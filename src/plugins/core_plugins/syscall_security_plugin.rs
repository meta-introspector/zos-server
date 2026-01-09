// Syscall Security Layer - Complexity-based Access Control
use crate::traits::auth_plugin::{AuthPlugin, AuthRequest, AuthResult};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Complexity levels for code and data access
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum ComplexityLevel {
    Safe = 0,      // Pure functions, immutable data
    Limited = 1,   // File I/O, network (rate limited)
    Privileged = 2, // System configuration
    Syscall = 3,   // Direct syscall access (root only)
}

/// Usage rate limits
#[derive(Debug, Clone)]
pub struct RateLimit {
    pub requests_per_minute: u32,
    pub data_bytes_per_minute: u64,
}

/// Syscall security filter
pub struct SyscallSecurityPlugin {
    user_complexity_limits: HashMap<String, ComplexityLevel>,
    user_rate_limits: HashMap<String, RateLimit>,
    usage_tracking: HashMap<String, UsageTracker>,
    forbidden_syscalls: Vec<String>,
}

#[derive(Debug, Clone)]
struct UsageTracker {
    requests_this_minute: u32,
    bytes_this_minute: u64,
    last_reset: u64,
}

impl SyscallSecurityPlugin {
    pub fn new() -> Self {
        let mut plugin = Self {
            user_complexity_limits: HashMap::new(),
            user_rate_limits: HashMap::new(),
            usage_tracking: HashMap::new(),
            forbidden_syscalls: Self::get_forbidden_syscalls(),
        };
        plugin.setup_default_limits();
        plugin
    }

    fn setup_default_limits(&mut self) {
        // Root can access everything
        self.user_complexity_limits.insert("root".to_string(), ComplexityLevel::Syscall);
        self.user_rate_limits.insert("root".to_string(), RateLimit {
            requests_per_minute: u32::MAX,
            data_bytes_per_minute: u64::MAX,
        });

        // Admin users get privileged access
        self.user_complexity_limits.insert("admin".to_string(), ComplexityLevel::Privileged);
        self.user_rate_limits.insert("admin".to_string(), RateLimit {
            requests_per_minute: 1000,
            data_bytes_per_minute: 100_000_000, // 100MB
        });

        // Regular users get limited access
        self.user_complexity_limits.insert("user".to_string(), ComplexityLevel::Limited);
        self.user_rate_limits.insert("user".to_string(), RateLimit {
            requests_per_minute: 100,
            data_bytes_per_minute: 10_000_000, // 10MB
        });
    }

    fn get_forbidden_syscalls() -> Vec<String> {
        vec![
            "execve".to_string(),
            "ptrace".to_string(),
            "mount".to_string(),
            "umount".to_string(),
            "chroot".to_string(),
            "setuid".to_string(),
            "setgid".to_string(),
            "mknod".to_string(),
            "reboot".to_string(),
            "syslog".to_string(),
        ]
    }

    /// Check if code contains forbidden syscalls or syn rewrites
    pub fn analyze_code_complexity(&self, code: &str, user_role: &str) -> Result<ComplexityLevel, String> {
        let user_limit = self.user_complexity_limits.get(user_role)
            .unwrap_or(&ComplexityLevel::Safe);

        // Check for direct syscall usage
        if self.contains_syscalls(code) {
            if *user_limit < ComplexityLevel::Syscall {
                return Err("Syscall access denied for non-root user".to_string());
            }
            return Ok(ComplexityLevel::Syscall);
        }

        // Check for syn AST manipulation (potential rewrite attacks)
        if self.contains_syn_rewrites(code) {
            return Err("Syn AST manipulation detected - potential security risk".to_string());
        }

        // Analyze complexity based on operations
        let detected_complexity = self.detect_complexity(code);

        if detected_complexity > *user_limit {
            return Err(format!("Code complexity {:?} exceeds user limit {:?}",
                detected_complexity, user_limit));
        }

        Ok(detected_complexity)
    }

    fn contains_syscalls(&self, code: &str) -> bool {
        // Check for direct syscall patterns
        let syscall_patterns = [
            "syscall(",
            "libc::",
            "unsafe {",
            "#[no_mangle]",
            "extern \"C\"",
        ];

        for pattern in &syscall_patterns {
            if code.contains(pattern) {
                return true;
            }
        }

        // Check for specific forbidden syscalls
        for syscall in &self.forbidden_syscalls {
            if code.contains(syscall) {
                return true;
            }
        }

        false
    }

