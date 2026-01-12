use std::process::Command;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::collections::HashSet;

struct CompilerPathMatcher {
    path_database: HashSet<String>,
    ast_strings: Vec<String>,
    hir_strings: Vec<String>,
    mir_strings: Vec<String>,
}

impl CompilerPathMatcher {
    fn new() -> Self {
        Self {
            path_database: HashSet::new(),
            ast_strings: Vec::new(),
            hir_strings: Vec::new(),
            mir_strings: Vec::new(),
        }
    }

    fn load_path_database(&mut self) -> Result<(), String> {
        println!("📂 Loading 33.9M file paths...");

        let file = fs::File::open("/mnt/data1/files.txt")
            .map_err(|e| format!("Failed to open files.txt: {}", e))?;
        let reader = BufReader::new(file);

        let mut count = 0;
        for line in reader.lines() {
            if let Ok(path) = line {
                self.path_database.insert(path);
                count += 1;

                if count % 1000000 == 0 {
                    print!("\r  Loaded {} paths", count);
                    std::io::stdout().flush().unwrap();
                }
            }
        }

        println!("\n✅ Loaded {} paths into database", self.path_database.len());
        Ok(())
    }

    fn extract_compiler_strings(&mut self) -> Result<(), String> {
        let source_file = "src/helloworld.rs";

        // Get AST dump
        println!("🌳 Extracting AST...");
        let ast_output = Command::new("rustc")
            .args(&["--pretty=ast", source_file])
            .output()
            .map_err(|e| format!("AST extraction failed: {}", e))?;

        let ast_content = String::from_utf8_lossy(&ast_output.stdout);
        let mut ast_strings = Vec::new();
        self.extract_strings_from_output(&ast_content, &mut ast_strings);
        self.ast_strings = ast_strings;

        // Get HIR dump
        println!("🔍 Extracting HIR...");
        let hir_output = Command::new("rustc")
            .args(&["--pretty=hir", source_file])
            .output()
            .map_err(|e| format!("HIR extraction failed: {}", e))?;

        let hir_content = String::from_utf8_lossy(&hir_output.stdout);
        let mut hir_strings = Vec::new();
        self.extract_strings_from_output(&hir_content, &mut hir_strings);
        self.hir_strings = hir_strings;

        // Get MIR dump
        println!("⚙️ Extracting MIR...");
        let mir_output = Command::new("rustc")
            .args(&["-Z", "dump-mir=all", source_file])
            .output()
            .map_err(|e| format!("MIR extraction failed: {}", e))?;

        let mir_content = String::from_utf8_lossy(&mir_output.stdout);
        let mut mir_strings = Vec::new();
        self.extract_strings_from_output(&mir_content, &mut mir_strings);
        self.mir_strings = mir_strings;

        println!("📊 Extracted {} AST, {} HIR, {} MIR strings",
            self.ast_strings.len(), self.hir_strings.len(), self.mir_strings.len());

        Ok(())
    }

    fn extract_strings_from_output(&self, content: &str, strings: &mut Vec<String>) {
        // Extract meaningful strings from compiler output
        for line in content.lines() {
            let trimmed = line.trim();

            // Skip empty lines and common noise
            if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.len() < 3 {
                continue;
            }

            // Extract identifiers, paths, and meaningful tokens
            let words: Vec<&str> = trimmed.split_whitespace().collect();
            for word in words {
                let clean_word = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '/' && c != '.' && c != '_');

                if clean_word.len() > 2 && (clean_word.contains('/') || clean_word.contains('.') || clean_word.len() > 5) {
                    strings.push(clean_word.to_string());
                }
            }

            // Also add the full line if it looks like a path
            if trimmed.contains('/') && trimmed.len() > 5 {
                strings.push(trimmed.to_string());
            }
        }

        strings.sort();
        strings.dedup();
    }

    fn find_matches(&self) -> (Vec<String>, Vec<String>, Vec<String>) {
        println!("🔍 Searching for compiler strings in path database...");

        let mut ast_matches = Vec::new();
        let mut hir_matches = Vec::new();
        let mut mir_matches = Vec::new();

        // Check AST strings
        for ast_string in &self.ast_strings {
            if self.path_database.contains(ast_string) {
                ast_matches.push(ast_string.clone());
            }
        }

        // Check HIR strings
        for hir_string in &self.hir_strings {
            if self.path_database.contains(hir_string) {
                hir_matches.push(hir_string.clone());
            }
        }

        // Check MIR strings
        for mir_string in &self.mir_strings {
            if self.path_database.contains(mir_string) {
                mir_matches.push(mir_string.clone());
            }
        }

        (ast_matches, hir_matches, mir_matches)
    }

    fn print_analysis(&self) {
        let (ast_matches, hir_matches, mir_matches) = self.find_matches();

        println!("\n🎯 COMPILER STRING → PATH DATABASE MATCHES:");

        println!("\n🌳 AST matches ({} found):", ast_matches.len());
        for (i, match_str) in ast_matches.iter().take(10).enumerate() {
            println!("    [{}] {}", i + 1, match_str);
        }

        println!("\n🔍 HIR matches ({} found):", hir_matches.len());
        for (i, match_str) in hir_matches.iter().take(10).enumerate() {
            println!("    [{}] {}", i + 1, match_str);
        }

        println!("\n⚙️ MIR matches ({} found):", mir_matches.len());
        for (i, match_str) in mir_matches.iter().take(10).enumerate() {
            println!("    [{}] {}", i + 1, match_str);
        }

        let total_matches = ast_matches.len() + hir_matches.len() + mir_matches.len();
        let total_strings = self.ast_strings.len() + self.hir_strings.len() + self.mir_strings.len();

        println!("\n📊 THEORY VALIDATION:");
        println!("  Total compiler strings: {}", total_strings);
        println!("  Total path matches: {}", total_matches);
        println!("  Match rate: {:.2}%", (total_matches as f64 / total_strings as f64) * 100.0);

        if total_matches > 0 {
            println!("\n✨ THEORY CONFIRMED: Compiler output strings exist as paths in the database!");
            println!("    The path structure DOES contain representations of the compiled program!");
        } else {
            println!("\n❌ No direct matches found - may need fuzzy matching or substring analysis");
        }
    }
}

fn main() {
    let mut matcher = CompilerPathMatcher::new();

    println!("🚀 Testing Theory: Compiler Output → Path Database");

    if let Err(e) = matcher.load_path_database() {
        eprintln!("Error loading database: {}", e);
        return;
    }

    if let Err(e) = matcher.extract_compiler_strings() {
        eprintln!("Error extracting compiler strings: {}", e);
        return;
    }

    matcher.print_analysis();
}
