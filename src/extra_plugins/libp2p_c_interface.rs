// LibP2P Plugin C Interface
// C-compatible functions for the libp2p plugin

use libp2p::{Swarm, gossipsub, identity, PeerId};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::ptr;

static mut SWARM: Option<Swarm<gossipsub::Behaviour>> = None;

#[no_mangle]
pub unsafe extern "C" fn p2p_init() -> c_int {
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());
    
    // Initialize gossipsub
    let gossipsub_config = gossipsub::ConfigBuilder::default()
        .heartbeat_interval(std::time::Duration::from_secs(10))
        .validation_mode(gossipsub::ValidationMode::Strict)
        .build()
        .expect("Valid config");
    
    let gossipsub = gossipsub::Behaviour::new(
        gossipsub::MessageAuthenticity::Signed(local_key.clone()),
        gossipsub_config,
    ).expect("Correct configuration");

    // Create swarm
    let swarm = libp2p::SwarmBuilder::with_existing_identity(local_key)
        .with_tokio()
        .with_tcp(
            libp2p::tcp::Config::default(),
            libp2p::noise::Config::new,
            libp2p::yamux::Config::default,
        )
        .unwrap()
        .with_behaviour(|_| gossipsub)
        .unwrap()
        .with_swarm_config(|c| c.with_idle_connection_timeout(std::time::Duration::from_secs(60)))
        .build();

    SWARM = Some(swarm);
    0 // Success
}

#[no_mangle]
pub unsafe extern "C" fn p2p_connect_peer(peer_id: *const c_char) -> c_int {
    if peer_id.is_null() {
        return -1;
    }
    
    let c_str = CStr::from_ptr(peer_id);
    let peer_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return -2,
    };
    
    // Implementation would connect to peer
    println!("Connecting to peer: {}", peer_str);
    0 // Success
}

#[no_mangle]
pub unsafe extern "C" fn p2p_disconnect_peer(peer_id: *const c_char) -> c_int {
    if peer_id.is_null() {
        return -1;
    }
    
    let c_str = CStr::from_ptr(peer_id);
    let peer_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return -2,
    };
    
    println!("Disconnecting from peer: {}", peer_str);
    0 // Success
}

#[no_mangle]
pub unsafe extern "C" fn p2p_send_message(
    peer_id: *const c_char,
    message: *const u8,
    message_len: usize,
) -> c_int {
    if peer_id.is_null() || message.is_null() {
        return -1;
    }
    
    let c_str = CStr::from_ptr(peer_id);
    let peer_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return -2,
    };
    
    let msg_slice = std::slice::from_raw_parts(message, message_len);
    println!("Sending {} bytes to peer: {}", message_len, peer_str);
    0 // Success
}

#[no_mangle]
pub unsafe extern "C" fn p2p_cleanup() -> c_int {
    SWARM = None;
    0 // Success
}
