use std::collections::HashMap;
use std::fs;
use std::process::Command;

fn main() {
    println!("🔍 Mapping Directories → Remotes → Forks → Crates");

    let mut mapper = RemoteForkMapper::new();
    mapper.scan_all_submodules();
    mapper.load_crate_names();
    mapper.create_mappings();
    mapper.report();
}

struct RemoteForkMapper {
    dir_to_remote: HashMap<String, String>,
    remote_to_crate: HashMap<String, String>,
    crate_to_dir: HashMap<String, String>,
}

impl RemoteForkMapper {
    fn new() -> Self {
        Self {
            dir_to_remote: HashMap::new(),
            remote_to_crate: HashMap::new(),
            crate_to_dir: HashMap::new(),
        }
    }

    fn scan_all_submodules(&mut self) {
        println!("📁 Scanning submodule directories for git remotes...");

        // Check if we have cached data first
        let cache_file = "directory_remote_cache.txt";
        if std::path::Path::new(cache_file).exists() {
            println!("   Loading from cache: {}", cache_file);
            if let Ok(content) = fs::read_to_string(cache_file) {
                for line in content.lines() {
                    if let Some((dir, remote)) = line.split_once('\t') {
                        self.dir_to_remote.insert(dir.to_string(), remote.to_string());
                    }
                }
                println!("   Loaded {} cached directory → remote mappings", self.dir_to_remote.len());
                return;
            }
        }

        // Scan fresh if no cache
        let submodules_path = "/mnt/data1/nix/vendor/rust/cargo2nix/submodules";
        let mut cache_content = String::new();

        if let Ok(entries) = fs::read_dir(submodules_path) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    if let Some(dir_name) = entry.file_name().to_str() {
                        let dir_path = entry.path();
                        if let Some(remote_url) = self.get_git_remote(&dir_path) {
                            self.dir_to_remote.insert(dir_name.to_string(), remote_url.clone());
                            cache_content.push_str(&format!("{}\t{}\n", dir_name, remote_url));
                        }
                    }
                }
            }
        }

        // Save cache
        let _ = fs::write(cache_file, cache_content);
        println!("   Found {} directories with git remotes (cached)", self.dir_to_remote.len());
    }

    fn get_git_remote(&self, dir_path: &std::path::Path) -> Option<String> {
        let output = Command::new("git")
            .args(&["-C", &dir_path.to_string_lossy(), "remote", "get-url", "origin"])
            .output()
            .ok()?;

        if output.status.success() {
            let url = String::from_utf8(output.stdout).ok()?.trim().to_string();
            Some(url)
        } else {
            None
        }
    }

    fn load_crate_names(&mut self) {
        println!("📦 Loading crate names from Cargo.toml files...");

        // Check cache first
        let cache_file = "crate_directory_cache.txt";
        if std::path::Path::new(cache_file).exists() {
            println!("   Loading from cache: {}", cache_file);
            if let Ok(content) = fs::read_to_string(cache_file) {
                for line in content.lines() {
                    if let Some((crate_name, dir_name)) = line.split_once('\t') {
                        self.crate_to_dir.insert(crate_name.to_string(), dir_name.to_string());
                    }
                }
                println!("   Loaded {} cached crate → directory mappings", self.crate_to_dir.len());
                return;
            }
        }

        // Scan fresh if no cache
        let mut cache_content = String::new();
        for (dir_name, _remote_url) in &self.dir_to_remote {
            let cargo_toml = format!("/mnt/data1/nix/vendor/rust/cargo2nix/submodules/{}/Cargo.toml", dir_name);
            if let Ok(content) = fs::read_to_string(&cargo_toml) {
                if let Some(crate_name) = self.extract_crate_name(&content) {
                    self.crate_to_dir.insert(crate_name.clone(), dir_name.clone());
                    cache_content.push_str(&format!("{}\t{}\n", crate_name, dir_name));
                }
            }
        }

        // Save cache
        let _ = fs::write(cache_file, cache_content);
        println!("   Mapped {} crate names to directories (cached)", self.crate_to_dir.len());
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

    fn create_mappings(&mut self) {
        println!("🔗 Creating remote → crate mappings...");

        for (crate_name, dir_name) in &self.crate_to_dir {
            if let Some(remote_url) = self.dir_to_remote.get(dir_name) {
                self.remote_to_crate.insert(remote_url.clone(), crate_name.clone());
            }
        }

        println!("   Created {} remote → crate mappings", self.remote_to_crate.len());
    }

    fn report(&self) {
        println!("\n📊 COMPLETE MAPPING REPORT");

        println!("\n🗂️  DIRECTORY → REMOTE → CRATE MAPPINGS:");
        let mut mappings: Vec<_> = self.dir_to_remote.iter().collect();
        mappings.sort_by_key(|(dir, _)| *dir);

        for (dir_name, remote_url) in mappings.iter().take(20) {
            let crate_name = self.crate_to_dir.iter()
                .find(|(_, d)| *d == *dir_name)
                .map(|(c, _)| c.as_str())
                .unwrap_or("unknown");

            let repo_name = self.extract_repo_name(remote_url);
            println!("   {} → {} → crate:{}", dir_name, repo_name, crate_name);
        }

        if self.dir_to_remote.len() > 20 {
            println!("   ... and {} more mappings", self.dir_to_remote.len() - 20);
        }

        println!("\n🔍 NAME MISMATCHES (dir ≠ crate):");
        let mut mismatches = 0;
        for (crate_name, dir_name) in &self.crate_to_dir {
            if crate_name != dir_name && !dir_name.contains(crate_name) && !crate_name.contains(dir_name) {
                if let Some(remote_url) = self.dir_to_remote.get(dir_name) {
                    let repo_name = self.extract_repo_name(remote_url);
                    println!("   crate:'{}' in dir:'{}' → {}", crate_name, dir_name, repo_name);
                    mismatches += 1;
                }
            }
        }

        if mismatches == 0 {
            println!("   No significant mismatches found");
        }

        println!("\n📈 SUMMARY:");
        println!("   Total directories: {}", self.dir_to_remote.len());
        println!("   Crates mapped: {}", self.crate_to_dir.len());
        println!("   Name mismatches: {}", mismatches);
    }

    fn extract_repo_name(&self, url: &str) -> String {
        if let Some(start) = url.rfind('/') {
            let repo = &url[start + 1..];
            if repo.ends_with(".git") {
                repo.strip_suffix(".git").unwrap_or(repo).to_string()
            } else {
                repo.to_string()
            }
        } else {
            url.to_string()
        }
    }
}
