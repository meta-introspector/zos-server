use std::collections::HashMap;
use std::fs;
use std::io::Read;

struct ModelClassifier {
    models: Vec<ModelInfo>,
    classifications: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone)]
struct ModelInfo {
    filename: String,
    total_transitions: u32,
    most_common_transition: (u32, u32, u32), // (from, to, count)
    file_size: u64,
    classification: String,
}

impl ModelClassifier {
    fn new() -> Self {
        Self {
            models: Vec::new(),
            classifications: HashMap::new(),
        }
    }

    fn scan_binary_models(&mut self) -> Result<(), String> {
        println!("🔍 Scanning binary models...");

        let entries = fs::read_dir(".").map_err(|e| format!("Cannot read directory: {}", e))?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "bin") {
                let filename = path.file_name().unwrap().to_string_lossy().to_string();

                match self.analyze_binary_model(&filename) {
                    Ok(model_info) => {
                        self.models.push(model_info);
                        print!(".");
                    }
                    Err(_) => print!("x"), // Skip corrupted files
                }
            }
        }

        println!("\n✅ Scanned {} models", self.models.len());
        Ok(())
    }

    fn analyze_binary_model(&self, filename: &str) -> Result<ModelInfo, String> {
        let mut file =
            fs::File::open(filename).map_err(|e| format!("Cannot open {}: {}", filename, e))?;

        let file_size = file.metadata().unwrap().len();

        // Read transition count
        let mut buffer = [0u8; 4];
        file.read_exact(&mut buffer)
            .map_err(|_| "Cannot read transition count")?;
        let total_transitions = u32::from_le_bytes(buffer);

        // Find most common transition
        let mut max_count = 0;
        let mut most_common = (0u32, 0u32, 0u32);

        // Read up to 100 transitions to find the most common
        for _ in 0..std::cmp::min(total_transitions, 100) {
            if file.read_exact(&mut buffer).is_ok() {
                let from = u32::from_le_bytes(buffer);
                if file.read_exact(&mut buffer).is_ok() {
                    let to = u32::from_le_bytes(buffer);
                    if file.read_exact(&mut buffer).is_ok() {
                        let count = u32::from_le_bytes(buffer);
                        if count > max_count {
                            max_count = count;
                            most_common = (from, to, count);
                        }
                    }
                }
            }
        }

        let classification = self.classify_model(filename, total_transitions, most_common);

        Ok(ModelInfo {
            filename: filename.to_string(),
            total_transitions,
            most_common_transition: most_common,
            file_size,
            classification,
        })
    }

    fn classify_model(
        &self,
        filename: &str,
        total_transitions: u32,
        most_common: (u32, u32, u32),
    ) -> String {
        // Classify by filename patterns and characteristics
        if filename.contains("rustc") {
            "rustc_compiler".to_string()
        } else if filename.contains("model") {
            match total_transitions {
                0..=1000 => "small_model".to_string(),
                1001..=10000 => "medium_model".to_string(),
                _ => "large_model".to_string(),
            }
        } else if filename.contains("report") {
            "reporting_system".to_string()
        } else {
            "unknown_model".to_string()
        }
    }

    fn organize_by_classification(&mut self) {
        println!("📊 Organizing models by classification...");

        for model in &self.models {
            self.classifications
                .entry(model.classification.clone())
                .or_insert_with(Vec::new)
                .push(model.filename.clone());
        }

        // Create a lookup map for transition counts
        let count_map: HashMap<String, u32> = self
            .models
            .iter()
            .map(|m| (m.filename.clone(), m.total_transitions))
            .collect();

        // Sort each classification by total count (descending)
        for (_, filenames) in &mut self.classifications {
            filenames.sort_by(|a, b| {
                let count_a = count_map.get(a).unwrap_or(&0);
                let count_b = count_map.get(b).unwrap_or(&0);
                count_b.cmp(count_a)
            });
        }
    }

    fn create_directory_structure(&self) -> Result<(), String> {
        println!("📁 Creating organized directory structure...");

        // Create base directory
        fs::create_dir_all("organized_models")
            .map_err(|e| format!("Cannot create directory: {}", e))?;

        for (classification, filenames) in &self.classifications {
            let class_dir = format!("organized_models/{}", classification);
            fs::create_dir_all(&class_dir)
                .map_err(|e| format!("Cannot create {}: {}", class_dir, e))?;

            println!("  📂 {}: {} models", classification, filenames.len());

            for filename in filenames {
                let src = filename;
                let dst = format!("{}/{}", class_dir, filename);

                // Copy file to organized location
                if let Err(_) = fs::copy(src, &dst) {
                    // If copy fails, create a symlink instead
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::symlink;
                        let abs_src = fs::canonicalize(src).unwrap();
                        symlink(abs_src, dst).ok();
                    }
                }
            }
        }

        Ok(())
    }

    fn print_analysis(&self) {
        println!("\n🌌 Model Classification Analysis:");

        // Sort models by total transitions (descending)
        let mut sorted_models = self.models.clone();
        sorted_models.sort_by(|a, b| b.total_transitions.cmp(&a.total_transitions));

        println!("\n🏆 Top 10 models by total transitions:");
        for (i, model) in sorted_models.iter().take(10).enumerate() {
            let (from_char, to_char) = (
                if model.most_common_transition.0 <= 127 {
                    char::from(model.most_common_transition.0 as u8)
                } else {
                    '?'
                },
                if model.most_common_transition.1 <= 127 {
                    char::from(model.most_common_transition.1 as u8)
                } else {
                    '?'
                },
            );

            println!(
                "  {}. {} ({}) - {} transitions, most common: '{}→{}' ({}x)",
                i + 1,
                model.filename,
                model.classification,
                model.total_transitions,
                from_char,
                to_char,
                model.most_common_transition.2
            );
        }

        println!("\n📊 Classification Summary:");
        let mut class_summary: Vec<_> = self.classifications.iter().collect();
        class_summary.sort_by(|a, b| b.1.len().cmp(&a.1.len()));

        for (classification, filenames) in class_summary {
            let total_transitions: u32 = filenames
                .iter()
                .filter_map(|f| self.models.iter().find(|m| &m.filename == f))
                .map(|m| m.total_transitions)
                .sum();

            println!(
                "  {}: {} models, {} total transitions",
                classification,
                filenames.len(),
                total_transitions
            );
        }

        println!("\n✅ Models organized in ./organized_models/ directory");
    }
}

fn main() {
    let mut classifier = ModelClassifier::new();

    println!("🚀 Model Classification and Organization System");

    if let Err(e) = classifier.scan_binary_models() {
        eprintln!("Scan failed: {}", e);
        return;
    }

    classifier.organize_by_classification();

    if let Err(e) = classifier.create_directory_structure() {
        eprintln!("Organization failed: {}", e);
        return;
    }

    classifier.print_analysis();
}
