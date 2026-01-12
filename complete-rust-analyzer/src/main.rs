use std::collections::{HashMap, HashSet};
use std::fs;

fn main() {
    println!("🎯 Complete Rust Dependency Coverage Analysis");

    let mut analyzer = CompleteRustAnalyzer::new();
    analyzer.load_all_rust_crates();
    analyzer.resolve_all_dependencies();
    analyzer.map_to_forks();
    analyzer.report_coverage();
}

struct CompleteRustAnalyzer {
    rust_crates: HashMap<String, CrateInfo>,
    all_dependencies: HashMap<String, DepSource>,
    our_forks: HashSet<String>,
    centrality_scores: HashMap<String, f64>,
}

#[derive(Debug, Clone)]
struct CrateInfo {
    name: String,
    path: String,
    dependencies: Vec<String>,
    is_workspace_member: bool,
}

#[derive(Debug, Clone)]
enum DepSource {
    Internal(String),      // Internal rust crate
    CratesIo(String),      // crates.io dependency
    GitHub(String),        // Direct GitHub dependency
    Git(String),           // Other git repository
}

impl CompleteRustAnalyzer {
    fn new() -> Self {
        Self {
            rust_crates: HashMap::new(),
            all_dependencies: HashMap::new(),
            our_forks: HashSet::new(),
            centrality_scores: HashMap::new(),
        }
    }

    fn load_all_rust_crates(&mut self) {
        println!("📦 Loading all Rust crates from workspace...");

        let rust_root = "/mnt/data1/nix/vendor/rust/cargo2nix/submodules/rust";
        self.scan_workspace(rust_root);

        println!("   Found {} crates in Rust workspace", self.rust_crates.len());
    }

    fn scan_workspace(&mut self, root_path: &str) {
        // Load workspace Cargo.toml
        let workspace_toml = format!("{}/Cargo.toml", root_path);
        if let Ok(content) = fs::read_to_string(&workspace_toml) {
            self.parse_workspace_members(&content, root_path);
        }

        // Also scan for any Cargo.toml files recursively
        if let Ok(output) = std::process::Command::new("find")
            .args(&[root_path, "-name", "Cargo.toml", "-type", "f"])
            .output() {
            if let Ok(files) = String::from_utf8(output.stdout) {
                for cargo_file in files.lines() {
                    self.parse_crate_toml(cargo_file);
                }
            }
        }
    }

    fn parse_workspace_members(&mut self, content: &str, root_path: &str) {
        let mut in_members = false;
        for line in content.lines() {
            let line = line.trim();
            if line == "members = [" {
                in_members = true;
                continue;
            }
            if in_members && line == "]" {
                break;
            }
            if in_members && line.starts_with('"') {
                if let Some(member) = line.strip_prefix('"').and_then(|s| s.strip_suffix("\",").or_else(|| s.strip_suffix('"'))) {
                    let member_path = format!("{}/{}", root_path, member);
                    let cargo_toml = format!("{}/Cargo.toml", member_path);
                    if std::path::Path::new(&cargo_toml).exists() {
                        self.parse_crate_toml(&cargo_toml);
                    }
                }
            }
        }
    }

    fn parse_crate_toml(&mut self, cargo_file: &str) {
        if let Ok(content) = fs::read_to_string(cargo_file) {
            let mut crate_name = String::new();
            let mut dependencies = Vec::new();
            let mut current_section = String::new();

            for line in content.lines() {
                let line = line.trim();

                if line.starts_with("name = ") {
                    crate_name = line.strip_prefix("name = ").unwrap_or("").trim_matches('"').to_string();
                } else if line == "[dependencies]" || line == "[dev-dependencies]" || line == "[build-dependencies]" {
                    current_section = line.to_string();
                } else if line.starts_with("[") {
                    current_section.clear();
                } else if !current_section.is_empty() && line.contains("=") {
                    if let Some(dep_name) = line.split('=').next() {
                        let dep_name = dep_name.trim();
                        if !dep_name.is_empty() && !dep_name.starts_with('#') {
                            dependencies.push(dep_name.to_string());
                        }
                    }
                }
            }

            if !crate_name.is_empty() {
                let crate_info = CrateInfo {
                    name: crate_name.clone(),
                    path: cargo_file.to_string(),
                    dependencies,
                    is_workspace_member: cargo_file.contains("/rust/"),
                };
                self.rust_crates.insert(crate_name, crate_info);
            }
        }
    }

    fn resolve_all_dependencies(&mut self) {
        println!("🔍 Resolving all dependencies to sources...");

        let mut all_deps = HashSet::new();
        for crate_info in self.rust_crates.values() {
            for dep in &crate_info.dependencies {
                all_deps.insert(dep.clone());
            }
        }

        for dep in all_deps {
            let source = self.determine_dependency_source(&dep);
            self.all_dependencies.insert(dep, source);
        }

        println!("   Resolved {} unique dependencies", self.all_dependencies.len());
    }

