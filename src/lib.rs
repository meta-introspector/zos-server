// ZOS Server - Zero Ontology System
// LMFDB Orbit-based system

pub mod lmfdb_orbits;
#[macro_use] 
pub mod orbit_macros;
pub mod zos_system;
pub mod automorphic_bootstrap;
pub mod rust_soul_eigenmatrix;

// Core plugins only
pub mod plugins;

// Optional modules for extended functionality
#[cfg(feature = "self-build")]
pub mod self_builder;
#[cfg(feature = "self-build")]
pub mod self_build_cli;

#[cfg(feature = "notebooklm")]
pub mod notebooklm_interface;
#[cfg(feature = "notebooklm")]
pub mod notebooklm_cli;

// Re-export orbit system
pub use lmfdb_orbits::*;
pub use orbit_macros::*;
