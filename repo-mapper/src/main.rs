use std::collections::{HashMap, HashSet};
use std::fs;
use std::process::Command;

fn main() {
    println!("🔍 Mapping crates to repositories and checking fork status");

    let rustc_deps = get_rustc_deps();
    let our_forks = get_our_forks();

    println!("Found {} rustc dependencies", rustc_deps.len());
    println!("Found {} existing forks", our_forks.len());

    let mut repo_to_crates: HashMap<String, Vec<String>> = HashMap::new();
    let mut have_repo_fork = 0;
    let mut missing_repo_fork = 0;
    let mut unknown_repo = 0;

    println!("\n📊 Repository -> Crate Mapping:");

    for dep in &rustc_deps {
        // Skip internal rustc crates
        if dep.starts_with("rustc_") || dep.starts_with("rustdoc") ||
           dep == "tidy" || dep == "clippy" || dep == "miri" ||
           dep == "cargo" || dep.starts_with("clippy_") {
            continue;
        }

        if let Some(repo_name) = get_crate_repository(dep) {
            repo_to_crates.entry(repo_name.clone()).or_default().push(dep.clone());

            if our_forks.contains(&repo_name) {
                println!("✅ {} -> {} (HAVE FORK)", dep, repo_name);
                have_repo_fork += 1;
            } else {
                println!("❌ {} -> {} (MISSING FORK)", dep, repo_name);
                missing_repo_fork += 1;
            }
        } else {
            println!("❓ {} -> UNKNOWN REPOSITORY", dep);
            unknown_repo += 1;
        }
    }

    println!("\n📈 Repository Fork Summary:");
    println!("Have repository fork: {}", have_repo_fork);
    println!("Missing repository fork: {}", missing_repo_fork);
    println!("Unknown repository: {}", unknown_repo);

    let total_external = have_repo_fork + missing_repo_fork + unknown_repo;
    if total_external > 0 {
        println!("Coverage: {:.1}%", (have_repo_fork as f64 / total_external as f64) * 100.0);
    }

    println!("\n🗂️ Repositories with multiple crates:");
    for (repo, crates) in &repo_to_crates {
        if crates.len() > 1 {
            println!("{} -> {} crates: {:?}", repo, crates.len(), crates);
        }
    }
}

fn get_rustc_deps() -> HashSet<String> {
    let mut deps = HashSet::new();

    let rustc_cargo_toml = "/mnt/data1/nix/vendor/rust/cargo2nix/submodules/rust/Cargo.toml";
    if let Ok(content) = fs::read_to_string(rustc_cargo_toml) {
        extract_dependencies(&content, &mut deps);
    }

    let rustc_cargo_lock = "/mnt/data1/nix/vendor/rust/cargo2nix/submodules/rust/Cargo.lock";
    if let Ok(content) = fs::read_to_string(rustc_cargo_lock) {
        extract_lock_dependencies(&content, &mut deps);
    }

    deps
}

fn get_our_forks() -> HashSet<String> {
    let mut forks = HashSet::new();
    let submodules_path = "/mnt/data1/nix/vendor/rust/cargo2nix/submodules";

    if let Ok(entries) = fs::read_dir(submodules_path) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    forks.insert(name.to_string());
                }
            }
        }
    }

    forks
}

fn get_crate_repository(crate_name: &str) -> Option<String> {
    // Try to get repository info from cargo metadata
    let output = Command::new("cargo")
        .args(&["search", crate_name, "--limit", "1"])
        .output()
        .ok()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Parse cargo search output to extract repository
        // This is a simplified approach - in practice you'd want to use crates.io API

        // For now, use common repository mappings
        match crate_name {
            // Serde ecosystem
            "serde" | "serde_derive" | "serde_json" => Some("serde".to_string()),

            // Tokio ecosystem
            "tokio" | "tokio-stream" | "tokio-util" => Some("tokio".to_string()),

            // Syn ecosystem
            "syn" | "quote" | "proc-macro2" => Some("syn".to_string()),

            // Regex
            "regex" | "regex-syntax" | "regex-automata" => Some("regex".to_string()),

            // Crossbeam
            name if name.starts_with("crossbeam") => Some("crossbeam".to_string()),

            // Windows crates
            name if name.starts_with("windows") => Some("windows-rs".to_string()),

            // Rayon
            "rayon" | "rayon-core" => Some("rayon".to_string()),

            // Individual crates - use crate name as repo name
            _ => Some(crate_name.replace("_", "-")),
        }
    } else {
        // Fallback: assume repo name is crate name with underscores replaced by hyphens
        Some(crate_name.replace("_", "-"))
    }
}

fn extract_dependencies(content: &str, deps: &mut HashSet<String>) {
    let mut in_deps_section = false;

    for line in content.lines() {
        let line = line.trim();

        if line == "[dependencies]" || line == "[dev-dependencies]" || line == "[build-dependencies]" {
            in_deps_section = true;
            continue;
        }

        if line.starts_with("[") && !line.contains("dependencies") {
            in_deps_section = false;
            continue;
        }

        if in_deps_section && line.contains("=") {
            if let Some(dep_name) = line.split('=').next() {
                let dep_name = dep_name.trim();
                if !dep_name.is_empty() && !dep_name.starts_with('#') {
                    deps.insert(dep_name.to_string());
                }
            }
        }
    }
}

fn extract_lock_dependencies(content: &str, deps: &mut HashSet<String>) {
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("name = ") {
            if let Some(name) = line.strip_prefix("name = ") {
                let name = name.trim_matches('"');
                deps.insert(name.to_string());
            }
        }
    }
}
