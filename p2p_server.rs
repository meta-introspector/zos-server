// LibP2P server that compiles and loads plugins on the fly
use crate::plugin_driver::{CompilerEvent, PluginDriver};
use libp2p::{gossipsub, mdns, swarm::SwarmEvent, Swarm};
use std::collections::HashMap;
use tokio::process::Command;

#[derive(Debug)]
pub enum P2PVerb {
    LoadSo(String, String),              // name, path
    RegisterEvent(String, u32),          // plugin_name, event_type
    AttachData(String, Vec<u8>),         // plugin_name, data
    RunWithFiles(String, Vec<String>),   // plugin_name, file_paths
    CaptureResult(String),               // plugin_name
    CompileSource(String, String),       // name, source_code
    CompileFile(String, String),         // name, file_path
    InvokeFunction(String, String, u32), // plugin_name, function_name, param
    // Telemetry verbs
    StartTrace(String, String),    // crate_name, phase (hir, mir, syn)
    StopTrace(String),             // crate_name
    PerfRecord(String, String),    // crate_name, command
    StraceRecord(String, String),  // crate_name, command
    CompareTraces(String, String), // trace1, trace2
    // Data manipulation verbs
    Intercept(String, String),  // target, pattern
    Extract(String, String),    // source, selector
    Exfiltrate(String, String), // data_id, destination
    Munge(String, String),      // data_id, transform
}

pub struct P2PPluginServer {
    driver: PluginDriver,
    event_registry: HashMap<String, Vec<u32>>, // plugin -> event_types
    stored_data: HashMap<String, Vec<u8>>,     // plugin -> data
    results: HashMap<String, Vec<u8>>,         // plugin -> results
}

impl P2PPluginServer {
    pub fn new() -> Self {
        Self {
            driver: PluginDriver::new(),
            event_registry: HashMap::new(),
            stored_data: HashMap::new(),
            results: HashMap::new(),
        }
    }

    // Execute P2P verbs
    pub async fn execute_verb(
        &mut self,
        verb: P2PVerb,
    ) -> Result<String, Box<dyn std::error::Error>> {
        match verb {
            P2PVerb::LoadSo(name, path) => {
                self.driver.load_plugin(&name, &path)?;
                Ok(format!("Loaded {}", name))
            }

            P2PVerb::RegisterEvent(plugin, event_type) => {
                self.event_registry
                    .entry(plugin.clone())
                    .or_insert_with(Vec::new)
                    .push(event_type);
                Ok(format!("Registered {} for event {}", plugin, event_type))
            }

            P2PVerb::AttachData(plugin, data) => {
                self.stored_data.insert(plugin.clone(), data);
                Ok(format!("Attached data to {}", plugin))
            }

            P2PVerb::RunWithFiles(plugin, files) => {
                for file in files {
                    let data = tokio::fs::read(&file).await?;
                    let event = CompilerEvent {
                        event_type: 2, // file processing
                        data: data.as_ptr(),
                        size: data.len(),
                    };
                    self.driver = std::mem::take::<PluginDriver>(&mut self.driver).react(event);
                    self.driver.execute_plugin(&plugin, "span_execute_c")?;
                }
                Ok(format!("Ran {} with files", plugin))
            }

            P2PVerb::CaptureResult(plugin) => {
                // Capture result from plugin execution
                let result = format!("Result from {}", plugin).into_bytes();
                self.results.insert(plugin.clone(), result.clone());
                Ok(format!(
                    "Captured result: {:?}",
                    String::from_utf8_lossy(&result)
                ))
            }

            P2PVerb::CompileSource(name, source) => {
                let so_path = format!("/tmp/{}.so", name);
                let rs_path = format!("/tmp/{}.rs", name);

                tokio::fs::write(&rs_path, source).await?;

                let output = Command::new("rustc")
                    .args(&["--crate-type", "cdylib", "-o", &so_path, &rs_path])
                    .output()
                    .await?;

                if output.status.success() {
                    self.driver.load_plugin(&name, &so_path)?;
                    Ok(format!("Compiled and loaded {}", name))
                } else {
                    Err(format!(
                        "Compilation failed: {}",
                        String::from_utf8_lossy(&output.stderr)
                    )
                    .into())
                }
            }

            P2PVerb::CompileFile(name, file_path) => {
                let source = tokio::fs::read_to_string(&file_path).await?;
                self.execute_verb(P2PVerb::CompileSource(name, source))
                    .await
            }

            P2PVerb::InvokeFunction(plugin, func_name, param) => {
                self.driver.execute_plugin(&plugin, &func_name)?;
                let result = format!("Invoked {}::{} with param {}", plugin, func_name, param);
                Ok(result)
            }
        }
    }

    // Process verb from network message
    pub async fn process_message(
        &mut self,
        msg: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let verb = self.parse_verb(msg)?;
        self.execute_verb(verb).await
    }

    fn parse_verb(&self, msg: &str) -> Result<P2PVerb, Box<dyn std::error::Error>> {
        let parts: Vec<&str> = msg.split_whitespace().collect();
        match parts[0] {
            "LOAD_SO" => Ok(P2PVerb::LoadSo(parts[1].to_string(), parts[2].to_string())),
            "REGISTER" => Ok(P2PVerb::RegisterEvent(
                parts[1].to_string(),
                parts[2].parse()?,
            )),
            "ATTACH" => Ok(P2PVerb::AttachData(
                parts[1].to_string(),
                parts[2].as_bytes().to_vec(),
            )),
            "RUN_FILES" => Ok(P2PVerb::RunWithFiles(
                parts[1].to_string(),
                parts[2..].iter().map(|s| s.to_string()).collect(),
            )),
            "CAPTURE" => Ok(P2PVerb::CaptureResult(parts[1].to_string())),
            "COMPILE_SRC" => Ok(P2PVerb::CompileSource(
                parts[1].to_string(),
                parts[2..].join(" "),
            )),
            "COMPILE_FILE" => Ok(P2PVerb::CompileFile(
                parts[1].to_string(),
                parts[2].to_string(),
            )),
            "INVOKE" => Ok(P2PVerb::InvokeFunction(
                parts[1].to_string(),
                parts[2].to_string(),
                parts.get(3).unwrap_or(&"0").parse().unwrap_or(0),
            )),
            // Telemetry verbs
            "START_TRACE" => Ok(P2PVerb::StartTrace(
                parts[1].to_string(),
                parts[2].to_string(),
            )),
            "STOP_TRACE" => Ok(P2PVerb::StopTrace(parts[1].to_string())),
            "PERF_RECORD" => Ok(P2PVerb::PerfRecord(
                parts[1].to_string(),
                parts[2..].join(" "),
            )),
            "STRACE_RECORD" => Ok(P2PVerb::StraceRecord(
                parts[1].to_string(),
                parts[2..].join(" "),
            )),
            "COMPARE_TRACES" => Ok(P2PVerb::CompareTraces(
                parts[1].to_string(),
                parts[2].to_string(),
            )),
            // Data manipulation verbs
            "INTERCEPT" => Ok(P2PVerb::Intercept(
                parts[1].to_string(),
                parts[2].to_string(),
            )),
            "EXTRACT" => Ok(P2PVerb::Extract(parts[1].to_string(), parts[2].to_string())),
            "EXFILTRATE" => Ok(P2PVerb::Exfiltrate(
                parts[1].to_string(),
                parts[2].to_string(),
            )),
            "MUNGE" => Ok(P2PVerb::Munge(parts[1].to_string(), parts[2].to_string())),
            _ => Err("Unknown verb".into()),
        }
    }
}
