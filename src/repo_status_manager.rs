use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RepoInfo {
    pub path: String,
    pub name: String,
    pub last_checked: Option<DateTime<Utc>>,
    pub status: Option<RepoStatus>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RepoStatus {
    pub branch: String,
    pub ahead: u32,
    pub modified: u32,
    pub untracked: u32,
    pub last_commit_date: DateTime<Utc>,
}

pub struct RepoStatusManager {
    repos: HashMap<String, RepoInfo>,
}

impl RepoStatusManager {
    pub fn new() -> Self {
        Self {
            repos: HashMap::new(),
        }
    }

    pub fn load_repo_list(&mut self) -> Result<(), String> {
        // Load from meta-introspector repos.txt
        if let Ok(content) = std::fs::read_to_string("/mnt/data1/meta-introspector/repos.txt") {
            for line in content.lines() {
                let path = line
                    .trim()
                    .replace("~", &std::env::var("HOME").unwrap_or_default());
                let name = Path::new(&path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                self.repos.insert(
                    path.clone(),
                    RepoInfo {
                        path: path.clone(),
                        name,
                        last_checked: None,
                        status: None,
                    },
                );
            }
        }

        // Scan canonical symlinks for more repos
        if let Ok(entries) = std::fs::read_dir("/mnt/data1/meta-introspector/canonical") {
            for entry in entries.flatten() {
                if let Some(repo_info) = self.scan_canonical_repo(&entry.path()) {
                    self.repos.insert(repo_info.path.clone(), repo_info);
                }
            }
        }

        Ok(())
    }

    fn scan_canonical_repo(&self, canonical_path: &Path) -> Option<RepoInfo> {
        let sources_path = canonical_path.join("sources/v1");
        if sources_path.exists() {
            if let Ok(real_path) = std::fs::read_link(&sources_path) {
                let path = real_path.to_string_lossy().to_string();
                let name = canonical_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                return Some(RepoInfo {
                    path,
                    name,
                    last_checked: None,
                    status: None,
                });
            }
        }
        None
    }

    pub fn get_all_repos(&self) -> Vec<&RepoInfo> {
        self.repos.values().collect()
    }

    pub fn get_repos_needing_refresh(&self) -> Vec<&RepoInfo> {
        self.repos
            .values()
            .filter(|repo| repo.last_checked.is_none() || repo.status.is_none())
            .collect()
    }

    pub fn get_repos_with_changes(&self) -> Vec<&RepoInfo> {
        self.repos
            .values()
            .filter(|repo| {
                if let Some(status) = &repo.status {
                    status.ahead > 0 || status.modified > 0 || status.untracked > 0
                } else {
                    false
                }
            })
            .collect()
    }

    // TODO: Add background task to refresh repo status
    pub fn queue_refresh(&mut self, path: &str) {
        // Mark for refresh in task queue
        if let Some(repo) = self.repos.get_mut(path) {
            repo.last_checked = None;
        }
    }
}
