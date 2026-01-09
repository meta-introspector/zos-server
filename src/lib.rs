// ZOS Server - Zero Ontology System
// Core stable modules only

pub mod lmfdb_orbits;
#[macro_use]
pub mod orbit_macros;
pub mod automorphic_bootstrap;
pub mod rust_soul_eigenmatrix;
pub mod zos_system;

// Core plugins only
pub mod plugins;

// Optional modules for extended functionality
pub mod compiler_integration;
pub mod llm_compiler_service;
#[cfg(feature = "self-build")]
pub mod self_build_cli;
#[cfg(feature = "self-build")]
pub mod self_builder;

#[cfg(feature = "notebooklm")]
pub mod notebooklm_cli;
#[cfg(feature = "notebooklm")]
pub mod notebooklm_interface;

// Experimental modules (optional)
#[cfg(feature = "experimental")]
#[allow(unexpected_cfgs)]
pub mod extras;

// Re-export orbit system
pub use lmfdb_orbits::*;
pub use orbit_macros::*;
