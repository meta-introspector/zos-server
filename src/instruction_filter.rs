// Code Complexity Analyzer and Filter
use crate::plugins::core_plugins::syscall_security_plugin::{SyscallSecurityPlugin, ComplexityLevel};
use std::collections::HashMap;

/// Code analysis result
#[derive(Debug, Clone)]
pub struct CodeAnalysis {
    pub complexity: ComplexityLevel,
    pub forbidden_patterns: Vec<String>,
    pub safe_to_execute: bool,
    pub required_role: String,
}

/// Instruction-level security filter
pub struct InstructionFilter {
    security_plugin: SyscallSecurityPlugin,
    complexity_cache: HashMap<String, CodeAnalysis>,
}

impl InstructionFilter {
    pub fn new() -> Self {
        Self {
            security_plugin: SyscallSecurityPlugin::new(),
            complexity_cache: HashMap::new(),
        }
    }

    /// Analyze and filter code before execution
    pub fn filter_code(&mut self, code: &str, user_role: &str) -> Result<CodeAnalysis, String> {
        // Check cache first
        let cache_key = format!("{}:{}", code.len(), user_role);
        if let Some(cached) = self.complexity_cache.get(&cache_key) {
            return Ok(cached.clone());
        }

        let mut analysis = CodeAnalysis {
            complexity: ComplexityLevel::Safe,
            forbidden_patterns: Vec::new(),
            safe_to_execute: true,
            required_role: "user".to_string(),
        };

        // Analyze complexity
        match self.security_plugin.analyze_code_complexity(code, user_role) {
            Ok(complexity) => {
                analysis.complexity = complexity.clone();
                analysis.required_role = match complexity {
                    ComplexityLevel::Safe => "user".to_string(),
                    ComplexityLevel::Limited => "user".to_string(),
                    ComplexityLevel::Privileged => "admin".to_string(),
                    ComplexityLevel::Syscall => "root".to_string(),
                };
            }
            Err(reason) => {
                analysis.safe_to_execute = false;
                analysis.forbidden_patterns.push(reason);
            }
        }

        // Additional security checks
        self.check_instruction_patterns(code, &mut analysis);

        // Cache result
        self.complexity_cache.insert(cache_key, analysis.clone());

        Ok(analysis)
    }

    /// Check for dangerous instruction patterns
    fn check_instruction_patterns(&self, code: &str, analysis: &mut CodeAnalysis) {
        let dangerous_patterns = [
            ("inline assembly", "asm!"),
            ("raw pointers", "*mut"),
            ("transmute", "std::mem::transmute"),
            ("forget", "std::mem::forget"),
            ("uninitialized", "std::mem::uninitialized"),
            ("raw syscall", "syscall("),
            ("process spawn", "std::process::Command"),
            ("dynamic loading", "libloading::"),
            ("ffi calls", "extern \"C\""),
        ];

        for (name, pattern) in &dangerous_patterns {
            if code.contains(pattern) {
                analysis.forbidden_patterns.push(format!("Dangerous pattern: {}", name));
                if analysis.complexity < ComplexityLevel::Privileged {
                    analysis.safe_to_execute = false;
                }
            }
        }
    }

    /// Remove code sections that exceed complexity threshold
    pub fn strip_complex_code(&self, code: &str, max_complexity: ComplexityLevel) -> String {
        let lines: Vec<&str> = code.lines().collect();
        let mut filtered_lines = Vec::new();

        for line in lines {
            let line_complexity = self.security_plugin.detect_complexity(line);
            if line_complexity <= max_complexity {
                filtered_lines.push(line);
            } else {
                filtered_lines.push(&format!("// REMOVED: complexity {:?} > {:?}",
                    line_complexity, max_complexity));
            }
        }

        filtered_lines.join("\n")
    }

    /// Check if user can execute code of given complexity
    pub fn can_user_execute(&self, user_role: &str, complexity: &ComplexityLevel) -> bool {
        match (user_role, complexity) {
            ("root", _) => true,
            ("admin", ComplexityLevel::Syscall) => false,
            ("admin", _) => true,
            ("developer", ComplexityLevel::Privileged | ComplexityLevel::Syscall) => false,
            ("developer", _) => true,
            ("user", ComplexityLevel::Safe | ComplexityLevel::Limited) => true,
            ("user", _) => false,
            _ => false,
        }
    }

    /// Apply rate limiting check
    pub fn check_rate_limit(&mut self, user_id: &str, code_size: u64) -> Result<(), String> {
        self.security_plugin.check_rate_limit(user_id, code_size)
    }

    /// Get complexity statistics
    pub fn get_complexity_stats(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        for analysis in self.complexity_cache.values() {
            let complexity_name = format!("{:?}", analysis.complexity);
            *stats.entry(complexity_name).or_insert(0) += 1;
        }
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_filtering() {
        let mut filter = InstructionFilter::new();

        let safe_code = "fn add(a: i32, b: i32) -> i32 { a + b }";
        let analysis = filter.filter_code(safe_code, "user").unwrap();
        assert!(analysis.safe_to_execute);
        assert_eq!(analysis.complexity, ComplexityLevel::Safe);

        let unsafe_code = "unsafe { syscall(1, 2, 3) }";
        let analysis = filter.filter_code(unsafe_code, "user").unwrap();
        assert!(!analysis.safe_to_execute);
    }

    #[test]
    fn test_complexity_stripping() {
        let filter = InstructionFilter::new();

        let mixed_code = r#"
fn safe_function() -> i32 { 42 }
unsafe { syscall(1) }
fn another_safe() -> String { "hello".to_string() }
"#;

        let stripped = filter.strip_complex_code(mixed_code, ComplexityLevel::Limited);
        assert!(stripped.contains("safe_function"));
        assert!(stripped.contains("REMOVED"));
        assert!(!stripped.contains("syscall"));
    }

    #[test]
    fn test_user_permissions() {
        let filter = InstructionFilter::new();

        assert!(filter.can_user_execute("root", &ComplexityLevel::Syscall));
        assert!(!filter.can_user_execute("user", &ComplexityLevel::Syscall));
        assert!(filter.can_user_execute("admin", &ComplexityLevel::Privileged));
        assert!(!filter.can_user_execute("user", &ComplexityLevel::Privileged));
    }
}
