// ZOS Server Macros - Zero Ontology System
// Utility macros for P2P operations

/// Macro to generate P2P verb handlers
#[macro_export]
macro_rules! handle_p2p_verb {
    ($verb:expr, $handler:expr, $args:expr) => {
        match $verb {
            P2PVerb::Connect => $handler.connect_peer(std::str::from_utf8($args)?),
            P2PVerb::Disconnect => $handler.disconnect_peer(std::str::from_utf8($args)?),
            P2PVerb::ListPeers => {
                let peers = $handler.list_peers();
                Ok(serde_json::to_vec(&peers)?)
            },
            _ => Err("Unsupported verb".to_string()),
        }
    };
}

/// Macro to create a P2P server with default configuration
#[macro_export]
macro_rules! create_p2p_server {
    ($server_type:ty) => {
        <$server_type>::new()
    };
    ($server_type:ty, $config:expr) => {
        <$server_type>::with_config($config)
    };
}

/// Macro for safe library function calls
#[macro_export]
macro_rules! safe_lib_call {
    ($lib:expr, $func:expr, $args:expr) => {
        match $lib.call_function($func, $args) {
            Ok(result) => result,
            Err(e) => {
                eprintln!("Library call failed: {}", e);
                return Err(e);
            }
        }
    };
}

/// Macro to log P2P events
#[macro_export]
macro_rules! log_p2p_event {
    ($event:expr) => {
        println!("[P2P] {}: {:?}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"), $event);
    };
    ($event:expr, $($arg:tt)*) => {
        println!("[P2P] {}: {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"), format!($($arg)*));
    };
}
