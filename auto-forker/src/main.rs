use std::collections::HashMap;
use std::fs;
use std::process::Command;

fn main() {
    println!("🍴 Auto-Fork Missing Rust Dependencies");

    let mut forker = AutoForker::new();
    forker.load_missing_deps();
    forker.check_existing_forks();
    forker.find_upstream_urls();
    forker.create_forks();
}

struct AutoForker {
    missing_deps: Vec<(String, usize)>, // (dep_name, ref_count)
    existing_forks: HashMap<String, String>, // crate_name -> dir_name
    upstream_urls: HashMap<String, String>, // crate_name -> upstream_url
}

impl AutoForker {
    fn new() -> Self {
        Self {
            missing_deps: Vec::new(),
            existing_forks: HashMap::new(),
            upstream_urls: HashMap::new(),
        }
    }

    fn load_missing_deps(&mut self) {
        println!("📋 Loading missing dependencies from analysis...");

        // Get missing deps from complete-rust-analyzer output
        let missing = vec![
            ("build_common", 12),
            ("tracing-subscriber", 12),
            ("cov-mark", 10),
            ("url", 10),
            ("crossbeam-channel", 8),
            ("getopts", 7),
            ("salsa-macros", 6),
        ];

        self.missing_deps = missing.into_iter()
            .map(|(name, count)| (name.to_string(), count))
            .collect();

        println!("   Loaded {} missing dependencies", self.missing_deps.len());
    }

    fn check_existing_forks(&mut self) {
        println!("🔍 Checking for existing forks...");

        // Load existing fork mappings
        if let Ok(content) = fs::read_to_string("../remote-fork-mapper/crate_directory_cache.txt") {
            for line in content.lines() {
                if let Some((crate_name, dir_name)) = line.split_once('\t') {
                    self.existing_forks.insert(crate_name.to_string(), dir_name.to_string());
                }
            }
        }

        // Check GitHub for existing forks
        for (dep_name, _count) in &self.missing_deps {
            if !self.existing_forks.contains_key(dep_name) {
                if self.check_github_fork_exists(dep_name) {
                    println!("   ✅ Fork exists on GitHub: {}", dep_name);
                    self.existing_forks.insert(dep_name.clone(), dep_name.clone());
                }
            }
        }

        println!("   Found {} existing forks", self.existing_forks.len());
    }

    fn check_github_fork_exists(&self, crate_name: &str) -> bool {
        let output = Command::new("gh")
            .args(&["repo", "view", &format!("meta-introspector/{}", crate_name)])
            .output();

        if let Ok(output) = output {
            output.status.success()
        } else {
            false
        }
    }

    fn find_upstream_urls(&mut self) {
        println!("🔗 Finding upstream URLs for missing dependencies...");

        for (dep_name, _count) in &self.missing_deps {
            if !self.existing_forks.contains_key(dep_name) {
                if let Some(upstream_url) = self.find_crates_io_repo(dep_name) {
                    self.upstream_urls.insert(dep_name.clone(), upstream_url);
                    println!("   📦 {}: {}", dep_name, self.upstream_urls[dep_name]);
                }
            }
        }

        println!("   Found {} upstream URLs", self.upstream_urls.len());
    }

    fn find_crates_io_repo(&self, crate_name: &str) -> Option<String> {
        // Query crates.io API for repository URL
        let output = Command::new("curl")
            .args(&["-s", &format!("https://crates.io/api/v1/crates/{}", crate_name)])
            .output()
            .ok()?;

        if output.status.success() {
            let json_str = String::from_utf8(output.stdout).ok()?;
            // Simple JSON parsing for repository field
            if let Some(start) = json_str.find("\"repository\":\"") {
                let start = start + 14;
                if let Some(end) = json_str[start..].find('"') {
                    let repo_url = &json_str[start..start + end];
                    if repo_url.contains("github.com") {
                        return Some(repo_url.to_string());
                    }
                }
            }
        }
        None
    }

    fn create_forks(&self) {
        println!("🚀 Creating forks and submodules...");

        let submodules_dir = "/mnt/data1/nix/vendor/rust/cargo2nix/submodules";

        for (dep_name, upstream_url) in &self.upstream_urls {
            println!("\n🍴 Processing: {}", dep_name);

            // Step 1: Fork on GitHub
            println!("   Forking {} to meta-introspector...", dep_name);
            let fork_result = Command::new("gh")
                .args(&["repo", "fork", upstream_url, "--remote", "--org", "meta-introspector"])
                .output();

            match fork_result {
                Ok(output) if output.status.success() => {
                    println!("   ✅ Fork created successfully");
                },
                Ok(output) => {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    if stderr.contains("already exists") {
                        println!("   ℹ️  Fork already exists");
                    } else {
                        println!("   ❌ Fork failed: {}", stderr);
                        continue;
                    }
                },
                Err(e) => {
                    println!("   ❌ Fork command failed: {}", e);
                    continue;
                }
            }

            // Step 2: Add as submodule
            let fork_url = format!("https://github.com/meta-introspector/{}", dep_name);
            let submodule_path = format!("{}/{}", submodules_dir, dep_name);

            println!("   Adding submodule: {}", dep_name);
            let submodule_result = Command::new("git")
                .args(&["-C", submodules_dir, "submodule", "add", &fork_url, dep_name])
                .output();

            match submodule_result {
                Ok(output) if output.status.success() => {
                    println!("   ✅ Submodule added: {}", submodule_path);
                },
                Ok(output) => {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    if stderr.contains("already exists") {
                        println!("   ℹ️  Submodule already exists");
                    } else {
                        println!("   ❌ Submodule failed: {}", stderr);
                    }
                },
                Err(e) => {
                    println!("   ❌ Submodule command failed: {}", e);
                }
            }
        }

        println!("\n🎉 Fork creation complete!");
        println!("📊 Summary:");
        println!("   Upstream URLs found: {}", self.upstream_urls.len());
        println!("   Forks attempted: {}", self.upstream_urls.len());
    }
}
