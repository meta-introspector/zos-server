// Authentication Plugin Manager
use crate::traits::auth_plugin::{AuthPlugin, AuthRequest, AuthResult};
use crate::plugins::core_plugins::ssh_auth_plugin::SshAuthPlugin;
use std::sync::{Arc, Mutex};

pub struct AuthManager {
    plugins: Vec<Box<dyn AuthPlugin>>,
}

impl AuthManager {
    pub fn new() -> Self {
        let mut manager = Self {
            plugins: Vec::new(),
        };

        // Register default SSH auth plugin
        manager.register_plugin(Box::new(SshAuthPlugin::new()));
        manager
    }

    pub fn register_plugin(&mut self, plugin: Box<dyn AuthPlugin>) {
        self.plugins.push(plugin);
    }

    pub fn authenticate(&self, request: &AuthRequest) -> Result<AuthResult, String> {
        // Try each plugin until one grants access
        for plugin in &self.plugins {
            match plugin.authenticate(request) {
                Ok(AuthResult::Granted { role, permissions }) => {
                    return Ok(AuthResult::Granted { role, permissions });
                }
                Ok(AuthResult::Denied { .. }) => continue,
                Err(e) => return Err(e),
            }
        }

        Ok(AuthResult::Denied {
            reason: "No plugin granted access".to_string(),
        })
    }

    pub fn register_key(&mut self, public_key: String, role: String) -> Result<(), String> {
        // Register with first available plugin (SSH auth)
        if let Some(plugin) = self.plugins.first_mut() {
            plugin.register_key(public_key, role)
        } else {
            Err("No auth plugins available".to_string())
        }
    }

    pub fn revoke_key(&mut self, public_key: &str) -> Result<(), String> {
        for plugin in &mut self.plugins {
            if plugin.revoke_key(public_key).is_ok() {
                return Ok(());
            }
        }
        Err("Key not found in any plugin".to_string())
    }

    pub fn list_keys(&self) -> Vec<(String, String, String)> {
        let mut all_keys = Vec::new();
        for plugin in &self.plugins {
            for (key, role) in plugin.list_keys() {
                all_keys.push((key, role, plugin.name().to_string()));
            }
        }
        all_keys
    }
}
