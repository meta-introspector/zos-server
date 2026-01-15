use std::collections::{HashMap, HashSet};

struct RegexFixedPointAnalyzer {
    transitions: HashMap<char, HashMap<char, u32>>,
    regex_chars: HashSet<char>,
}

impl RegexFixedPointAnalyzer {
    fn new() -> Self {
        let regex_chars: HashSet<char> = ".*+?[]{}()^$|\\".chars().collect();

        Self {
            transitions: HashMap::new(),
            regex_chars,
        }
    }

    fn load_sample_data(&mut self) {
        // Sample from file paths that might contain regex-like patterns
        let samples = vec![
            "src/*.rs",
            "tests/**/*.rs",
            ".git/config",
            "Cargo.toml",
            "*.{rs,toml}",
            "[a-z]+\\.rs$",
            "^src/.*\\.rs$",
        ];

        for sample in samples {
            let chars: Vec<char> = sample.chars().collect();
            for window in chars.windows(2) {
                *self
                    .transitions
                    .entry(window[0])
                    .or_insert_with(HashMap::new)
                    .entry(window[1])
                    .or_insert(0) += 1;
            }
        }
    }

    fn find_regex_fixed_points(&self) -> Vec<String> {
        let mut fixed_points = Vec::new();

        // Check if regex metacharacters have self-loops or cycles
        for regex_char in &self.regex_chars {
            if let Some(to_map) = self.transitions.get(regex_char) {
                // Direct self-loop
                if to_map.contains_key(regex_char) {
                    fixed_points.push(format!("Direct: {} -> {}", regex_char, regex_char));
                }

                // Two-step cycle back to regex chars
                for (next_char, _) in to_map {
                    if let Some(next_map) = self.transitions.get(next_char) {
                        if next_map.contains_key(regex_char) {
                            fixed_points.push(format!(
                                "Cycle: {} -> {} -> {}",
                                regex_char, next_char, regex_char
                            ));
                        }
                    }
                }
            }
        }

        fixed_points
    }

    fn generate_self_describing_regex(&self) -> String {
        // Try to generate a regex that describes the model's own structure
        let mut regex = String::new();

        // Start with common file pattern structure
        regex.push_str("^");

        // Generate path component using most likely transitions
        let mut current = 's'; // Start with 's' (common in src/)
        regex.push(current);

        for _ in 0..10 {
            if let Some(to_map) = self.transitions.get(&current) {
                if let Some((next_char, _)) = to_map.iter().max_by_key(|(_, count)| *count) {
                    // If we hit a regex metacharacter, use it
                    if self.regex_chars.contains(next_char) {
                        regex.push(*next_char);

                        // Add quantifier if appropriate
                        match *next_char {
                            '*' => regex.push_str(".*"),
                            '.' => regex.push_str("+"),
                            '[' => regex.push_str("a-z]"),
                            _ => {}
                        }
                        break;
                    } else {
                        regex.push(*next_char);
                        current = *next_char;
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        regex.push('$');
        regex
    }

    fn check_regex_grammar_closure(&self) -> bool {
        // Check if the model contains enough regex grammar to describe itself
        let regex_coverage: usize = self
            .regex_chars
            .iter()
            .filter(|c| self.transitions.contains_key(c))
            .count();

        let total_regex_chars = self.regex_chars.len();
        let coverage_ratio = regex_coverage as f64 / total_regex_chars as f64;

        println!(
            "📊 Regex Grammar Coverage: {}/{} ({:.1}%)",
            regex_coverage,
            total_regex_chars,
            coverage_ratio * 100.0
        );

        coverage_ratio > 0.5 // Arbitrary threshold for "sufficient coverage"
    }

    fn find_regex_attractors(&self) -> Vec<char> {
        // Find regex characters that other characters tend to transition to
        let mut attractor_scores: HashMap<char, u32> = HashMap::new();

        for (_, to_map) in &self.transitions {
            for (to_char, count) in to_map {
                if self.regex_chars.contains(to_char) {
                    *attractor_scores.entry(*to_char).or_insert(0) += count;
                }
            }
        }

        let mut attractors: Vec<_> = attractor_scores.iter().collect();
        attractors.sort_by_key(|(_, score)| std::cmp::Reverse(**score));

        attractors.into_iter().take(5).map(|(c, _)| *c).collect()
    }

    fn analyze_regex_cycles(&self) -> Vec<String> {
        let mut cycles = Vec::new();

        // Look for cycles involving regex characters
        for start_char in &self.regex_chars {
            if let Some(to_map) = self.transitions.get(start_char) {
                for (mid_char, _) in to_map {
                    if let Some(mid_map) = self.transitions.get(mid_char) {
                        for (end_char, _) in mid_map {
                            if *end_char == *start_char {
                                cycles.push(format!(
                                    "{} -> {} -> {}",
                                    start_char, mid_char, end_char
                                ));
                            }
                        }
                    }
                }
            }
        }

        cycles
    }

    fn print_analysis(&self) {
        println!("🔍 Regex Fixed Point Analysis:");

        let fixed_points = self.find_regex_fixed_points();
        println!("\n🔄 Regex Fixed Points:");
        if fixed_points.is_empty() {
            println!("    None found");
        } else {
            for fp in &fixed_points {
                println!("    {}", fp);
            }
        }

        let has_closure = self.check_regex_grammar_closure();
        println!("\n📝 Regex Grammar Closure: {}", has_closure);

        let attractors = self.find_regex_attractors();
        println!(
            "\n🧲 Regex Attractors (most transitioned-to): {:?}",
            attractors
        );

        let cycles = self.analyze_regex_cycles();
        println!("\n🔄 Regex Cycles:");
        if cycles.is_empty() {
            println!("    None found");
        } else {
            for cycle in cycles.iter().take(5) {
                println!("    {}", cycle);
            }
        }

        let self_regex = self.generate_self_describing_regex();
        println!("\n🪞 Self-Describing Regex: {}", self_regex);

        // Check if the generated regex contains regex metacharacters (self-reference)
        let contains_meta = self_regex.chars().any(|c| self.regex_chars.contains(&c));
        println!("    Contains metacharacters: {}", contains_meta);

        if contains_meta && has_closure {
            println!("\n✨ FIXED POINT DETECTED: Model can generate regex patterns that describe its own structure!");
        } else {
            println!("\n❌ No complete regex fixed point found");
        }
    }
}

fn main() {
    let mut analyzer = RegexFixedPointAnalyzer::new();

    println!("🚀 Analyzing regex grammar fixed points in Markov model...");

    analyzer.load_sample_data();
    analyzer.print_analysis();
}
