// SSH-based Authentication Plugin Implementation
use crate::traits::auth_plugin::{AuthPlugin, AuthRequest, AuthResult};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// SSH-based authentication plugin
pub struct SshAuthPlugin {
    registered_keys: HashMap<String, String>, // public_key -> role
    role_permissions: HashMap<String, Vec<String>>,
}

impl SshAuthPlugin {
    pub fn new() -> Self {
        let mut plugin = Self {
            registered_keys: HashMap::new(),
            role_permissions: HashMap::new(),
        };
        plugin.setup_default_roles();
        plugin.register_ssh_root_key();
        plugin
    }

    fn setup_default_roles(&mut self) {
        self.role_permissions.insert("admin".to_string(), vec![
            "auto_label".to_string(),
            "codebase_analysis".to_string(),
            "system_config".to_string(),
            "user_management".to_string(),
            "read_only".to_string(),
        ]);

        self.role_permissions.insert("developer".to_string(), vec![
            "auto_label".to_string(),
            "codebase_analysis".to_string(),
            "read_only".to_string(),
        ]);

        self.role_permissions.insert("user".to_string(), vec![
            "read_only".to_string(),
        ]);
    }

    fn register_ssh_root_key(&mut self) {
        if let Ok(ssh_key) = self.load_ssh_public_key() {
            self.registered_keys.insert(ssh_key, "admin".to_string());
        }
    }

    fn load_ssh_public_key(&self) -> Result<String, std::io::Error> {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let ssh_path = format!("{}/.ssh/id_rsa.pub", home);
        Ok(std::fs::read_to_string(ssh_path)?.trim().to_string())
    }

    fn verify_signature(&self, request: &AuthRequest) -> bool {
        // Simplified verification - in production use proper crypto
        let expected = format!("{}:{}:{}", request.public_key, request.function_name, request.timestamp);
        request.signature == format!("sig_{}", expected.len())
    }
}

impl AuthPlugin for SshAuthPlugin {
    fn name(&self) -> &'static str {
        "ssh_auth"
    }

    fn authenticate(&self, request: &AuthRequest) -> Result<AuthResult, String> {
        // Check if key is registered
        let role = self.registered_keys.get(&request.public_key)
            .ok_or("Public key not registered")?;

        // Verify signature
        if !self.verify_signature(request) {
            return Ok(AuthResult::Denied {
                reason: "Invalid signature".to_string()
            });
        }

        // Check timestamp (5 minute window)
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        if now - request.timestamp > 300 {
            return Ok(AuthResult::Denied {
                reason: "Request timestamp expired".to_string()
            });
        }

        // Get permissions for role
        let permissions = self.role_permissions.get(role)
            .cloned()
            .unwrap_or_default();

        Ok(AuthResult::Granted {
            role: role.clone(),
            permissions,
        })
    }

    fn register_key(&mut self, public_key: String, role: String) -> Result<(), String> {
        if self.registered_keys.contains_key(&public_key) {
            return Err("Public key already registered".to_string());
        }
        self.registered_keys.insert(public_key, role);
        Ok(())
    }

    fn revoke_key(&mut self, public_key: &str) -> Result<(), String> {
        self.registered_keys.remove(public_key)
            .ok_or("Public key not found")?;
        Ok(())
    }

    fn list_keys(&self) -> Vec<(String, String)> {
        self.registered_keys.iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }
}
