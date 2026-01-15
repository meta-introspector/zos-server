use std::collections::HashMap;
use std::fs;

struct SelfImage {
    own_patterns: HashMap<String, f64>,
    own_markov: HashMap<(char, char), f64>,
    own_signature: u64,
    self_knowledge: SelfKnowledge,
}

#[derive(Debug)]
struct SelfKnowledge {
    total_bytes: usize,
    unique_patterns: usize,
    complexity_score: f64,
    self_reference_count: usize,
}

impl SelfImage {
    fn analyze_self() -> Self {
        println!("🪞 SELF-INTROSPECTION: Analyzing own code...");

        // Read own source code
        let self_code = fs::read_to_string("byte_value_assigner.rs")
            .unwrap_or_else(|_| "fn main() {}".to_string());

        let own_patterns = Self::extract_patterns(&self_code);
        let own_markov = Self::build_markov(&self_code);
        let own_signature = Self::compute_signature(&self_code);

        let self_knowledge = SelfKnowledge {
            total_bytes: self_code.len(),
            unique_patterns: own_patterns.len(),
            complexity_score: own_markov.len() as f64,
            self_reference_count: self_code.matches("self").count(),
        };

        println!("📊 Self-analysis complete:");
        println!("   Bytes: {}", self_knowledge.total_bytes);
        println!("   Patterns: {}", self_knowledge.unique_patterns);
        println!("   Self-refs: {}", self_knowledge.self_reference_count);

        Self {
            own_patterns,
            own_markov,
            own_signature,
            self_knowledge,
        }
    }

    fn compare_to_external(&self, external_code: &str) -> ComparisonResult {
        let ext_patterns = Self::extract_patterns(external_code);
        let ext_markov = Self::build_markov(external_code);
        let ext_signature = Self::compute_signature(external_code);

        let similarity = self.calculate_similarity(&ext_patterns, &ext_markov);
        let novelty = self.calculate_novelty(&ext_patterns);
        let value = self.calculate_value(similarity, novelty);

        ComparisonResult {
            similarity,
            novelty,
            value,
            is_self_like: similarity > 0.7,
        }
    }

    fn extract_patterns(code: &str) -> HashMap<String, f64> {
        let mut patterns = HashMap::new();

        for line in code.lines() {
            let words: Vec<&str> = line.split_whitespace().collect();
            for window in words.windows(2) {
                let pattern = format!("{} {}", window[0], window[1]);
                *patterns.entry(pattern).or_insert(0.0) += 1.0;
            }
        }

        patterns
    }

    fn build_markov(code: &str) -> HashMap<(char, char), f64> {
        let chars: Vec<char> = code.chars().collect();
        let mut markov = HashMap::new();

        for window in chars.windows(2) {
            let pair = (window[0], window[1]);
            *markov.entry(pair).or_insert(0.0) += 1.0;
        }

        markov
    }

    fn compute_signature(code: &str) -> u64 {
        code.chars().fold(0, |acc, c| acc ^ (c as u64))
    }

    fn calculate_similarity(
        &self,
        ext_patterns: &HashMap<String, f64>,
        ext_markov: &HashMap<(char, char), f64>,
    ) -> f64 {
        let pattern_overlap = self
            .own_patterns
            .keys()
            .filter(|k| ext_patterns.contains_key(*k))
            .count() as f64;

        let markov_overlap = self
            .own_markov
            .keys()
            .filter(|k| ext_markov.contains_key(*k))
            .count() as f64;

        (pattern_overlap + markov_overlap)
            / (self.own_patterns.len() + self.own_markov.len()) as f64
    }

    fn calculate_novelty(&self, ext_patterns: &HashMap<String, f64>) -> f64 {
        let novel_patterns = ext_patterns
            .keys()
            .filter(|k| !self.own_patterns.contains_key(*k))
            .count() as f64;

        novel_patterns / ext_patterns.len() as f64
    }

    fn calculate_value(&self, similarity: f64, novelty: f64) -> f64 {
        // Value = balance of familiarity and novelty
        similarity * 0.3 + novelty * 0.7
    }
}

#[derive(Debug)]
struct ComparisonResult {
    similarity: f64,
    novelty: f64,
    value: f64,
    is_self_like: bool,
}

fn main() {
    println!("🧠 Self-Aware Code Analysis System");
    println!("{}", "=".repeat(40));

    // Step 1: Build self-image
    let self_image = SelfImage::analyze_self();

    // Step 2: Compare to external code samples
    let samples = vec![
        "fn hello() { println!(\"world\"); }",
        "struct Data { value: i32 }",
        "use std::collections::HashMap;",
    ];

    println!("\n🔍 Comparing external code to self:");
    for (i, sample) in samples.iter().enumerate() {
        let result = self_image.compare_to_external(sample);
        println!(
            "Sample {}: sim={:.2}, nov={:.2}, val={:.2}, self_like={}",
            i + 1,
            result.similarity,
            result.novelty,
            result.value,
            result.is_self_like
        );
    }

    println!("\n🎯 System now knows itself and can value other code!");
}
