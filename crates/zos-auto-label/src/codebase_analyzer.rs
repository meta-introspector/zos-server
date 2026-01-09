// Codebase analyzer that extracts and labels actual functions/modules
use crate::auto_label_system::{AutoLabelSystem, EigenLabel};
use std::fs;
use std::path::Path;

/// Analyzes codebase structure and applies auto-labeling
#[derive(Debug)]
pub struct CodebaseAnalyzer {
    label_system: AutoLabelSystem,
    functions: Vec<String>,
    modules: Vec<String>,
    structs: Vec<String>,
}

impl CodebaseAnalyzer {
    pub fn new() -> Self {
        Self {
            label_system: AutoLabelSystem::new(),
            functions: Vec::new(),
            modules: Vec::new(),
            structs: Vec::new(),
        }
    }

    /// Scan the codebase and extract all identifiers
    pub fn scan_codebase(&mut self, src_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.scan_directory(Path::new(src_path))?;
        Ok(())
    }

    /// Auto-label all discovered identifiers
    pub fn label_all(&mut self) -> Vec<(String, EigenLabel)> {
        let mut labeled = Vec::new();

        // Label functions
        for func in &self.functions {
            let label = self.label_system.auto_label(func);
            labeled.push((format!("fn::{}", func), label));
        }

        // Label modules
        for module in &self.modules {
            let label = self.label_system.auto_label(module);
            labeled.push((format!("mod::{}", module), label));
        }

        // Label structs
        for struct_name in &self.structs {
            let label = self.label_system.auto_label(struct_name);
            labeled.push((format!("struct::{}", struct_name), label));
        }

        labeled
    }

    /// Find structural duplicates across the codebase
    pub fn find_structural_duplicates(&self) -> Vec<(String, String, f64)> {
        self.label_system.find_duplicates(0.7)
    }

    /// Generate alignment report showing eigenmatrix mappings
    pub fn generate_alignment_report(&self) -> String {
        let mut report = String::new();
        report.push_str("# Codebase Alignment Report\n\n");

        report.push_str("## Discovered Identifiers\n");
        report.push_str(&format!("- Functions: {}\n", self.functions.len()));
        report.push_str(&format!("- Modules: {}\n", self.modules.len()));
        report.push_str(&format!("- Structs: {}\n", self.structs.len()));

        report.push_str("\n## Eigenmatrix Mappings\n");
        let ranked = self.label_system.rank_labels();
        for (i, (name, score)) in ranked.iter().take(10).enumerate() {
            if let Some(proof) = self.label_system.prove_label_mapping(name) {
                report.push_str(&format!("{}. {} (score: {:.3})\n", i + 1, name, score));
                report.push_str(&format!("   {}\n", proof));
            }
        }

        report.push_str("\n## Structural Duplicates\n");
        let duplicates = self.find_structural_duplicates();
        for (name1, name2, similarity) in duplicates.iter().take(5) {
            report.push_str(&format!(
                "- {} ≈ {} (similarity: {:.3})\n",
                name1, name2, similarity
            ));
        }

        report
    }

    fn scan_directory(&mut self, dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir() {
                    // Skip target and hidden directories
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if !name.starts_with('.') && name != "target" {
                            self.scan_directory(&path)?;
                        }
                    }
                } else if path.extension().map_or(false, |ext| ext == "rs") {
                    self.scan_rust_file(&path)?;
                }
            }
        }
        Ok(())
    }

    fn scan_rust_file(&mut self, file_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let content = fs::read_to_string(file_path)?;

        // Extract module name from file path
        if let Some(file_name) = file_path.file_stem().and_then(|n| n.to_str()) {
            if file_name != "main" && file_name != "lib" {
                self.modules.push(file_name.to_string());
            }
        }

        // Simple regex-like parsing for functions and structs
        for line in content.lines() {
            let trimmed = line.trim();

            // Extract function names
            if let Some(func_name) = self.extract_function_name(trimmed) {
                if !self.functions.contains(&func_name) {
                    self.functions.push(func_name);
                }
            }

            // Extract struct names
            if let Some(struct_name) = self.extract_struct_name(trimmed) {
                if !self.structs.contains(&struct_name) {
                    self.structs.push(struct_name);
                }
            }
        }

        Ok(())
    }

    fn extract_function_name(&self, line: &str) -> Option<String> {
        // Match patterns like "pub fn name(" or "fn name("
        if line.contains("fn ") && line.contains('(') {
            let parts: Vec<&str> = line.split("fn ").collect();
            if parts.len() > 1 {
                let after_fn = parts[1];
                if let Some(paren_pos) = after_fn.find('(') {
                    let name = after_fn[..paren_pos].trim();
                    if !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                        return Some(name.to_string());
                    }
                }
            }
        }
        None
    }

    fn extract_struct_name(&self, line: &str) -> Option<String> {
        // Match patterns like "pub struct Name" or "struct Name"
        if line.contains("struct ") && !line.contains("fn ") {
            let parts: Vec<&str> = line.split("struct ").collect();
            if parts.len() > 1 {
                let after_struct = parts[1].trim();
                let name = after_struct.split_whitespace().next()?;
                let clean_name = name.split('<').next()?.split(';').next()?;
                if !clean_name.is_empty()
                    && clean_name.chars().all(|c| c.is_alphanumeric() || c == '_')
                {
                    return Some(clean_name.to_string());
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_extraction() {
        let analyzer = CodebaseAnalyzer::new();

        assert_eq!(
            analyzer.extract_function_name("pub fn test_function() {"),
            Some("test_function".to_string())
        );
        assert_eq!(
            analyzer.extract_function_name("fn simple() -> bool {"),
            Some("simple".to_string())
        );
        assert_eq!(
            analyzer.extract_function_name("    fn indented(param: i32) {"),
            Some("indented".to_string())
        );
        assert_eq!(analyzer.extract_function_name("not a function"), None);
    }

    #[test]
    fn test_struct_extraction() {
        let analyzer = CodebaseAnalyzer::new();

        assert_eq!(
            analyzer.extract_struct_name("pub struct TestStruct {"),
            Some("TestStruct".to_string())
        );
        assert_eq!(
            analyzer.extract_struct_name("struct Simple;"),
            Some("Simple".to_string())
        );
        assert_eq!(
            analyzer.extract_struct_name("    struct Indented<T> {"),
            Some("Indented".to_string())
        );
        assert_eq!(analyzer.extract_struct_name("not a struct"), None);
    }
}
