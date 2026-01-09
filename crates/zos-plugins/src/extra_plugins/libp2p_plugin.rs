// LibP2P Plugin Interface
// Plugin that implements P2P networking via libloading

use crate::traits::P2PNetwork;
use crate::enums::NetworkEvent;
use crate::structs::PeerInfo;
use libloading::{Library, Symbol};
use std::collections::HashMap;

pub struct LibP2PPlugin {
    library: Library,
    peers: HashMap<String, PeerInfo>,
}

// C-compatible function signatures for the plugin
type ConnectPeerFn = unsafe extern "C" fn(*const i8) -> i32;
type DisconnectPeerFn = unsafe extern "C" fn(*const i8) -> i32;
type SendMessageFn = unsafe extern "C" fn(*const i8, *const u8, usize) -> i32;
type ListPeersFn = unsafe extern "C" fn(*mut *mut i8, *mut usize) -> i32;

impl LibP2PPlugin {
    pub fn new(plugin_path: &str) -> Result<Self, String> {
        let library = unsafe {
            Library::new(plugin_path)
                .map_err(|e| format!("Failed to load libp2p plugin: {}", e))?
        };

        Ok(LibP2PPlugin {
            library,
            peers: HashMap::new(),
        })
    }
}

impl P2PNetwork for LibP2PPlugin {
    fn connect_peer(&mut self, peer_id: &str) -> Result<(), String> {
        unsafe {
            let connect_fn: Symbol<ConnectPeerFn> = self.library
                .get(b"p2p_connect_peer")
                .map_err(|e| format!("Function not found: {}", e))?;

            let c_peer_id = std::ffi::CString::new(peer_id)
                .map_err(|e| format!("Invalid peer ID: {}", e))?;

            let result = connect_fn(c_peer_id.as_ptr());
            if result == 0 {
                Ok(())
            } else {
                Err(format!("Connection failed with code: {}", result))
            }
        }
    }

    fn disconnect_peer(&mut self, peer_id: &str) -> Result<(), String> {
        unsafe {
            let disconnect_fn: Symbol<DisconnectPeerFn> = self.library
                .get(b"p2p_disconnect_peer")
                .map_err(|e| format!("Function not found: {}", e))?;

            let c_peer_id = std::ffi::CString::new(peer_id)
                .map_err(|e| format!("Invalid peer ID: {}", e))?;

            let result = disconnect_fn(c_peer_id.as_ptr());
            if result == 0 {
                Ok(())
            } else {
                Err(format!("Disconnection failed with code: {}", result))
            }
        }
    }

    fn send_message(&mut self, peer_id: &str, message: &[u8]) -> Result<(), String> {
        unsafe {
            let send_fn: Symbol<SendMessageFn> = self.library
                .get(b"p2p_send_message")
                .map_err(|e| format!("Function not found: {}", e))?;

            let c_peer_id = std::ffi::CString::new(peer_id)
                .map_err(|e| format!("Invalid peer ID: {}", e))?;

            let result = send_fn(c_peer_id.as_ptr(), message.as_ptr(), message.len());
            if result == 0 {
                Ok(())
            } else {
                Err(format!("Send failed with code: {}", result))
            }
        }
    }

    fn list_peers(&self) -> Vec<PeerInfo> {
        self.peers.values().cloned().collect()
    }

    fn handle_event(&mut self, event: NetworkEvent) -> Result<(), String> {
        match event {
            NetworkEvent::PeerConnected(peer_id) => {
                let peer_info = PeerInfo {
                    peer_id: peer_id.clone(),
                    address: "unknown".to_string(),
                    connected: true,
                    last_seen: std::time::SystemTime::now(),
                    capabilities: vec![],
                };
                self.peers.insert(peer_id, peer_info);
            },
            NetworkEvent::PeerDisconnected(peer_id) => {
                self.peers.remove(&peer_id);
            },
            _ => {}
        }
        Ok(())
    }
}
