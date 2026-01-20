// ZOS Server - Minimal foundation library
// AGPL-3.0 License

// Foundation library for ZOS server
pub mod auth;
pub mod binary_classifier;
pub mod binary_inspector;
pub mod cache;
pub mod cargo2plugin_loader;
pub mod cicd_dashboard;
pub mod code_transformation_graph;
pub mod common;
pub mod core;
pub mod git_analyzer;
pub mod github_importer;
pub mod meta_introspector;
pub mod minimal_server_plugin;
pub mod mkbuildrs_patcher;
pub mod p2p_rustc_loader;
pub mod plugin_driver;
pub mod process_monitor_component;
pub mod project_watcher;
pub mod repo_status_manager;
pub mod security_audit;
pub mod self_bootstrap_system;
pub mod telemetry;
pub mod value_lattice_manager;
pub mod value_lattice_processor;
pub mod web;
pub mod zos_api;

// Clip2Secure modules
pub mod automorphic_compiler;
// pub mod clip2secure_lints;
pub mod complexity_types;
pub mod convergence_analyzer;
pub mod cpu_optimizer;
pub mod dual_model_prover;
pub mod iree_kleene_backend;
pub mod kleene_detector;
pub mod kleene_memory;
pub mod lean4_foundation;
pub mod meta_fixed_point;
pub mod meta_introspector_capstone;
pub mod modular_form_spec;
pub mod nidex_builder;
pub mod nvidia_kleene;
pub mod plantation_filter;

// Re-export key types for macro usage
pub use complexity_types::*;
pub mod plugin_registry;
pub mod plugins;
pub mod process_monitor;
pub mod services;
pub mod session;
pub mod traits;
pub mod version;
// Test auto-reload Sat Jan 10 06:31:40 PM EST 2026
// Auto-reload test Sat Jan 10 06:35:52 PM EST 2026
pub mod client_telemetry;
pub mod static_server;
