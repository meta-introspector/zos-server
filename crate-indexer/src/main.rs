use std::collections::HashMap;
use std::fs;
use std::process::Command;

fn main() {
    println!("📋 Creating Universal Crate Index");

    let mut indexer = CrateIndexer::new();
    indexer.scan_all_crates();
    indexer.check_git_status();
    indexer.generate_index();
}

struct CrateIndexer {
    crates: Vec<CrateEntry>,
}

#[derive(Debug, Clone)]
struct CrateEntry {
    crate_name: String,
    directory: String,
    local_path: String,
    fork_url: String,
    upstream_url: Option<String>,
    current_branch: String,
    commits_behind: Option<i32>,
    has_local_patches: bool,
}

impl CrateIndexer {
    fn new() -> Self {
        Self { crates: Vec::new() }
    }

    fn scan_all_crates(&mut self) {
        println!("🔍 Scanning all crates in submodules...");

        let submodules_path = "/mnt/data1/nix/vendor/rust/cargo2nix/submodules";
        if let Ok(entries) = fs::read_dir(submodules_path) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    if let Some(dir_name) = entry.file_name().to_str() {
                        let local_path = entry.path().to_string_lossy().to_string();

                        if let Some(crate_entry) = self.process_directory(dir_name, &local_path) {
                            self.crates.push(crate_entry);
                        }
                    }
                }
            }
        }

        println!("   Processed {} crate directories", self.crates.len());
    }

    fn process_directory(&self, dir_name: &str, local_path: &str) -> Option<CrateEntry> {
        // Get crate name from Cargo.toml
        let cargo_toml = format!("{}/Cargo.toml", local_path);
        let crate_name = if let Ok(content) = fs::read_to_string(&cargo_toml) {
            self.extract_crate_name(&content).unwrap_or_else(|| dir_name.to_string())
        } else {
            dir_name.to_string()
        };

        // Get git info
        let fork_url = self.get_git_remote(local_path)?;
        let current_branch = self.get_current_branch(local_path);

        Some(CrateEntry {
            crate_name,
            directory: dir_name.to_string(),
            local_path: local_path.to_string(),
            fork_url,
            upstream_url: None, // Will be filled in check_git_status
            current_branch,
            commits_behind: None,
            has_local_patches: false,
        })
    }

    fn extract_crate_name(&self, content: &str) -> Option<String> {
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("name = ") {
                if let Some(name) = line.strip_prefix("name = ") {
                    return Some(name.trim_matches('"').to_string());
                }
            }
        }
        None
    }

    fn get_git_remote(&self, path: &str) -> Option<String> {
        let output = Command::new("git")
            .args(&["-C", path, "remote", "get-url", "origin"])
            .output()
            .ok()?;

        if output.status.success() {
            Some(String::from_utf8(output.stdout).ok()?.trim().to_string())
        } else {
            None
        }
    }

    fn get_current_branch(&self, path: &str) -> String {
        let output = Command::new("git")
            .args(&["-C", path, "branch", "--show-current"])
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                return String::from_utf8(output.stdout)
                    .unwrap_or_default()
                    .trim()
                    .to_string();
            }
        }
        "unknown".to_string()
    }

    fn check_git_status(&mut self) {
        println!("🔄 Checking git status for all crates...");

        let paths: Vec<_> = self.crates.iter().map(|c| c.local_path.clone()).collect();

        for (i, path) in paths.iter().enumerate() {
            self.crates[i].upstream_url = self.get_upstream_remote(path);

            if self.crates[i].upstream_url.is_some() {
                self.crates[i].commits_behind = self.get_commits_behind(path);
            }

            self.crates[i].has_local_patches = self.has_local_commits(path);
        }

        println!("   Updated git status for {} crates", self.crates.len());
    }

    fn get_upstream_remote(&self, path: &str) -> Option<String> {
        let output = Command::new("git")
            .args(&["-C", path, "remote", "get-url", "upstream"])
            .output()
            .ok()?;

        if output.status.success() {
            Some(String::from_utf8(output.stdout).ok()?.trim().to_string())
        } else {
            None
        }
    }

    fn get_commits_behind(&self, path: &str) -> Option<i32> {
        // Fetch latest
        let _ = Command::new("git")
            .args(&["-C", path, "fetch", "upstream"])
            .output();

        // Count commits behind
        let output = Command::new("git")
            .args(&["-C", path, "rev-list", "--count", "HEAD..upstream/main"])
            .output()
            .ok()?;

        if output.status.success() {
            let count_str = String::from_utf8(output.stdout).ok()?;
            count_str.trim().parse().ok()
        } else {
            // Try master branch
            let output = Command::new("git")
                .args(&["-C", path, "rev-list", "--count", "HEAD..upstream/master"])
                .output()
                .ok()?;

            if output.status.success() {
                let count_str = String::from_utf8(output.stdout).ok()?;
                count_str.trim().parse().ok()
            } else {
                None
            }
        }
    }

    fn has_local_commits(&self, path: &str) -> bool {
        let output = Command::new("git")
            .args(&["-C", path, "log", "--oneline", "origin/main..HEAD"])
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                return !output.stdout.is_empty();
            }
        }

        // Try master branch
        let output = Command::new("git")
            .args(&["-C", path, "log", "--oneline", "origin/master..HEAD"])
            .output();

        if let Ok(output) = output {
            output.status.success() && !output.stdout.is_empty()
        } else {
            false
        }
    }

    fn generate_index(&self) {
        println!("📄 Generating universal crate index...");

        let mut index_content = String::new();
        index_content.push_str("# Universal Crate Index\n\n");
        index_content.push_str("| Crate | Directory | Fork URL | Upstream | Branch | Behind | Patches |\n");
        index_content.push_str("|-------|-----------|----------|----------|--------|--------|----------|\n");

        let mut sorted_crates = self.crates.clone();
        sorted_crates.sort_by(|a, b| a.crate_name.cmp(&b.crate_name));

        for crate_entry in &sorted_crates {
            let upstream = crate_entry.upstream_url.as_deref().unwrap_or("none");
            let behind = crate_entry.commits_behind.map_or("?".to_string(), |n: i32| n.to_string());
            let patches = if crate_entry.has_local_patches { "✅" } else { "❌" };

            index_content.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} | {} |\n",
                crate_entry.crate_name,
                crate_entry.directory,
                self.format_url(&crate_entry.fork_url),
                self.format_url(upstream),
                crate_entry.current_branch,
                behind,
                patches
            ));
        }

        // Add summary
        let total_crates = self.crates.len();
        let with_upstream = self.crates.iter().filter(|c| c.upstream_url.is_some()).count();
        let with_patches = self.crates.iter().filter(|c| c.has_local_patches).count();
        let behind_count = self.crates.iter().filter(|c| c.commits_behind.unwrap_or(0) > 0).count();

        index_content.push_str(&format!("\n## Summary\n"));
        index_content.push_str(&format!("- Total crates: {}\n", total_crates));
        index_content.push_str(&format!("- With upstream: {}\n", with_upstream));
        index_content.push_str(&format!("- With local patches: {}\n", with_patches));
        index_content.push_str(&format!("- Behind upstream: {}\n", behind_count));

        // Save index
        let _ = fs::write("universal_crate_index.md", &index_content);
        let _ = fs::write("universal_crate_index.json", &self.generate_json());

        println!("   ✅ Index saved to universal_crate_index.md and .json");
        println!("   📊 {} crates indexed", total_crates);
    }

    fn format_url(&self, url: &str) -> String {
        if url == "none" || url.is_empty() {
            "none".to_string()
        } else if url.len() > 50 {
            format!("{}...", &url[..47])
        } else {
            url.to_string()
        }
    }

    fn generate_json(&self) -> String {
        // Simple JSON generation
        let mut json = String::from("[\n");
        for (i, crate_entry) in self.crates.iter().enumerate() {
            if i > 0 { json.push_str(",\n"); }
            json.push_str(&format!(
                "  {{\n    \"crate_name\": \"{}\",\n    \"directory\": \"{}\",\n    \"local_path\": \"{}\",\n    \"fork_url\": \"{}\",\n    \"upstream_url\": \"{}\",\n    \"current_branch\": \"{}\",\n    \"commits_behind\": {},\n    \"has_local_patches\": {}\n  }}",
                crate_entry.crate_name,
                crate_entry.directory,
                crate_entry.local_path,
                crate_entry.fork_url,
                crate_entry.upstream_url.as_deref().unwrap_or("null"),
                crate_entry.current_branch,
                crate_entry.commits_behind.map_or("null".to_string(), |n| n.to_string()),
                crate_entry.has_local_patches
            ));
        }
        json.push_str("\n]");
        json
    }
}
