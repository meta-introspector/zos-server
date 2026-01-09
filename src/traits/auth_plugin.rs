// Authentication Plugin Trait for ZOS Server
use std::collections::HashMap;

/// Authentication result
#[derive(Debug, Clone)]
pub enum AuthResult {
    Granted { role: String, permissions: Vec<String> },
    Denied { reason: String },
}

/// Authentication request
#[derive(Debug, Clone)]
pub struct AuthRequest {
    pub public_key: String,
    pub function_name: String,
    pub timestamp: u64,
    pub signature: String,
}

/// Core authentication plugin trait
pub trait AuthPlugin: Send + Sync {
    /// Plugin name
    fn name(&self) -> &'static str;

    /// Authenticate a request
    fn authenticate(&self, request: &AuthRequest) -> Result<AuthResult, String>;

    /// Register a new key with role
    fn register_key(&mut self, public_key: String, role: String) -> Result<(), String>;

    /// Revoke a key
    fn revoke_key(&mut self, public_key: &str) -> Result<(), String>;

    /// List registered keys
    fn list_keys(&self) -> Vec<(String, String)>;
}
