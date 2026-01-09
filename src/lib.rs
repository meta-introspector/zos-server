// ZOS Server - Zero Ontology System
// Core macro-generated system

pub mod core_macros;
pub mod zos_system;

// Core plugins only
pub mod plugins;

// Self-building system
pub mod self_builder;
pub mod self_build_cli;

// NotebookLM integration
pub mod notebooklm_interface;
pub mod notebooklm_cli;

// Re-export core system
pub use zos_system::*;
