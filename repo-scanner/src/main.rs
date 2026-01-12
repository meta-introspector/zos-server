use rayon::prelude::*;
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};

fn main() {
    rayon::ThreadPoolBuilder::new().num_threads(24).build_global().unwrap();

    println!("🚀 Processing 20k+ repos with 24 cores...");

    let processed = AtomicUsize::new(0);
    let total_files = AtomicUsize::new(0);

    // Find all .git directories in parallel
    let git_dirs = find_git_repos("/mnt/data1/nix");
    println!("Found {} git repositories", git_dirs.len());

    // Process repos in parallel batches
    git_dirs.par_chunks(100).for_each(|batch| {
        for git_dir in batch {
            let repo_path = git_dir.parent().unwrap();
            let file_count = count_files_recursive(repo_path);

            total_files.fetch_add(file_count, Ordering::Relaxed);
            let count = processed.fetch_add(1, Ordering::Relaxed);

            if count % 100 == 0 {
                println!("📦 Processed {} repos, {} total files", count, total_files.load(Ordering::Relaxed));
            }
        }
    });

    println!("✅ Complete! {} repos, {} files processed",
        processed.load(Ordering::Relaxed),
        total_files.load(Ordering::Relaxed));
}

fn find_git_repos(root: &str) -> Vec<std::path::PathBuf> {
    let mut repos = Vec::new();
    scan_for_git(Path::new(root), &mut repos);
    repos
}

fn scan_for_git(dir: &Path, repos: &mut Vec<std::path::PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path.file_name().unwrap() == ".git" {
                    repos.push(path);
                } else {
                    scan_for_git(&path, repos);
                }
            }
        }
    }
}

fn count_files_recursive(dir: &Path) -> usize {
    let mut count = 0;
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                count += 1;
            } else if path.is_dir() && !path.file_name().unwrap().to_str().unwrap().starts_with('.') {
                count += count_files_recursive(&path);
            }
        }
    }
    count
}
