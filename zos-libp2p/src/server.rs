// LibP2P server that compiles and loads plugins on the fly
use crate::plugin_driver::{CompilerEvent, PluginDriver};
use libp2p::{gossipsub, mdns, swarm::SwarmEvent, Swarm};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::process::Command;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SyncIdentityKind {
    Artifact,
    Receipt,
}

impl SyncIdentityKind {
    fn canonical_prefix(self) -> &'static str {
        match self {
            SyncIdentityKind::Artifact => "artifact:",
            SyncIdentityKind::Receipt => "receipt:",
        }
    }

    fn counterpart_prefix(self) -> &'static str {
        match self {
            SyncIdentityKind::Artifact => "receipt:",
            SyncIdentityKind::Receipt => "artifact:",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SyncInventoryIdentityMessage {
    #[serde(default = "default_inventory_identity_verb")]
    pub verb: String,
    #[serde(default = "default_inventory_identity_plugin")]
    pub plugin: String,
    pub kind: SyncIdentityKind,
    #[serde(alias = "object_id", alias = "artifact_id", alias = "receipt_id")]
    pub identity: String,
    #[serde(alias = "content_digest", alias = "sha256", alias = "hash")]
    pub digest: String,
    #[serde(
        default,
        alias = "related_artifact_ref",
        alias = "artifact_ref",
        alias = "source_ref",
        skip_serializing_if = "Option::is_none"
    )]
    pub backing_ref: Option<String>,
}

impl SyncInventoryIdentityMessage {
    fn new(
        plugin: impl Into<String>,
        kind: SyncIdentityKind,
        identity: impl Into<String>,
        digest: impl Into<String>,
        backing_ref: Option<impl Into<String>>,
    ) -> Self {
        Self {
            verb: "attach_inventory_identity".to_string(),
            plugin: plugin.into(),
            kind,
            identity: identity.into(),
            digest: digest.into(),
            backing_ref: backing_ref.map(|value| value.into()),
        }
    }

    fn normalized(mut self) -> Result<Self, String> {
        self.verb = if self.verb.trim().is_empty() {
            default_inventory_identity_verb()
        } else {
            self.verb.trim().to_string()
        };
        self.plugin = if self.plugin.trim().is_empty() {
            default_inventory_identity_plugin()
        } else {
            self.plugin.trim().to_string()
        };
        self.identity = normalize_identity(self.kind.clone(), &self.identity)?;
        self.digest = normalize_digest(&self.digest)?;
        self.backing_ref = normalize_backing_ref(self.kind, self.backing_ref.as_deref());
        Ok(self)
    }
}

fn default_inventory_identity_verb() -> String {
    "attach_inventory_identity".to_string()
}

fn default_inventory_identity_plugin() -> String {
    "sync-runtime".to_string()
}

fn normalize_identity(kind: SyncIdentityKind, identity: &str) -> Result<String, String> {
    let identity = identity.trim();
    if identity.is_empty() {
        return Err("identity cannot be empty".to_string());
    }
    let canonical = kind.canonical_prefix();
    if identity.starts_with(canonical) {
        return Ok(identity.to_string());
    }
    if identity.starts_with("artifact:") || identity.starts_with("receipt:") {
        return Ok(identity.to_string());
    }
    Ok(format!("{canonical}{identity}"))
}

fn normalize_digest(digest: &str) -> Result<String, String> {
    let digest = digest.trim();
    if digest.is_empty() {
        return Err("digest cannot be empty".to_string());
    }
    if digest.starts_with("sha256:") {
        Ok(digest.to_string())
    } else {
        Ok(format!("sha256:{digest}"))
    }
}

fn normalize_backing_ref(kind: SyncIdentityKind, backing_ref: Option<&str>) -> Option<String> {
    let value = backing_ref?.trim();
    if value.is_empty() {
        return None;
    }
    if value.starts_with("artifact:") || value.starts_with("receipt:") {
        return Some(value.to_string());
    }
    Some(format!("{}{}", kind.counterpart_prefix(), value))
}

