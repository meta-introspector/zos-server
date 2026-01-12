use std::collections::HashMap;
use std::fs;
use std::path::Path;

fn main() {
    println!("🔍 Analyzing reverse transitions to find file endings...");

    let mut ending_patterns = HashMap::new();
    let mut models_processed = 0;

    // Process reverse models to find what leads to path endings
    if let Ok(entries) = fs::read_dir("models/reverse") {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "bin") {
                    if let Ok(model) = load_model(&path) {
                        // Look for transitions into common file ending characters
                        for ending_char in ['.', 's', 't', 'c', 'h', 'o', 'x', 'l', 'm', 'n'] {
                            if let Some(transitions) = model.get(&ending_char) {
                                for (from_char, count) in transitions {
                                    let pattern = format!("{}{}", from_char, ending_char);
                                    *ending_patterns.entry(pattern).or_insert(0) += *count as u64;
                                }
                            }
                        }

                        models_processed += 1;
                        if models_processed % 5000 == 0 {
                            println!("Processed {} reverse models...", models_processed);
                        }
                    }
                }
            }
        }
    }

    let mut sorted_endings: Vec<_> = ending_patterns.into_iter().collect();
    sorted_endings.sort_by(|a, b| b.1.cmp(&a.1));

    println!("\n🎯 TOP 20 FILE ENDING PATTERNS (from reverse transitions):");
    println!("Pattern | Count     | Likely File Types");
    println!("--------|-----------|------------------");

    for (i, (pattern, count)) in sorted_endings.iter().take(20).enumerate() {
        let file_types = match pattern.as_str() {
            "rs" => ".rs (Rust source)",
            "js" => ".js (JavaScript)",
            "ts" => ".ts (TypeScript)",
            "py" => ".py (Python)",
            "go" => ".go (Go source)",
            "cc" => ".cc (C++ source)",
            "cpp" => ".cpp (C++ source)",
            "hpp" => ".hpp (C++ header)",
            "txt" => ".txt (Text files)",
            "md" => ".md (Markdown)",
            "json" => ".json (JSON data)",
            "toml" => ".toml (TOML config)",
            "yaml" => ".yaml (YAML config)",
            "xml" => ".xml (XML data)",
            "html" => ".html (HTML)",
            "css" => ".css (Stylesheets)",
            "sh" => ".sh (Shell scripts)",
            "pl" => ".pl (Perl scripts)",
            "rb" => ".rb (Ruby scripts)",
            "php" => ".php (PHP scripts)",
            _ => "Various file types"
        };

        println!("{:7} | {:9} | {}", pattern, count, file_types);
    }

    println!("\n📊 SUMMARY:");
    println!("Reverse models processed: {}", models_processed);
    println!("Unique ending patterns: {}", sorted_endings.len());
    println!("Total ending transitions: {}", sorted_endings.iter().map(|(_, c)| c).sum::<u64>());
}

fn load_model(path: &Path) -> Result<HashMap<char, HashMap<char, u32>>, Box<dyn std::error::Error>> {
    let data = fs::read(path)?;
    let mut model = HashMap::new();

    if data.len() < 4 { return Ok(model); }

    let total_transitions = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
    let mut offset = 4;

    for _ in 0..total_transitions {
        if offset + 12 > data.len() { break; }

        let from_char = char::from_u32(u32::from_le_bytes([
            data[offset], data[offset + 1], data[offset + 2], data[offset + 3]
        ])).unwrap_or('\0');

        let to_char = char::from_u32(u32::from_le_bytes([
            data[offset + 4], data[offset + 5], data[offset + 6], data[offset + 7]
        ])).unwrap_or('\0');

        let count = u32::from_le_bytes([
            data[offset + 8], data[offset + 9], data[offset + 10], data[offset + 11]
        ]);

        model.entry(from_char).or_insert_with(HashMap::new).insert(to_char, count);
        offset += 12;
    }

    Ok(model)
}
