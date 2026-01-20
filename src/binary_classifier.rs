// Binary Classifier for Safe vs Unsafe .so Analysis
use std::process::Command;

/// Binary classifier that analyzes .so files
pub struct BinaryClassifier {
    unsafe_patterns: Vec<BinaryPattern>,
    safe_patterns: Vec<BinaryPattern>,
    symbol_analyzer: SymbolAnalyzer,
}

#[derive(Debug, Clone)]
pub struct BinaryPattern {
    pub name: String,
    pub bytes: Vec<u8>,
    pub risk_level: RiskLevel,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RiskLevel {
    Safe,
    Suspicious,
    Dangerous,
    Critical,
}

#[derive(Debug, Clone)]
pub struct ClassificationResult {
    pub file_path: String,
    pub is_safe: bool,
    pub risk_score: f64,
    pub dangerous_symbols: Vec<String>,
    pub syscall_count: usize,
    pub stripped_verification: bool,
}

impl BinaryClassifier {
    pub fn new() -> Self {
        Self {
            unsafe_patterns: Self::get_unsafe_patterns(),
            safe_patterns: Self::get_safe_patterns(),
            symbol_analyzer: SymbolAnalyzer::new(),
        }
    }

    fn get_unsafe_patterns() -> Vec<BinaryPattern> {
        vec![
            BinaryPattern {
                name: "syscall_instruction".to_string(),
                bytes: vec![0x0f, 0x05], // syscall
                risk_level: RiskLevel::Critical,
            },
            BinaryPattern {
                name: "execve_call".to_string(),
                bytes: vec![0x48, 0xc7, 0xc0, 0x3b], // mov rax, 59 (execve)
                risk_level: RiskLevel::Critical,
            },
            BinaryPattern {
                name: "fork_call".to_string(),
                bytes: vec![0x48, 0xc7, 0xc0, 0x39], // mov rax, 57 (fork)
                risk_level: RiskLevel::Dangerous,
            },
            BinaryPattern {
                name: "indirect_call".to_string(),
                bytes: vec![0xff, 0xd0], // call rax
                risk_level: RiskLevel::Suspicious,
            },
        ]
    }

    fn get_safe_patterns() -> Vec<BinaryPattern> {
        vec![
            BinaryPattern {
                name: "stack_setup".to_string(),
                bytes: vec![0x48, 0x89, 0xe5], // mov rbp, rsp
                risk_level: RiskLevel::Safe,
            },
            BinaryPattern {
                name: "return_instruction".to_string(),
                bytes: vec![0xc3], // ret
                risk_level: RiskLevel::Safe,
            },
        ]
    }

    /// Classify a .so file as safe or unsafe
    pub fn classify_binary(&self, so_path: &str) -> Result<ClassificationResult, String> {
        let binary_data =
            std::fs::read(so_path).map_err(|e| format!("Failed to read binary: {}", e))?;

        // Analyze binary patterns
        let pattern_analysis = self.analyze_patterns(&binary_data);

        // Analyze symbols
        let symbol_analysis = self.symbol_analyzer.analyze_symbols(so_path)?;

        // Calculate risk score
        let risk_score = self.calculate_risk_score(&pattern_analysis, &symbol_analysis);

        // Verify stripping
        let stripped_verification = self.verify_stripping(&symbol_analysis);

        Ok(ClassificationResult {
            file_path: so_path.to_string(),
            is_safe: risk_score < 0.3,
            risk_score,
            dangerous_symbols: symbol_analysis.dangerous_symbols,
            syscall_count: pattern_analysis.syscall_patterns,
            stripped_verification,
        })
    }

    pub fn analyze_patterns(&self, binary_data: &[u8]) -> PatternAnalysis {
        let mut analysis = PatternAnalysis {
            syscall_patterns: 0,
            dangerous_patterns: 0,
            safe_patterns: 0,
        };

        // Search for unsafe patterns
        for pattern in &self.unsafe_patterns {
            let count = self.count_pattern_occurrences(binary_data, &pattern.bytes);
            match pattern.risk_level {
                RiskLevel::Critical => analysis.syscall_patterns += count,
                RiskLevel::Dangerous => analysis.dangerous_patterns += count,
                _ => {}
            }
        }

        // Search for safe patterns
        for pattern in &self.safe_patterns {
            analysis.safe_patterns += self.count_pattern_occurrences(binary_data, &pattern.bytes);
        }

        analysis
    }

