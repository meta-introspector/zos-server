use std::fs;
use std::io::Read;
use std::collections::HashMap;

struct TransitionMatrix {
    models: Vec<ModelSample>,
}

#[derive(Debug)]
struct ModelSample {
    filename: String,
    top_transitions: Vec<(char, char, u32)>,
}

impl TransitionMatrix {
    fn new() -> Self {
        Self { models: Vec::new() }
    }

    fn sample_top_models(&mut self) -> Result<(), String> {
        println!("🔍 Sampling top 3 transitions from each model...");

        let entries = fs::read_dir(".")
            .map_err(|e| format!("Cannot read directory: {}", e))?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "bin") {
                let filename = path.file_name().unwrap().to_string_lossy().to_string();

                if let Ok(transitions) = self.extract_top_transitions(&filename) {
                    self.models.push(ModelSample {
                        filename,
                        top_transitions: transitions,
                    });
                }
            }
        }

        println!("✅ Sampled {} models", self.models.len());
        Ok(())
    }

    fn extract_top_transitions(&self, filename: &str) -> Result<Vec<(char, char, u32)>, String> {
        let mut file = fs::File::open(filename)
            .map_err(|e| format!("Cannot open {}: {}", filename, e))?;

        // Read transition count
        let mut buffer = [0u8; 4];
        file.read_exact(&mut buffer).map_err(|_| "Cannot read count")?;
        let total_transitions = u32::from_le_bytes(buffer);

        let mut transitions = Vec::new();

        // Read all transitions and find top 3
        for _ in 0..std::cmp::min(total_transitions, 1000) { // Limit to prevent memory issues
            if file.read_exact(&mut buffer).is_ok() {
                let from = u32::from_le_bytes(buffer);
                if file.read_exact(&mut buffer).is_ok() {
                    let to = u32::from_le_bytes(buffer);
                    if file.read_exact(&mut buffer).is_ok() {
                        let count = u32::from_le_bytes(buffer);

                        let from_char = if from <= 127 { char::from(from as u8) } else { '?' };
                        let to_char = if to <= 127 { char::from(to as u8) } else { '?' };

                        transitions.push((from_char, to_char, count));
                    }
                }
            }
        }

        // Sort by count and take top 3
        transitions.sort_by(|a, b| b.2.cmp(&a.2));
        transitions.truncate(3);

        Ok(transitions)
    }

    fn create_transition_matrix(&self) -> Vec<Vec<String>> {
        let mut matrix = Vec::new();

        // Header row
        let mut header = vec!["Model".to_string()];
        for i in 1..=3 {
            header.push(format!("Top{}_From", i));
            header.push(format!("Top{}_To", i));
            header.push(format!("Top{}_Count", i));
        }
        matrix.push(header);

        // Data rows
        for model in &self.models {
            let mut row = vec![model.filename.clone()];

            for i in 0..3 {
                if i < model.top_transitions.len() {
                    let (from, to, count) = model.top_transitions[i];
                    row.push(format!("'{}'", from));
                    row.push(format!("'{}'", to));
                    row.push(count.to_string());
                } else {
                    row.push("-".to_string());
                    row.push("-".to_string());
                    row.push("0".to_string());
                }
            }

            matrix.push(row);
        }

        matrix
    }

    fn print_matrix(&self) {
        let matrix = self.create_transition_matrix();

        println!("\n📊 Transition Matrix (Top 3 from each model):");
        println!("{}", "=".repeat(120));

        // Print header
        if let Some(header) = matrix.first() {
            println!("{:<25} {:<8} {:<8} {:<8} {:<8} {:<8} {:<8} {:<8} {:<8} {:<8}",
                header[0], header[1], header[2], header[3],
                header[4], header[5], header[6], header[7], header[8], header[9]);
            println!("{}", "-".repeat(120));
        }

        // Print top 20 rows
        for row in matrix.iter().skip(1).take(20) {
            if row.len() >= 10 {
                println!("{:<25} {:<8} {:<8} {:<8} {:<8} {:<8} {:<8} {:<8} {:<8} {:<8}",
                    &row[0][..std::cmp::min(24, row[0].len())],
                    row[1], row[2], row[3], row[4], row[5], row[6], row[7], row[8], row[9]);
            }
        }

        println!("{}", "=".repeat(120));
        println!("Showing top 20 of {} models", matrix.len() - 1);
    }

    fn save_matrix_csv(&self) -> Result<(), String> {
        let matrix = self.create_transition_matrix();
        let mut csv_content = String::new();

        for row in &matrix {
            csv_content.push_str(&row.join(","));
            csv_content.push('\n');
        }

        fs::write("transition_matrix.csv", csv_content)
            .map_err(|e| format!("Cannot write CSV: {}", e))?;

        println!("💾 Matrix saved to transition_matrix.csv");
        Ok(())
    }

    fn analyze_patterns(&self) {
        println!("\n🔍 Pattern Analysis:");

        let mut char_frequency: HashMap<char, u32> = HashMap::new();
        let mut transition_frequency: HashMap<(char, char), u32> = HashMap::new();

        for model in &self.models {
            for (from, to, count) in &model.top_transitions {
                *char_frequency.entry(*from).or_insert(0) += 1;
                *char_frequency.entry(*to).or_insert(0) += 1;
                *transition_frequency.entry((*from, *to)).or_insert(0) += std::cmp::min(*count, 1000000); // Prevent overflow
            }
        }

        // Most common characters across all models
        let mut char_sorted: Vec<_> = char_frequency.iter().collect();
        char_sorted.sort_by_key(|(_, count)| std::cmp::Reverse(**count));

        println!("  Most common characters:");
        for (c, count) in char_sorted.iter().take(10) {
            println!("    '{}': appears in {} models", c, count);
        }

        // Most common transitions across all models
        let mut trans_sorted: Vec<_> = transition_frequency.iter().collect();
        trans_sorted.sort_by_key(|(_, count)| std::cmp::Reverse(**count));

        println!("  Most common transitions:");
        for ((from, to), count) in trans_sorted.iter().take(5) {
            println!("    '{}' → '{}': total count {}", from, to, count);
        }
    }
}

fn main() {
    let mut matrix = TransitionMatrix::new();

    println!("🚀 Creating Transition Matrix from Binary Models");

    if let Err(e) = matrix.sample_top_models() {
        eprintln!("Sampling failed: {}", e);
        return;
    }

    matrix.print_matrix();
    matrix.analyze_patterns();

    if let Err(e) = matrix.save_matrix_csv() {
        eprintln!("Save failed: {}", e);
    }
}
