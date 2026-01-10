// Common utilities and shared functionality
pub mod ffi_plugin;
pub mod p2p_types;
pub mod server_utils;

// Re-export commonly used types and functions
pub use server_utils::{standard_html_footer, standard_html_header, ClientRecord, ServerState};