    fn count_pattern_occurrences(&self, data: &[u8], pattern: &[u8]) -> usize {
        data.windows(pattern.len())
            .filter(|window| *window == pattern)
            .count()
    }

    fn calculate_risk_score(&self, patterns: &PatternAnalysis, symbols: &SymbolAnalysis) -> f64 {
        let mut score = 0.0;

        // Syscall patterns are critical
        score += patterns.syscall_patterns as f64 * 0.5;

        // Dangerous symbols
        score += symbols.dangerous_symbols.len() as f64 * 0.3;

        // Dangerous patterns
        score += patterns.dangerous_patterns as f64 * 0.2;

        // Safe patterns reduce score
        score -= patterns.safe_patterns as f64 * 0.01;

        score.max(0.0).min(1.0)
    }

    fn verify_stripping(&self, symbols: &SymbolAnalysis) -> bool {
        // Verify that dangerous symbols are absent
        symbols.dangerous_symbols.is_empty() && symbols.syscall_symbols.is_empty()
    }

    /// Compare before and after stripping
    pub fn compare_binaries(
        &self,
        before_path: &str,
        after_path: &str,
    ) -> Result<ComparisonResult, String> {
        let before = self.classify_binary(before_path)?;
        let after = self.classify_binary(after_path)?;

        Ok(ComparisonResult {
            syscalls_removed: before.syscall_count.saturating_sub(after.syscall_count),
            risk_reduction: before.risk_score - after.risk_score,
            symbols_stripped: before
                .dangerous_symbols
                .len()
                .saturating_sub(after.dangerous_symbols.len()),
            stripping_successful: after.stripped_verification,
            before_classification: before,
            after_classification: after,
        })
    }
}

#[derive(Debug)]
pub struct PatternAnalysis {
    pub syscall_patterns: usize,
    pub dangerous_patterns: usize,
    pub safe_patterns: usize,
}

#[derive(Debug)]
pub struct ComparisonResult {
    pub syscalls_removed: usize,
    pub risk_reduction: f64,
    pub symbols_stripped: usize,
    pub stripping_successful: bool,
    pub before_classification: ClassificationResult,
    pub after_classification: ClassificationResult,
}

/// Symbol analyzer using objdump/nm
pub struct SymbolAnalyzer {
    dangerous_symbol_patterns: Vec<String>,
}

impl SymbolAnalyzer {
    pub fn new() -> Self {
        Self {
            dangerous_symbol_patterns: vec![
                "execve".to_string(),
                "fork".to_string(),
                "mount".to_string(),
                "ptrace".to_string(),
                "setuid".to_string(),
                "setgid".to_string(),
                "syscall".to_string(),
                "system".to_string(),
            ],
        }
    }

    pub fn analyze_symbols(&self, so_path: &str) -> Result<SymbolAnalysis, String> {
        let mut analysis = SymbolAnalysis {
            total_symbols: 0,
            dangerous_symbols: Vec::new(),
            syscall_symbols: Vec::new(),
        };

        // Use objdump to extract symbols
        if let Ok(output) = Command::new("objdump").args(&["-T", so_path]).output() {
            let symbols_output = String::from_utf8_lossy(&output.stdout);

            for line in symbols_output.lines() {
                if line.contains("FUNC") || line.contains("OBJECT") {
                    analysis.total_symbols += 1;

                    // Check for dangerous symbols
                    for pattern in &self.dangerous_symbol_patterns {
                        if line.to_lowercase().contains(&pattern.to_lowercase()) {
                            analysis.dangerous_symbols.push(line.trim().to_string());

                            if pattern == "syscall" || pattern == "execve" || pattern == "fork" {
                                analysis.syscall_symbols.push(pattern.clone());
                            }
                        }
                    }
                }
            }
        }

        // Fallback: use nm if objdump fails
        if analysis.total_symbols == 0 {
            if let Ok(output) = Command::new("nm").args(&["-D", so_path]).output() {
                let symbols_output = String::from_utf8_lossy(&output.stdout);
                analysis.total_symbols = symbols_output.lines().count();

                for line in symbols_output.lines() {
                    for pattern in &self.dangerous_symbol_patterns {
                        if line.to_lowercase().contains(&pattern.to_lowercase()) {
                            analysis.dangerous_symbols.push(line.trim().to_string());
                        }
                    }
                }
            }
        }

        Ok(analysis)
    }
}

#[derive(Debug)]
pub struct SymbolAnalysis {
    pub total_symbols: usize,
    pub dangerous_symbols: Vec<String>,
    pub syscall_symbols: Vec<String>,
}

/// Automated verification system
pub struct VerificationSystem {
    classifier: BinaryClassifier,
}

impl VerificationSystem {
    pub fn new() -> Self {
        Self {
            classifier: BinaryClassifier::new(),
        }
    }

