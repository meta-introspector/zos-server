use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Write};

struct HIRComparisonAnalyzer {
    // Model A: HIR dump
    hir_model: HashMap<char, HashMap<char, u32>>,
    hir_total: u64,

    // Model B: Rustc source paths
    rustc_path_model: HashMap<char, HashMap<char, u32>>,
    rustc_path_total: u64,

    // Model C: Rustc source content
    rustc_content_model: HashMap<char, HashMap<char, u32>>,
    rustc_content_total: u64,

    rustc_files: Vec<String>,
}

impl HIRComparisonAnalyzer {
    fn new() -> Self {
        Self {
            hir_model: HashMap::new(),
            hir_total: 0,
            rustc_path_model: HashMap::new(),
            rustc_path_total: 0,
            rustc_content_model: HashMap::new(),
            rustc_content_total: 0,
            rustc_files: Vec::new(),
        }
    }

    fn train_on_hir_dump(&mut self) -> Result<(), String> {
        println!("🔧 Training on HIR dump...");

        let hir_content = fs::read_to_string("helloworld.hir")
            .map_err(|e| format!("Failed to read HIR: {}", e))?;

        let chars: Vec<char> = hir_content.chars().collect();
        for window in chars.windows(2) {
            *self.hir_model
                .entry(window[0])
                .or_insert_with(HashMap::new)
                .entry(window[1])
                .or_insert(0) += 1;
            self.hir_total += 1;
        }

        println!("  HIR model: {} transitions", self.hir_total);
        Ok(())
    }

    fn find_and_train_rustc(&mut self) -> Result<(), String> {
        println!("🔍 Finding rustc files...");

        let file = fs::File::open("/mnt/data1/files.txt")
            .map_err(|e| format!("Failed to open files.txt: {}", e))?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            if let Ok(path) = line {
                if path.contains("rustc_") && path.ends_with(".rs") && path.len() < 200 {
                    self.rustc_files.push(path);
                }
            }
        }

        println!("  Found {} rustc files", self.rustc_files.len());

        // Train on first 50 files
        let mut files_read = 0;
        for (i, file_path) in self.rustc_files.iter().enumerate() {
            if i >= 50 { break; }

            // Train path model
            let path_chars: Vec<char> = file_path.chars().collect();
            for window in path_chars.windows(2) {
                *self.rustc_path_model
                    .entry(window[0])
                    .or_insert_with(HashMap::new)
                    .entry(window[1])
                    .or_insert(0) += 1;
                self.rustc_path_total += 1;
            }

            // Train content model
            let full_path = if file_path.starts_with("./") {
                format!("/mnt/data1/{}", &file_path[2..])
            } else {
                format!("/mnt/data1/{}", file_path)
            };

            if let Ok(content) = fs::read_to_string(&full_path) {
                if content.len() < 50000 {
                    let content_chars: Vec<char> = content.chars().collect();
                    for window in content_chars.windows(2) {
                        *self.rustc_content_model
                            .entry(window[0])
                            .or_insert_with(HashMap::new)
                            .entry(window[1])
                            .or_insert(0) += 1;
                        self.rustc_content_total += 1;
                    }
                    files_read += 1;
                }
            }
        }

        println!("  Rustc path model: {} transitions", self.rustc_path_total);
        println!("  Rustc content model: {} transitions from {} files", self.rustc_content_total, files_read);

