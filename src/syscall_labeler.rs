// Syscall Labeling and Auto-Security System
use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

/// Syscall definition from libssl
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyscallLabel {
    pub name: String,
    pub number: u64,
    pub risk_level: RiskLevel,
    pub required_caps: Vec<String>,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RiskLevel {
    Safe,      // read, write to owned files
    Medium,    // network, file creation
    High,      // process control, setuid
    Critical,  // mount, ptrace, reboot
}

/// Code path that leads to a syscall
#[derive(Debug, Clone)]
pub struct SyscallPath {
    pub crate_name: String,
    pub function_chain: Vec<String>,
    pub syscall: String,
    pub risk_level: RiskLevel,
}

/// Syscall labeling engine
pub struct SyscallLabeler {
    syscall_db: HashMap<String, SyscallLabel>,
    crate_paths: HashMap<String, Vec<SyscallPath>>,
    auto_policies: HashMap<String, SecurityPolicy>,
}

#[derive(Debug, Clone)]
pub struct SecurityPolicy {
    pub allowed_syscalls: HashSet<String>,
    pub max_risk_level: RiskLevel,
    pub requires_audit: bool,
}

impl SyscallLabeler {
    pub fn new() -> Self {
        let mut labeler = Self {
            syscall_db: HashMap::new(),
            crate_paths: HashMap::new(),
            auto_policies: HashMap::new(),
        };
        labeler.load_libssl_definitions();
        labeler
    }

    fn load_libssl_definitions(&mut self) {
        // Core syscalls with risk levels
        let syscalls = vec![
            ("read", 0, RiskLevel::Safe, vec![], "Read from file descriptor"),
            ("write", 1, RiskLevel::Safe, vec![], "Write to file descriptor"),
            ("open", 2, RiskLevel::Medium, vec![], "Open file"),
            ("close", 3, RiskLevel::Safe, vec![], "Close file descriptor"),
            ("execve", 59, RiskLevel::Critical, vec!["CAP_SYS_ADMIN".to_string()], "Execute program"),
            ("ptrace", 101, RiskLevel::Critical, vec!["CAP_SYS_PTRACE".to_string()], "Process trace"),
            ("mount", 165, RiskLevel::Critical, vec!["CAP_SYS_ADMIN".to_string()], "Mount filesystem"),
            ("setuid", 105, RiskLevel::High, vec!["CAP_SETUID".to_string()], "Set user ID"),
            ("socket", 41, RiskLevel::Medium, vec![], "Create socket"),
            ("bind", 49, RiskLevel::Medium, vec![], "Bind socket"),
            ("listen", 50, RiskLevel::Medium, vec![], "Listen on socket"),
        ];

        for (name, num, risk, caps, desc) in syscalls {
            self.syscall_db.insert(name.to_string(), SyscallLabel {
                name: name.to_string(),
                number: num,
                risk_level: risk,
                required_caps: caps,
                description: desc.to_string(),
            });
        }
    }

    /// Analyze a crate's syscall usage
    pub fn analyze_crate(&mut self, crate_name: &str, source_code: &str) -> Vec<SyscallPath> {
        let mut paths = Vec::new();

        // Find direct syscall usage
        for (syscall_name, label) in &self.syscall_db {
            if source_code.contains(&format!("syscall({}", label.number)) ||
               source_code.contains(&format!("{}(", syscall_name)) {
                paths.push(SyscallPath {
                    crate_name: crate_name.to_string(),
                    function_chain: vec!["direct".to_string()],
                    syscall: syscall_name.clone(),
                    risk_level: label.risk_level.clone(),
                });
            }
        }

        // Find libc usage
        if source_code.contains("libc::") {
            for line in source_code.lines() {
                if let Some(syscall) = self.extract_libc_call(line) {
                    if let Some(label) = self.syscall_db.get(&syscall) {
                        paths.push(SyscallPath {
                            crate_name: crate_name.to_string(),
                            function_chain: vec!["libc".to_string()],
                            syscall,
                            risk_level: label.risk_level.clone(),
                        });
                    }
                }
            }
        }

        self.crate_paths.insert(crate_name.to_string(), paths.clone());
        paths
    }

    fn extract_libc_call(&self, line: &str) -> Option<String> {
        if let Some(start) = line.find("libc::") {
            let rest = &line[start + 6..];
            if let Some(end) = rest.find('(') {
                return Some(rest[..end].to_string());
            }
        }
        None
    }

    /// Generate automatic security policy for a crate
    pub fn generate_auto_policy(&mut self, crate_name: &str) -> SecurityPolicy {
        let paths = self.crate_paths.get(crate_name).cloned().unwrap_or_default();

        let mut allowed_syscalls = HashSet::new();
        let mut max_risk = RiskLevel::Safe;
        let mut requires_audit = false;

        for path in &paths {
            allowed_syscalls.insert(path.syscall.clone());

            if path.risk_level == RiskLevel::Critical {
                max_risk = RiskLevel::Critical;
                requires_audit = true;
            } else if path.risk_level == RiskLevel::High && max_risk != RiskLevel::Critical {
                max_risk = RiskLevel::High;
                requires_audit = true;
            } else if path.risk_level == RiskLevel::Medium && max_risk == RiskLevel::Safe {
                max_risk = RiskLevel::Medium;
            }
        }

        let policy = SecurityPolicy {
            allowed_syscalls,
            max_risk_level: max_risk,
            requires_audit,
        };

        self.auto_policies.insert(crate_name.to_string(), policy.clone());
        policy
    }

    /// Check if crate usage is allowed for user role
    pub fn check_crate_access(&self, crate_name: &str, user_role: &str) -> Result<(), String> {
        let policy = self.auto_policies.get(crate_name)
            .ok_or("Crate not analyzed")?;

        match (user_role, &policy.max_risk_level) {
            ("root", _) => Ok(()),
            ("admin", RiskLevel::Critical) => Err("Critical syscalls require root".to_string()),
            ("admin", _) => Ok(()),
            ("developer", RiskLevel::High | RiskLevel::Critical) =>
                Err("High/Critical syscalls require admin+".to_string()),
            ("developer", _) => Ok(()),
            ("user", RiskLevel::Medium | RiskLevel::High | RiskLevel::Critical) =>
                Err("Only safe syscalls allowed for users".to_string()),
            ("user", RiskLevel::Safe) => Ok(()),
            _ => Err("Unknown role".to_string()),
        }
    }

    /// Get syscall report for crate
    pub fn get_crate_report(&self, crate_name: &str) -> Option<CrateReport> {
        let paths = self.crate_paths.get(crate_name)?;
        let policy = self.auto_policies.get(crate_name)?;

        Some(CrateReport {
            crate_name: crate_name.to_string(),
            syscall_count: paths.len(),
            risk_level: policy.max_risk_level.clone(),
            requires_audit: policy.requires_audit,
            syscalls: paths.iter().map(|p| p.syscall.clone()).collect(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct CrateReport {
    pub crate_name: String,
    pub syscall_count: usize,
    pub risk_level: RiskLevel,
    pub requires_audit: bool,
    pub syscalls: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syscall_detection() {
        let mut labeler = SyscallLabeler::new();

        let code = r#"
            use libc;
            unsafe { libc::open(path.as_ptr(), flags) }
            unsafe { syscall(59, prog, args, env) }
        "#;

        let paths = labeler.analyze_crate("test_crate", code);
        assert!(!paths.is_empty());
        assert!(paths.iter().any(|p| p.syscall == "open"));
    }

    #[test]
    fn test_auto_policy() {
        let mut labeler = SyscallLabeler::new();
        labeler.analyze_crate("safe_crate", "fn add(a: i32, b: i32) -> i32 { a + b }");

        let policy = labeler.generate_auto_policy("safe_crate");
        assert_eq!(policy.max_risk_level, RiskLevel::Safe);
        assert!(!policy.requires_audit);
    }
}
