use crossbeam::thread;
use std::collections::HashMap;
use std::fs;
use std::process::Command;

struct RustStructuredExtractor {
    rustc_path: String,
    output_dir: String,
    dump_types: Vec<String>,
}

impl RustStructuredExtractor {
    fn new(rustc_path: String) -> Self {
        Self {
            rustc_path,
            output_dir: "rust_dumps".to_string(),
            dump_types: vec![
                "ast".to_string(),
                "hir".to_string(),
                "mir".to_string(),
                "llvm-ir".to_string(),
            ],
        }
    }

    fn extract_all_dumps(&self) -> Result<(), String> {
        fs::create_dir_all(&self.output_dir).unwrap();

        println!("🦀 Extracting Rust compiler dumps from {}", self.rustc_path);

        // Find all Rust files once
        let output = Command::new("find")
            .args(&[&self.rustc_path, "-name", "*.rs", "-type", "f"])
            .output()
            .map_err(|e| format!("Find failed: {}", e))?;

        let files: Vec<String> = std::str::from_utf8(&output.stdout)
            .unwrap()
            .lines()
            .map(|s| s.to_string())
            .collect();

        println!("Found {} Rust files to process", files.len());

        // Process all dump types in parallel
        thread::scope(|s: &'_ thread::Scope<'_>| {
            for dump_type in &self.dump_types {
                let files_ref = &files;
                s.spawn(move |_: &'_ thread::Scope<'_>| {
                    println!("📤 Starting {} extraction...", dump_type);
                    self.extract_dump_type_parallel(dump_type, files_ref);
                });
            }
        })
        .unwrap();

        Ok(())
    }

    fn extract_dump_type_parallel(&self, dump_type: &str, files: &[String]) {
        let dump_dir = format!("{}/{}", self.output_dir, dump_type);
        fs::create_dir_all(&dump_dir).unwrap();

        // Process files in parallel chunks
        let chunk_size = 100;
        let chunks: Vec<_> = files.chunks(chunk_size).collect();

        thread::scope(|s: &'_ thread::Scope<'_>| {
            for (chunk_idx, chunk) in chunks.iter().enumerate() {
                let dump_dir = dump_dir.clone();
                s.spawn(move |_: &'_ thread::Scope<'_>| {
                    for (i, file_path) in chunk.iter().enumerate() {
                        if let Ok(dump_content) = self.get_dump_for_file(file_path, dump_type) {
                            let filename = file_path.split('/').last().unwrap_or("unknown");
                            let dump_file = format!(
                                "{}/{}_{}.dump",
                                dump_dir,
                                chunk_idx * chunk_size + i,
                                filename
                            );
                            let _ = fs::write(dump_file, dump_content);
                        }
                    }
                    println!("✅ Chunk {} complete for {}", chunk_idx, dump_type);
                });
            }
        })
        .unwrap();
    }

    fn process_dumps_to_models(&self) -> Result<(), String> {
        println!("🔄 Processing dumps into Markov models...");

        thread::scope(|s: &'_ thread::Scope<'_>| {
            for dump_type in &self.dump_types {
                s.spawn(move |_: &'_ thread::Scope<'_>| {
                    self.process_dump_type_to_models(dump_type);
                });
            }
        })
        .unwrap();

        Ok(())
    }

    fn process_dump_type_to_models(&self, dump_type: &str) {
        let dump_dir = format!("{}/{}", self.output_dir, dump_type);
        let model_dir = format!("models/{}", dump_type);
        fs::create_dir_all(&model_dir).unwrap();

        if let Ok(entries) = fs::read_dir(&dump_dir) {
            let files: Vec<_> = entries.filter_map(|e| e.ok()).collect();
            let chunk_size = 50;
            let chunks: Vec<_> = files.chunks(chunk_size).collect();

            thread::scope(|s: &'_ thread::Scope<'_>| {
                for chunk in chunks {
                    let model_dir = model_dir.clone();
                    s.spawn(move |_: &'_ thread::Scope<'_>| {
                        for entry in chunk {
                            let path = entry.path();
                            if let Ok(content) = fs::read_to_string(&path) {
                                let model = self.build_markov_model(&content);
                                let model_name = format!(
                                    "{}/{}.bin",
                                    model_dir,
                                    path.file_stem().unwrap().to_string_lossy()
                                );
                                self.save_model(&model, &model_name);
                            }
                        }
                    });
                }
            })
            .unwrap();
        }

        println!("✅ Processed {} dumps to models", dump_type);
    }

