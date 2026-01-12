use std::collections::{HashMap, HashSet};
use std::fs;

fn main() {
    println!("🔍 Analyzing Rust dependencies for forking strategy");

    let mut analyzer = RustDepAnalyzer::new();
    analyzer.load_rust_dependencies();

    // Query 1: All deps of rust (including transitive dev)
    println!("\n1️⃣ ALL DEPENDENCIES OF RUST (including transitive dev):");
    let all_deps = analyzer.get_all_rust_deps();
    println!("   Total dependencies: {}", all_deps.len());

    // Show breakdown by type
    let (direct, dev, build) = analyzer.categorize_deps(&all_deps);
    println!("   - Direct: {}", direct.len());
    println!("   - Dev: {}", dev.len());
    println!("   - Build: {}", build.len());

    // Query 2: All non-GitHub deps of rust
    println!("\n2️⃣ NON-GITHUB DEPENDENCIES OF RUST:");
    let non_github_deps = analyzer.get_non_github_deps(&all_deps);
    println!("   Non-GitHub dependencies: {}", non_github_deps.len());

    for (dep, source) in non_github_deps.iter().take(10) {
        println!("   - {} -> {}", dep, source);
    }
    if non_github_deps.len() > 10 {
        println!("   ... and {} more", non_github_deps.len() - 10);
    }

    // Query 3: Our GitHub mirrors of non-GitHub deps
    println!("\n3️⃣ OUR GITHUB MIRRORS OF NON-GITHUB DEPS:");
    let our_mirrors = analyzer.get_our_mirrors_of_non_github(&non_github_deps);
    println!("   We have mirrors for: {}", our_mirrors.len());

    for (dep, mirror_url) in &our_mirrors {
        println!("   ✅ {} -> {}", dep, mirror_url);
    }

    // Summary
    let missing_mirrors = non_github_deps.len() - our_mirrors.len();
    println!("\n📊 SUMMARY:");
    println!("   Total Rust deps: {}", all_deps.len());
    println!("   Non-GitHub deps: {}", non_github_deps.len());
    println!("   Our mirrors: {}", our_mirrors.len());
    println!("   Missing mirrors: {}", missing_mirrors);

    if missing_mirrors > 0 {
        println!("\n❌ MISSING MIRRORS RANKED BY EIGENVECTOR CENTRALITY:");
        let missing_with_usage = analyzer.rank_missing_by_usage(&non_github_deps, &our_mirrors);
        for (centrality_score, dep) in missing_with_usage.iter().take(20) {
            println!("   {}: {} (centrality: {})", centrality_score, dep, *centrality_score as f64 / 1000.0);
        }
        if missing_with_usage.len() > 20 {
            println!("   ... and {} more", missing_with_usage.len() - 20);
        }
    }
}

struct RustDepAnalyzer {
    rust_deps: HashMap<String, DepInfo>,
    our_forks: HashSet<String>,
    github_repos: HashSet<String>,
}

#[derive(Debug, Clone)]
struct DepInfo {
    name: String,
    dep_type: String, // "dependencies", "dev-dependencies", "build-dependencies"
    source: String,   // "github", "crates.io", "git", etc.
    url: Option<String>,
}

impl RustDepAnalyzer {
    fn new() -> Self {
        Self {
            rust_deps: HashMap::new(),
            our_forks: HashSet::new(),
            github_repos: HashSet::new(),
        }
    }

    fn load_rust_dependencies(&mut self) {
        println!("📥 Loading Rust dependencies...");

        // Load from rust Cargo.toml
        let rust_cargo_toml = "/mnt/data1/nix/vendor/rust/cargo2nix/submodules/rust/Cargo.toml";
        if let Ok(content) = fs::read_to_string(rust_cargo_toml) {
            self.parse_cargo_toml(&content);
        }

        // Load from rust Cargo.lock for transitive deps
        let rust_cargo_lock = "/mnt/data1/nix/vendor/rust/cargo2nix/submodules/rust/Cargo.lock";
        if let Ok(content) = fs::read_to_string(rust_cargo_lock) {
            self.parse_cargo_lock(&content);
        }

        // Load our existing forks
        self.load_our_forks();

        // Load GitHub repo list
        self.load_github_repos();

        println!("   Loaded {} dependencies", self.rust_deps.len());
    }

    fn parse_cargo_toml(&mut self, content: &str) {
        let mut current_section = String::new();

        for line in content.lines() {
            let line = line.trim();

            if line == "[dependencies]" {
                current_section = "dependencies".to_string();
                continue;
            } else if line == "[dev-dependencies]" {
                current_section = "dev-dependencies".to_string();
                continue;
            } else if line == "[build-dependencies]" {
                current_section = "build-dependencies".to_string();
                continue;
            } else if line.starts_with("[") {
                current_section.clear();
                continue;
            }

            if !current_section.is_empty() && line.contains("=") {
                if let Some(dep_name) = line.split('=').next() {
                    let dep_name = dep_name.trim();
                    if !dep_name.is_empty() && !dep_name.starts_with('#') {
                        let source = self.determine_source(line);
                        let dep_info = DepInfo {
                            name: dep_name.to_string(),
                            dep_type: current_section.clone(),
                            source: source.clone(),
                            url: self.extract_url(line),
                        };
                        self.rust_deps.insert(dep_name.to_string(), dep_info);
                    }
                }
            }
        }
    }

