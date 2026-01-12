use std::collections::HashMap;

struct Kleene2Markov2Godel {
    // Kleene algebra operations
    kleene_ops: HashMap<char, String>,

    // Markov transitions
    markov: HashMap<char, HashMap<char, f64>>,

    // Gödel numbering
    godel_map: HashMap<String, u64>,
    reverse_godel: HashMap<u64, String>,

    next_godel: u64,
}

impl Kleene2Markov2Godel {
    fn new() -> Self {
        let mut kleene_ops = HashMap::new();
        kleene_ops.insert('*', "kleene_star".to_string());
        kleene_ops.insert('+', "kleene_plus".to_string());
        kleene_ops.insert('?', "optional".to_string());
        kleene_ops.insert('|', "union".to_string());
        kleene_ops.insert('.', "concat".to_string());

        Self {
            kleene_ops,
            markov: HashMap::new(),
            godel_map: HashMap::new(),
            reverse_godel: HashMap::new(),
            next_godel: 1,
        }
    }

    fn kleene_to_markov(&mut self, pattern: &str) -> HashMap<char, HashMap<char, f64>> {
        let mut transitions = HashMap::new();
        let chars: Vec<char> = pattern.chars().collect();

        for i in 0..chars.len() {
            let current = chars[i];

            // Handle Kleene operations
            match current {
                '*' => {
                    if i > 0 {
                        let prev = chars[i-1];
                        // Self-loop for Kleene star
                        *transitions.entry(prev).or_insert_with(HashMap::new)
                            .entry(prev).or_insert(0.0) += 0.7;

                        // Optional continuation
                        if i + 1 < chars.len() {
                            let next = chars[i+1];
                            *transitions.entry(prev).or_insert_with(HashMap::new)
                                .entry(next).or_insert(0.0) += 0.3;
                        }
                    }
                },
                '+' => {
                    if i > 0 {
                        let prev = chars[i-1];
                        // At least one, then optional repeats
                        *transitions.entry(prev).or_insert_with(HashMap::new)
                            .entry(prev).or_insert(0.0) += 0.5;
                    }
                },
                '?' => {
                    if i > 0 && i + 1 < chars.len() {
                        let prev = chars[i-1];
                        let next = chars[i+1];
                        // Optional: skip or take
                        *transitions.entry(prev).or_insert_with(HashMap::new)
                            .entry(next).or_insert(0.0) += 0.5;
                    }
                },
                _ => {
                    // Regular character transition
                    if i + 1 < chars.len() {
                        let next = chars[i+1];
                        if !self.kleene_ops.contains_key(&next) {
                            *transitions.entry(current).or_insert_with(HashMap::new)
                                .entry(next).or_insert(0.0) += 1.0;
                        }
                    }
                }
            }
        }

        transitions
    }

    fn markov_to_godel(&mut self, transitions: &HashMap<char, HashMap<char, f64>>) -> Vec<u64> {
        let mut godel_numbers = Vec::new();

        for (from, to_map) in transitions {
            for (to, prob) in to_map {
                // Create transition string
                let transition = format!("{}->{}:{:.2}", from, to, prob);

                // Assign Gödel number if not exists
                if !self.godel_map.contains_key(&transition) {
                    self.godel_map.insert(transition.clone(), self.next_godel);
                    self.reverse_godel.insert(self.next_godel, transition.clone());
                    self.next_godel += 1;
                }

                godel_numbers.push(*self.godel_map.get(&transition).unwrap());
            }
        }

        godel_numbers
    }

    fn godel_to_kleene(&self, godel_num: u64) -> Option<String> {
        if let Some(transition) = self.reverse_godel.get(&godel_num) {
            // Extract pattern from transition
            if let Some(arrow_pos) = transition.find("->") {
                let from = &transition[..arrow_pos];
                let rest = &transition[arrow_pos + 2..];

                if let Some(colon_pos) = rest.find(':') {
                    let to = &rest[..colon_pos];
                    let prob: f64 = rest[colon_pos + 1..].parse().unwrap_or(0.0);

                    // Convert back to Kleene based on probability
                    if from == to && prob > 0.5 {
                        return Some(format!("{}*", from));
                    } else if prob > 0.8 {
                        return Some(format!("{}{}", from, to));
                    } else {
                        return Some(format!("{}?{}", from, to));
                    }
                }
            }
        }
        None
    }