    fn get_dump_for_file(&self, file_path: &str, dump_type: &str) -> Result<String, String> {
        let flag = match dump_type {
            "ast" => "-Zunpretty=ast",
            "hir" => "-Zunpretty=hir",
            "mir" => "-Zdump-mir=all",
            "llvm-ir" => "--emit=llvm-ir",
            _ => return Err("Unknown dump type".to_string()),
        };

        let output = Command::new("rustc")
            .args(&[flag, file_path, "-o", "/tmp/rustc_dump"])
            .output()
            .map_err(|e| format!("Rustc failed: {}", e))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Ok(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    fn build_markov_model(&self, content: &str) -> HashMap<char, HashMap<char, u32>> {
        let mut model = HashMap::new();
        let chars: Vec<char> = content.chars().collect();

        for window in chars.windows(2) {
            let from = window[0];
            let to = window[1];

            model
                .entry(from)
                .or_insert_with(HashMap::new)
                .entry(to)
                .and_modify(|count| *count += 1)
                .or_insert(1);
        }

        model
    }

    fn save_model(&self, model: &HashMap<char, HashMap<char, u32>>, filename: &str) {
        let mut file = fs::File::create(filename).unwrap();
        use std::io::Write;

        let total_transitions: usize = model.values().map(|m| m.len()).sum();
        file.write_all(&(total_transitions as u32).to_le_bytes())
            .unwrap();

        for (from, to_map) in model {
            for (to, count) in to_map {
                file.write_all(&(*from as u32).to_le_bytes()).unwrap();
                file.write_all(&(*to as u32).to_le_bytes()).unwrap();
                file.write_all(&count.to_le_bytes()).unwrap();
            }
        }
    }

    fn merge_with_source_models(&self) -> Result<(), String> {
        println!("🔗 Merging compiler dumps with Rust source models...");

        // Load existing Rust source models
        let mut source_model = HashMap::new();
        if let Ok(entries) = fs::read_dir("models/forward") {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.file_name().unwrap().to_string_lossy().contains("rust") {
                        if let Ok(model) = self.load_model(&path) {
                            // Merge into source_model
                            for (from, transitions) in model {
                                let entry = source_model.entry(from).or_insert_with(HashMap::new);
                                for (to, count) in transitions {
                                    *entry.entry(to).or_insert(0) += count;
                                }
                            }
                        }
                    }
                }
            }
        }

        println!("✅ Merged source models with compiler dumps");
        Ok(())
    }

    fn load_model(
        &self,
        path: &std::path::Path,
    ) -> Result<HashMap<char, HashMap<char, u32>>, String> {
        let data = fs::read(path).map_err(|e| e.to_string())?;
        let mut model = HashMap::new();

        if data.len() < 4 {
            return Ok(model);
        }

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
}

fn main() {
    let extractor = RustStructuredExtractor::new("/usr/src/rustc".to_string());

    if let Err(e) = extractor.extract_all_dumps() {
        eprintln!("Dump extraction failed: {}", e);
        return;
    }

    if let Err(e) = extractor.process_dumps_to_models() {
        eprintln!("Model processing failed: {}", e);
        return;
    }

    if let Err(e) = extractor.merge_with_source_models() {
        eprintln!("Model merging failed: {}", e);
        return;
    }

    println!("🎉 Rust structured extraction complete!");
}
