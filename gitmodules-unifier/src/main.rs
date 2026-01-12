use std::collections::{HashMap, HashSet};
use std::fs;

fn main() {
    println!("🔗 Reading .gitmodules files and uniting with stars/forks data");

    // Read .gitmodules list
    let gitmodules_list = fs::read_to_string("gitmodules_list.txt").unwrap();
    let gitmodules_repos: Vec<String> = gitmodules_list
        .lines()
        .map(|line| extract_repo_from_path(line))
        .filter(|repo| !repo.is_empty())
        .collect();

    println!("Found {} repos with .gitmodules", gitmodules_repos.len());

    // Read GitHub metadata
    let mut github_repos = HashMap::new();

    // Load starred repos
    if let Ok(starred_data) = fs::read_to_string("/mnt/data1/nix/index/starred.json") {
        parse_github_data(&starred_data, &mut github_repos);
        println!("Loaded starred repos");
    }

    // Load other GitHub data files
    for file in ["github_meta-introspector_repos.json", "github_metaintrospector_repos.json", "stars.json"] {
        let path = format!("/mnt/data1/nix/index/{}", file);
        if let Ok(data) = fs::read_to_string(&path) {
            parse_github_data(&data, &mut github_repos);
            println!("Loaded {}", file);
        }
    }

    println!("Total GitHub repos: {}", github_repos.len());

    // Unite the data
    let mut unified_repos = Vec::new();

    for gitmodule_repo in &gitmodules_repos {
        if let Some(github_data) = github_repos.get(gitmodule_repo) {
            unified_repos.push(UnifiedRepo {
                name: gitmodule_repo.clone(),
                has_submodules: true,
                stars: github_data.stars,
                forks: github_data.forks,
                language: github_data.language.clone(),
                owner: github_data.owner.clone(),
            });
        } else {
            unified_repos.push(UnifiedRepo {
                name: gitmodule_repo.clone(),
                has_submodules: true,
                stars: 0,
                forks: 0,
                language: None,
                owner: None,
            });
        }
    }

    println!("✅ Unified {} repos with submodules", unified_repos.len());

    // Show top repos by stars
    unified_repos.sort_by(|a, b| b.stars.cmp(&a.stars));
    println!("\n🌟 Top repos with submodules by stars:");
    for repo in unified_repos.iter().take(10) {
        println!("  {} - {} stars, {} forks", repo.name, repo.stars, repo.forks);
    }
}

#[derive(Debug)]
struct UnifiedRepo {
    name: String,
    has_submodules: bool,
    stars: u32,
    forks: u32,
    language: Option<String>,
    owner: Option<String>,
}

#[derive(Debug, Clone)]
struct GitHubRepo {
    stars: u32,
    forks: u32,
    language: Option<String>,
    owner: Option<String>,
}

fn extract_repo_from_path(path: &str) -> String {
    // Extract repo name from .gitmodules path
    if let Some(gitmodules_pos) = path.rfind("/.gitmodules") {
        let repo_path = &path[..gitmodules_pos];
        if let Some(last_slash) = repo_path.rfind('/') {
            return repo_path[last_slash + 1..].to_string();
        }
    }
    String::new()
}

fn parse_github_data(json_data: &str, repos: &mut HashMap<String, GitHubRepo>) {
    if let Ok(data) = serde_json::from_str::<serde_json::Value>(json_data) {
        if let Some(repo_array) = data.as_array() {
            for repo in repo_array {
                if let Some(full_name) = repo["full_name"].as_str() {
                    let github_repo = GitHubRepo {
                        stars: repo["stargazers_count"].as_u64().unwrap_or(0) as u32,
                        forks: repo["forks_count"].as_u64().unwrap_or(0) as u32,
                        language: repo["language"].as_str().map(|s| s.to_string()),
                        owner: repo["owner"]["login"].as_str().map(|s| s.to_string()),
                    };

                    // Store by full name
                    repos.insert(full_name.to_string(), github_repo.clone());

                    // Also store by repo name only for better matching
                    if let Some(repo_name) = full_name.split('/').last() {
                        repos.insert(repo_name.to_string(), github_repo.clone());
                    }

                    // Store by clone_url if available
                    if let Some(clone_url) = repo["clone_url"].as_str() {
                        repos.insert(clone_url.to_string(), github_repo.clone());
                    }

                    // Store by html_url if available
                    if let Some(html_url) = repo["html_url"].as_str() {
                        repos.insert(html_url.to_string(), github_repo);
                    }
                }
            }
        }
    }
}
