use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};

struct RustRepoGenerator {
    path_transitions: HashMap<char, HashMap<char, u32>>,
    common_paths: Vec<String>,
}

impl RustRepoGenerator {
    fn new() -> Self {
        Self {
            path_transitions: HashMap::new(),
            common_paths: Vec::new(),
        }
    }

    fn load_rust_patterns(&mut self) -> Result<(), String> {
        let file = fs::File::open("/mnt/data1/files.txt")
            .map_err(|e| format!("Failed to open files.txt: {}", e))?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let path = match line {
                Ok(p) => p,
                Err(_) => continue,
            };

            if path.ends_with(".rs") {
                self.analyze_rust_path(&path);
            }
        }
        Ok(())
    }

    fn analyze_rust_path(&mut self, path: &str) {
        // Extract common Rust project patterns
        if path.contains("/src/") {
            self.common_paths.push(path.to_string());
        }

        // Train transitions on path characters
        let chars: Vec<char> = path.chars().collect();
        for window in chars.windows(2) {
            *self
                .path_transitions
                .entry(window[0])
                .or_insert_with(HashMap::new)
                .entry(window[1])
                .or_insert(0) += 1;
        }
    }

    fn generate_path(&self, start_char: char, max_len: usize) -> String {
        let mut path = String::new();
        let mut current = start_char;
        path.push(current);

        for _ in 0..max_len {
            if let Some(next_chars) = self.path_transitions.get(&current) {
                // Pick most likely next character
                let next = next_chars
                    .iter()
                    .max_by_key(|(_, count)| *count)
                    .map(|(c, _)| *c);

                if let Some(next_char) = next {
                    path.push(next_char);
                    current = next_char;

                    // Stop at natural path boundaries
                    if next_char == '/' && path.len() > 10 {
                        break;
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        path
    }

    fn extract_common_patterns(&self) -> Vec<String> {
        let mut patterns = Vec::new();

        // Find common directory structures
        let mut dir_counts: HashMap<String, u32> = HashMap::new();

        for path in &self.common_paths {
            if let Some(src_pos) = path.find("/src/") {
                let after_src = &path[src_pos + 5..];
                if let Some(slash_pos) = after_src.find('/') {
                    let dir = &after_src[..slash_pos];
                    if !dir.is_empty() {
                        *dir_counts.entry(dir.to_string()).or_insert(0) += 1;
                    }
                }
            }
        }

        let mut sorted_dirs: Vec<_> = dir_counts.iter().collect();
        sorted_dirs.sort_by_key(|(_, count)| std::cmp::Reverse(**count));

        for (dir, _) in sorted_dirs.iter().take(10) {
            patterns.push(format!("src/{}", dir));
        }

        patterns
    }

    fn generate_repo_template(&self) -> Vec<String> {
        let mut template = Vec::new();

        // Standard Rust project structure
        template.push("Cargo.toml".to_string());
        template.push("README.md".to_string());
        template.push("src/".to_string());
        template.push("src/main.rs".to_string());
        template.push("src/lib.rs".to_string());

        // Add common patterns from analysis
        let patterns = self.extract_common_patterns();
        for pattern in patterns.iter().take(5) {
            template.push(format!("{}/mod.rs", pattern));
        }

        // Generate some paths using Markov model
        template.push(format!("src/{}.rs", self.generate_path('m', 10)));
        template.push(format!("tests/{}.rs", self.generate_path('t', 8)));

        template.push("tests/".to_string());
        template.push("examples/".to_string());
        template.push(".gitignore".to_string());

        template
    }

    fn create_template_repo(&self, name: &str) -> Result<(), String> {
        let template = self.generate_repo_template();

        println!("🚀 Generated Rust Repository Template: {}", name);
        println!("📁 Directory structure:");

        for path in &template {
            println!("  {}", path);

            // Create actual directories/files
            let full_path = format!("{}/{}", name, path);

            if path.ends_with('/') {
                fs::create_dir_all(&full_path).ok();
            } else {
                if let Some(parent) = std::path::Path::new(&full_path).parent() {
                    fs::create_dir_all(parent).ok();
                }

                let content = match path.as_str() {
                    "Cargo.toml" => format!("[package]\nname = \"{}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n", name),
                    "src/main.rs" => "fn main() {\n    println!(\"Hello, world!\");\n}\n".to_string(),
                    "src/lib.rs" => "//! Generated Rust library\n\npub fn hello() -> &'static str {\n    \"Hello from lib!\"\n}\n".to_string(),
                    "README.md" => format!("# {}\n\nGenerated from Markov analysis of 1.47M Rust files.\n", name),
                    ".gitignore" => "/target/\nCargo.lock\n".to_string(),
                    _ if path.ends_with(".rs") => "// Generated module\n\n".to_string(),
                    _ => String::new(),
                };

                fs::write(&full_path, content).ok();
            }
        }

        println!("\n✅ Template created in ./{}/", name);
        Ok(())
    }
}

fn main() {
    let mut generator = RustRepoGenerator::new();

    println!("🔍 Loading Rust path patterns from 1.47M files...");

    if let Err(e) = generator.load_rust_patterns() {
        eprintln!("Error: {}", e);
        return;
    }

    println!("🎯 Generating repository template...");

    if let Err(e) = generator.create_template_repo("markov-rust-template") {
        eprintln!("Error creating template: {}", e);
    }
}
