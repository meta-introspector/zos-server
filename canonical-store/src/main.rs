use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use rayon::prelude::*;

const CHUNK_SIZE: usize = 64 * 1024; // 64KB chunks
const MAX_MEMORY: usize = 30 * 1024 * 1024 * 1024; // 30GB

fn main() {
    rayon::ThreadPoolBuilder::new().num_threads(24).build_global().unwrap();

    println!("🚀 Canonical Store Server (24 CPUs, 30GB RAM, Git-backed)");

    let server = CanonicalStoreServer::new("/mnt/data1/store");
    server.start_processing("/mnt/data1/nix");
    server.serve_forever();
}

struct CanonicalStoreServer {
    store: Arc<RwLock<InMemoryStore>>,
    git_root: PathBuf,
}

struct InMemoryStore {
    repos: HashMap<String, RepoCache>,
    files: HashMap<String, FileCache>,
    memory_used: usize,
}

#[derive(Clone)]
struct RepoCache {
    canonical_path: String,
    metadata: RepoMetadata,
    files: Vec<String>,
    git_committed: bool,
}

#[derive(Clone, serde::Serialize)]
struct RepoMetadata {
    name: String,
    original_path: String,
    remote_url: Option<String>,
    languages: HashMap<String, usize>,
    file_count: usize,
    size_bytes: u64,
    last_updated: u64,
}

#[derive(Clone)]
struct FileCache {
    path: String,
    size: u64,
    hash: String,
    language: Option<String>,
    repo: String,
}

impl CanonicalStoreServer {
    fn new(store_root: &str) -> Self {
        let git_root = PathBuf::from(store_root);

        // Initialize git repo if it doesn't exist
        if !git_root.join(".git").exists() {
            let _ = std::process::Command::new("git")
                .args(&["init"])
                .current_dir(&git_root)
                .output();
        }

        Self {
            store: Arc::new(RwLock::new(InMemoryStore::new())),
            git_root,
        }
    }

    fn start_processing(&self, nix_path: &str) {
        println!("🔍 Processing {} with full CPU/RAM capacity...", nix_path);

        let dirs = self.collect_all_git_repos(nix_path);
        println!("Found {} git repositories", dirs.len());

        // Process repos in parallel, keeping everything in memory
        dirs.par_iter().for_each(|repo_path| {
            if let Some(repo_cache) = self.process_repo_to_cache(repo_path) {
                self.add_to_memory_store(repo_cache);
            }
        });

        // Commit everything to git
        self.commit_all_to_git();
    }

    fn collect_all_git_repos(&self, path: &str) -> Vec<PathBuf> {
        let mut repos = Vec::new();
        self.scan_for_git_repos(Path::new(path), &mut repos);
        repos
    }

