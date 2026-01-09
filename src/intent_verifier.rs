// Intent Verification System - Prove Safe and Intended Usage
use crate::plugins::core_plugins::syscall_security_plugin::ComplexityLevel;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Code path within an orbit
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CodePath {
    pub orbit: String,
    pub path: String,
    pub function: String,
}

/// User intent declaration
#[derive(Debug, Clone)]
pub struct UserIntent {
    pub user_id: String,
    pub code_path: CodePath,
    pub declared_purpose: String,
    pub expected_complexity: ComplexityLevel,
    pub usage_pattern: UsagePattern,
    pub timestamp: u64,
    pub proof_signature: String,
}

/// Allowed usage patterns
#[derive(Debug, Clone, PartialEq)]
pub enum UsagePattern {
    ReadOnly,           // Only reading data
    Computation,        // Pure computation
    DataTransform,      // Transform data without side effects
    FileAccess,         // Controlled file operations
    NetworkAccess,      // Controlled network operations
    SystemConfig,       // System configuration (admin only)
}

/// Intent verification result
#[derive(Debug, Clone)]
pub struct IntentVerification {
    pub allowed: bool,
    pub reason: String,
    pub required_proofs: Vec<String>,
    pub usage_constraints: Vec<String>,
}

/// Intent verification engine
pub struct IntentVerifier {
    orbit_policies: HashMap<String, OrbitPolicy>,
    user_intents: HashMap<String, Vec<UserIntent>>,
    verified_paths: HashMap<CodePath, VerifiedAccess>,
}

#[derive(Debug, Clone)]
struct OrbitPolicy {
    allowed_patterns: Vec<UsagePattern>,
    max_complexity: ComplexityLevel,
    requires_proof: bool,
    rate_limit_per_hour: u32,
}

#[derive(Debug, Clone)]
struct VerifiedAccess {
    user_id: String,
    verified_at: u64,
    usage_pattern: UsagePattern,
    access_count: u32,
    last_access: u64,
}

impl IntentVerifier {
    pub fn new() -> Self {
        let mut verifier = Self {
            orbit_policies: HashMap::new(),
            user_intents: HashMap::new(),
            verified_paths: HashMap::new(),
        };
        verifier.setup_default_policies();
        verifier
    }

    fn setup_default_policies(&mut self) {
        // Safe orbit - minimal restrictions
        self.orbit_policies.insert("safe".to_string(), OrbitPolicy {
            allowed_patterns: vec![
                UsagePattern::ReadOnly,
                UsagePattern::Computation,
                UsagePattern::DataTransform,
            ],
            max_complexity: ComplexityLevel::Safe,
            requires_proof: false,
            rate_limit_per_hour: 1000,
        });

        // System orbit - requires proof and admin access
        self.orbit_policies.insert("system".to_string(), OrbitPolicy {
            allowed_patterns: vec![
                UsagePattern::SystemConfig,
                UsagePattern::FileAccess,
            ],
            max_complexity: ComplexityLevel::Privileged,
            requires_proof: true,
            rate_limit_per_hour: 10,
        });

        // Network orbit - controlled network access
        self.orbit_policies.insert("network".to_string(), OrbitPolicy {
            allowed_patterns: vec![
                UsagePattern::NetworkAccess,
                UsagePattern::DataTransform,
            ],
            max_complexity: ComplexityLevel::Limited,
            requires_proof: true,
            rate_limit_per_hour: 100,
        });

        // Kernel orbit - syscall access (root only)
        self.orbit_policies.insert("kernel".to_string(), OrbitPolicy {
            allowed_patterns: vec![],  // No patterns allowed by default
            max_complexity: ComplexityLevel::Syscall,
            requires_proof: true,
            rate_limit_per_hour: 1,
        });
    }

