use std::collections::HashMap;
use std::fs;
use std::process::Command;
use crossbeam::thread;

struct MultiRepoExtractor {
    sources: Vec<String>,
    output_dir: String,
}

impl MultiRepoExtractor {
    fn new() -> Self {
        Self {
            sources: vec![
                "/home/mdupont/nix/vendor/rust/cargo2nix/submodules/rust-build".to_string(),
                "/home/mdupont/nix/vendor/rust/cargo2nix/submodules/rust-build/compiler/zombie_driver".to_string(),
                "/home/mdupont/zos-server".to_string(),
                "/home/mdupont/meta-introspector".to_string(),
            ],
            output_dir: "multi_repo_analysis".to_string(),
        }
    }

    fn scan_all_sources(&self) -> Result<(), String> {
        fs::create_dir_all(&self.output_dir).unwrap();

        println!("🔍 Scanning all source repositories...");

        thread::scope(|s| {
            for source in &self.sources {
                s.spawn(move |_| {
                    println!("📂 Processing: {}", source);
                    self.scan_source_parallel(source);
                });
            }
        }).unwrap();

        Ok(())
    }

    fn scan_source_parallel(&self, source_path: &str) {
        let source_name = source_path.split('/').last().unwrap_or("unknown");
        let output_subdir = format!("{}/{}", self.output_dir, source_name);
        fs::create_dir_all(&output_subdir).unwrap();

        // Find all files (not just Rust)
        let output = Command::new("find")
            .args(&[source_path, "-type", "f"])
            .output();

        if let Ok(output) = output {
            let files: Vec<String> = std::str::from_utf8(&output.stdout)
                .unwrap_or("")
                .lines()
                .map(|s| s.to_string())
                .collect();

            println!("Found {} files in {}", files.len(), source_name);

            // Group by file extension
            let mut by_extension: HashMap<String, Vec<String>> = HashMap::new();
            for file in files {
                let ext = file.split('.').last().unwrap_or("no_ext").to_string();
                by_extension.entry(ext).or_insert_with(Vec::new).push(file);
            }

            // Process each extension type in parallel
            thread::scope(|s| {
                for (ext, files) in by_extension {
                    s.spawn(move |_| {
                        self.process_extension_type(&output_subdir, &ext, &files);
                    });
                }
            }).unwrap();
        }
    }

    fn process_extension_type(&self, output_dir: &str, extension: &str, files: &[String]) {
        let ext_dir = format!("{}/{}", output_dir, extension);
        fs::create_dir_all(&ext_dir).unwrap();

        println!("Processing {} {} files", files.len(), extension);

        // Process files in chunks
        let chunk_size = 100;
        let chunks: Vec<_> = files.chunks(chunk_size).collect();

        thread::scope(|s| {
            for (chunk_idx, chunk) in chunks.iter().enumerate() {
                s.spawn(move |_| {
                    for (i, file_path) in chunk.iter().enumerate() {
                        if let Ok(content) = fs::read_to_string(file_path) {
                            let model = self.build_markov_model(&content);
                            let model_file = format!("{}/{}_{}.bin", ext_dir, chunk_idx * chunk_size + i,
                                file_path.split('/').last().unwrap_or("unknown"));
                            self.save_model(&model, &model_file);
                        }
                    }
                });
            }
        }).unwrap();

        println!("✅ Completed {} files for {}", files.len(), extension);
    }

    fn build_markov_model(&self, content: &str) -> HashMap<char, HashMap<char, u32>> {
        let mut model = HashMap::new();
        let chars: Vec<char> = content.chars().collect();

        for window in chars.windows(2) {
            let from = window[0];
            let to = window[1];

            model.entry(from)
                .or_insert_with(HashMap::new)
                .entry(to)
                .and_modify(|count| *count += 1)
                .or_insert(1);
        }

        model
    }

    fn save_model(&self, model: &HashMap<char, HashMap<char, u32>>, filename: &str) {
        if let Ok(mut file) = fs::File::create(filename) {
            use std::io::Write;

            let total_transitions: usize = model.values().map(|m| m.len()).sum();
            let _ = file.write_all(&(total_transitions as u32).to_le_bytes());

            for (from, to_map) in model {
                for (to, count) in to_map {
                    let _ = file.write_all(&(*from as u32).to_le_bytes());
                    let _ = file.write_all(&(*to as u32).to_le_bytes());
                    let _ = file.write_all(&count.to_le_bytes());
                }
            }
        }
    }

    fn generate_summary(&self) -> Result<(), String> {
        println!("📊 Generating analysis summary...");

        let mut summary = String::new();
        summary.push_str("# Multi-Repository Analysis Summary\n\n");

        for source in &self.sources {
            let source_name = source.split('/').last().unwrap_or("unknown");
            let source_dir = format!("{}/{}", self.output_dir, source_name);

            if let Ok(entries) = fs::read_dir(&source_dir) {
                summary.push_str(&format!("## {}\n", source_name));

                for entry in entries {
                    if let Ok(entry) = entry {
                        let ext_name = entry.file_name().to_string_lossy();
                        if let Ok(models) = fs::read_dir(entry.path()) {
                            let count = models.count();
                            summary.push_str(&format!("- {}: {} models\n", ext_name, count));
                        }
                    }
                }
                summary.push('\n');
            }
        }

        fs::write(format!("{}/ANALYSIS_SUMMARY.md", self.output_dir), summary).unwrap();
        println!("✅ Summary saved to ANALYSIS_SUMMARY.md");

        Ok(())
    }
}

fn main() {
    let extractor = MultiRepoExtractor::new();

    if let Err(e) = extractor.scan_all_sources() {
        eprintln!("Scanning failed: {}", e);
        return;
    }

    if let Err(e) = extractor.generate_summary() {
        eprintln!("Summary generation failed: {}", e);
        return;
    }

    println!("🎉 Multi-repository analysis complete!");
}
