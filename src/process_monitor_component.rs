use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub cpu_usage: f64,
    pub memory_mb: f64,
    pub command: String,
}

pub struct ProcessMonitorComponent;

impl ProcessMonitorComponent {
    pub fn new() -> Self {
        Self
    }

    pub fn get_running_processes(&self) -> HashMap<String, ProcessInfo> {
        let mut processes = HashMap::new();

        // Get process list using ps command
        if let Ok(output) = Command::new("ps").args(&["aux", "--no-headers"]).output() {
            let output_str = String::from_utf8_lossy(&output.stdout);

            for line in output_str.lines().take(20) {
                // Limit to top 20 processes
                let fields: Vec<&str> = line.split_whitespace().collect();
                if fields.len() >= 11 {
                    if let (Ok(pid), Ok(cpu), Ok(mem)) = (
                        fields[1].parse::<u32>(),
                        fields[2].parse::<f64>(),
                        fields[3].parse::<f64>(),
                    ) {
                        let command = fields[10..].join(" ");
                        let name = fields[10]
                            .split('/')
                            .last()
                            .unwrap_or(fields[10])
                            .to_string();

                        // Convert memory from % to MB (rough estimate)
                        let memory_mb = mem * 16.0; // Assuming 16GB total RAM

                        processes.insert(
                            name.clone(),
                            ProcessInfo {
                                pid,
                                cpu_usage: cpu,
                                memory_mb,
                                command,
                            },
                        );
                    }
                }
            }
        }

        processes
    }

    pub fn get_zos_user_processes(&self) -> HashMap<String, ProcessInfo> {
        let mut processes = HashMap::new();

        // Get processes running as zos user
        if let Ok(output) = Command::new("ps")
            .args(&["-u", "zos", "-o", "pid,pcpu,pmem,comm", "--no-headers"])
            .output()
        {
            let output_str = String::from_utf8_lossy(&output.stdout);

            for line in output_str.lines() {
                let fields: Vec<&str> = line.trim().split_whitespace().collect();
                if fields.len() >= 4 {
                    if let (Ok(pid), Ok(cpu), Ok(mem)) = (
                        fields[0].parse::<u32>(),
                        fields[1].parse::<f64>(),
                        fields[2].parse::<f64>(),
                    ) {
                        let name = fields[3].to_string();
                        let memory_mb = mem * 16.0; // Rough estimate

                        processes.insert(
                            name.clone(),
                            ProcessInfo {
                                pid,
                                cpu_usage: cpu,
                                memory_mb,
                                command: name.clone(),
                            },
                        );
                    }
                }
            }
        }

        processes
    }
}
