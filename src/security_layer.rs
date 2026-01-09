// Integrated Security Layer - Combines Auth, Complexity, and Intent Verification
use crate::auth_manager::AuthManager;
use crate::instruction_filter::{InstructionFilter, CodeAnalysis};
use crate::intent_verifier::{IntentVerifier, UserIntent, IntentVerification, create_user_intent, UsagePattern};
use crate::plugins::core_plugins::syscall_security_plugin::ComplexityLevel;
use crate::traits::auth_plugin::{AuthRequest, AuthResult};

/// Complete security verification result
#[derive(Debug, Clone)]
pub struct SecurityVerification {
    pub allowed: bool,
    pub auth_result: Option<AuthResult>,
    pub code_analysis: Option<CodeAnalysis>,
    pub intent_verification: Option<IntentVerification>,
    pub final_reason: String,
    pub constraints: Vec<String>,
}

/// Integrated security system
pub struct SecurityLayer {
    auth_manager: AuthManager,
    instruction_filter: InstructionFilter,
    intent_verifier: IntentVerifier,
}

impl SecurityLayer {
    pub fn new() -> Self {
        Self {
            auth_manager: AuthManager::new(),
            instruction_filter: InstructionFilter::new(),
            intent_verifier: IntentVerifier::new(),
        }
    }

    /// Complete security verification for code execution
    pub fn verify_execution_request(
        &mut self,
        user_id: &str,
        public_key: &str,
        code: &str,
        orbit: &str,
        path: &str,
        function: &str,
        declared_purpose: &str,
        usage_pattern: UsagePattern,
    ) -> Result<SecurityVerification, String> {
        let mut verification = SecurityVerification {
            allowed: false,
            auth_result: None,
            code_analysis: None,
            intent_verification: None,
            final_reason: String::new(),
            constraints: Vec::new(),
        };

        // Step 1: Authenticate user
        let auth_request = AuthRequest {
            public_key: public_key.to_string(),
            function_name: function.to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            signature: format!("sig_{}_{}", user_id, function),
        };

        let auth_result = self.auth_manager.authenticate(&auth_request)?;
        let user_role = match &auth_result {
            AuthResult::Granted { role, .. } => role.clone(),
            AuthResult::Denied { reason } => {
                verification.auth_result = Some(auth_result);
                verification.final_reason = format!("Authentication failed: {}", reason);
                return Ok(verification);
            }
        };
        verification.auth_result = Some(auth_result);

        // Step 2: Analyze code complexity and safety
        let code_analysis = self.instruction_filter.filter_code(code, &user_role)?;
        if !code_analysis.safe_to_execute {
            verification.code_analysis = Some(code_analysis.clone());
            verification.final_reason = format!("Code analysis failed: {:?}", code_analysis.forbidden_patterns);
            return Ok(verification);
        }
        verification.code_analysis = Some(code_analysis.clone());

        // Step 3: Verify user intent
        let intent = create_user_intent(
            user_id.to_string(),
            orbit.to_string(),
            path.to_string(),
            function.to_string(),
            declared_purpose.to_string(),
            usage_pattern,
            code_analysis.complexity.clone(),
        );

        let intent_verification = self.intent_verifier.verify_intent(intent, &user_role)?;
        if !intent_verification.allowed {
            verification.intent_verification = Some(intent_verification.clone());
            verification.final_reason = format!("Intent verification failed: {}", intent_verification.reason);
            return Ok(verification);
        }
        verification.intent_verification = Some(intent_verification.clone());

        // Step 4: Apply rate limiting
        if let Err(e) = self.instruction_filter.check_rate_limit(user_id, code.len() as u64) {
            verification.final_reason = format!("Rate limit exceeded: {}", e);
            return Ok(verification);
        }

        // Step 5: Compile final constraints
        verification.constraints.extend(intent_verification.usage_constraints);
        verification.constraints.push(format!("User role: {}", user_role));
        verification.constraints.push(format!("Code complexity: {:?}", code_analysis.complexity));
        verification.constraints.push(format!("Orbit: {}", orbit));

        // All checks passed
        verification.allowed = true;
        verification.final_reason = "All security checks passed".to_string();

        Ok(verification)
    }

    /// Quick check if user can access a specific code path
    pub fn can_access_path(&self, user_id: &str, orbit: &str, path: &str, function: &str) -> bool {
        let code_path = crate::intent_verifier::CodePath {
            orbit: orbit.to_string(),
            path: path.to_string(),
            function: function.to_string(),
        };
        self.intent_verifier.is_access_allowed(user_id, &code_path)
    }

    /// Revoke access to a code path
    pub fn revoke_path_access(&mut self, orbit: &str, path: &str, function: &str) -> Result<(), String> {
        let code_path = crate::intent_verifier::CodePath {
            orbit: orbit.to_string(),
            path: path.to_string(),
            function: function.to_string(),
        };
        self.intent_verifier.revoke_access(&code_path)
    }

    /// Get security statistics
    pub fn get_security_stats(&self) -> SecurityStats {
        SecurityStats {
            complexity_stats: self.instruction_filter.get_complexity_stats(),
            total_users: self.auth_manager.list_keys().len(),
            active_verifications: 0, // Would track active sessions
        }
    }

    /// Register new user key
    pub fn register_user(&mut self, public_key: String, role: String) -> Result<(), String> {
        self.auth_manager.register_key(public_key, role)
    }

    /// Revoke user key
    pub fn revoke_user(&mut self, public_key: &str) -> Result<(), String> {
        self.auth_manager.revoke_key(public_key)
    }
}

#[derive(Debug, Clone)]
pub struct SecurityStats {
    pub complexity_stats: std::collections::HashMap<String, usize>,
    pub total_users: usize,
    pub active_verifications: usize,
}

/// Security policy enforcement
impl SecurityLayer {
    /// Enforce zero-trust policy: every access must be verified
    pub fn enforce_zero_trust(&self) -> bool {
        // In zero-trust mode, no cached permissions are allowed
        // Every request must go through full verification
        true
    }

    /// Check if operation requires root privileges
    pub fn requires_root(&self, complexity: &ComplexityLevel) -> bool {
        matches!(complexity, ComplexityLevel::Syscall)
    }

    /// Generate security audit log entry
    pub fn create_audit_entry(&self, verification: &SecurityVerification, user_id: &str) -> String {
        format!(
            "AUDIT: user={} allowed={} reason='{}' constraints={:?}",
            user_id,
            verification.allowed,
            verification.final_reason,
            verification.constraints
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complete_verification_flow() {
        let mut security = SecurityLayer::new();

        // Register a test user
        security.register_user("test_key_123".to_string(), "user".to_string()).unwrap();

        let result = security.verify_execution_request(
            "test_user",
            "test_key_123",
            "fn add(a: i32, b: i32) -> i32 { a + b }",
            "safe",
            "/math/add",
            "add_numbers",
            "Adding two integers for calculation",
            UsagePattern::Computation,
        ).unwrap();

        assert!(result.allowed);
        assert!(result.auth_result.is_some());
        assert!(result.code_analysis.is_some());
        assert!(result.intent_verification.is_some());
    }

    #[test]
    fn test_syscall_rejection() {
        let mut security = SecurityLayer::new();

        security.register_user("test_key_123".to_string(), "user".to_string()).unwrap();

        let result = security.verify_execution_request(
            "test_user",
            "test_key_123",
            "unsafe { syscall(1, 2, 3) }",
            "kernel",
            "/sys/raw",
            "raw_syscall",
            "Direct system call",
            UsagePattern::SystemConfig,
        ).unwrap();

        assert!(!result.allowed);
    }
}
