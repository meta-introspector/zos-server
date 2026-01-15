use std::collections::{HashMap, HashSet};
use std::fs;

struct MarkovAnalyzer {
    transitions: HashMap<char, HashMap<char, u32>>,
    states: HashSet<char>,
    total_transitions: u64,
}

impl MarkovAnalyzer {
    fn new() -> Self {
        Self {
            transitions: HashMap::new(),
            states: HashSet::new(),
            total_transitions: 0,
        }
    }

    fn load_from_simple_model(&mut self) -> Result<(), String> {
        // Use our existing simple model data
        let sample_text = "src/main.rs src/lib.rs Cargo.toml README.md .gitignore";

        let chars: Vec<char> = sample_text.chars().collect();
        for window in chars.windows(2) {
            let from = window[0];
            let to = window[1];

            self.states.insert(from);
            self.states.insert(to);

            *self
                .transitions
                .entry(from)
                .or_insert_with(HashMap::new)
                .entry(to)
                .or_insert(0) += 1;

            self.total_transitions += 1;
        }
        Ok(())
    }

    fn find_fixed_points(&self) -> Vec<char> {
        let mut fixed_points = Vec::new();

        for (from, to_map) in &self.transitions {
            if let Some(count) = to_map.get(from) {
                // Self-transition exists
                let total_from: u32 = to_map.values().sum();
                let self_prob = *count as f64 / total_from as f64;

                if self_prob > 0.5 {
                    // Strong self-loop
                    fixed_points.push(*from);
                }
            }
        }

        fixed_points
    }

    fn extract_regex_patterns(&self) -> Vec<String> {
        let mut patterns = Vec::new();

        // Find common sequences
        for (from, to_map) in &self.transitions {
            let total: u32 = to_map.values().sum();

            for (to, count) in to_map {
                let prob = *count as f64 / total as f64;

                if prob > 0.8 {
                    // High probability transition
                    // Check if this continues a pattern
                    if let Some(next_map) = self.transitions.get(to) {
                        let next_total: u32 = next_map.values().sum();

                        for (next_to, next_count) in next_map {
                            let next_prob = *next_count as f64 / next_total as f64;

                            if next_prob > 0.8 {
                                patterns.push(format!("{}{}{}", from, to, next_to));
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

    fn compute_state_complexity(&self) -> HashMap<char, f64> {
        let mut complexity = HashMap::new();

        for (state, to_map) in &self.transitions {
            let num_transitions = to_map.len();
            let total_count: u32 = to_map.values().sum();

            // Entropy calculation
            let mut entropy = 0.0;
            for count in to_map.values() {
                let prob = *count as f64 / total_count as f64;
                entropy -= prob * prob.log2();
            }

            complexity.insert(*state, entropy);
        }

        complexity
    }

    fn find_strongly_connected_components(&self) -> Vec<Vec<char>> {
        // Simple cycle detection
        let mut components = Vec::new();
        let mut visited = HashSet::new();

        for start_state in &self.states {
            if visited.contains(start_state) {
                continue;
            }

            let mut component = Vec::new();
            let mut stack = vec![*start_state];
            let mut local_visited = HashSet::new();

            while let Some(current) = stack.pop() {
                if local_visited.contains(&current) {
                    continue;
                }

                local_visited.insert(current);
                component.push(current);

                if let Some(to_map) = self.transitions.get(&current) {
                    for next_state in to_map.keys() {
                        if !local_visited.contains(next_state) {
                            stack.push(*next_state);
                        }
                    }
                }
            }

            if component.len() > 1 {
                components.push(component);
                for state in components.last().unwrap() {
                    visited.insert(*state);
                }
            }
        }

        components
    }

    fn check_self_reference(&self) -> bool {
        // Check if the model contains patterns that describe itself
        let model_chars: HashSet<char> = "markov".chars().collect();
        let state_overlap: usize = self.states.intersection(&model_chars).count();

        state_overlap > 0
    }

    fn generate_dot_graph(&self) -> String {
        let mut dot = String::from("digraph MarkovModel {\n");
        dot.push_str("  rankdir=LR;\n");
        dot.push_str("  node [shape=circle];\n");

        // Add nodes with complexity coloring
        let complexity = self.compute_state_complexity();
        for state in &self.states {
            let comp = complexity.get(state).unwrap_or(&0.0);
            let color = if *comp > 2.0 {
                "red"
            } else if *comp > 1.0 {
                "orange"
            } else {
                "lightblue"
            };

            let state_str = if *state == ' ' {
                "SPACE".to_string()
            } else {
                state.to_string()
            };
            dot.push_str(&format!(
                "  \"{}\" [fillcolor={}, style=filled];\n",
                state_str, color
            ));
        }

        // Add edges with weights
        for (from, to_map) in &self.transitions {
            let total: u32 = to_map.values().sum();

            for (to, count) in to_map {
                let prob = *count as f64 / total as f64;
                let weight = if prob > 0.5 { "bold" } else { "normal" };

                let from_label = if *from == ' ' {
                    "SPACE"
                } else {
                    &from.to_string()
                };
                let to_label = if *to == ' ' { "SPACE" } else { &to.to_string() };

                dot.push_str(&format!(
                    "  \"{}\" -> \"{}\" [label=\"{:.2}\", style={}];\n",
                    from_label, to_label, prob, weight
                ));
            }
        }

        dot.push_str("}\n");
        dot
    }

    fn print_analysis(&self) {
        println!("🔍 Markov Model Analysis:");
        println!("  States: {}", self.states.len());
        println!("  Total transitions: {}", self.total_transitions);
        println!(
            "  Transition density: {:.2}",
            self.total_transitions as f64 / (self.states.len() * self.states.len()) as f64
        );

        let fixed_points = self.find_fixed_points();
        println!("\n🔄 Fixed Points (self-loops): {:?}", fixed_points);

        let patterns = self.extract_regex_patterns();
        println!("\n📝 Regex Patterns (high probability sequences):");
        for pattern in patterns.iter().take(5) {
            println!("    {}", pattern);
        }

        let complexity = self.compute_state_complexity();
        let mut comp_sorted: Vec<_> = complexity.iter().collect();
        comp_sorted.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());

        println!("\n🧠 State Complexity (entropy):");
        for (state, comp) in comp_sorted.iter().take(5) {
            let state_display = if **state == ' ' {
                "SPACE"
            } else {
                &state.to_string()
            };
            println!("    {}: {:.2} bits", state_display, comp);
        }

        let components = self.find_strongly_connected_components();
        println!("\n🔗 Strongly Connected Components: {}", components.len());
        for (i, component) in components.iter().take(3).enumerate() {
            println!("    Component {}: {:?}", i + 1, component);
        }

        println!("\n🪞 Self-Reference: {}", self.check_self_reference());

        println!("\n📊 Graph Properties:");
        println!(
            "  Average out-degree: {:.2}",
            self.transitions.values().map(|m| m.len()).sum::<usize>() as f64
                / self.states.len() as f64
        );
    }
}

fn main() {
    let mut analyzer = MarkovAnalyzer::new();

    println!("🚀 Analyzing Markov model structure...");

    if let Err(e) = analyzer.load_from_simple_model() {
        eprintln!("Error: {}", e);
        return;
    }

    analyzer.print_analysis();

    // Generate visualization
    let dot_graph = analyzer.generate_dot_graph();
    if let Err(e) = fs::write("markov_graph.dot", dot_graph) {
        eprintln!("Failed to write DOT file: {}", e);
    } else {
        println!("\n💾 Graph saved to markov_graph.dot");
        println!("   Run: dot -Tpng markov_graph.dot -o markov_graph.png");
    }
}
