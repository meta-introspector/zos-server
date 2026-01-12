use std::collections::HashSet;
use std::fs;
use std::path::Path;

fn main() {
    println!("🔍 Checking for meta-introspector remotes in existing forks");

    let submodules_path = "/mnt/data1/nix/vendor/rust/cargo2nix/submodules";
    let mut have_meta_remote = 0;
    let mut missing_meta_remote = 0;

    if let Ok(entries) = fs::read_dir(submodules_path) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                if let Some(repo_name) = entry.file_name().to_str() {
                    let git_config_path = entry.path().join(".git").join("config");

                    if git_config_path.exists() {
                        if let Ok(config_content) = fs::read_to_string(&git_config_path) {
                            if has_meta_introspector_remote(&config_content) {
                                println!("✅ {} - HAS meta-introspector remote", repo_name);
                                have_meta_remote += 1;
                            } else {
                                println!("❌ {} - MISSING meta-introspector remote", repo_name);
                                missing_meta_remote += 1;

                                // Show existing remotes
                                let remotes = extract_remotes(&config_content);
                                if !remotes.is_empty() {
                                    println!("   Existing remotes: {:?}", remotes);
                                }
                            }
                        }
                    } else {
                        println!("❓ {} - No .git/config found", repo_name);
                    }
                }
            }
        }
    }

    println!("\n📊 Meta-introspector Remote Summary:");
    println!("Have meta-introspector remote: {}", have_meta_remote);
    println!("Missing meta-introspector remote: {}", missing_meta_remote);

    let total = have_meta_remote + missing_meta_remote;
    if total > 0 {
        println!("Coverage: {:.1}%", (have_meta_remote as f64 / total as f64) * 100.0);
    }
}

fn has_meta_introspector_remote(config_content: &str) -> bool {
    config_content.contains("meta-introspector")
}

fn extract_remotes(config_content: &str) -> Vec<String> {
    let mut remotes = Vec::new();
    let mut current_remote = None;

    for line in config_content.lines() {
        let line = line.trim();

        // Check for remote section
        if line.starts_with("[remote ") {
            if let Some(start) = line.find('"') {
                if let Some(end) = line.rfind('"') {
                    if start < end {
                        current_remote = Some(line[start + 1..end].to_string());
                    }
                }
            }
        }

        // Check for URL in current remote
        if line.starts_with("url = ") && current_remote.is_some() {
            let url = line[6..].trim();
            remotes.push(format!("{}: {}", current_remote.as_ref().unwrap(), url));
            current_remote = None;
        }
    }

    remotes
}
