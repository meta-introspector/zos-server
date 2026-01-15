use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;

struct RustcHIRExtractor {
    // Structured artifacts
    hir_artifacts: HashMap<String, HIRArtifact>,
    source_artifacts: HashMap<String, SourceArtifact>,
    build_manifest: BuildManifest,

    // Statistics
    total_files: u32,
    successful_hir: u32,
    failed_hir: u32,
}

#[derive(Debug)]
struct HIRArtifact {
    source_path: String,
    hir_content: String,
    hir_size: usize,
    compilation_status: CompilationStatus,
    markov_transitions: HashMap<char, HashMap<char, u32>>,
    transition_count: u64,
}

#[derive(Debug)]
struct SourceArtifact {
    path: String,
    content: String,
    size: usize,
    module_type: ModuleType,
    markov_transitions: HashMap<char, HashMap<char, u32>>,
    transition_count: u64,
}

#[derive(Debug)]
struct BuildManifest {
    rustc_root: String,
    total_modules: u32,
    hir_success_rate: f64,
    build_timestamp: String,
    artifacts_generated: Vec<String>,
}

#[derive(Debug)]
enum CompilationStatus {
    Success,
    Failed(String),
    Skipped(String),
}

#[derive(Debug)]
enum ModuleType {
    Library,
    Binary,
    Test,
    Benchmark,
    Example,
    Unknown,
}

impl RustcHIRExtractor {
    fn new() -> Self {
        Self {
            hir_artifacts: HashMap::new(),
            source_artifacts: HashMap::new(),
            build_manifest: BuildManifest {
                rustc_root: String::new(),
                total_modules: 0,
                hir_success_rate: 0.0,
                build_timestamp: chrono::Utc::now().to_rfc3339(),
                artifacts_generated: Vec::new(),
            },
            total_files: 0,
            successful_hir: 0,
            failed_hir: 0,
        }
    }

    fn extract_all_rustc_hir(&mut self) -> Result<(), String> {
        let rustc_root = "/home/mdupont/nix/vendor/rust/cargo2nix/submodules/rust-build/compiler";
        self.build_manifest.rustc_root = rustc_root.to_string();

        println!("🏗️ Starting Rustc HIR CI/CD Build Process");
        println!("📂 Scanning: {}", rustc_root);

        // Find all rustc_* directories
        let rustc_dirs = self.find_rustc_directories(rustc_root)?;
        println!("📦 Found {} rustc modules", rustc_dirs.len());

        for (i, dir) in rustc_dirs.iter().enumerate() {
            let progress = (i + 1) as f64 / rustc_dirs.len() as f64 * 100.0;
            println!("\n🔧 [{:.1}%] Processing module: {}", progress, dir);

            self.process_rustc_module(dir)?;
        }

        self.finalize_build()?;
        Ok(())
    }

