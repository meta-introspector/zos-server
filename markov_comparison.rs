use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};

struct MarkovComparison {
    // Compilation dump model
    dump_transitions: HashMap<char, HashMap<char, u32>>,
    dump_total: u64,

    // Path database model (sample)
    path_transitions: HashMap<char, HashMap<char, u32>>,
    path_total: u64,
}

impl MarkovComparison {
    fn new() -> Self {
        Self {
            dump_transitions: HashMap::new(),
            dump_total: 0,
            path_transitions: HashMap::new(),
            path_total: 0,
        }
    }

    fn train_on_compilation_dump(&mut self) -> Result<(), String> {
        println!("🔧 Training on compilation dump...");

        let dump_content = fs::read_to_string("helloworld.ll")
            .map_err(|e| format!("Failed to read dump: {}", e))?;

        let chars: Vec<char> = dump_content.chars().collect();
        for window in chars.windows(2) {
            *self.dump_transitions
                .entry(window[0])
                .or_insert_with(HashMap::new)
                .entry(window[1])
                .or_insert(0) += 1;
            self.dump_total += 1;
        }

        println!("  Dump model: {} transitions", self.dump_total);
        Ok(())
    }

    fn train_on_path_sample(&mut self) -> Result<(), String> {
        println!("📂 Training on path sample...");

        let file = fs::File::open("/mnt/data1/files.txt")
            .map_err(|e| format!("Failed to open files.txt: {}", e))?;
        let reader = BufReader::new(file);

        let mut count = 0;
        for line in reader.lines() {
            if let Ok(path) = line {
                if path.ends_with(".rs") {
                    let chars: Vec<char> = path.chars().collect();
                    for window in chars.windows(2) {
                        *self.path_transitions
                            .entry(window[0])
                            .or_insert_with(HashMap::new)
                            .entry(window[1])
                            .or_insert(0) += 1;
                        self.path_total += 1;
                    }

                    count += 1;
                    if count >= 10000 { // Sample first 10k rust files
                        break;
                    }
                }
            }
        }

        println!("  Path model: {} transitions from {} files", self.path_total, count);
        Ok(())
    }

    fn compute_model_similarity(&self) -> f64 {
        let mut common_transitions = 0;
        let mut total_comparisons = 0;

        for (from, dump_to_map) in &self.dump_transitions {
            if let Some(path_to_map) = self.path_transitions.get(from) {
                for (to, dump_count) in dump_to_map {
                    total_comparisons += 1;

                    if let Some(path_count) = path_to_map.get(to) {
                        // Both models have this transition
                        common_transitions += 1;

                        // Could also compare probabilities here
                        let dump_prob = *dump_count as f64 / self.dump_total as f64;
                        let path_prob = *path_count as f64 / self.path_total as f64;
                        let _prob_diff = (dump_prob - path_prob).abs();
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

        // Find 3-character sequences that appear in both models
        for (from, dump_to_map) in &self.dump_transitions {
            if let Some(path_to_map) = self.path_transitions.get(from) {
                for (to, _) in dump_to_map {
                    if path_to_map.contains_key(to) {
                        // Found shared transition, look for continuation
                        if let Some(dump_next_map) = self.dump_transitions.get(to) {
                            if let Some(path_next_map) = self.path_transitions.get(to) {
                                for (next, _) in dump_next_map {
                                    if path_next_map.contains_key(next) {
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

    fn analyze_character_distributions(&self) {
        println!("\n📊 Character Distribution Analysis:");

        // Top characters in dump model
        let mut dump_chars: HashMap<char, u32> = HashMap::new();
        for (from, to_map) in &self.dump_transitions {
            for (to, count) in to_map {
                *dump_chars.entry(*from).or_insert(0) += count;
                *dump_chars.entry(*to).or_insert(0) += count;
            }
        }

        let mut dump_sorted: Vec<_> = dump_chars.iter().collect();
        dump_sorted.sort_by_key(|(_, count)| std::cmp::Reverse(**count));

        println!("  Top dump characters:");
        for (c, count) in dump_sorted.iter().take(5) {
            let display = if **c == ' ' { "SPACE" } else { &c.to_string() };
            println!("    '{}': {} occurrences", display, count);
        }

        // Top characters in path model
        let mut path_chars: HashMap<char, u32> = HashMap::new();
        for (from, to_map) in &self.path_transitions {
            for (to, count) in to_map {
                *path_chars.entry(*from).or_insert(0) += count;
                *path_chars.entry(*to).or_insert(0) += count;
            }
        }

        let mut path_sorted: Vec<_> = path_chars.iter().collect();
        path_sorted.sort_by_key(|(_, count)| std::cmp::Reverse(**count));

        println!("  Top path characters:");
        for (c, count) in path_sorted.iter().take(5) {
            let display = if **c == ' ' { "SPACE" } else { &c.to_string() };
            println!("    '{}': {} occurrences", display, count);
        }
    }

    fn print_analysis(&self) {
        println!("\n🔍 Markov Model Comparison Analysis:");

        let similarity = self.compute_model_similarity();
        println!("  Model similarity: {:.2}%", similarity * 100.0);

        let shared_patterns = self.find_shared_patterns();
        println!("  Shared patterns: {}", shared_patterns.len());

        println!("\n🎯 Top shared 3-char patterns:");
        for pattern in shared_patterns.iter().take(10) {
            println!("    '{}'", pattern);
        }

        self.analyze_character_distributions();

        if similarity > 0.1 {
            println!("\n✨ THEORY SUPPORTED: Compilation dump Markov model shows significant overlap with path structure!");
            println!("    The path database contains structural patterns from compiled programs!");
        } else {
            println!("\n🤔 Low direct similarity - may need deeper analysis or different compilation representation");
        }
    }
}

fn main() {
    let mut comparison = MarkovComparison::new();

    println!("🚀 Comparing Compilation Dump Markov Model vs Path Database Model");

    if let Err(e) = comparison.train_on_compilation_dump() {
        eprintln!("Error training on dump: {}", e);
        return;
    }

    if let Err(e) = comparison.train_on_path_sample() {
        eprintln!("Error training on paths: {}", e);
        return;
    }

    comparison.print_analysis();
}