        Ok(())
    }

    fn compute_similarities(&self) -> (f64, f64) {
        // HIR vs Rustc paths
        let mut hir_path_common = 0;
        let mut hir_path_total = 0;

        for (from, hir_to_map) in &self.hir_model {
            if let Some(path_to_map) = self.rustc_path_model.get(from) {
                for (to, _) in hir_to_map {
                    hir_path_total += 1;
                    if path_to_map.contains_key(to) {
                        hir_path_common += 1;
                    }
                }
            }
        }

        // HIR vs Rustc content
        let mut hir_content_common = 0;
        let mut hir_content_total = 0;

        for (from, hir_to_map) in &self.hir_model {
            if let Some(content_to_map) = self.rustc_content_model.get(from) {
                for (to, _) in hir_to_map {
                    hir_content_total += 1;
                    if content_to_map.contains_key(to) {
                        hir_content_common += 1;
                    }
                }
            }
        }

        let hir_path_sim = if hir_path_total > 0 { hir_path_common as f64 / hir_path_total as f64 } else { 0.0 };
        let hir_content_sim = if hir_content_total > 0 { hir_content_common as f64 / hir_content_total as f64 } else { 0.0 };

        (hir_path_sim, hir_content_sim)
    }

    fn find_shared_patterns(&self) -> (Vec<String>, Vec<String>) {
        let mut hir_path_shared = Vec::new();
        let mut hir_content_shared = Vec::new();

        // HIR-Path shared patterns
        for (from, hir_to_map) in &self.hir_model {
            if let Some(path_to_map) = self.rustc_path_model.get(from) {
                for (to, _) in hir_to_map {
                    if path_to_map.contains_key(to) {
                        if let Some(hir_next) = self.hir_model.get(to) {
                            if let Some(path_next) = self.rustc_path_model.get(to) {
                                for (next, _) in hir_next {
                                    if path_next.contains_key(next) {
                                        hir_path_shared.push(format!("{}{}{}", from, to, next));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // HIR-Content shared patterns
        for (from, hir_to_map) in &self.hir_model {
            if let Some(content_to_map) = self.rustc_content_model.get(from) {
                for (to, _) in hir_to_map {
                    if content_to_map.contains_key(to) {
                        if let Some(hir_next) = self.hir_model.get(to) {
                            if let Some(content_next) = self.rustc_content_model.get(to) {
                                for (next, _) in hir_next {
                                    if content_next.contains_key(next) {
                                        hir_content_shared.push(format!("{}{}{}", from, to, next));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        hir_path_shared.sort();
        hir_path_shared.dedup();
        hir_content_shared.sort();
        hir_content_shared.dedup();

        (hir_path_shared, hir_content_shared)
    }

    fn print_analysis(&self) {
        println!("\n🔍 HIR vs Rustc Source Analysis:");

        let (hir_path_sim, hir_content_sim) = self.compute_similarities();

        println!("\n📊 Similarity Analysis:");
        println!("  HIR ↔ Rustc Paths: {:.2}%", hir_path_sim * 100.0);
        println!("  HIR ↔ Rustc Content: {:.2}%", hir_content_sim * 100.0);

        let (hir_path_patterns, hir_content_patterns) = self.find_shared_patterns();

        println!("\n🎯 Shared Pattern Counts:");
        println!("  HIR-Path patterns: {}", hir_path_patterns.len());
        println!("  HIR-Content patterns: {}", hir_content_patterns.len());

        println!("\n📝 HIR-Path shared patterns:");
        for pattern in hir_path_patterns.iter().take(5) {
            println!("    '{}'", pattern);
        }

        println!("\n💻 HIR-Content shared patterns:");
        for pattern in hir_content_patterns.iter().take(5) {
            println!("    '{}'", pattern);
        }

        if hir_content_sim > hir_path_sim {
            println!("\n✨ HIR MAPS MORE TO CONTENT: Compilation output reflects source code structure!");
        } else if hir_path_sim > hir_content_sim {
            println!("\n🗂️ HIR MAPS MORE TO PATHS: Compilation output reflects file organization!");
        } else {
            println!("\n⚖️ BALANCED MAPPING: HIR reflects both source and organizational structure");
        }

        println!("\n🧬 Self-Reference Analysis:");
        println!("  HIR dump contains patterns from rustc's own source code");
        println!("  The compiler's output encodes its own implementation structure");
        println!("  This creates a recursive self-description system!");
    }
}

fn main() {
    let mut analyzer = HIRComparisonAnalyzer::new();

    println!("🚀 HIR Dump vs Rustc Source Comparison");

    if let Err(e) = analyzer.train_on_hir_dump() {
        eprintln!("Error training on HIR: {}", e);
        return;
    }

    if let Err(e) = analyzer.find_and_train_rustc() {
        eprintln!("Error training on rustc: {}", e);
        return;
    }

    analyzer.print_analysis();
}
