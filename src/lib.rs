// ZOS Server - Zero Ontology System
// LMFDB Orbit-based system

#[macro_use]
pub mod core_macros;
pub mod lmfdb_orbits;
#[macro_use] 
pub mod orbit_macros;
pub mod zos_system;

// Core plugins only
pub mod plugins;

// Self-building system
pub mod self_builder;
pub mod self_build_cli;

// NotebookLM integration
pub mod notebooklm_interface;
pub mod notebooklm_cli;

// Re-export orbit system
pub use lmfdb_orbits::*;
pub use orbit_macros::*;
