use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::sync::{Arc, RwLock};

fn main() {
    rayon::ThreadPoolBuilder::new().num_threads(24).build_global().unwrap();

    println!("📊 PHASE 1: Loading existing repo metadata");
    let mut graph = RepoGraph::new();

    // Step 1: Read existing JSON metadata files
    graph.load_from_json_files("/mnt/data1/nix/index");

    println!("📊 PHASE 2: Building repository graph");
    // Step 2: Build connections between repos (forks, languages, owners)
    graph.build_connections();

    println!("📊 PHASE 3: Scanning disk for missing repos");
    // Step 3: Find repos on disk that aren't in our graph yet
    let disk_repos = find_all_git_repos("/mnt/data1/nix");
    let missing_repos = graph.find_missing_repos(&disk_repos);

    println!("Found {} repos on disk, {} missing from graph", disk_repos.len(), missing_repos.len());

    println!("📊 PHASE 4: Adding missing repos to graph");
    // Step 4: Process missing repos and add to graph
    graph.add_missing_repos(missing_repos);

    println!("✅ Complete! Graph has {} repos with {} connections",
        graph.repo_count(), graph.connection_count());
}

struct RepoGraph {
    repos: HashMap<String, RepoNode>,
    connections: HashMap<String, HashSet<String>>,
}

struct RepoNode {
    name: String,
    path: String,
    remote_url: Option<String>,
    language: Option<String>,
    owner: Option<String>,
    stars: u32,
    forks: u32,
    file_count: usize,
}

impl RepoGraph {
    fn new() -> Self {
        Self {
            repos: HashMap::new(),
            connections: HashMap::new(),
        }
    }

    fn load_from_json_files(&mut self, index_dir: &str) {
        println!("🔍 Loading JSON metadata files...");

        if let Ok(entries) = fs::read_dir(index_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "json") {
                    println!("Loading: {}", path.display());

                    if let Ok(data) = fs::read_to_string(&path) {
                        self.parse_json_data(&data);
                    }
                }
            }
        }

        println!("✅ Loaded {} repos from JSON", self.repos.len());
    }

    fn parse_json_data(&mut self, json_data: &str) {
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(json_data) {
            if let Some(repos) = data.as_array() {
                for repo in repos {
                    if let Some(full_name) = repo["full_name"].as_str() {
                        let node = RepoNode {
                            name: full_name.to_string(),
                            path: String::new(), // Will fill from disk scan
                            remote_url: repo["clone_url"].as_str().map(|s| s.to_string()),
                            language: repo["language"].as_str().map(|s| s.to_string()),
                            owner: repo["owner"]["login"].as_str().map(|s| s.to_string()),
                            stars: repo["stargazers_count"].as_u64().unwrap_or(0) as u32,
                            forks: repo["forks_count"].as_u64().unwrap_or(0) as u32,
                            file_count: 0, // Will count from disk
                        };

                        self.repos.insert(full_name.to_string(), node);
                    }
                }
            }
        }
    }

    fn build_connections(&mut self) {
        println!("🔗 Building repository connections...");

        for (repo_name, repo) in &self.repos {
            let mut connections = HashSet::new();

            // Connect by language
            if let Some(lang) = &repo.language {
                let lang_key = format!("lang:{}", lang);
                connections.insert(lang_key);
            }

            // Connect by owner
            if let Some(owner) = &repo.owner {
                let owner_key = format!("user:{}", owner);
                connections.insert(owner_key);
            }

            self.connections.insert(repo_name.clone(), connections);
        }

        println!("✅ Built connections for {} repos", self.connections.len());
    }

    fn find_missing_repos(&self, disk_repos: &[String]) -> Vec<String> {
        disk_repos.iter()
            .filter(|repo| !self.repos.contains_key(*repo))
            .cloned()
            .collect()
    }

    fn add_missing_repos(&mut self, missing_repos: Vec<String>) {
        println!("➕ Adding {} missing repos...", missing_repos.len());

        missing_repos.par_iter().for_each(|repo_path| {
            // Process each missing repo
            if let Some(repo_name) = Path::new(repo_path).file_name().and_then(|n| n.to_str()) {
                // This would be filled in with actual repo processing
                println!("Processing: {}", repo_name);
            }
        });
    }

    fn repo_count(&self) -> usize {
        self.repos.len()
    }

    fn connection_count(&self) -> usize {
        self.connections.values().map(|c| c.len()).sum()
    }
}

fn find_all_git_repos(root: &str) -> Vec<String> {
    println!("🔍 Scanning {} for git repositories...", root);

    let repos = Arc::new(RwLock::new(Vec::new()));

    // Parallel directory scanning
    scan_directory_parallel(Path::new(root), repos.clone());

    let final_repos = repos.read().unwrap().clone();
    println!("✅ Found {} git repositories on disk", final_repos.len());

    final_repos
}

fn scan_directory_parallel(dir: &Path, repos: Arc<RwLock<Vec<String>>>) {
    if let Ok(entries) = fs::read_dir(dir) {
        let entries: Vec<_> = entries.collect();

        entries.par_iter().for_each(|entry| {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    if path.join(".git").exists() {
                        repos.write().unwrap().push(path.to_string_lossy().to_string());
                    } else {
                        scan_directory_parallel(&path, repos.clone());
                    }
                }
            }
        });
    }
}
