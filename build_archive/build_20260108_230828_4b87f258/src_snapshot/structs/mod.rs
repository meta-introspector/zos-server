// ZOS Server Structs - Zero Ontology System
// Consolidated structs from all P2P modules

use std::collections::HashMap;
use libp2p::PeerId;

#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub peer_id: String,
    pub address: String,
    pub connected: bool,
    pub last_seen: std::time::SystemTime,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DatasetSeed {
    pub name: String,
    pub version: String,
    pub hash: String,
    pub size: u64,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub struct LoadedSo {
    pub name: String,
    pub path: String,
    pub handle: *mut std::ffi::c_void,
    pub functions: HashMap<String, *mut std::ffi::c_void>,
}

#[derive(Debug)]
pub struct P2PLibWrapper {
    pub loaded_libs: HashMap<String, LoadedSo>,
    pub peer_id: Option<PeerId>,
}

#[derive(Debug)]
pub struct RustcDriverWrapper {
    pub driver_path: String,
    pub temp_dir: std::path::PathBuf,
    pub compilation_cache: HashMap<String, String>,
}

#[derive(Debug)]
pub struct P2PRustcLoader {
    pub rustc_wrapper: RustcDriverWrapper,
    pub network: P2PLibWrapper,
}

#[derive(Debug)]
pub struct LibP2P2Server {
    pub swarm: Option<libp2p::Swarm<libp2p::gossipsub::Behaviour>>,
    pub peers: HashMap<String, PeerInfo>,
    pub datasets: HashMap<String, DatasetSeed>,
}

#[derive(Debug)]
pub struct P2PPluginServer {
    pub plugins: HashMap<String, LoadedSo>,
    pub network: P2PLibWrapper,
}

#[derive(Debug)]
pub struct AnalysisData {
    pub compilation_time: std::time::Duration,
    pub binary_size: u64,
    pub dependencies: Vec<String>,
    pub errors: Vec<String>,
}
