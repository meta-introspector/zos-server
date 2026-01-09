// ZOS Server - Zero Ontology System
// Main library module with organized structure

pub mod enums;
pub mod structs;
pub mod traits;
pub mod macros;
pub mod functions;
pub mod plugins;
pub mod verb_export;
pub mod plugin_registry;
pub mod protocol_exports;
pub mod mini_sdf_server;
pub mod node_coordinator;
pub mod blockchain_ingestor;
pub mod universal_plugin_loader;
pub mod verified_plugin_loader;
pub mod notebooklm_interface;
pub mod notebooklm_cli;

// Re-export commonly used items
pub use enums::*;
pub use structs::*;
pub use traits::*;
pub use functions::*;
pub use plugins::*;
pub use verb_export::*;
pub use plugin_registry::*;

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = "ZOS Server - Zero Ontology System";

/// Initialize the ZOS server system
pub fn init() -> Result<(), String> {
    println!("{} v{}", NAME, VERSION);
    println!("Initializing Zero Ontology System...");
    Ok(())
}