    fn determine_dependency_source(&self, dep_name: &str) -> DepSource {
        let clean_dep = dep_name.replace(".workspace", "");

        // Check if it's an internal rust crate
        if self.rust_crates.contains_key(&clean_dep) {
            return DepSource::Internal(clean_dep);
        }

        // Load the mapping cache to check if we have it forked
        let mut have_fork = false;
        if let Ok(content) = fs::read_to_string("../remote-fork-mapper/crate_directory_cache.txt") {
            for line in content.lines() {
                if let Some((crate_name, _dir_name)) = line.split_once('\t') {
                    if crate_name == clean_dep {
                        have_fork = true;
                        break;
                    }
                }
            }
        }

        if have_fork {
            DepSource::GitHub(format!("https://github.com/meta-introspector/{}", clean_dep))
        } else {
            DepSource::CratesIo(clean_dep)
        }
    }

    fn map_to_forks(&mut self) {
        println!("🍴 Loading existing forks...");

        // Load our existing forks
        let submodules_path = "/mnt/data1/nix/vendor/rust/cargo2nix/submodules";
        if let Ok(entries) = fs::read_dir(submodules_path) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    if let Some(name) = entry.file_name().to_str() {
                        self.our_forks.insert(name.to_string());
                    }
                }
            }
        }

        // Load centrality scores
        let centrality_file = "../graph-loader/eigenvector_centrality.txt";
        if let Ok(content) = fs::read_to_string(centrality_file) {
            for line in content.lines() {
                if let Some((name, score)) = line.split_once(':') {
                    if let Ok(score) = score.trim().parse::<f64>() {
                        self.centrality_scores.insert(name.trim().to_string(), score);
                    }
                }
            }
        }

        println!("   Found {} existing forks", self.our_forks.len());
    }

    fn report_coverage(&self) {
        println!("\n📊 COMPLETE RUST DEPENDENCY COVERAGE REPORT");

        let mut internal_count = 0;
        let mut github_forked = 0;
        let mut crates_io_forked = 0;
        let mut crates_io_missing = Vec::new();
        let mut github_missing = Vec::new();

        // Count references for each dependency
        let mut dep_ref_counts = HashMap::new();
        for crate_info in self.rust_crates.values() {
            for dep in &crate_info.dependencies {
                let clean_dep = dep.replace(".workspace", "");
                *dep_ref_counts.entry(clean_dep).or_insert(0) += 1;
            }
        }

        for (dep_name, source) in &self.all_dependencies {
            let clean_dep = dep_name.replace(".workspace", "");
            let ref_count = dep_ref_counts.get(&clean_dep).unwrap_or(&0);

            match source {
                DepSource::Internal(_) => internal_count += 1,
                DepSource::GitHub(_) => github_forked += 1,
                DepSource::CratesIo(_) => {
                    if self.our_forks.contains(&clean_dep) {
                        crates_io_forked += 1;
                    } else {
                        crates_io_missing.push((clean_dep.clone(), *ref_count));
                    }
                }
                DepSource::Git(_url) => {
                    if !self.our_forks.contains(&clean_dep) {
                        github_missing.push((clean_dep.clone(), *ref_count));
                    }
                }
            }
        }

        // Sort by reference count (highest first)
        crates_io_missing.sort_by(|a, b| b.1.cmp(&a.1));
        github_missing.sort_by(|a, b| b.1.cmp(&a.1));

        let total_deps = self.all_dependencies.len();
        let forked_deps = github_forked + crates_io_forked;
        let coverage_percent = (forked_deps as f64 / (total_deps - internal_count) as f64) * 100.0;

        println!("📈 COVERAGE STATISTICS:");
        println!("   Total dependencies: {}", total_deps);
        println!("   Internal (rust workspace): {}", internal_count);
        println!("   External dependencies: {}", total_deps - internal_count);
        println!("   Already forked: {}", forked_deps);
        println!("   Coverage: {:.1}%", coverage_percent);

        println!("\n🎯 TOP 10 MOST REFERENCED MISSING FORKS:");
        for (i, (dep, ref_count)) in crates_io_missing.iter().take(10).enumerate() {
            println!("   {}. {} ({} references)", i+1, dep, ref_count);
        }

        if !github_missing.is_empty() {
            println!("\n🐙 GitHub dependencies needing forks:");
            for (dep, ref_count) in github_missing.iter().take(5) {
                println!("   {} ({} references)", dep, ref_count);
            }
        }

        println!("\n🚀 NEXT ACTIONS FOR 100% COVERAGE:");
        println!("   Need to fork {} more dependencies", crates_io_missing.len() + github_missing.len());
        println!("   Start with top 10 most referenced dependencies above");
    }
}