    fn parse_cargo_lock(&mut self, content: &str) {
        // Parse Cargo.lock for transitive dependencies
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("name = ") {
                if let Some(name) = line.strip_prefix("name = ") {
                    let name = name.trim_matches('"');
                    if !self.rust_deps.contains_key(name) {
                        let dep_info = DepInfo {
                            name: name.to_string(),
                            dep_type: "transitive".to_string(),
                            source: "crates.io".to_string(), // Most transitive deps are from crates.io
                            url: None,
                        };
                        self.rust_deps.insert(name.to_string(), dep_info);
                    }
                }
            }
        }
    }

    fn determine_source(&self, line: &str) -> String {
        if line.contains("github") {
            "github".to_string()
        } else if line.contains("git =") {
            "git".to_string()
        } else if line.contains("path =") {
            "local".to_string()
        } else {
            "crates.io".to_string()
        }
    }

    fn extract_url(&self, line: &str) -> Option<String> {
        if let Some(start) = line.find("git = \"") {
            if let Some(end) = line[start + 7..].find('"') {
                return Some(line[start + 7..start + 7 + end].to_string());
            }
        }
        None
    }

    fn load_our_forks(&mut self) {
        let submodules_path = "/mnt/data1/nix/vendor/rust/cargo2nix/submodules";
        if let Ok(entries) = fs::read_dir(submodules_path) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    if let Some(name) = entry.file_name().to_str() {
                        self.our_forks.insert(name.to_string());
                    }
                }
            }
        }
    }

    fn load_github_repos(&mut self) {
        // Load from our collected GitHub URLs
        if let Ok(content) = fs::read_to_string("../url-extractor/all_github_urls.txt") {
            for line in content.lines() {
                if line.contains("github.com") {
                    if let Some(repo_name) = self.extract_repo_name_from_url(line) {
                        self.github_repos.insert(repo_name);
                    }
                }
            }
        }
    }

    fn extract_repo_name_from_url(&self, url: &str) -> Option<String> {
        if let Some(start) = url.find("github.com/") {
            let path = &url[start + 11..];
            let parts: Vec<&str> = path.split('/').collect();
            if parts.len() >= 2 {
                return Some(format!("{}/{}", parts[0], parts[1]));
            }
        }
        None
    }

    // Query methods
    fn get_all_rust_deps(&self) -> Vec<String> {
        self.rust_deps.keys().cloned().collect()
    }

    fn categorize_deps(&self, deps: &[String]) -> (Vec<String>, Vec<String>, Vec<String>) {
        let mut direct = Vec::new();
        let mut dev = Vec::new();
        let mut build = Vec::new();

        for dep in deps {
            if let Some(info) = self.rust_deps.get(dep) {
                match info.dep_type.as_str() {
                    "dependencies" => direct.push(dep.clone()),
                    "dev-dependencies" => dev.push(dep.clone()),
                    "build-dependencies" => build.push(dep.clone()),
                    _ => {} // transitive
                }
            }
        }

        (direct, dev, build)
    }

    fn get_non_github_deps(&self, deps: &[String]) -> HashMap<String, String> {
        let mut non_github = HashMap::new();

        for dep in deps {
            if let Some(info) = self.rust_deps.get(dep) {
                if info.source != "github" {
                    non_github.insert(dep.clone(), info.source.clone());
                }
            }
        }

        non_github
    }

    fn get_our_mirrors_of_non_github(&self, non_github_deps: &HashMap<String, String>) -> HashMap<String, String> {
        let mut mirrors = HashMap::new();

        for (dep, _source) in non_github_deps {
            // Check if we have a fork of this dependency
            if self.our_forks.contains(dep) {
                mirrors.insert(dep.clone(), format!("https://github.com/meta-introspector/{}", dep));
            }

            // Also check with common name transformations
            let dep_with_hyphens = dep.replace("_", "-");
            if self.our_forks.contains(&dep_with_hyphens) {
                mirrors.insert(dep.clone(), format!("https://github.com/meta-introspector/{}", dep_with_hyphens));
            }
        }

        mirrors
    }

    fn rank_missing_by_usage(&self, non_github_deps: &HashMap<String, String>, our_mirrors: &HashMap<String, String>) -> Vec<(usize, String)> {
        let mut usage_counts = Vec::new();

        // Load precomputed eigenvector centrality from graph-loader
        let centrality_file = "../graph-loader/eigenvector_centrality.txt";
        let mut centrality_map = HashMap::new();

        if let Ok(content) = fs::read_to_string(centrality_file) {
            for line in content.lines() {
                if let Some((name, score)) = line.split_once(':') {
                    if let Ok(score) = score.trim().parse::<f64>() {
                        centrality_map.insert(name.trim().to_string(), score);
                    }
                }
            }
        }

        for (dep, _source) in non_github_deps {
            if !our_mirrors.contains_key(dep) {
                let centrality = centrality_map.get(dep).unwrap_or(&0.0);
                let usage_score = (centrality * 1000.0) as usize; // Scale for ranking
                usage_counts.push((usage_score, dep.clone()));
            }
        }

        usage_counts.sort_by(|a, b| b.0.cmp(&a.0));
        usage_counts
    }

    fn count_usage_across_repos(&self, dep_name: &str) -> usize {
        let mut count = 0;

        // Count in current rust dependencies
        if self.rust_deps.contains_key(dep_name) {
            count += 1;
        }

        // Search across all Cargo.toml files in the repository network
        if let Ok(output) = std::process::Command::new("find")
            .args(&["/mnt/data1/nix", "-name", "Cargo.toml", "-type", "f"])
            .output() {
            if let Ok(files) = String::from_utf8(output.stdout) {
                for file_path in files.lines() {
                    if let Ok(content) = fs::read_to_string(file_path) {
                        // Look for dependency declarations
                        if content.contains(&format!("{} =", dep_name)) ||
                           content.contains(&format!("\"{}\"", dep_name)) ||
                           content.contains(&format!("{} ", dep_name)) {
                            count += 1;
                        }
                    }
                }
            }
        }

        count
    }
}