    fn find_rustc_directories(&self, root: &str) -> Result<Vec<String>, String> {
        let mut rustc_dirs = Vec::new();

        if let Ok(entries) = fs::read_dir(root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(name) = path.file_name() {
                        let name_str = name.to_string_lossy();
                        if name_str.starts_with("rustc_") {
                            rustc_dirs.push(path.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }

        rustc_dirs.sort();
        Ok(rustc_dirs)
    }

    fn process_rustc_module(&mut self, module_dir: &str) -> Result<(), String> {
        let src_dir = format!("{}/src", module_dir);

        if !Path::new(&src_dir).exists() {
            println!("  ⚠️ No src directory found, skipping");
            return Ok(());
        }

        // Find all .rs files in the module
        let rs_files = self.find_rust_files(&src_dir)?;
        println!("  📄 Found {} Rust files", rs_files.len());

        for (i, file_path) in rs_files.iter().enumerate() {
            if i % 10 == 0 {
                print!("\r    Processing file {}/{}", i + 1, rs_files.len());
                std::io::stdout().flush().unwrap();
            }

            self.process_rust_file(file_path)?;
        }

        println!(
            "\r    ✅ Processed {}/{} files",
            rs_files.len(),
            rs_files.len()
        );
        Ok(())
    }

    fn find_rust_files(&self, dir: &str) -> Result<Vec<String>, String> {
        let mut rust_files = Vec::new();

        fn walk_dir(dir: &str, files: &mut Vec<String>, depth: u32) -> Result<(), String> {
            if depth > 10 {
                return Ok(());
            } // Prevent infinite recursion

            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() && path.extension().map_or(false, |ext| ext == "rs") {
                        files.push(path.to_string_lossy().to_string());
                    } else if path.is_dir() {
                        walk_dir(&path.to_string_lossy(), files, depth + 1)?;
                    }
                }
            }
            Ok(())
        }

        walk_dir(dir, &mut rust_files, 0)?;
        Ok(rust_files)
    }

    fn process_rust_file(&mut self, file_path: &str) -> Result<(), String> {
        self.total_files += 1;

        // Read source file
        let source_content = match fs::read_to_string(file_path) {
            Ok(content) => content,
            Err(_) => return Ok(()), // Skip unreadable files
        };

        // Create source artifact
        let source_artifact = self.create_source_artifact(file_path, &source_content);
        let artifact_key = self.get_artifact_key(file_path);
        self.source_artifacts
            .insert(artifact_key.clone(), source_artifact);

        // Generate HIR
        let hir_artifact = self.generate_hir_artifact(file_path, &source_content);
        self.hir_artifacts.insert(artifact_key, hir_artifact);

        Ok(())
    }

    fn create_source_artifact(&self, file_path: &str, content: &str) -> SourceArtifact {
        let mut markov_transitions = HashMap::new();
        let mut transition_count = 0;

        // Build Markov model of source
        let chars: Vec<char> = content.chars().collect();
        for window in chars.windows(2) {
            *markov_transitions
                .entry(window[0])
                .or_insert_with(HashMap::new)
                .entry(window[1])
                .or_insert(0) += 1;
            transition_count += 1;
        }

        SourceArtifact {
            path: file_path.to_string(),
            content: content.to_string(),
            size: content.len(),
            module_type: self.determine_module_type(file_path),
            markov_transitions,
            transition_count,
        }
    }

    fn generate_hir_artifact(&mut self, file_path: &str, source_content: &str) -> HIRArtifact {
        // Attempt HIR generation
        let hir_result = Command::new("rustc")
            .args(&["-Z", "unpretty=hir", file_path])
            .output();

        let (hir_content, status) = match hir_result {
            Ok(output) if output.status.success() => {
                let content = String::from_utf8_lossy(&output.stdout).to_string();
                self.successful_hir += 1;
                (content, CompilationStatus::Success)
            }
            Ok(output) => {
                let error = String::from_utf8_lossy(&output.stderr).to_string();
                self.failed_hir += 1;
                (String::new(), CompilationStatus::Failed(error))
            }
            Err(e) => {
                self.failed_hir += 1;
                (String::new(), CompilationStatus::Failed(e.to_string()))
            }
        };

        // Build Markov model of HIR
        let mut markov_transitions = HashMap::new();
        let mut transition_count = 0;

        if !hir_content.is_empty() {
            let chars: Vec<char> = hir_content.chars().collect();
            for window in chars.windows(2) {
                *markov_transitions
                    .entry(window[0])
                    .or_insert_with(HashMap::new)
                    .entry(window[1])
                    .or_insert(0) += 1;
                transition_count += 1;
            }
        }

        HIRArtifact {
            source_path: file_path.to_string(),
            hir_content,
            hir_size: hir_content.len(),
            compilation_status: status,
            markov_transitions,
            transition_count,
        }
    }

    fn determine_module_type(&self, file_path: &str) -> ModuleType {
        if file_path.contains("/tests/") {
            ModuleType::Test
        } else if file_path.contains("/benches/") {
            ModuleType::Benchmark
        } else if file_path.contains("/examples/") {
            ModuleType::Example
        } else if file_path.ends_with("/main.rs") {
            ModuleType::Binary
        } else if file_path.ends_with("/lib.rs") {
            ModuleType::Library
        } else {
            ModuleType::Unknown
        }
    }

    fn get_artifact_key(&self, file_path: &str) -> String {
        // Create unique key from file path
        file_path.replace("/", "_").replace(".", "_")
    }

    fn finalize_build(&mut self) -> Result<(), String> {
        self.build_manifest.total_modules = self.total_files;
        self.build_manifest.hir_success_rate = if self.total_files > 0 {
            self.successful_hir as f64 / self.total_files as f64
        } else {
            0.0
        };

        // Save structured artifacts
        self.save_artifacts()?;
        self.print_build_report();

        Ok(())
    }

    fn save_artifacts(&mut self) -> Result<(), String> {
        // Save HIR artifacts as JSON
        let hir_json = serde_json::to_string_pretty(&self.hir_artifacts)
            .map_err(|e| format!("HIR serialization error: {}", e))?;
        fs::write("rustc_hir_artifacts.json", hir_json)
            .map_err(|e| format!("HIR write error: {}", e))?;
        self.build_manifest
            .artifacts_generated
            .push("rustc_hir_artifacts.json".to_string());

        // Save source artifacts as JSON
        let source_json = serde_json::to_string_pretty(&self.source_artifacts)
            .map_err(|e| format!("Source serialization error: {}", e))?;
        fs::write("rustc_source_artifacts.json", source_json)
            .map_err(|e| format!("Source write error: {}", e))?;
        self.build_manifest
            .artifacts_generated
            .push("rustc_source_artifacts.json".to_string());

        // Save build manifest
        let manifest_json = serde_json::to_string_pretty(&self.build_manifest)
            .map_err(|e| format!("Manifest serialization error: {}", e))?;
        fs::write("rustc_build_manifest.json", manifest_json)
            .map_err(|e| format!("Manifest write error: {}", e))?;

        println!("💾 Artifacts saved:");
        for artifact in &self.build_manifest.artifacts_generated {
            println!("    {}", artifact);
        }

        Ok(())
    }

    fn print_build_report(&self) {
        println!("\n🎯 Rustc HIR CI/CD Build Report:");
        println!("  Total files processed: {}", self.total_files);
        println!("  HIR generation successful: {}", self.successful_hir);
        println!("  HIR generation failed: {}", self.failed_hir);
        println!(
            "  Success rate: {:.2}%",
            self.build_manifest.hir_success_rate * 100.0
        );

        // Analyze artifacts
        let total_hir_transitions: u64 = self
            .hir_artifacts
            .values()
            .map(|a| a.transition_count)
            .sum();
        let total_source_transitions: u64 = self
            .source_artifacts
            .values()
            .map(|a| a.transition_count)
            .sum();

        println!("\n📊 Markov Analysis:");
        println!("  Total source transitions: {}", total_source_transitions);
        println!("  Total HIR transitions: {}", total_hir_transitions);

        if total_source_transitions > 0 && total_hir_transitions > 0 {
            let expansion_ratio = total_hir_transitions as f64 / total_source_transitions as f64;
            println!("  HIR expansion ratio: {:.2}x", expansion_ratio);
        }

        println!("\n🔑 Automorphic Field Theory:");
        println!("  Rustc HIR artifacts contain the complete structural DNA");
        println!("  Each module's HIR preserves and transforms source patterns");
        println!("  The collection forms a complete automorphic field");
        println!("  This IS the mathematical key to Rust's self-description");
    }
}

fn main() {
    let mut extractor = RustcHIRExtractor::new();

    if let Err(e) = extractor.extract_all_rustc_hir() {
        eprintln!("Build failed: {}", e);
        std::process::exit(1);
    }

    println!("\n✅ Rustc HIR CI/CD Build Complete!");
}
