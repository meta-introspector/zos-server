use std::collections::{HashMap, HashSet};
use std::fs;

fn main() {
    println!("🦀 Analyzing Rust crate ecosystem");

    let mut crate_graph = CrateGraph::new();

    // Focus ecosystems
    let focus_crates = ["nix", "lean4", "minizinc", "solana", "anchor", "spl"];

    println!("📦 Processing Cargo.toml files...");
    let cargo_toml_list = fs::read_to_string("cargo_toml_list.txt").unwrap();
    let mut processed = 0;

    for cargo_path in cargo_toml_list.lines() {
        if let Ok(content) = fs::read_to_string(cargo_path) {
            let repo_path = extract_repo_path(cargo_path);
            crate_graph.process_cargo_toml(&content, &repo_path);
            processed += 1;

            if processed % 1000 == 0 {
                println!("Processed {} Cargo.toml files", processed);
            }
        }
    }

    println!("🔒 Processing Cargo.lock files...");
    let cargo_lock_list = fs::read_to_string("cargo_lock_list.txt").unwrap();
    processed = 0;

    for lock_path in cargo_lock_list.lines() {
        if let Ok(content) = fs::read_to_string(lock_path) {
            let repo_path = extract_repo_path(lock_path);
            crate_graph.process_cargo_lock(&content, &repo_path);
            processed += 1;

            if processed % 1000 == 0 {
                println!("Processed {} Cargo.lock files", processed);
            }
        }
    }

    println!("\n📊 Crate Graph Statistics:");
    println!("Total crates: {}", crate_graph.crates.len());
    println!("Total repos: {}", crate_graph.repo_crates.len());
    println!("Dependencies: {}", crate_graph.crate_deps.values().map(|d| d.len()).sum::<usize>());

    // Focus on our target ecosystems
    println!("\n🎯 Focus Ecosystem Analysis:");
    for focus in &focus_crates {
        if let Some(repos) = crate_graph.crate_repos.get(*focus) {
            println!("{}: {} repos use this crate", focus, repos.len());
            for repo in repos.iter().take(5) {
                println!("  - {}", repo);
            }
        }
    }

    // Most popular crates
    let mut crate_popularity: Vec<_> = crate_graph.crate_repos.iter()
        .map(|(crate_name, repos)| (crate_name, repos.len()))
        .collect();
    crate_popularity.sort_by(|a, b| b.1.cmp(&a.1));

    println!("\n🔥 Most popular crates:");
    for (crate_name, repo_count) in crate_popularity.iter().take(20) {
        println!("  {} -> {} repos", crate_name, repo_count);
    }
}

struct CrateGraph {
    crates: HashSet<String>,
    repo_crates: HashMap<String, HashSet<String>>, // repo -> crates
    crate_repos: HashMap<String, HashSet<String>>, // crate -> repos
    crate_deps: HashMap<String, HashSet<String>>,  // crate -> dependencies
}

impl CrateGraph {
    fn new() -> Self {
        Self {
            crates: HashSet::new(),
            repo_crates: HashMap::new(),
            crate_repos: HashMap::new(),
            crate_deps: HashMap::new(),
        }
    }

    fn process_cargo_toml(&mut self, content: &str, repo_path: &str) {
        // Simple TOML parsing for dependencies
        let mut in_deps_section = false;
        let mut current_crate = None;

        for line in content.lines() {
            let line = line.trim();

            // Check for package name
            if line.starts_with("name = ") {
                if let Some(name) = extract_quoted_value(line) {
                    current_crate = Some(name.clone());
                    self.add_crate_to_repo(&name, repo_path);
                }
            }

            // Check for dependency sections
            if line == "[dependencies]" || line == "[dev-dependencies]" || line == "[build-dependencies]" {
                in_deps_section = true;
                continue;
            }

            if line.starts_with("[") && line != "[dependencies]" && line != "[dev-dependencies]" && line != "[build-dependencies]" {
                in_deps_section = false;
                continue;
            }

            // Parse dependencies
            if in_deps_section && line.contains("=") {
                if let Some(dep_name) = extract_dependency_name(line) {
                    if let Some(ref crate_name) = current_crate {
                        self.crate_deps.entry(crate_name.clone())
                            .or_default()
                            .insert(dep_name);
                    }
                }
            }
        }
    }

    fn process_cargo_lock(&mut self, content: &str, repo_path: &str) {
        // Parse Cargo.lock for exact dependency versions
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("name = ") {
                if let Some(crate_name) = extract_quoted_value(line) {
                    self.add_crate_to_repo(&crate_name, repo_path);
                }
            }
        }
    }

    fn add_crate_to_repo(&mut self, crate_name: &str, repo_path: &str) {
        self.crates.insert(crate_name.to_string());

        self.repo_crates.entry(repo_path.to_string())
            .or_default()
            .insert(crate_name.to_string());

        self.crate_repos.entry(crate_name.to_string())
            .or_default()
            .insert(repo_path.to_string());
    }
}

fn extract_repo_path(file_path: &str) -> String {
    // Extract repo name from file path
    let parts: Vec<&str> = file_path.split('/').collect();
    for i in 0..parts.len() {
        if parts[i] == "nix" && i + 1 < parts.len() {
            return parts[i + 1..].join("/");
        }
    }
    file_path.to_string()
}

fn extract_quoted_value(line: &str) -> Option<String> {
    if let Some(eq_pos) = line.find('=') {
        let value = line[eq_pos + 1..].trim();
        let value = value.trim_matches('"').trim_matches('\'');
        if !value.is_empty() {
            return Some(value.to_string());
        }
    }
    None
}

fn extract_dependency_name(line: &str) -> Option<String> {
    if let Some(eq_pos) = line.find('=') {
        let dep_name = line[..eq_pos].trim();
        if !dep_name.is_empty() {
            return Some(dep_name.to_string());
        }
    }
    None
}
