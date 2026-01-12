use std::collections::{HashMap, HashSet};
use std::fs;

fn main() {
    println!("🔗 Building repository dependency graph");

    let mut graph = DependencyGraph::new();

    // Step 1: Load all URLs and normalize them
    println!("📊 Loading URL datasets...");
    let github_urls = load_urls("../url-extractor/all_github_urls.txt");
    let gitmodules_urls = load_urls("../gitmodules-extractor/gitmodules_urls.txt");
    let git_remote_urls = load_urls("../git-config-extractor/git_remote_urls.txt");

    println!("GitHub URLs: {}", github_urls.len());
    println!("Gitmodules URLs: {}", gitmodules_urls.len());
    println!("Git remote URLs: {}", git_remote_urls.len());

    // Step 2: Build canonical URL mapping
    println!("\n🏗️ Building canonical URL mapping...");
    for url in &github_urls {
        graph.add_canonical_url(url);
    }
    for url in &gitmodules_urls {
        graph.add_canonical_url(url);
    }
    for url in &git_remote_urls {
        graph.add_canonical_url(url);
    }

    // Step 3: Build dependency relationships
    println!("\n🔗 Building dependency relationships...");

    // Read .gitmodules files to build parent->child relationships
    let gitmodules_list = fs::read_to_string("../gitmodules_list.txt").unwrap();
    for gitmodules_path in gitmodules_list.lines() {
        if let Ok(content) = fs::read_to_string(gitmodules_path) {
            let parent_repo = extract_parent_repo(gitmodules_path);
            let submodule_urls = extract_submodule_urls(&content);

            for submodule_url in submodule_urls {
                graph.add_dependency(&parent_repo, &submodule_url);
            }
        }
    }

    println!("\n📊 Graph Statistics:");
    println!("Canonical repos: {}", graph.canonical_urls.len());
    println!("Dependencies: {}", graph.dependencies.values().map(|deps| deps.len()).sum::<usize>());

    // Find most referenced repos
    let mut reference_counts: HashMap<String, usize> = HashMap::new();
    for deps in graph.dependencies.values() {
        for dep in deps {
            *reference_counts.entry(dep.clone()).or_insert(0) += 1;
        }
    }

    let mut sorted_refs: Vec<_> = reference_counts.iter().collect();
    sorted_refs.sort_by(|a, b| b.1.cmp(a.1));

    println!("\n🔥 Most referenced repositories:");
    for (repo, count) in sorted_refs.iter().take(10) {
        println!("  {} -> {} references", repo, count);
    }

    // Find repos with most dependencies
    let mut dep_counts: Vec<_> = graph.dependencies.iter()
        .map(|(repo, deps)| (repo, deps.len()))
        .collect();
    dep_counts.sort_by(|a, b| b.1.cmp(&a.1));

    println!("\n📦 Repositories with most dependencies:");
    for (repo, count) in dep_counts.iter().take(10) {
        println!("  {} -> {} dependencies", repo, count);
    }
}

struct DependencyGraph {
    canonical_urls: HashMap<String, String>, // URL -> canonical form
    dependencies: HashMap<String, HashSet<String>>, // parent -> children
}

impl DependencyGraph {
    fn new() -> Self {
        Self {
            canonical_urls: HashMap::new(),
            dependencies: HashMap::new(),
        }
    }

    fn add_canonical_url(&mut self, url: &str) {
        let canonical = normalize_url(url);
        self.canonical_urls.insert(url.to_string(), canonical);
    }

    fn add_dependency(&mut self, parent: &str, child_url: &str) {
        let parent_canonical = self.canonical_urls.get(parent)
            .cloned()
            .unwrap_or_else(|| normalize_url(parent));
        let child_canonical = self.canonical_urls.get(child_url)
            .cloned()
            .unwrap_or_else(|| normalize_url(child_url));

        self.dependencies.entry(parent_canonical)
            .or_default()
            .insert(child_canonical);
    }
}

fn load_urls(file_path: &str) -> Vec<String> {
    fs::read_to_string(file_path)
        .unwrap_or_default()
        .lines()
        .map(|s| s.to_string())
        .collect()
}

fn normalize_url(url: &str) -> String {
    let clean = url.replace("https://", "").replace("http://", "").replace(".git", "");

    // Convert to canonical path format
    if let Some(start) = clean.find("github.com/") {
        let path = &clean[start + 11..];
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() >= 2 {
            return format!("com/github/{}/{}", parts[0], parts[1]);
        }
    }

    if let Some(start) = clean.find("gitlab.com/") {
        let path = &clean[start + 11..];
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() >= 2 {
            return format!("com/gitlab/{}/{}", parts[0], parts[1]);
        }
    }

    // Generic fallback
    clean.replace("/", "_").replace(".", "_")
}

fn extract_parent_repo(gitmodules_path: &str) -> String {
    if let Some(gitmodules_pos) = gitmodules_path.rfind("/.gitmodules") {
        let repo_path = &gitmodules_path[..gitmodules_pos];
        if let Some(last_slash) = repo_path.rfind('/') {
            return repo_path[last_slash + 1..].to_string();
        }
    }
    "unknown".to_string()
}

fn extract_submodule_urls(content: &str) -> Vec<String> {
    let mut urls = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("url = ") {
            let url = line[6..].trim().trim_matches('"').trim_matches('\'');
            if !url.is_empty() {
                urls.push(url.to_string());
            }
        }
    }
    urls
}
