use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Write};

struct MultiFileMarkov {
    // Models per file type/name
    models: HashMap<String, HashMap<char, HashMap<char, u32>>>,
    reverse_models: HashMap<String, HashMap<char, HashMap<char, u32>>>,
    file_counts: HashMap<String, u32>,
}

impl MultiFileMarkov {
    fn new() -> Self {
        Self {
            models: HashMap::new(),
            reverse_models: HashMap::new(),
            file_counts: HashMap::new(),
        }
    }

    fn analyze_rust_ecosystem(&mut self) -> Result<(), String> {
        let file = fs::File::open("/mnt/data1/files.txt")
            .map_err(|e| format!("Failed to open files.txt: {}", e))?;
        let reader = BufReader::new(file);

        let mut processed = 0u64;

        for line in reader.lines() {
            let path = match line {
                Ok(p) => p,
                Err(_) => continue,
            };

            // Filter for any file type
            self.analyze_file_path(&path);
            processed += 1;

            if processed % 50000 == 0 {
                print!(
                    "\r🔍 Processed {} files, {} file types",
                    processed,
                    self.file_counts.len()
                );
                std::io::stdout().flush().unwrap();
            }
        }

        println!(
            "\n✅ Analyzed {} files across {} file types",
            processed,
            self.file_counts.len()
        );
        Ok(())
    }

    fn analyze_file_path(&mut self, path: &str) {
        // Determine file type key
        let file_key = if path.ends_with(".rs") {
            "rs".to_string()
        } else if path.ends_with("Cargo.toml") {
            "Cargo.toml".to_string()
        } else if path.ends_with("Cargo.lock") {
            "Cargo.lock".to_string()
        } else if path.ends_with(".gitignore") {
            ".gitignore".to_string()
        } else if path.ends_with(".gitmodules") {
            ".gitmodules".to_string()
        } else if path.contains("/.git/config") {
            ".git/config".to_string()
        } else if path.ends_with(".md") {
            "md".to_string()
        } else if path.ends_with(".toml") {
            "toml".to_string()
        } else if path.ends_with(".json") {
            "json".to_string()
        } else if path.ends_with(".yaml") || path.ends_with(".yml") {
            "yaml".to_string()
        } else if let Some(ext) = path.split('.').last() {
            if ext.len() <= 10 && ext.chars().all(|c| c.is_alphanumeric()) {
                ext.to_string()
            } else {
                "unknown".to_string()
            }
        } else {
            "no_ext".to_string()
        };

        *self.file_counts.entry(file_key.clone()).or_insert(0) += 1;

        // Train models
        let chars: Vec<char> = path.chars().collect();
        for window in chars.windows(2) {
            let from = window[0];
            let to = window[1];

            // Forward model
            *self
                .models
                .entry(file_key.clone())
                .or_insert_with(HashMap::new)
                .entry(from)
                .or_insert_with(HashMap::new)
                .entry(to)
                .or_insert(0) += 1;

            // Reverse model
            *self
                .reverse_models
                .entry(file_key.clone())
                .or_insert_with(HashMap::new)
                .entry(to)
                .or_insert_with(HashMap::new)
                .entry(from)
                .or_insert(0) += 1;
        }
    }

    fn save_all_models(&self) -> Result<(), String> {
        for (file_type, model) in &self.models {
            let filename = format!(
                "{}_forward.bin",
                file_type.replace("/", "_").replace(".", "_")
            );
            self.save_binary_model(&filename, model)?;
        }

        for (file_type, model) in &self.reverse_models {
            let filename = format!(
                "{}_reverse.bin",
                file_type.replace("/", "_").replace(".", "_")
            );
            self.save_binary_model(&filename, model)?;
        }

        // Save file counts
        let mut file = fs::File::create("file_counts.bin")
            .map_err(|e| format!("Create file_counts error: {}", e))?;

        file.write_all(&(self.file_counts.len() as u32).to_le_bytes())
            .unwrap();
        for (file_type, count) in &self.file_counts {
            file.write_all(&(file_type.len() as u32).to_le_bytes())
                .unwrap();
            file.write_all(file_type.as_bytes()).unwrap();
            file.write_all(&count.to_le_bytes()).unwrap();
        }

        println!(
            "💾 Saved {} forward models, {} reverse models",
            self.models.len(),
            self.reverse_models.len()
        );
        Ok(())
    }

    fn save_binary_model(
        &self,
        filename: &str,
        model: &HashMap<char, HashMap<char, u32>>,
    ) -> Result<(), String> {
        let mut file =
            fs::File::create(filename).map_err(|e| format!("Create {} error: {}", filename, e))?;

        // Count total transitions
        let total_transitions: usize = model.values().map(|m| m.len()).sum();
        file.write_all(&(total_transitions as u32).to_le_bytes())
            .unwrap();

        for (from, to_map) in model {
            for (to, count) in to_map {
                file.write_all(&(*from as u32).to_le_bytes()).unwrap();
                file.write_all(&(*to as u32).to_le_bytes()).unwrap();
                file.write_all(&count.to_le_bytes()).unwrap();
            }
        }

        Ok(())
    }

    fn generate_forward(
        &self,
        model: &HashMap<char, HashMap<char, u32>>,
        start: char,
        len: usize,
    ) -> String {
        let mut result = String::new();
        let mut current = start;
        result.push(current);

        for _ in 0..len {
            if let Some(next_chars) = model.get(&current) {
                if let Some((next_char, _)) = next_chars.iter().max_by_key(|(_, count)| *count) {
                    result.push(*next_char);
                    current = *next_char;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        result
    }

    fn generate_reverse(
        &self,
        model: &HashMap<char, HashMap<char, u32>>,
        end: char,
        len: usize,
    ) -> String {
        let mut result = String::new();
        let mut current = end;
        result.push(current);

        for _ in 0..len {
            if let Some(prev_chars) = model.get(&current) {
                if let Some((prev_char, _)) = prev_chars.iter().max_by_key(|(_, count)| *count) {
                    result.insert(0, *prev_char);
                    current = *prev_char;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        result
    }

    fn print_analysis(&self) {
        println!("\n🌳 Multi-File Markov Analysis:");
        println!("  Total file types: {}", self.file_counts.len());

        let mut sorted_counts: Vec<_> = self.file_counts.iter().collect();
        sorted_counts.sort_by_key(|(_, count)| std::cmp::Reverse(**count));

        println!("\n🔧 Top 15 file types:");
        for (file_type, count) in sorted_counts.iter().take(15) {
            println!("    {}: {} files", file_type, count);
        }

        println!("\n🎯 Generation examples:");
        if let Some(rs_model) = self.models.get("rs") {
            println!(
                "  Rust forward from 's': {}",
                self.generate_forward(rs_model, 's', 15)
            );
        }
        if let Some(rs_reverse) = self.reverse_models.get("rs") {
            println!(
                "  Rust reverse to 's': {}",
                self.generate_reverse(rs_reverse, 's', 15)
            );
        }
        if let Some(cargo_model) = self.models.get("Cargo.toml") {
            println!(
                "  Cargo.toml forward from 'C': {}",
                self.generate_forward(cargo_model, 'C', 15)
            );
        }
    }
}

fn main() {
    let mut analyzer = MultiFileMarkov::new();

    println!("🚀 Analyzing ALL file types from 33.9M files...");

    if let Err(e) = analyzer.analyze_rust_ecosystem() {
        eprintln!("Error: {}", e);
        return;
    }

    analyzer.print_analysis();

    if let Err(e) = analyzer.save_all_models() {
        eprintln!("Error saving models: {}", e);
    }
}
