#!/usr/bin/env cargo
//! ```cargo
//! [dependencies]
//! serde = { version = "1.0", features = ["derive"] }
//! serde_json = "1.0"
//! chrono = { version = "0.4", features = ["serde"] }
//! shellexpand = "3.1"
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Serialize, Deserialize)]
pub struct RepoStatus {
    pub path: String,
    pub branch: String,
    pub ahead: u32,
    pub behind: u32,
    pub modified: u32,
    pub untracked: u32,
    pub last_commit: String,
    pub last_commit_date: DateTime<Utc>,
    pub unpushed_commits: Vec<CommitInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommitInfo {
    pub hash: String,
    pub date: DateTime<Utc>,
    pub message: String,
}

fn main() {
    let repos = scan_all_repos();

    println!("📊 Repository Status (Chronological)");
    println!("=====================================");

    for repo in &repos {
        println!("📁 {}", repo.path);
        println!(
            "   Branch: {} | Ahead: {} | Modified: {} | Untracked: {}",
            repo.branch, repo.ahead, repo.modified, repo.untracked
        );
        println!("   Last commit: {}", repo.last_commit_date);
        if !repo.unpushed_commits.is_empty() {
            println!("   Unpushed commits: {}", repo.unpushed_commits.len());
        }
        println!();
    }

    println!("\n🚨 Unpushed Changes Summary");
    println!("===========================");
    let unpushed: HashMap<String, u32> = repos
        .into_iter()
        .filter(|r| r.ahead > 0 || r.modified > 0 || r.untracked > 0)
        .map(|r| (r.path, r.ahead + r.modified + r.untracked))
        .collect();

    for (path, count) in unpushed {
        println!("📁 {} -> {} changes", path, count);
    }
}

fn scan_all_repos() -> Vec<RepoStatus> {
    let mut repos = Vec::new();

    // Read repos from meta-introspector
    if let Ok(content) = std::fs::read_to_string("/mnt/data1/meta-introspector/repos.txt") {
        for line in content.lines() {
            if let Some(status) = get_repo_status(line.trim()) {
                repos.push(status);
            }
        }
    }

    repos.sort_by(|a, b| b.last_commit_date.cmp(&a.last_commit_date));
    repos
}

fn get_repo_status(path: &str) -> Option<RepoStatus> {
    let expanded_path = shellexpand::tilde(path).to_string();
    let repo_path = Path::new(&expanded_path);

    if !repo_path.join(".git").exists() {
        return None;
    }

    let branch = git_cmd(&repo_path, &["branch", "--show-current"])?;
    let status_output = git_cmd(&repo_path, &["status", "--porcelain", "-b"])?;

    let (ahead, behind) = parse_ahead_behind(&status_output);
    let (modified, untracked) = count_changes(&status_output);

    let last_commit = git_cmd(&repo_path, &["log", "-1", "--format=%H"])?;
    let last_commit_date_str = git_cmd(&repo_path, &["log", "-1", "--format=%cI"])?;
    let last_commit_date = DateTime::parse_from_rfc3339(&last_commit_date_str)
        .ok()?
        .with_timezone(&Utc);

    let unpushed_commits = get_unpushed_commits(&repo_path);

    Some(RepoStatus {
        path: expanded_path,
        branch,
        ahead,
        behind,
        modified,
        untracked,
        last_commit,
        last_commit_date,
        unpushed_commits,
    })
}

fn git_cmd(repo_path: &Path, args: &[&str]) -> Option<String> {
    Command::new("git")
        .current_dir(repo_path)
        .args(args)
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
            } else {
                None
            }
        })
}

fn parse_ahead_behind(status: &str) -> (u32, u32) {
    for line in status.lines() {
        if line.starts_with("##") && line.contains("[") {
            if let Some(bracket_content) = line.split('[').nth(1) {
                if let Some(content) = bracket_content.split(']').next() {
                    if content.contains("ahead") && content.contains("behind") {
                        let parts: Vec<&str> = content.split(", ").collect();
                        let ahead = parts[0]
                            .split_whitespace()
                            .nth(1)
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(0);
                        let behind = parts[1]
                            .split_whitespace()
                            .nth(1)
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(0);
                        return (ahead, behind);
                    } else if content.contains("ahead") {
                        let ahead = content
                            .split_whitespace()
                            .nth(1)
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(0);
                        return (ahead, 0);
                    }
                }
            }
        }
    }
    (0, 0)
}

fn count_changes(status: &str) -> (u32, u32) {
    let mut modified = 0;
    let mut untracked = 0;

    for line in status.lines() {
        if line.starts_with("##") {
            continue;
        }
        if line.starts_with("??") {
            untracked += 1;
        } else if !line.trim().is_empty() {
            modified += 1;
        }
    }

    (modified, untracked)
}

fn get_unpushed_commits(repo_path: &Path) -> Vec<CommitInfo> {
    let output = git_cmd(
        repo_path,
        &["log", "--oneline", "@{u}..", "--format=%H|%cI|%s"],
    );

    if let Some(output) = output {
        output
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.splitn(3, '|').collect();
                if parts.len() == 3 {
                    let date = DateTime::parse_from_rfc3339(parts[1])
                        .ok()?
                        .with_timezone(&Utc);
                    Some(CommitInfo {
                        hash: parts[0].to_string(),
                        date,
                        message: parts[2].to_string(),
                    })
                } else {
                    None
                }
            })
            .collect()
    } else {
        Vec::new()
    }
}
