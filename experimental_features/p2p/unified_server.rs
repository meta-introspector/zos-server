use crate::common::p2p_types::*;
use libloading::Library;
use std::collections::HashMap;
// use std::ffi::CString;
use tokio::process::Command;

pub struct UnifiedP2PServer {
    peers: HashMap<String, PeerInfo>,
    datasets: HashMap<String, DatasetSeed>,
    loaded_libraries: HashMap<String, LoadedSo>,
    active_traces: HashMap<String, String>,
}

impl UnifiedP2PServer {
    pub fn new() -> Self {
        Self {
            peers: HashMap::new(),
            datasets: HashMap::new(),
            loaded_libraries: HashMap::new(),
            active_traces: HashMap::new(),
        }
    }

    pub async fn execute_verb(
        &mut self,
        verb: P2PVerb,
    ) -> Result<String, Box<dyn std::error::Error>> {
        match verb {
            P2PVerb::LoadSo(name, path) => self.load_so(&name, &path).await,
            P2PVerb::CompileSource(name, source) => self.compile_rust_source(&name, &source).await,
            P2PVerb::StartTrace(crate_name, phase) => self.start_trace(&crate_name, &phase).await,
            P2PVerb::StopTrace(crate_name) => self.stop_trace(&crate_name).await,
            P2PVerb::ConnectPeer(peer_addr) => self.connect_peer(&peer_addr).await,
            P2PVerb::LoadDataset(name, url) => self.load_dataset(&name, &url).await,
            _ => Err("Verb not implemented".into()),
        }
    }

    async fn load_so(
        &mut self,
        name: &str,
        path: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let library = unsafe { Library::new(path)? };
        let symbols = self.extract_symbols(&library)?;

        let loaded_so = LoadedSo {
            name: name.to_string(),
            path: path.to_string(),
            symbols: symbols.clone(),
            loaded_at: chrono::Utc::now(),
            size_bytes: std::fs::metadata(path)?.len(),
            checksum: format!("{:x}", md5::compute(std::fs::read(path)?)),
        };

        self.loaded_libraries.insert(name.to_string(), loaded_so);
        Ok(format!("Loaded {} with {} symbols", name, symbols.len()))
    }

    async fn compile_rust_source(
        &self,
        name: &str,
        source: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let temp_dir = tempfile::tempdir()?;
        let source_path = temp_dir.path().join(format!("{}.rs", name));
        std::fs::write(&source_path, source)?;

        let output = Command::new("rustc")
            .args(&["--crate-type", "cdylib", "-o"])
            .arg(temp_dir.path().join(format!("lib{}.so", name)))
            .arg(&source_path)
            .output()
            .await?;

        if output.status.success() {
            Ok(format!("Compiled {} successfully", name))
        } else {
            Err(format!(
                "Compilation failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )
            .into())
        }
    }

    async fn start_trace(
        &mut self,
        crate_name: &str,
        phase: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        self.active_traces
            .insert(crate_name.to_string(), phase.to_string());
        Ok(format!("Started {} trace for {}", phase, crate_name))
    }

    async fn stop_trace(&mut self, crate_name: &str) -> Result<String, Box<dyn std::error::Error>> {
        if let Some(phase) = self.active_traces.remove(crate_name) {
            Ok(format!("Stopped {} trace for {}", phase, crate_name))
        } else {
            Err(format!("No active trace for {}", crate_name).into())
        }
    }

    async fn connect_peer(
        &mut self,
        peer_addr: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let peer_info = PeerInfo {
            peer_id: format!("peer_{}", self.peers.len()),
            multiaddr: peer_addr.to_string(),
            protocols: vec!["zos/1.0.0".to_string()],
            last_seen: chrono::Utc::now(),
            reputation: 1.0,
            capabilities: vec!["compile".to_string(), "trace".to_string()],
        };

        self.peers
            .insert(peer_info.peer_id.clone(), peer_info.clone());
        Ok(format!("Connected to peer {}", peer_info.peer_id))
    }

    async fn load_dataset(
        &mut self,
        name: &str,
        url: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let dataset = DatasetSeed {
            name: name.to_string(),
            description: format!("Dataset loaded from {}", url),
            source_url: url.to_string(),
            hash: format!("{:x}", md5::compute(url.as_bytes())),
            size_bytes: 0, // Would be populated after download
            format: "unknown".to_string(),
            schema: None,
            tags: vec![],
            created_at: chrono::Utc::now(),
        };

        self.datasets.insert(name.to_string(), dataset);
        Ok(format!("Loaded dataset {}", name))
    }

    fn extract_symbols(
        &self,
        _library: &Library,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        // Simplified symbol extraction
        Ok(vec!["main".to_string(), "init".to_string()])
    }
}
