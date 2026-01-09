// ZOS Server - Zero Ontology System
// Core stable modules only

pub mod lmfdb_orbits;
#[macro_use]
pub mod orbit_macros;
pub mod automorphic_bootstrap;
#[cfg(feature = "serde")]
pub mod rust_soul_eigenmatrix;
pub mod zos_system;

// Core plugins only
pub mod plugins;

// Optional modules for extended functionality
pub mod compiler_integration;
#[cfg(all(feature = "tokio", feature = "chrono", feature = "serde"))]
pub mod llm_compiler_service;
#[cfg(all(feature = "axum", feature = "tokio", feature = "serde"))]
pub mod secure_api_server;
#[cfg(all(feature = "libp2p", feature = "tokio", feature = "serde"))]
pub mod secure_libp2p_api;
#[cfg(feature = "self-build")]
pub mod self_build_cli;
#[cfg(feature = "self-build")]
pub mod self_builder;
#[cfg(all(feature = "tokio", feature = "serde", feature = "uuid"))]
pub mod task_modes;
#[cfg(all(feature = "chrono", feature = "serde", feature = "uuid"))]
pub mod task_registry;

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