    /// Verify user intent to access a code path
    pub fn verify_intent(&mut self, intent: UserIntent, user_role: &str) -> Result<IntentVerification, String> {
        let policy = self.orbit_policies.get(&intent.code_path.orbit)
            .ok_or("Unknown orbit")?;

        let mut verification = IntentVerification {
            allowed: false,
            reason: String::new(),
            required_proofs: Vec::new(),
            usage_constraints: Vec::new(),
        };

        // Check if usage pattern is allowed in this orbit
        if !policy.allowed_patterns.contains(&intent.usage_pattern) {
            verification.reason = format!("Usage pattern {:?} not allowed in orbit {}",
                intent.usage_pattern, intent.code_path.orbit);
            return Ok(verification);
        }

        // Check complexity limits
        if intent.expected_complexity > policy.max_complexity {
            verification.reason = format!("Expected complexity {:?} exceeds orbit limit {:?}",
                intent.expected_complexity, policy.max_complexity);
            return Ok(verification);
        }

        // Check user role permissions
        if !self.check_role_permissions(user_role, &intent.code_path.orbit, &intent.usage_pattern) {
            verification.reason = format!("Role {} lacks permission for {:?} in orbit {}",
                user_role, intent.usage_pattern, intent.code_path.orbit);
            return Ok(verification);
        }

        // Check rate limits
        if !self.check_rate_limits(&intent.user_id, &intent.code_path.orbit, policy) {
            verification.reason = "Rate limit exceeded for this orbit".to_string();
            return Ok(verification);
        }

        // Verify proof if required
        if policy.requires_proof {
            match self.verify_intent_proof(&intent) {
                Ok(proofs) => verification.required_proofs = proofs,
                Err(e) => {
                    verification.reason = format!("Proof verification failed: {}", e);
                    return Ok(verification);
                }
            }
        }

        // Generate usage constraints
        verification.usage_constraints = self.generate_constraints(&intent, policy);

        // Record verified intent
        self.record_verified_access(&intent);

        verification.allowed = true;
        verification.reason = "Intent verified successfully".to_string();
        Ok(verification)
    }

    fn check_role_permissions(&self, role: &str, orbit: &str, pattern: &UsagePattern) -> bool {
        match (role, orbit, pattern) {
            ("root", _, _) => true,
            ("admin", "kernel", _) => false,  // Even admin can't access kernel
            ("admin", _, UsagePattern::SystemConfig) => true,
            ("admin", _, _) => true,
            ("developer", "system", _) => false,
            ("developer", _, UsagePattern::SystemConfig) => false,
            ("developer", _, _) => true,
            ("user", "safe", UsagePattern::ReadOnly | UsagePattern::Computation) => true,
            ("user", _, _) => false,
            _ => false,
        }
    }

    fn check_rate_limits(&self, user_id: &str, orbit: &str, policy: &OrbitPolicy) -> bool {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let current_hour = now / 3600;

        let access_count = self.verified_paths.values()
            .filter(|access| {
                access.user_id == user_id &&
                access.last_access / 3600 == current_hour
            })
            .count() as u32;

        access_count < policy.rate_limit_per_hour
    }

    fn verify_intent_proof(&self, intent: &UserIntent) -> Result<Vec<String>, String> {
        let mut required_proofs = Vec::new();

        // Verify signature matches intent
        let expected_sig = self.generate_intent_signature(intent);
        if intent.proof_signature != expected_sig {
            return Err("Invalid intent signature".to_string());
        }

        // Check timestamp freshness (5 minute window)
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        if now - intent.timestamp > 300 {
            return Err("Intent timestamp expired".to_string());
        }

        // Require specific proofs based on usage pattern
        match intent.usage_pattern {
            UsagePattern::SystemConfig => {
                required_proofs.push("Administrative authorization required".to_string());
            }
            UsagePattern::NetworkAccess => {
                required_proofs.push("Network access justification required".to_string());
            }
            UsagePattern::FileAccess => {
                required_proofs.push("File access scope must be specified".to_string());
            }
            _ => {}
        }

        Ok(required_proofs)
    }

    fn generate_intent_signature(&self, intent: &UserIntent) -> String {
        // Simplified signature generation
        format!("intent_{}_{}_{}_{}",
            intent.user_id,
            intent.code_path.orbit,
            intent.declared_purpose.len(),
            intent.timestamp)
    }

