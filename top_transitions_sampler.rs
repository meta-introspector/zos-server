use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
struct TransitionSample {
    from_char: char,
    to_char: char,
    count: u32,
    model_file: String,
}

fn main() {
    println!("🔍 Sampling top 10 highest transitions across all models...");

    let mut all_transitions = Vec::new();
    let mut models_processed = 0;

    // Process forward models
    if let Ok(entries) = fs::read_dir("models/forward") {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "bin") {
                    if let Ok(model) = load_model(&path) {
                        let filename = path.file_name().unwrap().to_string_lossy().to_string();

                        for (from_char, transitions) in model {
                            for (to_char, count) in transitions {
                                all_transitions.push(TransitionSample {
                                    from_char,
                                    to_char,
                                    count,
                                    model_file: filename.clone(),
                                });
                            }
                        }

                        models_processed += 1;
                        if models_processed % 1000 == 0 {
                            println!("Processed {} models...", models_processed);
                        }
                    }
                }
            }
        }
    }

    // Process reverse models
    if let Ok(entries) = fs::read_dir("models/reverse") {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "bin") {
                    if let Ok(model) = load_model(&path) {
                        let filename = path.file_name().unwrap().to_string_lossy().to_string();

                        for (from_char, transitions) in model {
                            for (to_char, count) in transitions {
                                all_transitions.push(TransitionSample {
                                    from_char,
                                    to_char,
                                    count,
                                    model_file: filename.clone(),
                                });
                            }
                        }

                        models_processed += 1;
                        if models_processed % 1000 == 0 {
                            println!("Processed {} models...", models_processed);
                        }
                    }
                }
            }
        }
    }

    println!("📊 Total transitions found: {}", all_transitions.len());
    println!("📁 Models processed: {}", models_processed);

    // Sort by count descending
    all_transitions.sort_by(|a, b| b.count.cmp(&a.count));

    println!("\n🏆 TOP 10 HIGHEST TRANSITIONS ACROSS ALL MODELS:");
    println!("Rank | From → To | Count     | Model File");
    println!("-----|-----------|-----------|----------------------------------");

    for (i, transition) in all_transitions.iter().take(10).enumerate() {
        let from_display = if transition.from_char.is_control() {
            format!("\\x{:02x}", transition.from_char as u8)
        } else {
            transition.from_char.to_string()
        };

        let to_display = if transition.to_char.is_control() {
            format!("\\x{:02x}", transition.to_char as u8)
        } else {
            transition.to_char.to_string()
        };

        println!(
            "{:4} | {:4} → {:4} | {:9} | {}",
            i + 1,
            from_display,
            to_display,
            transition.count,
            transition.model_file.chars().take(30).collect::<String>()
        );
    }

    // Aggregate statistics
    let mut char_pair_totals: HashMap<(char, char), u64> = HashMap::new();
    for transition in &all_transitions {
        *char_pair_totals
            .entry((transition.from_char, transition.to_char))
            .or_insert(0) += transition.count as u64;
    }

    let mut aggregated: Vec<_> = char_pair_totals.into_iter().collect();
    aggregated.sort_by(|a, b| b.1.cmp(&a.1));

    println!("\n🌍 TOP 10 TRANSITIONS AGGREGATED ACROSS ALL MODELS:");
    println!("Rank | From → To | Total Count | Frequency");
    println!("-----|-----------|-------------|----------");

    let total_transitions: u64 = aggregated.iter().map(|(_, count)| count).sum();

    for (i, ((from_char, to_char), total_count)) in aggregated.iter().take(10).enumerate() {
        let from_display = if from_char.is_control() {
            format!("\\x{:02x}", *from_char as u8)
        } else {
            from_char.to_string()
        };

        let to_display = if to_char.is_control() {
            format!("\\x{:02x}", *to_char as u8)
        } else {
            to_char.to_string()
        };

        let frequency = (*total_count as f64 / total_transitions as f64) * 100.0;

        println!(
            "{:4} | {:4} → {:4} | {:11} | {:6.2}%",
            i + 1,
            from_display,
            to_display,
            total_count,
            frequency
        );
    }

    println!("\n📈 SUMMARY:");
    println!("Total unique transitions: {}", aggregated.len());
    println!("Total transition instances: {}", total_transitions);
    println!("Models analyzed: {}", models_processed);
}

fn load_model(
    path: &Path,
) -> Result<HashMap<char, HashMap<char, u32>>, Box<dyn std::error::Error>> {
    let data = fs::read(path)?;
    let mut model = HashMap::new();

    if data.len() < 4 {
        return Ok(model);
    }

    // Read total transitions count
    let total_transitions = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;

    let mut offset = 4;
    for _ in 0..total_transitions {
        if offset + 12 > data.len() {
            break;
        }

        let from_char = char::from_u32(u32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]))
        .unwrap_or('\0');

        let to_char = char::from_u32(u32::from_le_bytes([
            data[offset + 4],
            data[offset + 5],
            data[offset + 6],
            data[offset + 7],
        ]))
        .unwrap_or('\0');

        let count = u32::from_le_bytes([
            data[offset + 8],
            data[offset + 9],
            data[offset + 10],
            data[offset + 11],
        ]);

        model
            .entry(from_char)
            .or_insert_with(HashMap::new)
            .insert(to_char, count);

        offset += 12;
    }

    Ok(model)
}