#[derive(Debug)]
pub enum P2PVerb {
    LoadSo(String, String),              // name, path
    RegisterEvent(String, u32),          // plugin_name, event_type
    AttachData(String, Vec<u8>),         // plugin_name, data
    AttachInventoryIdentity(SyncInventoryIdentityMessage),
    RunWithFiles(String, Vec<String>),   // plugin_name, file_paths
    CaptureResult(String),               // plugin_name
    CompileSource(String, String),       // name, source_code
    CompileFile(String, String),         // name, file_path
    InvokeFunction(String, String, u32), // plugin_name, function_name, param
}

pub struct P2PPluginServer {
    driver: PluginDriver,
    event_registry: HashMap<String, Vec<u32>>, // plugin -> event_types
    stored_data: HashMap<String, Vec<u8>>,     // plugin -> data
    inventory_identities: HashMap<String, Vec<SyncInventoryIdentityMessage>>, // plugin -> sync identities
    results: HashMap<String, Vec<u8>>,         // plugin -> results
}

impl P2PPluginServer {
    pub fn new() -> Self {
        Self {
            driver: PluginDriver::new(),
            event_registry: HashMap::new(),
            stored_data: HashMap::new(),
            inventory_identities: HashMap::new(),
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

            P2PVerb::AttachInventoryIdentity(message) => {
                let message = message
                    .normalized()
                    .map_err(|err| format!("normalize sync inventory identity: {err}"))?;
                let plugin = message.plugin.clone();
                let encoded = serde_json::to_vec(&message)?;
                self.stored_data.insert(plugin.clone(), encoded);
                self.inventory_identities
                    .entry(plugin.clone())
                    .or_insert_with(Vec::new)
                    .push(message.clone());
                Ok(format!(
                    "Attached {} inventory identity {} to {}",
                    match message.kind {
                        SyncIdentityKind::Artifact => "artifact",
                        SyncIdentityKind::Receipt => "receipt",
                    },
                    message.identity,
                    plugin,
                ))
            }

            P2PVerb::RunWithFiles(plugin, files) => {
                for file in files {
                    let data = tokio::fs::read(&file).await?;
                    let event = CompilerEvent {
                        event_type: 2, // file processing
                        data: data.as_ptr(),
                        size: data.len(),
                    };
                    self.driver = std::mem::take(&mut self.driver).react(event);
                    self.driver.execute_plugin(&plugin, "span_execute_c")?;
                }
                Ok(format!("Ran {} with files", plugin))
            }

            P2PVerb::CaptureResult(plugin) => self.capture_result(&plugin),

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
        if let Some(verb) = self.parse_inventory_identity_json(msg)? {
            return Ok(verb);
        }

        let parts: Vec<&str> = msg.split_whitespace().collect();
        if parts.is_empty() {
            return Err("Empty message".into());
        }

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
            "ATTACH_INVENTORY" => Ok(P2PVerb::AttachInventoryIdentity(
                SyncInventoryIdentityMessage::new(
                    parts[1],
                    Self::parse_sync_identity_kind(parts[2])?,
                    parts[3],
                    parts[4],
                    parts.get(5).copied(),
                ),
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
            _ => Err("Unknown verb".into()),
        }
    }

    fn capture_result(&mut self, plugin: &str) -> Result<String, Box<dyn std::error::Error>> {
        if let Some(messages) = self.inventory_identities.get(plugin) {
            let result = serde_json::to_vec(messages)?;
            self.results.insert(plugin.to_string(), result.clone());
            return Ok(format!(
                "Captured {} inventory identities for {}",
                messages.len(),
                plugin
            ));
        }

        let result = format!("Result from {}", plugin).into_bytes();
        self.results.insert(plugin.to_string(), result.clone());
        Ok(format!(
            "Captured result: {:?}",
            String::from_utf8_lossy(&result)
        ))
    }

    fn parse_inventory_identity_json(
        &self,
        msg: &str,
    ) -> Result<Option<P2PVerb>, Box<dyn std::error::Error>> {
        let parsed: SyncInventoryIdentityMessage = match serde_json::from_str(msg) {
            Ok(value) => value,
            Err(_) => return Ok(None),
        };
        if parsed.verb != "attach_inventory_identity" {
            return Ok(None);
        }
        let parsed = parsed
            .normalized()
            .map_err(|err| format!("normalize sync inventory identity: {err}"))?;
        Ok(Some(P2PVerb::AttachInventoryIdentity(parsed)))
    }

    fn parse_sync_identity_kind(raw: &str) -> Result<SyncIdentityKind, Box<dyn std::error::Error>> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "artifact" => Ok(SyncIdentityKind::Artifact),
            "erdfa_manifest" => Ok(SyncIdentityKind::Artifact),
            "manifest" => Ok(SyncIdentityKind::Artifact),
            "shard" => Ok(SyncIdentityKind::Artifact),
            "receipt" => Ok(SyncIdentityKind::Receipt),
            "observation" => Ok(SyncIdentityKind::Receipt),
            "zkperf_observation" => Ok(SyncIdentityKind::Receipt),
            "proof" => Ok(SyncIdentityKind::Receipt),
            _ => Err("Unknown sync identity kind".into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_inventory_identity_json_message() {
        let server = P2PPluginServer::new();
        let msg = serde_json::to_string(&SyncInventoryIdentityMessage::new(
            "sync-runtime",
            SyncIdentityKind::Artifact,
            "artifact:wikidata-2026-demo",
            "sha256:stream-container-example",
            Some("receipt:zkperf-run-20260329-a"),
        ))
        .expect("encode");

        let parsed = server.parse_verb(&msg).expect("parse inventory identity json");
        match parsed {
            P2PVerb::AttachInventoryIdentity(message) => {
                assert_eq!(message.plugin, "sync-runtime");
                assert_eq!(message.kind, SyncIdentityKind::Artifact);
                assert_eq!(message.identity, "artifact:wikidata-2026-demo");
                assert_eq!(message.digest, "sha256:stream-container-example");
                assert_eq!(
                    message.backing_ref.as_deref(),
                    Some("receipt:zkperf-run-20260329-a")
                );
            }
            _ => panic!("expected inventory identity verb"),
        }
    }

    #[test]
    fn parse_inventory_identity_json_message_normalizes_producer_shape() {
        let server = P2PPluginServer::new();
        let msg = r#"{
            "verb":"attach_inventory_identity",
            "kind":"receipt",
            "object_id":"zkperf-run-20260329-a",
            "hash":"obs-0001",
            "related_artifact_ref":"wikidata-2026-demo"
        }"#;

        let parsed = server.parse_verb(msg).expect("parse inventory identity json");
        match parsed {
            P2PVerb::AttachInventoryIdentity(message) => {
                assert_eq!(message.plugin, "sync-runtime");
                assert_eq!(message.kind, SyncIdentityKind::Receipt);
                assert_eq!(message.identity, "receipt:zkperf-run-20260329-a");
                assert_eq!(message.digest, "sha256:obs-0001");
                assert_eq!(
                    message.backing_ref.as_deref(),
                    Some("artifact:wikidata-2026-demo")
                );
            }
            _ => panic!("expected inventory identity verb"),
        }
    }

    #[test]
    fn parse_sync_identity_kind_accepts_producer_aliases() {
        assert_eq!(
            P2PPluginServer::parse_sync_identity_kind("manifest").expect("manifest"),
            SyncIdentityKind::Artifact
        );
        assert_eq!(
            P2PPluginServer::parse_sync_identity_kind("zkperf_observation").expect("observation"),
            SyncIdentityKind::Receipt
        );
    }

    #[test]
    fn capture_result_prefers_inventory_identity_payloads() {
        let mut server = P2PPluginServer::new();
        server
            .inventory_identities
            .entry("sync-runtime".to_string())
            .or_insert_with(Vec::new)
            .push(SyncInventoryIdentityMessage::new(
                "sync-runtime",
                SyncIdentityKind::Receipt,
                "receipt:zkperf-run-20260329-a",
                "sha256:obs-0001",
                Some("artifact:wikidata-2026-demo"),
            ));

        let result = server.capture_result("sync-runtime").expect("capture result");
        assert_eq!(result, "Captured 1 inventory identities for sync-runtime");

        let stored = server.results.get("sync-runtime").expect("stored result");
        let decoded: Vec<SyncInventoryIdentityMessage> =
            serde_json::from_slice(stored).expect("decode stored inventory identities");
        assert_eq!(decoded.len(), 1);
        assert_eq!(decoded[0].identity, "receipt:zkperf-run-20260329-a");
    }
}
