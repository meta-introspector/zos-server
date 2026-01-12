use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

fn main() {
    println!("🔍 Checking rustc dependency fork status");

    // Step 1: Find rustc dependencies
    let rustc_deps = find_rustc_dependencies();
    println!("Found {} rustc dependencies", rustc_deps.len());

    // Step 2: Check what we already have in cargo2nix/submodules/
    let existing_forks = find_existing_forks();
    println!("Found {} existing forks in cargo2nix/submodules/", existing_forks.len());

    // Step 3: Find missing forks
    let missing_forks: Vec<_> = rustc_deps.iter()
        .filter(|dep| !existing_forks.contains(*dep))
        .collect();

    println!("\n📊 Fork Status Report:");
    println!("Total rustc deps: {}", rustc_deps.len());
    println!("Already forked: {}", rustc_deps.len() - missing_forks.len());
    println!("Missing forks: {}", missing_forks.len());

    if !missing_forks.is_empty() {
        println!("\n❌ Missing forks that need to be created:");
        for dep in &missing_forks {
            println!("  - {}", dep);
        }

        // Generate fork commands
        println!("\n🔧 Commands to fork missing dependencies:");
        for dep in &missing_forks {
            if let Some(url) = find_crate_repo_url(dep) {
                println!("git submodule add {} cargo2nix/submodules/{}", url, dep);
            } else {
                println!("# Need to find repo URL for: {}", dep);
            }
        }
    } else {
        println!("\n✅ All rustc dependencies are already forked!");
    }

    // Step 4: Check for extra forks we don't need
    let extra_forks: Vec<_> = existing_forks.iter()
        .filter(|fork| !rustc_deps.contains(*fork))
        .collect();

    if !extra_forks.is_empty() {
        println!("\n🔄 Extra forks (not rustc deps):");
        for fork in &extra_forks {
            println!("  - {}", fork);
        }
    }
}

fn find_rustc_dependencies() -> HashSet<String> {
    let mut deps = HashSet::new();

    // Look for rustc Cargo.toml in the actual location
    let rustc_cargo_toml = "/mnt/data1/nix/vendor/rust/cargo2nix/submodules/rust/Cargo.toml";
    if let Ok(content) = fs::read_to_string(rustc_cargo_toml) {
        println!("Found rustc Cargo.toml at: {}", rustc_cargo_toml);
        extract_dependencies(&content, &mut deps);
    }

    // Also check Cargo.lock
    let rustc_cargo_lock = "/mnt/data1/nix/vendor/rust/cargo2nix/submodules/rust/Cargo.lock";
    if let Ok(content) = fs::read_to_string(rustc_cargo_lock) {
        println!("Found rustc Cargo.lock at: {}", rustc_cargo_lock);
        extract_lock_dependencies(&content, &mut deps);
    }

    deps
}

fn find_existing_forks() -> HashSet<String> {
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

    // Also check other potential locations
    let other_paths = [
        "/mnt/data1/nix/time/*/cargo2nix/submodules",
        "/mnt/data1/nix/cargo2nix/submodules"
    ];

    for pattern in &other_paths {
        if let Ok(paths) = glob::glob(pattern) {
            for submodules_dir in paths.flatten() {
                if let Ok(entries) = fs::read_dir(&submodules_dir) {
                    for entry in entries.flatten() {
                        if entry.path().is_dir() {
                            if let Some(name) = entry.file_name().to_str() {
                                forks.insert(name.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    forks
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

fn find_crate_repo_url(crate_name: &str) -> Option<String> {
    // Try to find the crate's repository URL
    // This is a simplified version - in practice you'd query crates.io API
    Some(format!("https://github.com/rust-lang/{}", crate_name))
}
