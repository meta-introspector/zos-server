// ZOS Server Functions - Zero Ontology System
// Common utility functions

use crate::structs::{PeerInfo, DatasetSeed, LoadedSo};
use crate::enums::{P2PVerb, LibVerb};
use std::collections::HashMap;

/// Initialize a new P2P network configuration
pub fn init_p2p_config() -> HashMap<String, String> {
    let mut config = HashMap::new();
    config.insert("listen_addr".to_string(), "/ip4/0.0.0.0/tcp/0".to_string());
    config.insert("bootstrap_nodes".to_string(), "".to_string());
    config.insert("max_peers".to_string(), "50".to_string());
    config
}

/// Validate peer information
pub fn validate_peer_info(peer: &PeerInfo) -> Result<(), String> {
    if peer.peer_id.is_empty() {
        return Err("Peer ID cannot be empty".to_string());
    }
    if peer.address.is_empty() {
        return Err("Peer address cannot be empty".to_string());
    }
    Ok(())
}

/// Create a dataset seed from metadata
pub fn create_dataset_seed(name: &str, version: &str, metadata: HashMap<String, String>) -> DatasetSeed {
    DatasetSeed {
        name: name.to_string(),
        version: version.to_string(),
        hash: calculate_hash(&format!("{}{}", name, version)),
        size: 0,
        metadata,
    }
}

/// Calculate a simple hash for identification
pub fn calculate_hash(input: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

/// Parse P2P verb from string
pub fn parse_p2p_verb(verb_str: &str) -> Result<P2PVerb, String> {
    match verb_str.to_lowercase().as_str() {
        "connect" => Ok(P2PVerb::Connect),
        "disconnect" => Ok(P2PVerb::Disconnect),
        "send_message" => Ok(P2PVerb::SendMessage),
        "receive_message" => Ok(P2PVerb::ReceiveMessage),
        "list_peers" => Ok(P2PVerb::ListPeers),
        "get_peer_info" => Ok(P2PVerb::GetPeerInfo),
        "load_dataset" => Ok(P2PVerb::LoadDataset),
        "unload_dataset" => Ok(P2PVerb::UnloadDataset),
        "query_dataset" => Ok(P2PVerb::QueryDataset),
        "load_library" => Ok(P2PVerb::LoadLibrary),
        "unload_library" => Ok(P2PVerb::UnloadLibrary),
        "call_function" => Ok(P2PVerb::CallFunction),
        "compile_rust" => Ok(P2PVerb::CompileRust),
        "load_binary" => Ok(P2PVerb::LoadBinary),
        "execute_function" => Ok(P2PVerb::ExecuteFunction),
        _ => Err(format!("Unknown verb: {}", verb_str)),
    }
}

/// Format error message with context
pub fn format_error(context: &str, error: &str) -> String {
    format!("[{}] Error: {}", context, error)
}

/// Create a default loaded SO structure
pub fn create_loaded_so(name: &str, path: &str) -> LoadedSo {
    LoadedSo {
        name: name.to_string(),
        path: path.to_string(),
        handle: std::ptr::null_mut(),
        functions: HashMap::new(),
    }
}
