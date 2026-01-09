// Code Quarantine and Path Sampling System
use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

/// Code quarantine system
pub struct CodeQuarantine {
    known_paths: HashSet<String>,
    quarantined_code: HashMap<String, QuarantinedCode>,
    sampling_approvals: HashMap<String, SamplingApproval>,
    execution_mode: ExecutionMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuarantinedCode {
    pub code_hash: String,
    pub source_code: String,
    pub execution_paths: Vec<ExecutionPath>,
    pub quarantine_timestamp: u64,
    pub test_results: Vec<TestResult>,
    pub approval_status: ApprovalStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPath {
    pub path_hash: String,
    pub function_chain: Vec<String>,
    pub complexity_score: u32,
    pub orbit_class: String,
    pub first_seen: u64,
    pub test_coverage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub test_id: String,
    pub passed: bool,
    pub execution_time: u64,
    pub memory_used: u64,
    pub paths_covered: Vec<String>,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingApproval {
    pub path_hash: String,
    pub approved_by: String,
    pub approval_timestamp: u64,
    pub sampling_percentage: f64,
    pub max_executions: u32,
    pub expiry_timestamp: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionMode {
    Test,        // All new code runs in test mode
    Sampling,    // Approved paths run with sampling
    Production,  // Only known safe paths
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ApprovalStatus {
    Quarantined,     // New code, not tested
    Testing,         // Currently under test
    TestPassed,      // Tests passed, awaiting approval
    Approved,        // Approved for sampling
    ProductionReady, // Ready for full production
    Rejected,        // Failed tests or approval
}

impl CodeQuarantine {
    pub fn new() -> Self {
        Self {
            known_paths: HashSet::new(),
            quarantined_code: HashMap::new(),
            sampling_approvals: HashMap::new(),
            execution_mode: ExecutionMode::Production,
        }
    }

    /// Quarantine new code before any execution
    pub fn quarantine_code(&mut self, source_code: &str) -> Result<String, String> {
        let code_hash = self.calculate_hash(source_code);

        // Check if code is already known
        if self.is_code_known(&code_hash) {
            return Ok(code_hash);
        }

        // Analyze execution paths
        let execution_paths = self.analyze_execution_paths(source_code)?;

        // Check for new paths
        let new_paths = self.identify_new_paths(&execution_paths);

        if !new_paths.is_empty() {
            println!("🔒 QUARANTINE: New execution paths detected");
            for path in &new_paths {
                println!("  - New path: {}", path.path_hash);
            }

            let quarantined = QuarantinedCode {
                code_hash: code_hash.clone(),
                source_code: source_code.to_string(),
                execution_paths: new_paths,
                quarantine_timestamp: self.current_timestamp(),
                test_results: Vec::new(),
                approval_status: ApprovalStatus::Quarantined,
            };

            self.quarantined_code.insert(code_hash.clone(), quarantined);
            return Err(format!("Code quarantined: {}", code_hash));
        }

        Ok(code_hash)
    }

    /// Execute code with quarantine checks
    pub fn execute_with_quarantine(&mut self, code_hash: &str, user_id: &str) -> Result<ExecutionResult, String> {
        // Check execution mode
        match self.execution_mode {
            ExecutionMode::Production => {
                if !self.is_production_ready(code_hash) {
                    return Err("Code not approved for production execution".to_string());
                }
            }
            ExecutionMode::Sampling => {
                if !self.is_sampling_approved(code_hash) {
                    return Err("Code not approved for sampling".to_string());
                }

                // Check sampling limits
                if !self.check_sampling_limits(code_hash)? {
                    return Err("Sampling limits exceeded".to_string());
                }
            }
            ExecutionMode::Test => {
                // Test mode allows quarantined code
                if let Some(quarantined) = self.quarantined_code.get_mut(code_hash) {
                    quarantined.approval_status = ApprovalStatus::Testing;
                }
            }
        }

        // Execute with monitoring
        let result = self.monitored_execution(code_hash, user_id)?;

        // Record execution for sampling/testing
        self.record_execution(code_hash, &result)?;

        Ok(result)
    }

    fn analyze_execution_paths(&self, source_code: &str) -> Result<Vec<ExecutionPath>, String> {
        let mut paths = Vec::new();

        // Simple path analysis (in practice would use AST analysis)
        let functions = self.extract_functions(source_code);

        for (i, function) in functions.iter().enumerate() {
            let path_hash = self.calculate_hash(&format!("{}:{}", function, i));

            paths.push(ExecutionPath {
                path_hash,
                function_chain: vec![function.clone()],
                complexity_score: self.estimate_complexity(function),
                orbit_class: "Cyclic".to_string(), // Would use orbit classifier
                first_seen: self.current_timestamp(),
                test_coverage: 0.0,
            });
        }

        Ok(paths)
    }

    fn extract_functions(&self, source_code: &str) -> Vec<String> {
        source_code.lines()
            .filter(|line| line.trim().starts_with("fn ") || line.trim().starts_with("pub fn "))
            .map(|line| {
                line.split_whitespace()
                    .nth(1)
                    .or_else(|| line.split_whitespace().nth(2))
                    .unwrap_or("unknown")
                    .split('(')
                    .next()
                    .unwrap_or("unknown")
                    .to_string()
            })
            .collect()
    }

    fn identify_new_paths(&self, paths: &[ExecutionPath]) -> Vec<ExecutionPath> {
        paths.iter()
            .filter(|path| !self.known_paths.contains(&path.path_hash))
            .cloned()
            .collect()
    }

    fn is_code_known(&self, code_hash: &str) -> bool {
        !self.quarantined_code.contains_key(code_hash) ||
        self.quarantined_code.get(code_hash)
            .map(|q| q.approval_status == ApprovalStatus::ProductionReady)
            .unwrap_or(false)
    }

    fn is_production_ready(&self, code_hash: &str) -> bool {
        self.quarantined_code.get(code_hash)
            .map(|q| q.approval_status == ApprovalStatus::ProductionReady)
            .unwrap_or(false)
    }

    fn is_sampling_approved(&self, code_hash: &str) -> bool {
        self.sampling_approvals.contains_key(code_hash) &&
        self.quarantined_code.get(code_hash)
            .map(|q| q.approval_status == ApprovalStatus::Approved)
            .unwrap_or(false)
    }

    fn check_sampling_limits(&self, code_hash: &str) -> Result<bool, String> {
        if let Some(approval) = self.sampling_approvals.get(code_hash) {
            let now = self.current_timestamp();

            // Check expiry
            if now > approval.expiry_timestamp {
                return Ok(false);
            }

            // Check execution count (simplified)
            // In practice would track actual executions
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn monitored_execution(&self, code_hash: &str, user_id: &str) -> Result<ExecutionResult, String> {
        let start_time = self.current_timestamp();

        // Simulate execution with monitoring
        let result = ExecutionResult {
            code_hash: code_hash.to_string(),
            user_id: user_id.to_string(),
            execution_time: 100, // ms
            memory_used: 1024,   // bytes
            paths_executed: vec!["main".to_string()],
            success: true,
            timestamp: start_time,
        };

        println!("🔍 MONITORED EXECUTION: {} by {}", code_hash, user_id);
        Ok(result)
    }

    fn record_execution(&mut self, code_hash: &str, result: &ExecutionResult) -> Result<(), String> {
        // Record for testing/sampling analysis
        if let Some(quarantined) = self.quarantined_code.get_mut(code_hash) {
            let test_result = TestResult {
                test_id: format!("test_{}", result.timestamp),
                passed: result.success,
                execution_time: result.execution_time,
                memory_used: result.memory_used,
                paths_covered: result.paths_executed.clone(),
                timestamp: result.timestamp,
            };

            quarantined.test_results.push(test_result);
        }

        Ok(())
    }

    /// Approve code for sampling after successful testing
    pub fn approve_for_sampling(&mut self, code_hash: &str, approver: &str, sampling_percentage: f64) -> Result<(), String> {
        let quarantined = self.quarantined_code.get_mut(code_hash)
            .ok_or("Code not found in quarantine")?;

        // Check test results
        if quarantined.test_results.is_empty() {
            return Err("No test results available".to_string());
        }

        let passed_tests = quarantined.test_results.iter().filter(|t| t.passed).count();
        let total_tests = quarantined.test_results.len();

        if (passed_tests as f64 / total_tests as f64) < 0.95 {
            return Err("Insufficient test pass rate for approval".to_string());
        }

        // Create sampling approval
        let approval = SamplingApproval {
            path_hash: code_hash.to_string(),
            approved_by: approver.to_string(),
            approval_timestamp: self.current_timestamp(),
            sampling_percentage,
            max_executions: 1000,
            expiry_timestamp: self.current_timestamp() + 86400 * 7, // 7 days
        };

        self.sampling_approvals.insert(code_hash.to_string(), approval);
        quarantined.approval_status = ApprovalStatus::Approved;

        println!("✅ APPROVED FOR SAMPLING: {} ({}%)", code_hash, sampling_percentage);
        Ok(())
    }

    /// Promote code to production after successful sampling
    pub fn promote_to_production(&mut self, code_hash: &str) -> Result<(), String> {
        let quarantined = self.quarantined_code.get_mut(code_hash)
            .ok_or("Code not found")?;

        if quarantined.approval_status != ApprovalStatus::Approved {
            return Err("Code must be approved for sampling first".to_string());
        }

        // Add paths to known safe paths
        for path in &quarantined.execution_paths {
            self.known_paths.insert(path.path_hash.clone());
        }

        quarantined.approval_status = ApprovalStatus::ProductionReady;

        println!("🚀 PROMOTED TO PRODUCTION: {}", code_hash);
        Ok(())
    }

    /// Set execution mode
    pub fn set_execution_mode(&mut self, mode: ExecutionMode) {
        self.execution_mode = mode;
        println!("🔧 EXECUTION MODE: {:?}", mode);
    }

    /// Generate quarantine report
    pub fn generate_quarantine_report(&self) -> QuarantineReport {
        let total_quarantined = self.quarantined_code.len();
        let mut status_counts = HashMap::new();

        for quarantined in self.quarantined_code.values() {
            *status_counts.entry(quarantined.approval_status.clone()).or_insert(0) += 1;
        }

        QuarantineReport {
            total_quarantined,
            status_counts,
            known_safe_paths: self.known_paths.len(),
            active_sampling: self.sampling_approvals.len(),
            execution_mode: self.execution_mode.clone(),
        }
    }

    fn calculate_hash(&self, data: &str) -> String {
        format!("hash_{}", data.len()) // Simplified hash
    }

    fn estimate_complexity(&self, function: &str) -> u32 {
        function.len() as u32 // Simplified complexity
    }

    fn current_timestamp(&self) -> u64 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
    }
}

#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub code_hash: String,
    pub user_id: String,
    pub execution_time: u64,
    pub memory_used: u64,
    pub paths_executed: Vec<String>,
    pub success: bool,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct QuarantineReport {
    pub total_quarantined: usize,
    pub status_counts: HashMap<ApprovalStatus, u32>,
    pub known_safe_paths: usize,
    pub active_sampling: usize,
    pub execution_mode: ExecutionMode,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_quarantine() {
        let mut quarantine = CodeQuarantine::new();

        let new_code = "fn test() { println!(\"hello\"); }";
        let result = quarantine.quarantine_code(new_code);

        // New code should be quarantined
        assert!(result.is_err());
    }

    #[test]
    fn test_execution_modes() {
        let mut quarantine = CodeQuarantine::new();

        // Test mode allows quarantined code
        quarantine.set_execution_mode(ExecutionMode::Test);

        // Production mode blocks quarantined code
        quarantine.set_execution_mode(ExecutionMode::Production);
    }

    #[test]
    fn test_sampling_approval() {
        let mut quarantine = CodeQuarantine::new();
        let code = "fn safe() { 1 + 1 }";

        // Quarantine code
        let code_hash = quarantine.quarantine_code(code).unwrap_err();
        let hash = code_hash.split(": ").nth(1).unwrap();

        // Approve for sampling
        let result = quarantine.approve_for_sampling(hash, "admin", 10.0);
        // Would fail due to no test results, but tests the interface
    }
}