    fn generate_constraints(&self, intent: &UserIntent, policy: &OrbitPolicy) -> Vec<String> {
        let mut constraints = Vec::new();

        constraints.push(format!("Maximum complexity: {:?}", policy.max_complexity));
        constraints.push(format!("Usage pattern: {:?}", intent.usage_pattern));
        constraints.push(format!("Rate limit: {} accesses per hour", policy.rate_limit_per_hour));

        match intent.usage_pattern {
            UsagePattern::FileAccess => {
                constraints.push("File access limited to declared paths".to_string());
            }
            UsagePattern::NetworkAccess => {
                constraints.push("Network access logged and monitored".to_string());
            }
            UsagePattern::SystemConfig => {
                constraints.push("All system changes must be auditable".to_string());
            }
            _ => {}
        }

        constraints
    }

    fn record_verified_access(&mut self, intent: &UserIntent) {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        let access = VerifiedAccess {
            user_id: intent.user_id.clone(),
            verified_at: now,
            usage_pattern: intent.usage_pattern.clone(),
            access_count: 1,
            last_access: now,
        };

        self.verified_paths.insert(intent.code_path.clone(), access);

        // Store user intent history
        self.user_intents.entry(intent.user_id.clone())
            .or_insert_with(Vec::new)
            .push(intent.clone());
    }

    /// Check if a code path access is currently allowed
    pub fn is_access_allowed(&self, user_id: &str, code_path: &CodePath) -> bool {
        if let Some(access) = self.verified_paths.get(code_path) {
            access.user_id == user_id &&
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() - access.verified_at < 3600 // 1 hour validity
        } else {
            false
        }
    }

    /// Get user's intent history
    pub fn get_user_intent_history(&self, user_id: &str) -> Vec<UserIntent> {
        self.user_intents.get(user_id).cloned().unwrap_or_default()
    }

    /// Revoke access to a code path
    pub fn revoke_access(&mut self, code_path: &CodePath) -> Result<(), String> {
        self.verified_paths.remove(code_path)
            .ok_or("Access not found")?;
        Ok(())
    }
}

/// Helper to create user intent
pub fn create_user_intent(
    user_id: String,
    orbit: String,
    path: String,
    function: String,
    purpose: String,
    pattern: UsagePattern,
    complexity: ComplexityLevel,
) -> UserIntent {
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let proof_signature = format!("intent_{}_{}_{}_{}", user_id, orbit, purpose.len(), timestamp);

    UserIntent {
        user_id,
        code_path: CodePath { orbit, path, function },
        declared_purpose: purpose,
        expected_complexity: complexity,
        usage_pattern: pattern,
        timestamp,
        proof_signature,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_orbit_access() {
        let mut verifier = IntentVerifier::new();

        let intent = create_user_intent(
            "user123".to_string(),
            "safe".to_string(),
            "/math/add".to_string(),
            "add_numbers".to_string(),
            "Adding two integers".to_string(),
            UsagePattern::Computation,
            ComplexityLevel::Safe,
        );

        let result = verifier.verify_intent(intent, "user").unwrap();
        assert!(result.allowed);
    }

    #[test]
    fn test_kernel_orbit_denied() {
        let mut verifier = IntentVerifier::new();

        let intent = create_user_intent(
            "user123".to_string(),
            "kernel".to_string(),
            "/sys/syscall".to_string(),
            "raw_syscall".to_string(),
            "Direct system call".to_string(),
            UsagePattern::SystemConfig,
            ComplexityLevel::Syscall,
        );

        let result = verifier.verify_intent(intent, "user").unwrap();
        assert!(!result.allowed);
    }

    #[test]
    fn test_admin_system_access() {
        let mut verifier = IntentVerifier::new();

        let intent = create_user_intent(
            "admin123".to_string(),
            "system".to_string(),
            "/config/update".to_string(),
            "update_config".to_string(),
            "Updating system configuration".to_string(),
            UsagePattern::SystemConfig,
            ComplexityLevel::Privileged,
        );

        let result = verifier.verify_intent(intent, "admin").unwrap();
        assert!(result.allowed);
        assert!(!result.required_proofs.is_empty());
    }
}
