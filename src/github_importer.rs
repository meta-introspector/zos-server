use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Deserialize)]
struct GitHubRepo {
    name: String,
    url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RepoOwnershipData {
    pub name: String,
    pub owner: String,
    pub url: String,
    pub is_fork: bool,
    pub organization: String,
}

pub struct GitHubDataImporter {
    repos: Vec<RepoOwnershipData>,
}

impl GitHubDataImporter {
    pub fn new() -> Self {
        Self { repos: Vec::new() }
    }

    pub fn load_from_index(&mut self, index_path: &str) -> Result<(), String> {
        let content = fs::read_to_string(index_path)
            .map_err(|e| format!("Failed to read index file: {}", e))?;

        let github_repos: Vec<GitHubRepo> =
            serde_json::from_str(&content).map_err(|e| format!("Failed to parse JSON: {}", e))?;

        for repo in github_repos {
            if let Some(ownership) = self.parse_github_url(&repo.url) {
                self.repos.push(RepoOwnershipData {
                    name: repo.name,
                    owner: ownership.0,
                    url: repo.url,
                    is_fork: ownership.2,
                    organization: ownership.1,
                });
            }
        }

        Ok(())
    }

    fn parse_github_url(&self, url: &str) -> Option<(String, String, bool)> {
        // Parse GitHub URL: https://github.com/owner/repo
        if let Some(path) = url.strip_prefix("https://github.com/") {
            let parts: Vec<&str> = path.split('/').collect();
            if parts.len() >= 2 {
                let owner = parts[0].to_string();
                let org = owner.clone(); // For GitHub, owner and org are often the same
                let is_fork = owner == "meta-introspector"; // Our forks
                return Some((owner, org, is_fork));
            }
        }
        None
    }

    pub fn get_owner_stats(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        for repo in &self.repos {
            *stats.entry(repo.owner.clone()).or_insert(0) += 1;
        }
        stats
    }

    pub fn get_top_owners(&self, limit: usize) -> Vec<(String, usize)> {
        let mut stats: Vec<_> = self.get_owner_stats().into_iter().collect();
        stats.sort_by(|a, b| b.1.cmp(&a.1));
        stats.into_iter().take(limit).collect()
    }

    pub fn get_repos(&self) -> &[RepoOwnershipData] {
        &self.repos
    }

    pub fn generate_ownership_report(&self) -> String {
        let top_owners = self.get_top_owners(10);
        let total_repos = self.repos.len();
        let fork_count = self.repos.iter().filter(|r| r.is_fork).count();

        format!(
            r#"
# GitHub Repository Ownership Report

## Summary
- **Total Repositories**: {}
- **Forked Repositories**: {}
- **Original Repositories**: {}

## Top 10 Repository Owners
{}

## Fork Analysis
- **meta-introspector forks**: {} repositories
- **External repositories**: {} repositories

## Organization Distribution
{}
        "#,
            total_repos,
            fork_count,
            total_repos - fork_count,
            top_owners
                .iter()
                .enumerate()
                .map(|(i, (owner, count))| format!(
                    "{}. **{}**: {} repositories",
                    i + 1,
                    owner,
                    count
                ))
                .collect::<Vec<_>>()
                .join("\n"),
            fork_count,
            total_repos - fork_count,
            self.get_owner_stats()
                .iter()
                .map(|(org, count)| format!("- {}: {} repos", org, count))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}
