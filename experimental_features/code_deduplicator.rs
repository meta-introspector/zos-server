use std::collections::HashMap;
use std::fs;
use std::path::Path;
use syn::{parse_file, visit::Visit, Expr, Stmt};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct CodePattern {
    pub pattern_type: String,
    pub canonical_form: String,
    pub length: usize,
}

pub struct CodeDeduplicator {
    patterns: HashMap<CodePattern, Vec<String>>, // pattern -> file locations
    canonical_numbers: HashMap<CodePattern, u64>,
    next_canonical_id: u64,
}

impl CodeDeduplicator {
    pub fn new() -> Self {
        Self {
            patterns: HashMap::new(),
            canonical_numbers: HashMap::new(),
            next_canonical_id: 1,
        }
    }

    pub fn analyze_file(&mut self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = fs::read_to_string(file_path)?;
        let syntax_tree = parse_file(&content)?;

        let mut visitor = PatternVisitor::new(file_path);
        visitor.visit_file(&syntax_tree);

        for pattern in visitor.patterns {
            self.patterns
                .entry(pattern.clone())
                .or_insert_with(Vec::new)
                .push(file_path.to_string());

            if !self.canonical_numbers.contains_key(&pattern) {
                self.canonical_numbers
                    .insert(pattern, self.next_canonical_id);
                self.next_canonical_id += 1;
            }
        }

        Ok(())
    }

    pub fn generate_canonical_refactor(&self) -> String {
        let mut refactor = String::new();

        // Generate canonical constants module
        refactor.push_str("// Generated canonical constants\n");
        refactor.push_str("pub mod canonical {\n");

        for (pattern, canonical_id) in &self.canonical_numbers {
            if pattern.pattern_type == "println_repeat" {
                refactor.push_str(&format!(
                    "    pub const SEPARATOR_{}: &str = \"{}\";\n",
                    canonical_id, pattern.canonical_form
                ));
            }
        }

        refactor.push_str("}\n\n");

        // Generate usage report
        refactor.push_str("// Duplicate patterns found:\n");
        for (pattern, locations) in &self.patterns {
            if locations.len() > 1 {
                let canonical_id = self.canonical_numbers[pattern];
                refactor.push_str(&format!(
                    "// Pattern #{} ({}): {} duplicates\n",
                    canonical_id,
                    pattern.pattern_type,
                    locations.len()
                ));
                for location in locations {
                    refactor.push_str(&format!("//   {}\n", location));
                }
            }
        }

        refactor
    }
}

struct PatternVisitor {
    file_path: String,
    patterns: Vec<CodePattern>,
}

impl PatternVisitor {
    fn new(file_path: &str) -> Self {
        Self {
            file_path: file_path.to_string(),
            patterns: Vec::new(),
        }
    }
}

impl<'ast> Visit<'ast> for PatternVisitor {
    fn visit_stmt(&mut self, stmt: &'ast Stmt) {
        if let Stmt::Expr(expr, _) = stmt {
            if let Expr::Macro(macro_expr) = expr {
                if macro_expr.mac.path.is_ident("println") {
                    let tokens = macro_expr.mac.tokens.to_string();
                    if tokens.contains("repeat") {
                        let pattern = CodePattern {
                            pattern_type: "println_repeat".to_string(),
                            canonical_form: self.extract_repeat_pattern(&tokens),
                            length: tokens.len(),
                        };
                        self.patterns.push(pattern);
                    }
                }
            }
        }
        syn::visit::visit_stmt(self, stmt);
    }
}

impl PatternVisitor {
    fn extract_repeat_pattern(&self, tokens: &str) -> String {
        // Extract the canonical form: "char".repeat(n)
        if let Some(start) = tokens.find('"') {
            if let Some(end) = tokens[start + 1..].find('"') {
                let char = &tokens[start + 1..start + 1 + end];
                if let Some(repeat_start) = tokens.find("repeat(") {
                    if let Some(repeat_end) = tokens[repeat_start..].find(')') {
                        let count = &tokens[repeat_start + 7..repeat_start + repeat_end];
                        return format!("\"{}\" × {}", char, count);
                    }
                }
            }
        }
        tokens.to_string()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut deduplicator = CodeDeduplicator::new();

    // Analyze all Rust files in src/
    for entry in fs::read_dir("src")? {
        let entry = entry?;
        if entry.path().extension().map_or(false, |ext| ext == "rs") {
            if let Err(e) = deduplicator.analyze_file(&entry.path().to_string_lossy()) {
                eprintln!("Error analyzing {}: {}", entry.path().display(), e);
            }
        }
    }

    // Generate canonical refactor
    let refactor = deduplicator.generate_canonical_refactor();
    fs::write("canonical_refactor.rs", refactor)?;

    println!("Generated canonical refactor in canonical_refactor.rs");
    println!(
        "Found {} unique patterns",
        deduplicator.canonical_numbers.len()
    );

    Ok(())
}