    fn contains_syn_rewrites(&self, code: &str) -> bool {
        // Detect syn-based AST manipulation
        let syn_patterns = [
            "syn::",
            "parse_quote!",
            "quote!",
            "TokenStream",
            "proc_macro",
            "Attribute::parse",
            "Item::parse",
        ];

        for pattern in &syn_patterns {
            if code.contains(pattern) {
                return true;
            }
        }

        false
    }

    fn detect_complexity(&self, code: &str) -> ComplexityLevel {
        // File system operations
        if code.contains("std::fs") || code.contains("File::") {
            return ComplexityLevel::Limited;
        }

        // Network operations
        if code.contains("std::net") || code.contains("TcpStream") {
            return ComplexityLevel::Limited;
        }

        // Process operations
        if code.contains("std::process") || code.contains("Command::") {
            return ComplexityLevel::Privileged;
        }

        // Environment manipulation
        if code.contains("std::env::set") {
            return ComplexityLevel::Privileged;
        }

        ComplexityLevel::Safe
    }

    /// Check and update rate limits
    pub fn check_rate_limit(&mut self, user_id: &str, data_size: u64) -> Result<(), String> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let current_minute = now / 60;

        let rate_limit = self.user_rate_limits.get(user_id)
            .ok_or("No rate limit configured for user")?;

        let usage = self.usage_tracking.entry(user_id.to_string())
            .or_insert(UsageTracker {
                requests_this_minute: 0,
                bytes_this_minute: 0,
                last_reset: current_minute,
            });

        // Reset counters if we're in a new minute
        if usage.last_reset < current_minute {
            usage.requests_this_minute = 0;
            usage.bytes_this_minute = 0;
            usage.last_reset = current_minute;
        }

        // Check limits
        if usage.requests_this_minute >= rate_limit.requests_per_minute {
            return Err("Request rate limit exceeded".to_string());
        }

        if usage.bytes_this_minute + data_size > rate_limit.data_bytes_per_minute {
            return Err("Data rate limit exceeded".to_string());
        }

        // Update usage
        usage.requests_this_minute += 1;
        usage.bytes_this_minute += data_size;

        Ok(())
    }
}

impl AuthPlugin for SyscallSecurityPlugin {
    fn name(&self) -> &'static str {
        "syscall_security"
    }

    fn authenticate(&self, request: &AuthRequest) -> Result<AuthResult, String> {
        // This plugin focuses on code analysis rather than key authentication
        // It should be used in conjunction with other auth plugins
        Ok(AuthResult::Granted {
            role: "security_checked".to_string(),
            permissions: vec!["code_analysis".to_string()],
        })
    }

    fn register_key(&mut self, _public_key: String, _role: String) -> Result<(), String> {
        Err("This plugin does not handle key registration".to_string())
    }

    fn revoke_key(&mut self, _public_key: &str) -> Result<(), String> {
        Err("This plugin does not handle key revocation".to_string())
    }

    fn list_keys(&self) -> Vec<(String, String)> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syscall_detection() {
        let plugin = SyscallSecurityPlugin::new();

        let safe_code = "fn add(a: i32, b: i32) -> i32 { a + b }";
        assert_eq!(plugin.detect_complexity(safe_code), ComplexityLevel::Safe);

        let file_code = "use std::fs::File;";
        assert_eq!(plugin.detect_complexity(file_code), ComplexityLevel::Limited);

        let syscall_code = "unsafe { syscall(1, 2, 3) }";
        assert!(plugin.contains_syscalls(syscall_code));
    }

    #[test]
    fn test_syn_rewrite_detection() {
        let plugin = SyscallSecurityPlugin::new();

        let normal_code = "fn test() {}";
        assert!(!plugin.contains_syn_rewrites(normal_code));

        let syn_code = "use syn::parse_quote;";
        assert!(plugin.contains_syn_rewrites(syn_code));
    }

    #[test]
    fn test_complexity_limits() {
        let plugin = SyscallSecurityPlugin::new();

        let syscall_code = "unsafe { syscall(1) }";
        assert!(plugin.analyze_code_complexity(syscall_code, "user").is_err());
        assert!(plugin.analyze_code_complexity(syscall_code, "root").is_ok());
    }
}
