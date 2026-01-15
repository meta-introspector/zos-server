use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
struct ByteValue {
    used_by_rustc: bool,
    used_by_system: bool,
    is_new: bool,
    is_duplicate: bool,
    markov_score: f64,
    value_score: f64,
}

struct ValueAssigner {
    rustc_patterns: HashSet<String>,
    system_patterns: HashSet<String>,
    known_hashes: HashSet<u64>,
    markov_transitions: HashMap<(char, char), f64>,
}

impl ValueAssigner {
    fn new() -> Self {
        Self {
            rustc_patterns: Self::load_rustc_patterns(),
            system_patterns: Self::load_system_patterns(),
            known_hashes: HashSet::new(),
            markov_transitions: HashMap::new(),
        }
    }

    fn assign_byte_values(&mut self, content: &str, _file_path: &str) -> Vec<ByteValue> {
        let chars: Vec<char> = content.chars().collect();
        let mut values = Vec::new();

        for (i, &ch) in chars.iter().enumerate() {
            let context = self.get_context(&chars, i);
            let markov_node = if i > 0 {
                (chars[i - 1], ch)
            } else {
                ('\0', ch)
            };

            let byte_value = ByteValue {
                used_by_rustc: self.is_used_by_rustc(&context),
                used_by_system: self.is_used_by_system(&context),
                is_new: self.is_new_pattern(&context),
                is_duplicate: self.is_duplicate(&context),
                markov_score: self.get_markov_score(markov_node),
                value_score: 0.0, // Will calculate
            };

            values.push(byte_value);
        }

        // Calculate final value scores
        for value in &mut values {
            value.value_score = self.calculate_value_score(value);
        }

        values
    }

    fn get_context(&self, chars: &[char], pos: usize) -> String {
        let start = pos.saturating_sub(10);
        let end = (pos + 10).min(chars.len());
        chars[start..end].iter().collect()
    }

    fn is_used_by_rustc(&self, context: &str) -> bool {
        self.rustc_patterns
            .iter()
            .any(|pattern| context.contains(pattern))
    }

    fn is_used_by_system(&self, context: &str) -> bool {
        self.system_patterns
            .iter()
            .any(|pattern| context.contains(pattern))
    }

    fn is_new_pattern(&self, context: &str) -> bool {
        let hash = self.hash_context(context);
        !self.known_hashes.contains(&hash)
    }

    fn is_duplicate(&mut self, context: &str) -> bool {
        let hash = self.hash_context(context);
        if self.known_hashes.contains(&hash) {
            true
        } else {
            self.known_hashes.insert(hash);
            false
        }
    }

    fn get_markov_score(&self, node: (char, char)) -> f64 {
        self.markov_transitions.get(&node).copied().unwrap_or(0.0)
    }

    fn calculate_value_score(&self, value: &ByteValue) -> f64 {
        let mut score = 0.0;

        if value.used_by_rustc {
            score += 10.0;
        }
        if value.used_by_system {
            score += 5.0;
        }
        if value.is_new {
            score += 3.0;
        }
        if value.is_duplicate {
            score -= 2.0;
        }

        score += value.markov_score * 2.0;
        score
    }

    fn load_rustc_patterns() -> HashSet<String> {
        ["fn ", "struct ", "impl ", "use ", "mod ", "pub "]
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    fn load_system_patterns() -> HashSet<String> {
        ["main(", "println!", "std::", "Vec<", "HashMap"]
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    fn hash_context(&self, context: &str) -> u64 {
        context.chars().fold(0, |acc, c| acc ^ (c as u64))
    }
}

fn main() {
    println!("💎 Byte Value Assignment for 1.4M Files");
    println!("🔍 Analyzing: rustc usage, system usage, novelty, duplicates");

    let mut assigner = ValueAssigner::new();

    // Sample analysis
    let sample_code = "fn main() { println!(\"Hello\"); }";
    let values = assigner.assign_byte_values(sample_code, "test.rs");

    println!("📊 Sample byte values:");
    for (i, value) in values.iter().take(10).enumerate() {
        println!(
            "  Byte {}: score={:.1}, rustc={}, sys={}, new={}",
            i, value.value_score, value.used_by_rustc, value.used_by_system, value.is_new
        );
    }

    println!("\n🎯 Each Markov node now has usage value!");
}
