use std::collections::{HashMap, HashSet};
use std::fs;

fn main() {
    let starred_data = fs::read_to_string("~/nix/index/starred.json").unwrap_or_default();
    let github_data = fs::read_to_string("~/nix/index/github_meta-introspector_repos.json").unwrap_or_default();

    let mut user_repos: HashMap<String, HashSet<String>> = HashMap::new();
    let mut repo_users: HashMap<String, HashSet<String>> = HashMap::new();

    // Parse starred repos (user -> repos they starred)
    if let Ok(starred) = serde_json::from_str::<serde_json::Value>(&starred_data) {
        if let Some(repos) = starred.as_array() {
            for repo in repos {
                if let Some(full_name) = repo["full_name"].as_str() {
                    if let Some(owner) = repo["owner"]["login"].as_str() {
                        user_repos.entry("starred_user".to_string())
                            .or_default()
                            .insert(full_name.to_string());
                        repo_users.entry(full_name.to_string())
                            .or_default()
                            .insert("starred_user".to_string());
                    }
                }
            }
        }
    }

    // Parse github repos (extract contributors/stargazers if available)
    if let Ok(github) = serde_json::from_str::<serde_json::Value>(&github_data) {
        if let Some(repos) = github.as_array() {
            for repo in repos {
                if let Some(full_name) = repo["full_name"].as_str() {
                    if let Some(owner) = repo["owner"]["login"].as_str() {
                        user_repos.entry(owner.to_string())
                            .or_default()
                            .insert(full_name.to_string());
                        repo_users.entry(full_name.to_string())
                            .or_default()
                            .insert(owner.to_string());
                    }
                }
            }
        }
    }

    // Simple eigenvector approximation: repos with shared users
    let mut repo_similarity: HashMap<String, HashMap<String, f64>> = HashMap::new();

    for (repo1, users1) in &repo_users {
        for (repo2, users2) in &repo_users {
            if repo1 != repo2 {
                let intersection: HashSet<_> = users1.intersection(users2).collect();
                let union_size = users1.len() + users2.len() - intersection.len();

                if union_size > 0 {
                    let similarity = intersection.len() as f64 / union_size as f64;
                    if similarity > 0.1 {
                        repo_similarity.entry(repo1.clone())
                            .or_default()
                            .insert(repo2.clone(), similarity);
                    }
                }
            }
        }
    }

    // Find top similar repos
    println!("Repository Similarity Graph:");
    for (repo, similar) in repo_similarity {
        if similar.len() > 0 {
            println!("\n{}", repo);
            let mut sorted: Vec<_> = similar.iter().collect();
            sorted.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());
            for (similar_repo, score) in sorted.iter().take(3) {
                println!("  -> {} (score: {:.3})", similar_repo, score);
            }
        }
    }

    println!("\nTotal repos: {}", repo_users.len());
    println!("Total users: {}", user_repos.len());
}
