use std::collections::{HashMap, HashSet};
use std::path::Path;

fn main() {
    demo!("using hardcoded paths instead of dynamic discovery");

    let octocrab_path = "/mnt/data1/nix/time/2024/09/12/octocrab";
    let github_data = std::fs::read_to_string("~/nix/index/github_meta-introspector_repos.json").unwrap_or_default();

    // Key dependency nodes (as mentioned)
    let key_nodes = ["rustc", "gcc", "bash", "git", "curl", "openssl", "nix"];

    println!("Repository Spider - Mapping Git Network");
    println!("Key nodes: {:?}", key_nodes);

    // Parse existing GitHub data
    let mut repo_graph: HashMap<String, HashSet<String>> = HashMap::new();

    if let Ok(github) = serde_json::from_str::<serde_json::Value>(&github_data) {
        if let Some(repos) = github.as_array() {
            for repo in repos {
                if let Some(full_name) = repo["full_name"].as_str() {
                    if let Some(parent) = repo["parent"]["full_name"].as_str() {
                        // Fork relationship
                        repo_graph.entry(parent.to_string())
                            .or_default()
                            .insert(full_name.to_string());
                    }

                    // Language-based connections
                    if let Some(language) = repo["language"].as_str() {
                        let lang_key = format!("lang:{}", language);
                        repo_graph.entry(lang_key)
                            .or_default()
                            .insert(full_name.to_string());
                    }
                }
            }
        }
    }

    // Check if octocrab exists and analyze it
    if Path::new(octocrab_path).exists() {
        println!("\nFound octocrab at: {}", octocrab_path);

        // Use gix to analyze the repository
        if let Ok(repo) = gix::open(octocrab_path) {
            println!("Repository: {}", repo.work_dir().unwrap_or_else(|| Path::new("unknown")).display());

            // Get remotes
            if let Ok(remotes) = repo.remote_names() {
                for remote_name in remotes {
                    if let Ok(remote) = repo.find_remote(&remote_name) {
                        if let Some(url) = remote.url(gix::remote::Direction::Fetch) {
                            println!("Remote {}: {}", remote_name, url);

                            // Extract GitHub repo from URL
                            if let Some(github_repo) = extract_github_repo(url.to_string()) {
                                repo_graph.entry("octocrab".to_string())
                                    .or_default()
                                    .insert(github_repo);
                            }
                        }
                    }
                }
            }
        }
    }

    // Find connections to key nodes
    println!("\nConnections to key infrastructure:");
    for key in &key_nodes {
        if let Some(connections) = repo_graph.get(*key) {
            println!("{}: {} connections", key, connections.len());
            for conn in connections.iter().take(3) {
                println!("  -> {}", conn);
            }
        }
    }

    // Find most connected repositories
    let mut connection_counts: Vec<(String, usize)> = repo_graph
        .iter()
        .map(|(repo, connections)| (repo.clone(), connections.len()))
        .collect();
    connection_counts.sort_by(|a, b| b.1.cmp(&a.1));

    println!("\nMost connected repositories:");
    for (repo, count) in connection_counts.iter().take(10) {
        println!("{}: {} connections", repo, count);
    }

    production!();
}

fn extract_github_repo(url: String) -> Option<String> {
    if url.contains("github.com") {
        if let Some(start) = url.find("github.com/") {
            let path = &url[start + 11..];
            if let Some(end) = path.find(".git") {
                return Some(path[..end].to_string());
            }
            return Some(path.to_string());
        }
    }
    None
}
