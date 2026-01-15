use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Write};

struct VennMarkovAnalyzer {
    // Model A: Rustc file paths
    path_model: HashMap<char, HashMap<char, u32>>,
    path_total: u64,

    // Model B: Rustc file content
    content_model: HashMap<char, HashMap<char, u32>>,
    content_total: u64,

    // Model C: Combined (paths + content)
    combined_model: HashMap<char, HashMap<char, u32>>,
    combined_total: u64,

    rustc_files: Vec<String>,
}

impl VennMarkovAnalyzer {
    fn new() -> Self {
        Self {
            path_model: HashMap::new(),
            path_total: 0,
            content_model: HashMap::new(),
            content_total: 0,
            combined_model: HashMap::new(),
            combined_total: 0,
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
                if path.contains("rustc_") && path.ends_with(".rs") && path.len() < 200 {
                    self.rustc_files.push(path);
                }
            }
        }

        println!("  Found {} rustc source files", self.rustc_files.len());
        Ok(())
    }

    fn train_models(&mut self) -> Result<(), String> {
        println!("📖 Training three models...");

        let mut files_processed = 0;

        for (i, file_path) in self.rustc_files.iter().enumerate() {
            if i >= 100 {
                break;
            } // Limit to first 100 files

            // Model A: Train on file path
            let path_chars: Vec<char> = file_path.chars().collect();
            for window in path_chars.windows(2) {
                *self
                    .path_model
                    .entry(window[0])
                    .or_insert_with(HashMap::new)
                    .entry(window[1])
                    .or_insert(0) += 1;
                self.path_total += 1;

                // Model C: Also add to combined
                *self
                    .combined_model
                    .entry(window[0])
                    .or_insert_with(HashMap::new)
                    .entry(window[1])
                    .or_insert(0) += 1;
                self.combined_total += 1;
            }

            // Model B: Train on file content (if readable)
            let full_path = if file_path.starts_with("./") {
                format!("/mnt/data1/{}", &file_path[2..])
            } else {
                format!("/mnt/data1/{}", file_path)
            };

            if let Ok(content) = fs::read_to_string(&full_path) {
                if content.len() < 50000 {
                    // Reasonable size limit
                    let content_chars: Vec<char> = content.chars().collect();
                    for window in content_chars.windows(2) {
                        *self
                            .content_model
                            .entry(window[0])
                            .or_insert_with(HashMap::new)
                            .entry(window[1])
                            .or_insert(0) += 1;
                        self.content_total += 1;

                        // Model C: Also add to combined
                        *self
                            .combined_model
                            .entry(window[0])
                            .or_insert_with(HashMap::new)
                            .entry(window[1])
                            .or_insert(0) += 1;
                        self.combined_total += 1;
                    }
                    files_processed += 1;
                }
            }

            if i % 20 == 0 {
                print!("\r  Processed {} files", i);
                std::io::stdout().flush().unwrap();
            }
        }

        println!("\n  Path model: {} transitions", self.path_total);
        println!(
            "  Content model: {} transitions from {} files",
            self.content_total, files_processed
        );
        println!("  Combined model: {} transitions", self.combined_total);

        // Save models
        self.save_models()?;

        Ok(())
    }

    fn save_models(&self) -> Result<(), String> {
        use std::io::Write;

        // Save rustc path model
        let mut file = fs::File::create("rustc_path_model.bin")
            .map_err(|e| format!("Create path model error: {}", e))?;

        file.write_all(&(self.path_model.len() as u32).to_le_bytes())
            .unwrap();
        for (from, to_map) in &self.path_model {
            for (to, count) in to_map {
                file.write_all(&(*from as u32).to_le_bytes()).unwrap();
                file.write_all(&(*to as u32).to_le_bytes()).unwrap();
                file.write_all(&count.to_le_bytes()).unwrap();
            }
        }

        // Save rustc content model
        let mut file = fs::File::create("rustc_content_model.bin")
            .map_err(|e| format!("Create content model error: {}", e))?;

        file.write_all(&(self.content_model.len() as u32).to_le_bytes())
            .unwrap();
        for (from, to_map) in &self.content_model {
            for (to, count) in to_map {
                file.write_all(&(*from as u32).to_le_bytes()).unwrap();
                file.write_all(&(*to as u32).to_le_bytes()).unwrap();
                file.write_all(&count.to_le_bytes()).unwrap();
            }
        }

        println!("💾 Saved rustc_path_model.bin and rustc_content_model.bin");

        Ok(())
    }

    fn compute_venn_overlaps(&self) -> (usize, usize, usize, usize) {
        // A only, B only, A∩B, A∪B
        let mut a_only = 0;
        let mut b_only = 0;
        let mut intersection = 0;
        let mut union_set = std::collections::HashSet::new();

        // Get all transitions from both models
        for (from, to_map) in &self.path_model {
            for (to, _) in to_map {
                let transition = (*from, *to);
                union_set.insert(transition);

                if self
                    .content_model
                    .get(from)
                    .and_then(|m| m.get(to))
                    .is_some()
                {
                    intersection += 1;
                } else {
                    a_only += 1;
                }
            }
        }

        for (from, to_map) in &self.content_model {
            for (to, _) in to_map {
                let transition = (*from, *to);
                union_set.insert(transition);

                if !self.path_model.get(from).and_then(|m| m.get(to)).is_some() {
                    b_only += 1;
                }
            }
        }

        (a_only, b_only, intersection, union_set.len())
    }

    fn find_model_specific_patterns(&self) -> (Vec<String>, Vec<String>, Vec<String>) {
        let mut path_only = Vec::new();
        let mut content_only = Vec::new();
        let mut shared = Vec::new();

        // Find 3-char patterns unique to each model
        for (from, path_to_map) in &self.path_model {
            for (to, _) in path_to_map {
                if let Some(path_next_map) = self.path_model.get(to) {
                    for (next, _) in path_next_map {
                        let pattern = format!("{}{}{}", from, to, next);

                        // Check if this pattern exists in content model
                        let in_content = self
                            .content_model
                            .get(from)
                            .and_then(|m| m.get(to))
                            .and_then(|_| self.content_model.get(to))
                            .and_then(|m| m.get(next))
                            .is_some();

                        if in_content {
                            shared.push(pattern);
                        } else {
                            path_only.push(pattern);
                        }
                    }
                }
            }
        }

        // Find content-only patterns
        for (from, content_to_map) in &self.content_model {
            for (to, _) in content_to_map {
                if let Some(content_next_map) = self.content_model.get(to) {
                    for (next, _) in content_next_map {
                        let pattern = format!("{}{}{}", from, to, next);

                        let in_path = self
                            .path_model
                            .get(from)
                            .and_then(|m| m.get(to))
                            .and_then(|_| self.path_model.get(to))
                            .and_then(|m| m.get(next))
                            .is_some();

                        if !in_path {
                            content_only.push(pattern);
                        }
                    }
                }
            }
        }

        path_only.sort();
        path_only.dedup();
        content_only.sort();
        content_only.dedup();
        shared.sort();
        shared.dedup();

        (path_only, content_only, shared)
    }

    fn print_venn_analysis(&self) {
        println!("\n🔍 Venn Diagram Analysis of Three Markov Models:");

        let (a_only, b_only, intersection, union_size) = self.compute_venn_overlaps();

        println!("\n📊 Transition Overlaps:");
        println!("  Path-only transitions: {}", a_only);
        println!("  Content-only transitions: {}", b_only);
        println!("  Shared transitions (A∩B): {}", intersection);
        println!("  Total unique transitions (A∪B): {}", union_size);

        let overlap_ratio = intersection as f64 / union_size as f64;
        println!("  Overlap ratio: {:.2}%", overlap_ratio * 100.0);

        let (path_patterns, content_patterns, shared_patterns) =
            self.find_model_specific_patterns();

        println!("\n🎯 Pattern Analysis:");
        println!("  Path-only patterns: {}", path_patterns.len());
        println!("  Content-only patterns: {}", content_patterns.len());
        println!("  Shared patterns: {}", shared_patterns.len());

        println!("\n📝 Sample Path-only patterns:");
        for pattern in path_patterns.iter().take(5) {
            println!("    '{}'", pattern);
        }

        println!("\n💻 Sample Content-only patterns:");
        for pattern in content_patterns.iter().take(5) {
            println!("    '{}'", pattern);
        }

        println!("\n🤝 Sample Shared patterns:");
        for pattern in shared_patterns.iter().take(5) {
            println!("    '{}'", pattern);
        }

        // Combined model analysis
        let combined_unique = self.combined_model.len();
        println!("\n🔄 Combined Model:");
        println!("  Unique states: {}", combined_unique);
        println!("  Total transitions: {}", self.combined_total);

        if overlap_ratio > 0.3 {
            println!("\n✨ HIGH OVERLAP: Rustc paths and content share significant structure!");
        } else if overlap_ratio > 0.1 {
            println!("\n🤔 MODERATE OVERLAP: Some structural similarity between paths and content");
        } else {
            println!("\n❌ LOW OVERLAP: Paths and content have distinct structures");
        }

        println!("\n🧬 Self-Reference Implications:");
        println!("  The compiler's file paths encode its own source structure");
        println!("  Shared patterns suggest deep structural correspondence");
        println!("  Combined model captures both organizational and implementation patterns");
    }
}

fn main() {
    let mut analyzer = VennMarkovAnalyzer::new();

    println!("🚀 Venn Diagram Analysis: Rustc Paths vs Content vs Combined");

    if let Err(e) = analyzer.find_rustc_files() {
        eprintln!("Error finding files: {}", e);
        return;
    }

    if let Err(e) = analyzer.train_models() {
        eprintln!("Error training models: {}", e);
        return;
    }

    analyzer.print_venn_analysis();
}
