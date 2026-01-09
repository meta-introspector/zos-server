use libp2p::{PeerId, Multiaddr};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct LibP2PVerb {
    pub verb_id: String,
    pub service_name: String,
    pub protocol: String,
    pub handler: String,
    pub access_level: AccessLevel,
    pub seat_number: u32,
}

#[derive(Debug, Clone)]
pub enum AccessLevel {
    Public,
    SeatHolders,
    Chamber(String),
    Specific(Vec<u32>),
}

pub struct VerbRegistry {
    pub verbs: HashMap<String, LibP2PVerb>,
    pub plugins: HashMap<String, PluginInfo>,
    pub peer_seats: HashMap<PeerId, u32>,
}

#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub name: String,
    pub protocol: String,
    pub seat_owner: u32,
    pub guarded: bool,
    pub permissions: Vec<String>,
}

impl VerbRegistry {
    pub fn new() -> Self {
        Self {
            verbs: HashMap::new(),
            plugins: HashMap::new(),
            peer_seats: HashMap::new(),
        }
    }

    pub fn register_plugin(&mut self, plugin: PluginInfo) {
        println!("🔌 Plugin registered: {} (seat: {}, guarded: {})",
                 plugin.name, plugin.seat_owner, plugin.guarded);
        self.plugins.insert(plugin.name.clone(), plugin);
    }

    pub fn register_verb(&mut self, verb: LibP2PVerb) -> Result<(), String> {
        // Check if plugin exists and seat has permission
        if let Some(plugin) = self.plugins.get(&verb.service_name) {
            if plugin.seat_owner != verb.seat_number && plugin.guarded {
                return Err("Access denied to guarded plugin".to_string());
            }
        }

        println!("📡 LibP2P verb registered: {} → {}", verb.verb_id, verb.protocol);
        self.verbs.insert(verb.verb_id.clone(), verb);
        Ok(())
    }

    pub fn can_access_verb(&self, verb_id: &str, requester_seat: u32) -> bool {
        if let Some(verb) = self.verbs.get(verb_id) {
            match &verb.access_level {
                AccessLevel::Public => true,
                AccessLevel::SeatHolders => requester_seat > 0,
                AccessLevel::Chamber(chamber) => {
                    match chamber.as_str() {
                        "senate" => requester_seat >= 1 && requester_seat <= 100,
                        "representatives" => requester_seat >= 101 && requester_seat <= 600,
                        "vendors" => requester_seat >= 601 && requester_seat <= 1618,
                        _ => false,
                    }
                },
                AccessLevel::Specific(seats) => seats.contains(&requester_seat),
            }
        } else {
            false
        }
    }

    pub fn handle_verb_request(&self, verb_id: &str, requester_peer: PeerId, data: &[u8]) -> Result<Vec<u8>, String> {
        let requester_seat = self.peer_seats.get(&requester_peer)
            .copied()
            .unwrap_or(0);

        if !self.can_access_verb(verb_id, requester_seat) {
            return Err("Access denied".to_string());
        }

        let verb = self.verbs.get(verb_id)
            .ok_or("Verb not found")?;

        // Route to appropriate plugin handler
        match verb.service_name.as_str() {
            "rustc_driver" => self.handle_rustc_request(verb, requester_seat, data),
            "compiler" => self.handle_compiler_request(verb, requester_seat, data),
            "build" => self.handle_build_request(verb, requester_seat, data),
            _ => Err("Unknown service".to_string()),
        }
    }

    fn handle_rustc_request(&self, verb: &LibP2PVerb, requester_seat: u32, data: &[u8]) -> Result<Vec<u8>, String> {
        let plugin = self.plugins.get("rustc_driver")
            .ok_or("rustc_driver plugin not found")?;

        // Check if requester has access to guarded rustc_driver
        if plugin.guarded && plugin.seat_owner != requester_seat {
            // Check if requester has dev center access
            if !self.has_dev_center_access(requester_seat) {
                return Err("Dev center access required for rustc_driver".to_string());
            }
        }

        // Parse compilation request
        let request: CompileRequest = serde_json::from_slice(data)
            .map_err(|e| format!("Invalid request: {}", e))?;

        // Execute compilation in sandboxed environment
        let result = self.execute_rustc_compilation(request, requester_seat)?;

        Ok(serde_json::to_vec(&result).unwrap())
    }

    fn has_dev_center_access(&self, seat_number: u32) -> bool {
        // Dev center access rules
        match seat_number {
            0 => true,                    // Root always has access
            1..=100 => true,             // Senate has dev access
            101..=600 => seat_number % 10 == 1, // Every 10th representative
            _ => false,                  // Vendors need explicit permission
        }
    }

    fn execute_rustc_compilation(&self, request: CompileRequest, seat: u32) -> Result<CompileResult, String> {
        // Create isolated compilation environment
        let workspace = format!("/tmp/zos_compile_seat_{}", seat);
        std::fs::create_dir_all(&workspace)
            .map_err(|e| format!("Failed to create workspace: {}", e))?;

        // Write source files
        for (path, content) in &request.sources {
            let file_path = format!("{}/{}", workspace, path);
            if let Some(parent) = std::path::Path::new(&file_path).parent() {
                std::fs::create_dir_all(parent).ok();
            }
            std::fs::write(&file_path, content)
                .map_err(|e| format!("Failed to write {}: {}", path, e))?;
        }

        // Execute rustc with resource limits
        let output = std::process::Command::new("timeout")
            .args(&["30s", "rustc"])
            .args(&request.args)
            .current_dir(&workspace)
            .output()
            .map_err(|e| format!("Compilation failed: {}", e))?;

        // Clean up workspace
        std::fs::remove_dir_all(&workspace).ok();

        Ok(CompileResult {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            artifacts: if output.status.success() {
                vec!["binary".to_string()]
            } else {
                vec![]
            },
        })
    }

    fn handle_compiler_request(&self, _verb: &LibP2PVerb, _seat: u32, _data: &[u8]) -> Result<Vec<u8>, String> {
        // Handle other compiler requests
        Ok(b"compiler response".to_vec())
    }

    fn handle_build_request(&self, _verb: &LibP2PVerb, _seat: u32, _data: &[u8]) -> Result<Vec<u8>, String> {
        // Handle build system requests
        Ok(b"build response".to_vec())
    }
}

#[derive(serde::Deserialize)]
pub struct CompileRequest {
    pub sources: HashMap<String, String>,
    pub args: Vec<String>,
    pub target: Option<String>,
}

#[derive(serde::Serialize)]
pub struct CompileResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub artifacts: Vec<String>,
}
