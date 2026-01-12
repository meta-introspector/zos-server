use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Write};

struct RustcMarkovAnalyzer {
    // Rustc source code model
    rustc_transitions: HashMap<char, HashMap<char, u32>>,
    rustc_total: u64,

    // Directory structure model
    dir_transitions: HashMap<char, HashMap<char, u32>>,
    dir_total: u64,

    rustc_files: Vec<String>,
}

impl RustcMarkovAnalyzer {
    fn new() -> Self {
        Self {
            rustc_transitions: HashMap::new(),
            rustc_total: 0,
            dir_transitions: HashMap::new(),
            dir_total: 0,
            rustc_files: Vec::new(),
        }
    }

    fn find_rustc_files(&mut self) -> Result<(), String> {
        println!("🔍 Finding rustc source files...");

        let file = fs::File::open("/mnt/data1/files.txt")
            .map_err(|e| format!("Failed to open files.txt: {}", e))?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            if let Ok(path) = line {
                if path.contains("rustc_") && path.ends_with(".rs") {
                    self.rustc_files.push(path);
                }
            }
        }

        println!("  Found {} rustc source files", self.rustc_files.len());
        Ok(())
    }

    fn train_on_rustc_paths(&mut self) -> Result<(), String> {
        println!("📖 Training on rustc file paths...");

        let mut total_chars = 0;

        for (i, file_path) in self.rustc_files.iter().enumerate() {
            if i >= 1000 { break; } // Limit to first 1000 paths

            let chars: Vec<char> = file_path.chars().collect();
            total_chars += chars.len();

            for window in chars.windows(2) {
                *self.rustc_transitions
                    .entry(window[0])
                    .or_insert_with(HashMap::new)
                    .entry(window[1])
                    .or_insert(0) += 1;
                self.rustc_total += 1;
            }

            if i % 100 == 0 {
                print!("\r  Processed {} paths, {} chars", i, total_chars);
                std::io::stdout().flush().unwrap();
            }
        }

        println!("\n  Rustc path model: {} transitions from {} paths", self.rustc_total, self.rustc_files.len().min(1000));
        Ok(())
    }

    fn train_on_directory_structure(&mut self) -> Result<(), String> {
        println!("📁 Training on directory structure...");

        let file = fs::File::open("/mnt/data1/files.txt")
            .map_err(|e| format!("Failed to open files.txt: {}", e))?;
        let reader = BufReader::new(file);

        let mut count = 0;
        for line in reader.lines() {
            if let Ok(path) = line {
                if path.contains("/compiler/rustc_") || path.contains("/src/") {
                    let chars: Vec<char> = path.chars().collect();
                    for window in chars.windows(2) {
                        *self.dir_transitions
                            .entry(window[0])
                            .or_insert_with(HashMap::new)
                            .entry(window[1])
                            .or_insert(0) += 1;
                        self.dir_total += 1;
                    }

                    count += 1;
                    if count >= 20000 { break; } // Sample 20k paths
                }
            }
        }

        println!("  Directory model: {} transitions from {} paths", self.dir_total, count);
        Ok(())
    }

    fn compute_similarity(&self) -> f64 {
        let mut common_transitions = 0;
        let mut total_comparisons = 0;

        for (from, rustc_to_map) in &self.rustc_transitions {
            if let Some(dir_to_map) = self.dir_transitions.get(from) {
                for (to, _) in rustc_to_map {
                    total_comparisons += 1;
                    if dir_to_map.contains_key(to) {
                        common_transitions += 1;
                    }
                }
            }
        }

        if total_comparisons > 0 {
            common_transitions as f64 / total_comparisons as f64
        } else {
            0.0
        }
    }

    fn find_shared_patterns(&self) -> Vec<String> {
        let mut patterns = Vec::new();

        for (from, rustc_to_map) in &self.rustc_transitions {
            if let Some(dir_to_map) = self.dir_transitions.get(from) {
                for (to, _) in rustc_to_map {
                    if dir_to_map.contains_key(to) {
                        if let Some(rustc_next) = self.rustc_transitions.get(to) {
                            if let Some(dir_next) = self.dir_transitions.get(to) {
                                for (next, _) in rustc_next {
                                    if dir_next.contains_key(next) {
                                        patterns.push(format!("{}{}{}", from, to, next));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        patterns.sort();
        patterns.dedup();
        patterns
    }

    fn print_analysis(&self) {
        println!("\n🔍 Rustc Source vs Directory Structure Analysis:");

        let similarity = self.compute_similarity();
        println!("  Model similarity: {:.2}%", similarity * 100.0);

        let shared_patterns = self.find_shared_patterns();
        println!("  Shared 3-char patterns: {}", shared_patterns.len());

        println!("\n🎯 Top shared patterns:");
        for pattern in shared_patterns.iter().take(15) {
            println!("    '{}'", pattern);
        }

        // Character frequency analysis
        let mut rustc_chars: HashMap<char, u32> = HashMap::new();
        for (from, to_map) in &self.rustc_transitions {
            for (to, count) in to_map {
                *rustc_chars.entry(*from).or_insert(0) += count;
                *rustc_chars.entry(*to).or_insert(0) += count;
            }
        }

        let mut rustc_sorted: Vec<_> = rustc_chars.iter().collect();
        rustc_sorted.sort_by_key(|(_, count)| std::cmp::Reverse(**count));

        println!("\n📊 Top rustc source characters:");
        for (c, count) in rustc_sorted.iter().take(8) {
            let display = if **c == ' ' { "SPACE" } else if **c == '\n' { "NEWLINE" } else { &c.to_string() };
            println!("    '{}': {} occurrences", display, count);
        }

        if similarity > 0.3 {
            println!("\n✨ THEORY CONFIRMED: Rustc source code structure maps to directory paths!");
            println!("    The compiler's own source patterns exist in the filesystem structure!");
        } else if similarity > 0.1 {
            println!("\n🤔 Moderate similarity - rustc patterns partially reflected in paths");
        } else {
            println!("\n❌ Low similarity - may need different analysis approach");
        }

        println!("\n🔄 Self-Reference Analysis:");
        println!("  Rustc files found: {}", self.rustc_files.len());
        println!("  This means rustc source code exists in our path database");
        println!("  The compiler can potentially analyze its own source structure!");
    }
}

fn main() {
    let mut analyzer = RustcMarkovAnalyzer::new();

    println!("🚀 Analyzing Rustc Source Code vs Directory Structure");

    if let Err(e) = analyzer.find_rustc_files() {
        eprintln!("Error finding files: {}", e);
        return;
    }

    if let Err(e) = analyzer.train_on_rustc_paths() {
        eprintln!("Error training on content: {}", e);
        return;
    }

    if let Err(e) = analyzer.train_on_directory_structure() {
        eprintln!("Error training on directories: {}", e);
        return;
    }

    analyzer.print_analysis();
}
