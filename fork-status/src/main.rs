use std::collections::HashSet;
use std::fs;

fn main() {
    println!("🔍 Checking if we have forks for each rustc dependency");

    // Get rustc dependencies
    let rustc_deps = get_rustc_deps();
    println!("Found {} rustc dependencies", rustc_deps.len());

    // Get our existing forks
    let our_forks = get_our_forks();
    println!("Found {} existing forks", our_forks.len());

    // Check each dependency
    let mut have_fork = 0;
    let mut missing_fork = 0;

    println!("\n📊 Fork Status Check (External Dependencies Only):");
    for dep in &rustc_deps {
        // Skip internal rustc crates - they're part of the rust repo itself
        if dep.starts_with("rustc_") || dep.starts_with("rustdoc") ||
           dep == "tidy" || dep == "clippy" || dep == "miri" ||
           dep == "cargo" || dep.starts_with("clippy_") {
            continue;
        }

        if our_forks.contains(dep) {
            println!("✅ {} - HAVE FORK", dep);
            have_fork += 1;
        } else {
            println!("❌ {} - MISSING FORK", dep);
            missing_fork += 1;
        }
    }

    println!("\n📈 Summary:");
    println!("Have fork: {}", have_fork);
    println!("Missing fork: {}", missing_fork);
    println!("Coverage: {:.1}%", (have_fork as f64 / rustc_deps.len() as f64) * 100.0);
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
