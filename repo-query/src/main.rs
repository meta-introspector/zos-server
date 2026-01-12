use std::collections::{HashMap, HashSet};
use std::fs;

fn main() {
    println!("🔍 Loading repository data for querying...");

    let mut db = RepoDatabase::new();
    db.load_all_data();

    println!("✅ Database loaded!");
    println!("📊 Stats: {} repos, {} dependencies, {} crates",
        db.repos.len(), db.dependencies.len(), db.crates.len());

    // Example queries
    println!("\n🔍 Sample Queries:");

    // Query 1: Most popular crates
    println!("\n1. Top 10 most used crates:");
    let popular_crates = db.query_most_popular_crates(10);
    for (crate_name, count) in popular_crates {
        println!("   {} -> {} repos", crate_name, count);
    }

    // Query 2: Repos we need to fork for rustc
    println!("\n2. Missing rustc dependencies we need to fork:");
    let missing = db.query_missing_rustc_deps();
    println!("   {} missing dependencies", missing.len());
    for dep in missing.iter().take(5) {
        println!("   - {}", dep);
    }

    // Query 3: Most connected repositories
    println!("\n3. Most connected repositories:");
    let connected = db.query_most_connected_repos(5);
    for (repo, connections) in connected {
        println!("   {} -> {} connections", repo, connections);
    }

    // Query 4: Repositories by language
    println!("\n4. Rust repositories:");
    let rust_repos = db.query_repos_by_language("Rust");
    println!("   {} Rust repositories found", rust_repos.len());

    // Interactive query mode
    println!("\n💡 Available queries:");
    println!("   - most_popular_crates(n)");
    println!("   - missing_rustc_deps()");
    println!("   - most_connected_repos(n)");
    println!("   - repos_by_language(lang)");
    println!("   - crates_in_repo(repo_name)");
    println!("   - dependencies_of(crate_name)");
}

struct RepoDatabase {
    repos: HashMap<String, RepoInfo>,
    dependencies: HashMap<String, HashSet<String>>,
    crates: HashMap<String, CrateInfo>,
    github_urls: HashSet<String>,
    gitmodule_urls: HashSet<String>,
    git_remote_urls: HashSet<String>,
    missing_deps: HashSet<String>,
    existing_forks: HashSet<String>,
}

#[derive(Debug)]
struct RepoInfo {
    name: String,
    url: Option<String>,
    language: Option<String>,
    stars: u32,
    forks: u32,
    has_submodules: bool,
}

#[derive(Debug)]
struct CrateInfo {
    name: String,
    repos: HashSet<String>,
    dependencies: HashSet<String>,
}

impl RepoDatabase {
    fn new() -> Self {
        Self {
            repos: HashMap::new(),
            dependencies: HashMap::new(),
            crates: HashMap::new(),
            github_urls: HashSet::new(),
            gitmodule_urls: HashSet::new(),
            git_remote_urls: HashSet::new(),
            missing_deps: HashSet::new(),
            existing_forks: HashSet::new(),
        }
    }

    fn load_all_data(&mut self) {
        println!("📥 Loading GitHub URLs...");
        self.load_urls("../url-extractor/all_github_urls.txt", &mut self.github_urls);

        println!("📥 Loading gitmodule URLs...");
        self.load_urls("../gitmodules-extractor/gitmodules_urls.txt", &mut self.gitmodule_urls);

        println!("📥 Loading git remote URLs...");
        self.load_urls("../git-config-extractor/git_remote_urls.txt", &mut self.git_remote_urls);

        println!("📥 Loading missing dependencies...");
        self.load_urls("../fork-status/missing_external_deps.txt", &mut self.missing_deps);

        println!("📥 Loading existing forks...");
        self.load_existing_forks();

        println!("📥 Loading GitHub metadata...");
        self.load_github_metadata();
    }

    fn load_urls(&mut self, file_path: &str, target: &mut HashSet<String>) {
        if let Ok(content) = fs::read_to_string(file_path) {
            for line in content.lines() {
                if !line.trim().is_empty() {
                    target.insert(line.trim().to_string());
                }
            }
        }
    }

    fn load_existing_forks(&mut self) {
        let submodules_path = "/mnt/data1/nix/vendor/rust/cargo2nix/submodules";
        if let Ok(entries) = fs::read_dir(submodules_path) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    if let Some(name) = entry.file_name().to_str() {
                        self.existing_forks.insert(name.to_string());
                    }
                }
            }
        }
    }

    fn load_github_metadata(&mut self) {
        // Load from GitHub JSON files
        let files = [
            "/mnt/data1/nix/index/starred.json",
            "/mnt/data1/nix/index/github_meta-introspector_repos.json"
        ];

        for file in &files {
            if let Ok(content) = fs::read_to_string(file) {
                if let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(repos) = data.as_array() {
                        for repo in repos {
                            if let Some(full_name) = repo["full_name"].as_str() {
                                let repo_info = RepoInfo {
                                    name: full_name.to_string(),
                                    url: repo["clone_url"].as_str().map(|s| s.to_string()),
                                    language: repo["language"].as_str().map(|s| s.to_string()),
                                    stars: repo["stargazers_count"].as_u64().unwrap_or(0) as u32,
                                    forks: repo["forks_count"].as_u64().unwrap_or(0) as u32,
                                    has_submodules: false, // Will be updated later
                                };
                                self.repos.insert(full_name.to_string(), repo_info);
                            }
                        }
                    }
                }
            }
        }
    }

    // Query methods
    fn query_most_popular_crates(&self, limit: usize) -> Vec<(String, usize)> {
        let mut crate_counts: Vec<_> = self.crates.iter()
            .map(|(name, info)| (name.clone(), info.repos.len()))
            .collect();
        crate_counts.sort_by(|a, b| b.1.cmp(&a.1));
        crate_counts.into_iter().take(limit).collect()
    }

    fn query_missing_rustc_deps(&self) -> Vec<String> {
        self.missing_deps.iter().cloned().collect()
    }

    fn query_most_connected_repos(&self, limit: usize) -> Vec<(String, usize)> {
        let mut connections: Vec<_> = self.dependencies.iter()
            .map(|(repo, deps)| (repo.clone(), deps.len()))
            .collect();
        connections.sort_by(|a, b| b.1.cmp(&a.1));
        connections.into_iter().take(limit).collect()
    }

    fn query_repos_by_language(&self, language: &str) -> Vec<String> {
        self.repos.iter()
            .filter(|(_, info)| info.language.as_deref() == Some(language))
            .map(|(name, _)| name.clone())
            .collect()
    }

    fn query_crates_in_repo(&self, repo_name: &str) -> Vec<String> {
        self.crates.iter()
            .filter(|(_, info)| info.repos.contains(repo_name))
            .map(|(name, _)| name.clone())
            .collect()
    }

    fn query_dependencies_of(&self, crate_name: &str) -> Vec<String> {
        if let Some(crate_info) = self.crates.get(crate_name) {
            crate_info.dependencies.iter().cloned().collect()
        } else {
            Vec::new()
        }
    }
}
