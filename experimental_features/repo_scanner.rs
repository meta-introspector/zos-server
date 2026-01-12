use std::collections::HashMap;
use std::fs;
use std::path::Path;

use std::collections::HashMap;
use std::fs;
use std::path::Path;

fn main() {
    let index_dir = std::env::var("HOME").unwrap() + "/nix/index";

    // Read GitHub repos JSON to get fork information
    let github_repos_path = format!("{}/github_meta-introspector_repos.json", index_dir);
    let mut fork_repos = std::collections::HashSet::new();

    if let Ok(content) = fs::read_to_string(&github_repos_path) {
        if let Ok(repos) = serde_json::from_str::<Vec<serde_json::Value>>(&content) {
            for repo in repos {
                if let Some(name) = repo["name"].as_str() {
                    fork_repos.insert(name.to_string());
                }
            }
        }
    }

    println!("Found {} forked repos in GitHub", fork_repos.len());

    // Read the allrs.txt file which contains all Rust file paths
    let allrs_path = format!("{}/allrs.txt", index_dir);
    println!("Reading repo directories from {}", allrs_path);

    let mut repo_dirs = std::collections::HashSet::new();

    if let Ok(content) = fs::read_to_string(&allrs_path) {
        for line in content.lines().take(1000) { // Limit for testing
            if let Some(path) = extract_repo_dir(line) {
                repo_dirs.insert(path);
            }
        }
    }

    println!("Found {} unique repo directories", repo_dirs.len());

    // Sample first 10 repos and check if they're forks
    for (i, repo_dir) in repo_dirs.iter().take(10).enumerate() {
        println!("\n=== REPO {}: {} ===", i + 1, repo_dir);

        // Check if it's a git repo
        let git_dir = Path::new(repo_dir).join(".git");
        if git_dir.exists() {
            println!("✓ Git repo");

            // Check if it's a fork
            let repo_name = Path::new(repo_dir).file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");

            if fork_repos.contains(repo_name) {
                println!("🍴 FORK of meta-introspector/{}", repo_name);
            } else {
                println!("📁 Local repo");
            }

            let stats = scan_repo_quick(repo_dir);
            print_quick_stats(&stats);
        } else {
            println!("✗ NOT a git repo");
        }
    }
}

fn extract_repo_dir(file_path: &str) -> Option<String> {
    let path = Path::new(file_path);

    // Walk up the directory tree to find a .git directory
    let mut current = path.parent();
    while let Some(dir) = current {
        if dir.join(".git").exists() {
            return Some(dir.to_string_lossy().to_string());
        }
        current = dir.parent();
    }
    None
}

fn scan_repo_quick(repo_path: &str) -> HashMap<String, u32> {
    let mut file_counts = HashMap::new();
    scan_dir_quick(Path::new(repo_path), &mut file_counts);
    file_counts
}

fn scan_dir_quick(dir: &Path, counts: &mut HashMap<String, u32>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().unwrap().to_string_lossy();
                if !matches!(name.as_ref(), ".git" | "target" | "node_modules" | ".lake" | "build") {
                    scan_dir_quick(&path, counts);
                }
            } else {
                let file_type = get_file_type(&path);
                *counts.entry(file_type).or_insert(0) += 1;
            }
        }
    }
}

fn get_file_type(path: &Path) -> String {
    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        match name {
            "Cargo.toml" => "cargo_toml".to_string(),
            "Cargo.lock" => "cargo_lock".to_string(),
            ".gitmodules" => "gitmodules".to_string(),
            _ => {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    match ext {
                        "rs" => "rust".to_string(),
                        "md" => "markdown".to_string(),
                        "toml" => "toml".to_string(),
                        "json" => "json".to_string(),
                        "sh" => "shell".to_string(),
                        _ => format!("other_{}", ext),
                    }
                } else {
                    "no_ext".to_string()
                }
            }
        }
    } else {
        "unknown".to_string()
    }
}

fn print_quick_stats(counts: &HashMap<String, u32>) {
    let mut sorted: Vec<_> = counts.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(a.1));

    let total: u32 = counts.values().sum();
    println!("  Total files: {}", total);

    for (file_type, count) in sorted.iter().take(5) {
        println!("  {}: {}", file_type, count);
    }
}