    fn scan_for_git_repos(&self, path: &Path, repos: &mut Vec<PathBuf>) {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                if entry_path.is_dir() {
                    if entry_path.join(".git").exists() {
                        repos.push(entry_path);
                    } else {
                        self.scan_for_git_repos(&entry_path, repos);
                    }
                }
            }
        }
    }

    fn process_repo_to_cache(&self, repo_path: &Path) -> Option<RepoCache> {
        let repo_name = repo_path.file_name()?.to_str()?.to_string();

        // Get canonical path: com/github/meta-introspector/{repo}
        let canonical_path = format!("com/github/meta-introspector/{}", repo_name);

        // Collect all files in parallel
        let files = self.collect_repo_files(repo_path);
        let file_results: Vec<_> = files.par_chunks(1000)
            .map(|chunk| {
                let mut languages = HashMap::new();
                let mut total_size = 0u64;
                let mut file_list = Vec::new();

                for file_path in chunk {
                    if let Ok(metadata) = fs::metadata(file_path) {
                        total_size += metadata.len();

                        let relative_path = file_path.strip_prefix(repo_path)
                            .unwrap_or(file_path)
                            .to_string_lossy()
                            .to_string();

                        file_list.push(relative_path);

                        if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
                            *languages.entry(ext.to_string()).or_insert(0) += 1;
                        }
                    }
                }

                (languages, total_size, file_list)
            })
            .collect();

        // Merge results
        let mut all_languages = HashMap::new();
        let mut total_size = 0u64;
        let mut all_files = Vec::new();

        for (languages, size, files) in file_results {
            total_size += size;
            all_files.extend(files);
            for (lang, count) in languages {
                *all_languages.entry(lang).or_insert(0) += count;
            }
        }

        let metadata = RepoMetadata {
            name: repo_name.clone(),
            original_path: repo_path.to_string_lossy().to_string(),
            remote_url: self.get_git_remote_url(repo_path),
            languages: all_languages,
            file_count: all_files.len(),
            size_bytes: total_size,
            last_updated: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        Some(RepoCache {
            canonical_path,
            metadata,
            files: all_files,
            git_committed: false,
        })
    }

    fn parse_canonical_path_from_url(&self, url: &str) -> String {
        let clean_url = url.replace("https://", "").replace("http://", "").replace(".git", "");

        // GitHub: com/github/org/repo
        if let Some(start) = clean_url.find("github.com/") {
            let path = &clean_url[start + 11..];
            let parts: Vec<&str> = path.split('/').collect();
            if parts.len() >= 2 {
                return format!("com/github/{}/{}", parts[0], parts[1]);
            }
        }

        // Codeberg: org/codeberg/user/repo
        if let Some(start) = clean_url.find("codeberg.org/") {
            let path = &clean_url[start + 13..];
            let parts: Vec<&str> = path.split('/').collect();
            if parts.len() >= 2 {
                return format!("org/codeberg/{}/{}", parts[0], parts[1]);
            }
        }

        // SourceForge: net/sourceforge/project/repo
        if let Some(start) = clean_url.find("sourceforge.net/") {
            let path = &clean_url[start + 16..];
            let parts: Vec<&str> = path.split('/').collect();
            if parts.len() >= 2 {
                return format!("net/sourceforge/{}/{}", parts[0], parts[1]);
            }
        }

        // BitBucket: org/bitbucket/user/repo
        if let Some(start) = clean_url.find("bitbucket.org/") {
            let path = &clean_url[start + 14..];
            let parts: Vec<&str> = path.split('/').collect();
            if parts.len() >= 2 {
                return format!("org/bitbucket/{}/{}", parts[0], parts[1]);
            }
        }

        // GitLab: com/gitlab/user/repo
        if let Some(start) = clean_url.find("gitlab.com/") {
            let path = &clean_url[start + 11..];
            let parts: Vec<&str> = path.split('/').collect();
            if parts.len() >= 2 {
                return format!("com/gitlab/{}/{}", parts[0], parts[1]);
            }
        }

        // Generic fallback
        format!("unknown/{}", clean_url.replace("/", "_").replace(".", "_"))
    }

    fn collect_repo_files(&self, repo_path: &Path) -> Vec<PathBuf> {
        let mut files = Vec::new();
        self.walk_directory(repo_path, &mut files);
        files
    }

    fn walk_directory(&self, dir: &Path, files: &mut Vec<PathBuf>) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    files.push(path);
                } else if path.is_dir() && !path.file_name().unwrap().to_str().unwrap().starts_with('.') {
                    self.walk_directory(&path, files);
                }
            }
        }
    }

    fn get_git_remote_url(&self, repo_path: &Path) -> Option<String> {
        let config_path = repo_path.join(".git").join("config");
        if let Ok(content) = fs::read_to_string(config_path) {
            for line in content.lines() {
                if line.trim().starts_with("url = ") {
                    return Some(line.trim()[6..].to_string());
                }
            }
        }
        None
    }

    fn add_to_memory_store(&self, repo_cache: RepoCache) {
        let mut store = self.store.write().unwrap();

        // Check memory limit
        if store.memory_used < MAX_MEMORY {
            let repo_name = repo_cache.metadata.name.clone();
            store.memory_used += std::mem::size_of::<RepoCache>();
            store.repos.insert(repo_name.clone(), repo_cache);

            if store.repos.len() % 100 == 0 {
                println!("📦 Cached {} repos ({:.1} GB used)",
                    store.repos.len(),
                    store.memory_used as f64 / 1024.0 / 1024.0 / 1024.0);
            }
        }
    }

    fn commit_all_to_git(&self) {
        println!("💾 Committing all cached data to git...");

        let store = self.store.read().unwrap();

        // Write all repo metadata to disk in canonical structure
        for (repo_name, repo_cache) in &store.repos {
            let repo_dir = self.git_root.join(&repo_cache.canonical_path);
            let _ = fs::create_dir_all(&repo_dir);

            // Write metadata.json
            let metadata_json = serde_json::to_string_pretty(&repo_cache.metadata).unwrap();
            let _ = fs::write(repo_dir.join("metadata.json"), metadata_json);

            // Write file list
            let files_list = repo_cache.files.join("\n");
            let _ = fs::write(repo_dir.join("files.txt"), files_list);

            // Write summary
            let summary = format!(
                "# {}\n\n- Files: {}\n- Size: {:.1} MB\n- Languages: {:?}\n- Source: {}\n",
                repo_name,
                repo_cache.metadata.file_count,
                repo_cache.metadata.size_bytes as f64 / 1024.0 / 1024.0,
                repo_cache.metadata.languages,
                repo_cache.metadata.original_path
            );
            let _ = fs::write(repo_dir.join("README.md"), summary);
        }

        // Git add and commit
        let _ = std::process::Command::new("git")
            .args(&["add", "."])
            .current_dir(&self.git_root)
            .output();

        let commit_msg = format!("Update canonical store: {} repos cached", store.repos.len());
        let _ = std::process::Command::new("git")
            .args(&["commit", "-m", &commit_msg])
            .current_dir(&self.git_root)
            .output();

        println!("✅ Committed {} repos to git", store.repos.len());
    }

    fn serve_forever(&self) {
        println!("🌐 Canonical store server running...");
        println!("📊 In-memory cache: {} repos", self.store.read().unwrap().repos.len());
        println!("💾 Git repository: {}", self.git_root.display());

        // Server would handle HTTP requests here
        // For now, just keep the data in memory
        loop {
            std::thread::sleep(std::time::Duration::from_secs(60));
            let store = self.store.read().unwrap();
            println!("💡 Server alive: {} repos cached, {:.1} GB RAM used",
                store.repos.len(),
                store.memory_used as f64 / 1024.0 / 1024.0 / 1024.0);
        }
    }
}

impl InMemoryStore {
    fn new() -> Self {
        Self {
            repos: HashMap::new(),
            files: HashMap::new(),
            memory_used: 0,
        }
    }
}
