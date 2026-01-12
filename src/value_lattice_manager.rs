use serde::{Deserialize, Serialize};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatticeProcessStatus {
    pub pid: Option<u32>,
    pub status: String, // "stopped", "starting", "running", "failed"
    pub started_at: Option<SystemTime>,
    pub restart_count: u32,
    pub last_error: Option<String>,
}

pub struct ValueLatticeManager {
    process: Arc<Mutex<Option<Child>>>,
    status: Arc<Mutex<LatticeProcessStatus>>,
    binary_path: String,
}

impl ValueLatticeManager {
    pub fn new(binary_path: String) -> Self {
        Self {
            process: Arc::new(Mutex::new(None)),
            status: Arc::new(Mutex::new(LatticeProcessStatus {
                pid: None,
                status: "stopped".to_string(),
                started_at: None,
                restart_count: 0,
                last_error: None,
            })),
            binary_path,
        }
    }

    pub fn start(&self) -> Result<(), String> {
        let mut process_guard = self.process.lock().unwrap();
        let mut status_guard = self.status.lock().unwrap();

        if status_guard.status == "running" {
            return Ok(());
        }

        status_guard.status = "starting".to_string();
        drop(status_guard);

        match Command::new(&self.binary_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(child) => {
                let pid = child.id();
                *process_guard = Some(child);

                let mut status_guard = self.status.lock().unwrap();
                status_guard.pid = Some(pid);
                status_guard.status = "running".to_string();
                status_guard.started_at = Some(SystemTime::now());
                status_guard.restart_count += 1;
                status_guard.last_error = None;

                println!("Value lattice process started with PID: {}", pid);
                Ok(())
            }
            Err(e) => {
                let mut status_guard = self.status.lock().unwrap();
                status_guard.status = "failed".to_string();
                status_guard.last_error = Some(e.to_string());
                Err(format!("Failed to start value lattice: {}", e))
            }
        }
    }

    pub fn stop(&self) -> Result<(), String> {
        let mut process_guard = self.process.lock().unwrap();
        let mut status_guard = self.status.lock().unwrap();

        if let Some(mut child) = process_guard.take() {
            match child.kill() {
                Ok(_) => {
                    let _ = child.wait();
                    status_guard.status = "stopped".to_string();
                    status_guard.pid = None;
                    println!("Value lattice process stopped");
                    Ok(())
                }
                Err(e) => Err(format!("Failed to stop process: {}", e)),
            }
        } else {
            status_guard.status = "stopped".to_string();
            Ok(())
        }
    }

    pub fn restart(&self) -> Result<(), String> {
        self.stop()?;
        thread::sleep(Duration::from_secs(1));
        self.start()
    }

    pub fn status(&self) -> LatticeProcessStatus {
        self.status.lock().unwrap().clone()
    }

    pub fn compile_binary(&self, source_path: &str) -> Result<(), String> {
        let output = Command::new("cargo")
            .args(&["build", "--release", "--bin", "value_lattice_server"])
            .current_dir(source_path)
            .output()
            .map_err(|e| format!("Failed to run cargo: {}", e))?;

        if output.status.success() {
            println!("Value lattice binary compiled successfully");
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("Compilation failed: {}", stderr))
        }
    }
}