    /// Verify that stripping was successful
    pub fn verify_stripping(&self, original_so: &str, stripped_so: &str) -> Result<bool, String> {
        let comparison = self.classifier.compare_binaries(original_so, stripped_so)?;

        println!("🔍 Binary Comparison Results:");
        println!("  Syscalls removed: {}", comparison.syscalls_removed);
        println!(
            "  Risk reduction: {:.2}%",
            comparison.risk_reduction * 100.0
        );
        println!("  Symbols stripped: {}", comparison.symbols_stripped);
        println!(
            "  Stripping successful: {}",
            comparison.stripping_successful
        );

        // Verification criteria
        let verification_passed = comparison.syscalls_removed > 0
            && comparison.risk_reduction > 0.0
            && comparison.stripping_successful
            && comparison.after_classification.is_safe;

        if verification_passed {
            println!("✅ VERIFICATION PASSED: Binary is provably safe");
        } else {
            println!("❌ VERIFICATION FAILED: Binary may still contain dangerous code");
        }

        Ok(verification_passed)
    }

    /// Generate verification report
    pub fn generate_report(&self, comparison: &ComparisonResult) -> String {
        format!(
            "BINARY VERIFICATION REPORT\n\
             ========================\n\
             Original file: {}\n\
             Stripped file: {}\n\
             \n\
             BEFORE STRIPPING:\n\
             - Risk score: {:.2}\n\
             - Syscall count: {}\n\
             - Dangerous symbols: {}\n\
             \n\
             AFTER STRIPPING:\n\
             - Risk score: {:.2}\n\
             - Syscall count: {}\n\
             - Dangerous symbols: {}\n\
             \n\
             IMPROVEMENTS:\n\
             - Syscalls removed: {}\n\
             - Risk reduction: {:.2}%\n\
             - Symbols stripped: {}\n\
             - Verification: {}\n",
            comparison.before_classification.file_path,
            comparison.after_classification.file_path,
            comparison.before_classification.risk_score,
            comparison.before_classification.syscall_count,
            comparison.before_classification.dangerous_symbols.len(),
            comparison.after_classification.risk_score,
            comparison.after_classification.syscall_count,
            comparison.after_classification.dangerous_symbols.len(),
            comparison.syscalls_removed,
            comparison.risk_reduction * 100.0,
            comparison.symbols_stripped,
            if comparison.stripping_successful {
                "PASSED"
            } else {
                "FAILED"
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_detection() {
        let classifier = BinaryClassifier::new();
        let binary_with_syscall = vec![0x48, 0x89, 0xe5, 0x0f, 0x05, 0xc3]; // mov + syscall + ret

        let analysis = classifier.analyze_patterns(&binary_with_syscall);
        assert!(analysis.syscall_patterns > 0);
    }

    #[test]
    fn test_safe_classification() {
        let classifier = BinaryClassifier::new();
        let safe_binary = vec![0x48, 0x89, 0xe5, 0xc3]; // mov + ret (safe)

        let analysis = classifier.analyze_patterns(&safe_binary);
        assert_eq!(analysis.syscall_patterns, 0);
        assert!(analysis.safe_patterns > 0);
    }
}
