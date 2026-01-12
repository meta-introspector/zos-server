use std::collections::HashMap;
use std::fs;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct SimpleMarkovModel {
    transitions: HashMap<char, HashMap<char, u32>>,
    total_chars: u64,
    model_metadata: ModelMetadata,
}

#[derive(Serialize, Deserialize)]
pub struct ModelMetadata {
    created_at: String,
    repos_scanned: Vec<String>,
    total_files: u32,
    model_version: String,
}

impl SimpleMarkovModel {
    pub fn new() -> Self {
        Self {
            transitions: HashMap::new(),
            total_chars: 0,
            model_metadata: ModelMetadata {
                created_at: chrono::Utc::now().to_rfc3339(),
                repos_scanned: Vec::new(),
                total_files: 0,
                model_version: "1.0.0".to_string(),
            },
        }
    }

    pub fn save_model(&self, path: &str) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Serialization error: {}", e))?;

        fs::write(path, json)
            .map_err(|e| format!("Write error: {}", e))?;

        println!("💾 Model saved to: {}", path);
        Ok(())
    }

    pub fn load_model(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Read error: {}", e))?;

        let model: Self = serde_json::from_str(&content)
            .map_err(|e| format!("Deserialization error: {}", e))?;

        println!("📂 Model loaded from: {}", path);
        Ok(model)
    }

    pub fn train_from_repos(&mut self) -> Result<(), String> {
        let repos_dir = "/mnt/data1/meta-introspector/repos";

        if let Ok(entries) = fs::read_dir(repos_dir) {
            for entry in entries.flatten() {
                if entry.file_type().map(|ft| ft.is_symlink()).unwrap_or(false) {
                    if let Ok(real_path) = fs::read_link(entry.path()) {
                        self.scan_repo(&real_path)?;
                    }
                }
            }
        }

        println!("📊 Markov model trained on {} characters", self.total_chars);
        Ok(())
    }

    fn scan_repo(&mut self, repo_path: &std::path::Path) -> Result<(), String> {
        if let Ok(entries) = fs::read_dir(repo_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "rs") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        self.train_on_text(&content);
                        self.model_metadata.total_files += 1;
                    }
                }
            }
        }
        self.model_metadata.repos_scanned.push(repo_path.to_string_lossy().to_string());
                }
            }
        }
        Ok(())
    }

    fn train_on_text(&mut self, text: &str) {
        let chars: Vec<char> = text.chars().collect();

        for window in chars.windows(2) {
            let from = window[0];
            let to = window[1];

            *self.transitions
                .entry(from)
                .or_insert_with(HashMap::new)
                .entry(to)
                .or_insert(0) += 1;

            self.total_chars += 1;
        }
    }

    pub fn get_transition_probability(&self, from: char, to: char) -> f64 {
        if let Some(from_map) = self.transitions.get(&from) {
            let from_total: u32 = from_map.values().sum();
            if let Some(&count) = from_map.get(&to) {
                return count as f64 / from_total as f64;
            }
        }
        0.0
    }

    pub fn generate_text(&self, start: char, length: usize) -> String {
        let mut result = String::new();
        let mut current = start;
        result.push(current);

        for _ in 0..length {
            if let Some(transitions) = self.transitions.get(&current) {
                // Simple: pick most likely next character
                if let Some((&next_char, _)) = transitions.iter().max_by_key(|(_, &count)| count) {
                    result.push(next_char);
                    current = next_char;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        result
    }

    pub fn find_word_stats(&self, word: &str) -> (u32, Vec<(String, u32)>) {
        let word_chars: Vec<char> = word.chars().collect();
        let mut word_count = 0;
        let mut following_patterns = HashMap::new();

        // Look for the word pattern in our transitions
        for (from_char, to_map) in &self.transitions {
            if *from_char == word_chars[0] {
                // Found potential start of word, check if full word follows
                if self.check_word_sequence(&word_chars, *from_char) {
                    word_count += 1;

                    // Find what comes after the word
                    if let Some(last_char) = word_chars.last() {
                        if let Some(after_map) = self.transitions.get(last_char) {
                            for (next_char, count) in after_map {
                                let pattern = format!("{}{}", word, next_char);
                                *following_patterns.entry(pattern).or_insert(0) += count;
                            }
                        }
                    }
                }
            }
        }

        let mut sorted_patterns: Vec<_> = following_patterns.into_iter().collect();
        sorted_patterns.sort_by_key(|(_, count)| std::cmp::Reverse(*count));

        (word_count, sorted_patterns.into_iter().take(10).collect())
    }

    fn check_word_sequence(&self, word_chars: &[char], start_char: char) -> bool {
        if word_chars.is_empty() || word_chars[0] != start_char {
            return false;
        }

        let mut current = start_char;
        for &next_char in &word_chars[1..] {
            if let Some(transitions) = self.transitions.get(&current) {
                if transitions.contains_key(&next_char) {
                    current = next_char;
                } else {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }
        println!("🔍 Markov Model Statistics:");
        println!("  Total characters processed: {}", self.total_chars);
        println!("  Unique characters: {}", self.transitions.len());

        // Show top transitions
        let mut all_transitions: Vec<_> = self.transitions.iter()
            .flat_map(|(from, to_map)| {
                to_map.iter().map(move |(to, count)| ((*from, *to), *count))
            })
            .collect();
        all_transitions.sort_by_key(|(_, count)| std::cmp::Reverse(*count));

        println!("  Top 10 character transitions:");
        for ((from, to), count) in all_transitions.iter().take(10) {
            println!("    '{}' → '{}': {} times", from, to, count);
        }
    }
}

fn main() {
    let mut model = SimpleMarkovModel::new();

    println!("🚀 Building Markov model from all repositories...");

    if let Err(e) = model.train_from_repos() {
        eprintln!("Error: {}", e);
        return;
    }

    // Save the model
    let model_path = "/mnt/data1/meta-introspector/markov_model.json";
    if let Err(e) = model.save_model(model_path) {
        eprintln!("Failed to save model: {}", e);
    }

    model.print_stats();


    // Find 'enum' statistics
    let (enum_count, enum_patterns) = model.find_word_stats("enum");
    println!("\n🔍 'enum' Statistics:");
    println!("  Found 'enum' sequences: {} times", enum_count);
    println!("  Top patterns following 'enum':");
    for (pattern, count) in enum_patterns {
        println!("    '{}': {} times", pattern, count);
    }
}
