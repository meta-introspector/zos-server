use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

#[derive(serde::Deserialize)]
struct Value(serde_json::Value);

fn main() {
    let index_dir = std::env::var("HOME").unwrap() + "/nix/index";

    // Load all repo categories
    let forks = load_repo_names(&format!("{}/github_meta-introspector_repos.json", index_dir));
    let stars = load_repo_names(&format!("{}/starred.json", index_dir));

    println!("📊 Repository Analysis:");
    println!("🍴 Forks: {}", forks.len());
    println!("⭐ Stars: {}", stars.len());

    // Find crates used in Cargo.toml files
    let mut all_crates = HashSet::new();
    scan_cargo_dependencies(".", &mut all_crates);
    scan_cargo_dependencies("~/zombie_driver2", &mut all_crates);

    println!("\n📦 Crates used: {}", all_crates.len());

    // Find crates that are NOT forked
    let mut need_to_fork = Vec::new();
    for crate_name in &all_crates {
        if !forks.contains(crate_name) && !stars.contains(crate_name) {
            need_to_fork.push(crate_name.clone());
        }
    }

    println!("\n🎯 Crates to fork ({}):", need_to_fork.len());
    for (i, crate_name) in need_to_fork.iter().take(20).enumerate() {
        println!("  {}. {}", i + 1, crate_name);
    }

    // Show overlap analysis
    let fork_star_overlap: HashSet<_> = forks.intersection(&stars).collect();
    println!("\n🔄 Fork+Star overlap: {}", fork_star_overlap.len());

    let crate_fork_overlap: HashSet<_> = all_crates.intersection(&forks).collect();
    println!("✅ Crates already forked: {}", crate_fork_overlap.len());
}

fn load_repo_names(path: &str) -> HashSet<String> {
    let mut names = HashSet::new();

    if let Ok(content) = fs::read_to_string(path) {
        if let Ok(repos) = serde_json::from_str::<Vec<serde_json::Value>>(&content) {
            for repo in repos {
                if let Some(name) = repo["name"].as_str() {
                    names.insert(name.to_string());
                }
                // Also try full_name for starred repos
                if let Some(full_name) = repo["full_name"].as_str() {
                    if let Some(repo_name) = full_name.split('/').last() {
                        names.insert(repo_name.to_string());
                    }
                }
            }
        }
    }

    names
}

fn scan_cargo_dependencies(repo_path: &str, crates: &mut HashSet<String>) {
    scan_dir_for_cargo(Path::new(repo_path), crates);
}

fn scan_dir_for_cargo(dir: &Path, crates: &mut HashSet<String>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().unwrap().to_string_lossy();
                if !matches!(name.as_ref(), ".git" | "target" | "node_modules" | ".lake") {
                    scan_dir_for_cargo(&path, crates);
                }
            } else if path.file_name().and_then(|n| n.to_str()) == Some("Cargo.toml") {
                extract_dependencies(&path, crates);
            }
        }
    }
}

fn extract_dependencies(cargo_toml: &Path, crates: &mut HashSet<String>) {
    if let Ok(content) = fs::read_to_string(cargo_toml) {
        if let Ok(parsed) = content.parse::<toml::Value>() {
            // Extract from [dependencies]
            if let Some(deps) = parsed.get("dependencies").and_then(|d| d.as_table()) {
                for key in deps.keys() {
                    crates.insert(key.clone());
                }
            }

            // Extract from [dev-dependencies]
            if let Some(dev_deps) = parsed.get("dev-dependencies").and_then(|d| d.as_table()) {
                for key in dev_deps.keys() {
                    crates.insert(key.clone());
                }
            }

            // Extract from [build-dependencies]
            if let Some(build_deps) = parsed.get("build-dependencies").and_then(|d| d.as_table()) {
                for key in build_deps.keys() {
                    crates.insert(key.clone());
                }
            }
        }
    }
}
