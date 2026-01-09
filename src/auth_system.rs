// Public key authentication and role-based access control
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::auth_manager::AuthManager;
use crate::traits::auth_plugin::{AuthRequest, AuthResult};

/// Public key for authentication
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PublicKey(pub String);

/// User roles with different permission levels
#[derive(Debug, Clone, PartialEq)]
pub enum Role {
    Admin,
    Developer,
    Auditor,
    User,
}

/// Function permissions required for access
#[derive(Debug, Clone, PartialEq)]
pub enum Permission {
    AutoLabel,
    CodebaseAnalysis,
    SystemConfig,
    UserManagement,
    ReadOnly,
}

/// Signed request with public key authentication
#[derive(Debug)]
pub struct SignedRequest {
    pub public_key: PublicKey,
    pub function_name: String,
    pub timestamp: u64,
    pub signature: String,
}

/// Authentication and authorization system (now using plugin architecture)
#[derive(Debug)]
pub struct AuthSystem {
    auth_manager: AuthManager,
}

impl AuthSystem {
    pub fn new() -> Self {
        Self {
            auth_manager: AuthManager::new(),
        }
    }

    /// Register a public key with a role
    pub fn register_key(&mut self, public_key: PublicKey, role: Role) -> Result<(), String> {
        let role_str = match role {
            Role::Admin => "admin",
            Role::Developer => "developer",
            Role::Auditor => "auditor",
            Role::User => "user",
        }.to_string();

        self.auth_manager.register_key(public_key.0, role_str)
    }

    /// Authenticate and authorize a signed request
    pub fn authenticate(&self, request: &SignedRequest) -> Result<(), String> {
        let auth_request = AuthRequest {
            public_key: request.public_key.0.clone(),
            function_name: request.function_name.clone(),
            timestamp: request.timestamp,
            signature: request.signature.clone(),
        };

        match self.auth_manager.authenticate(&auth_request)? {
            AuthResult::Granted { .. } => Ok(()),
            AuthResult::Denied { reason } => Err(reason),
        }
    }

    /// Revoke a public key
    pub fn revoke_key(&mut self, public_key: &PublicKey) -> Result<(), String> {
        self.auth_manager.revoke_key(&public_key.0)
    }

    /// List all registered keys and their roles
    pub fn list_keys(&self) -> Vec<(PublicKey, Role)> {
        self.auth_manager.list_keys()
            .into_iter()
            .map(|(key, role, _plugin)| {
                let role_enum = match role.as_str() {
                    "admin" => Role::Admin,
                    "developer" => Role::Developer,
                    "auditor" => Role::Auditor,
                    _ => Role::User,
                };
                (PublicKey(key), role_enum)
            })
            .collect()
    }

    /// Check if request is from root SSH key
    pub fn is_root_request(&self, public_key: &PublicKey) -> bool {
        if let Ok(ssh_key) = self.load_ssh_public_key() {
            public_key.0 == ssh_key
        } else {
            false
        }
    }

    /// Load SSH public key from ~/.ssh/id_rsa.pub
    fn load_ssh_public_key(&self) -> Result<String, std::io::Error> {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let ssh_path = format!("{}/.ssh/id_rsa.pub", home);
        std::fs::read_to_string(ssh_path)?.trim().to_string().parse().map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid SSH key format")
        })
    }
}

/// Helper to create SSH-signed request using private key
pub fn create_ssh_signed_request(function_name: String) -> Result<SignedRequest, Box<dyn std::error::Error>> {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let pub_path = format!("{}/.ssh/id_rsa.pub", home);
    let priv_path = format!("{}/.ssh/id_rsa", home);

    // Load public key
    let public_key_content = std::fs::read_to_string(pub_path)?.trim().to_string();
    let public_key = PublicKey(public_key_content);

    // Create message to sign
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs();
    let message = format!("{}:{}:{}", public_key.0, function_name, timestamp);

    // Sign with private key using ssh-keygen
    let signature = sign_with_ssh_key(&priv_path, &message)?;

    Ok(SignedRequest {
        public_key,
        function_name,
        timestamp,
        signature,
    })
}

/// Sign message using SSH private key via ssh-keygen
fn sign_with_ssh_key(private_key_path: &str, message: &str) -> Result<String, Box<dyn std::error::Error>> {
    use std::process::Command;
    use std::io::Write;

    // Create temporary file for message
    let temp_file = format!("/tmp/zos_sign_{}", timestamp_nanos());
    std::fs::write(&temp_file, message)?;

    // Sign using ssh-keygen
    let output = Command::new("ssh-keygen")
        .args(&["-Y", "sign", "-f", private_key_path, "-n", "zos-server", &temp_file])
        .output()?;

    // Clean up temp file
    let _ = std::fs::remove_file(&temp_file);

    if !output.status.success() {
        return Err(format!("SSH signing failed: {}",
            String::from_utf8_lossy(&output.stderr)).into());
    }

    // Read signature file
    let sig_file = format!("{}.sig", temp_file);
    let signature = std::fs::read_to_string(&sig_file)?;
    let _ = std::fs::remove_file(&sig_file);

    Ok(signature.trim().to_string())
}

fn timestamp_nanos() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
}
pub fn create_signed_request(
    public_key: PublicKey,
    function_name: String,
) -> SignedRequest {
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let signature = format!("sig_{}",
        format!("{}:{}:{}", public_key.0, function_name, timestamp).len());

    SignedRequest {
        public_key,
        function_name,
        timestamp,
        signature,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_registration() {
        let mut auth = AuthSystem::new();
        let key = PublicKey("admin_key_123".to_string());

        assert!(auth.register_key(key.clone(), Role::Admin).is_ok());
        assert!(auth.register_key(key, Role::User).is_err()); // Duplicate
    }

    #[test]
    fn test_authentication() {
        let mut auth = AuthSystem::new();
        let admin_key = PublicKey("admin_key_123".to_string());
        let user_key = PublicKey("user_key_456".to_string());

        auth.register_key(admin_key.clone(), Role::Admin).unwrap();
        auth.register_key(user_key.clone(), Role::User).unwrap();

        // Admin can access auto_label
        let admin_request = create_signed_request(admin_key, "auto_label".to_string());
        assert!(auth.authenticate(&admin_request).is_ok());

        // User cannot access auto_label
        let user_request = create_signed_request(user_key, "auto_label".to_string());
        assert!(auth.authenticate(&user_request).is_err());
    }

    #[test]
    fn test_key_revocation() {
        let mut auth = AuthSystem::new();
        let key = PublicKey("test_key".to_string());

        auth.register_key(key.clone(), Role::Developer).unwrap();
        assert!(auth.revoke_key(&key).is_ok());
        assert!(auth.revoke_key(&key).is_err()); // Already revoked
    }
}
