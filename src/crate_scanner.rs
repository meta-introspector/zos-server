// Crate Security Scanner - Auto-analyze dependencies
use crate::syscall_labeler::{SyscallLabeler, RiskLevel};
use std::collections::HashMap;
use std::path::Path;

pub struct CrateScanner {
    labeler: SyscallLabeler,
    scanned_crates: HashMap<String, ScanResult>,
}

#[derive(Debug, Clone)]
pub struct ScanResult {
    pub crate_name: String,
    pub version: String,
    pub risk_level: RiskLevel,
    pub syscalls_found: Vec<String>,
    pub auto_secured: bool,
}

impl CrateScanner {
    pub fn new() -> Self {
        Self {
            labeler: SyscallLabeler::new(),
            scanned_crates: HashMap::new(),
        }
    }

    /// Scan all crates in Cargo.toml
    pub fn scan_dependencies(&mut self, cargo_toml_path: &str) -> Result<Vec<ScanResult>, String> {
        let cargo_content = std::fs::read_to_string(cargo_toml_path)
            .map_err(|e| format!("Failed to read Cargo.toml: {}", e))?;

        let mut results = Vec::new();

        // Extract dependencies (simplified parser)
        for line in cargo_content.lines() {
            if let Some(crate_name) = self.extract_dependency(line) {
                if let Ok(result) = self.scan_crate(&crate_name) {
                    results.push(result);
                }
            }
        }

        Ok(results)
    }

    fn extract_dependency(&self, line: &str) -> Option<String> {
        if line.contains("=") && !line.starts_with('#') {
            if let Some(name) = line.split('=').next() {
                return Some(name.trim().trim_matches('"').to_string());
            }
        }
        None
    }

    /// Scan individual crate
    pub fn scan_crate(&mut self, crate_name: &str) -> Result<ScanResult, String> {
        // Try to find crate source in common locations
        let source_paths = vec![
            format!("~/.cargo/registry/src/*/{}*/src/lib.rs", crate_name),
            format!("./target/debug/deps/{}-*/src/lib.rs", crate_name),
            format!("./src/{}.rs", crate_name),
        ];

        let mut source_code = String::new();
        for path_pattern in source_paths {
            if let Ok(content) = std::fs::read_to_string(&path_pattern) {
                source_code = content;
                break;
            }
        }

        if source_code.is_empty() {
            // Fallback: analyze based on known crate patterns
            source_code = self.get_known_crate_patterns(crate_name);
        }

        // Analyze syscall usage
        let paths = self.labeler.analyze_crate(crate_name, &source_code);
        let policy = self.labeler.generate_auto_policy(crate_name);

        let result = ScanResult {
            crate_name: crate_name.to_string(),
            version: "unknown".to_string(),
            risk_level: policy.max_risk_level,
            syscalls_found: paths.iter().map(|p| p.syscall.clone()).collect(),
            auto_secured: true,
        };

        self.scanned_crates.insert(crate_name.to_string(), result.clone());
        Ok(result)
    }

    fn get_known_crate_patterns(&self, crate_name: &str) -> String {
        // Known syscall patterns for common crates
        match crate_name {
            "tokio" => "libc::epoll_create libc::socket libc::bind".to_string(),
            "std" => "libc::read libc::write libc::open libc::close".to_string(),
            "libc" => "syscall execve ptrace mount setuid".to_string(),
            "nix" => "libc::fork libc::execve libc::wait".to_string(),
            "openssl" => "libc::socket libc::read libc::write".to_string(),
            _ => String::new(),
        }
    }

    /// Generate security report
    pub fn generate_report(&self) -> SecurityReport {
        let mut critical_crates = Vec::new();
        let mut high_risk_crates = Vec::new();
        let mut medium_risk_crates = Vec::new();
        let mut safe_crates = Vec::new();

        for result in self.scanned_crates.values() {
            match result.risk_level {
                RiskLevel::Critical => critical_crates.push(result.crate_name.clone()),
                RiskLevel::High => high_risk_crates.push(result.crate_name.clone()),
                RiskLevel::Medium => medium_risk_crates.push(result.crate_name.clone()),
                RiskLevel::Safe => safe_crates.push(result.crate_name.clone()),
            }
        }

        SecurityReport {
            total_crates: self.scanned_crates.len(),
            critical_crates,
            high_risk_crates,
            medium_risk_crates,
            safe_crates,
        }
    }

    /// Check if user can use specific crate
    pub fn check_crate_permission(&self, crate_name: &str, user_role: &str) -> Result<(), String> {
        self.labeler.check_crate_access(crate_name, user_role)
    }
}

#[derive(Debug)]
pub struct SecurityReport {
    pub total_crates: usize,
    pub critical_crates: Vec<String>,
    pub high_risk_crates: Vec<String>,
    pub medium_risk_crates: Vec<String>,
    pub safe_crates: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crate_scanning() {
        let mut scanner = CrateScanner::new();
        let result = scanner.scan_crate("libc").unwrap();
        assert_eq!(result.risk_level, RiskLevel::Critical);
    }
}