    fn find_self_reference(&self) -> Option<u64> {
        // Look for Gödel number that encodes its own structure
        for (godel_num, transition) in &self.reverse_godel {
            let godel_str = godel_num.to_string();

            // Check if transition contains its own Gödel number
            if transition.contains(&godel_str) {
                return Some(*godel_num);
            }

            // Check if Gödel number appears in Kleene pattern
            if let Some(kleene) = self.godel_to_kleene(*godel_num) {
                if kleene.contains(&godel_str) {
                    return Some(*godel_num);
                }
            }
        }
        None
    }

    fn compute_path_godel(&self, path: &str) -> u64 {
        // Compute composite Gödel number using prime factorization method
        let primes = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71];
        let mut godel_number = 1u64;

        for (i, c) in path.chars().enumerate() {
            if i < primes.len() {
                let char_code = (c as u32) as u64;
                godel_number *= primes[i].pow(char_code as u32);

                // Prevent overflow by using modular arithmetic
                if godel_number > 1_000_000_000 {
                    godel_number %= 1_000_000_007; // Large prime
                }
            }
        }

        godel_number
    }

    fn compute_fixed_point(&mut self) -> Option<String> {
        // Try to find a pattern that generates itself
        let test_patterns = vec!["a*", "a+b*", "x?y*", "(ab)*"];

        for pattern in test_patterns {
            let markov = self.kleene_to_markov(pattern);
            let godel_nums = self.markov_to_godel(&markov);

            // Check if any Gödel number appears in the pattern
            for godel_num in godel_nums {
                if pattern.contains(&godel_num.to_string()) {
                    return Some(format!("Fixed point: {} ↔ Gödel #{}", pattern, godel_num));
                }
            }
        }

        None
    }
        // Try to find a pattern that generates itself
        let test_patterns = vec!["a*", "a+b*", "x?y*", "(ab)*"];

        for pattern in test_patterns {
            let markov = self.kleene_to_markov(pattern);
            let godel_nums = self.markov_to_godel(&markov);

            // Check if any Gödel number appears in the pattern
            for godel_num in godel_nums {
                if pattern.contains(&godel_num.to_string()) {
                    return Some(format!("Fixed point: {} ↔ Gödel #{}", pattern, godel_num));
                }
            }
        }

        None
    }

    fn print_analysis(&mut self) {
        println!("🔄 Kleene → Markov → Gödel Analysis:");

        let test_pattern = "src/helloworld.rs";
        println!("\n📝 Computing Gödel number for: {}", test_pattern);

        // Kleene to Markov
        let markov = self.kleene_to_markov(test_pattern);
        println!("  Markov transitions: {}", markov.len());

        for (from, to_map) in &markov {
            for (to, prob) in to_map {
                println!("    {} → {} (p={:.2})", from, to, prob);
            }
        }

        // Markov to Gödel
        let godel_nums = self.markov_to_godel(&markov);
        println!("\n🔢 Gödel numbers: {:?}", godel_nums);

        // Compute composite Gödel number for entire path
        let composite_godel = self.compute_path_godel(test_pattern);
        println!("\n✨ Gödel number of '{}': {}", test_pattern, composite_godel);

        // Show character-by-character encoding
        println!("\n📊 Character breakdown:");
        for (i, c) in test_pattern.chars().enumerate() {
            let char_godel = (c as u32) as u64;
            println!("    [{}] '{}' → Gödel #{}", i, c, char_godel);
        }
    }
}

fn main() {
    let mut transformer = Kleene2Markov2Godel::new();

    println!("🚀 Kleene Algebra → Markov Chains → Gödel Numbers");

    transformer.print_analysis();
}
