use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

#[derive(Debug, Serialize, Deserialize)]
pub struct GitProjectAnalysis {
    pub name: String,
    pub path: String,
    pub is_fork: bool,
    pub git_status: String,
    pub last_commit_date: String,
    pub last_commit_hash: String,
    pub last_commit_message: String,
    pub commits_last_12_months: usize,
    pub branch: String,
    pub remote_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitAnalysisReport {
    pub total_projects: usize,
    pub forks_count: usize,
    pub active_projects: usize,
    pub projects: Vec<GitProjectAnalysis>,
    pub recent_commits: Vec<CommitEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommitEntry {
    pub project: String,
    pub date: String,
    pub hash: String,
    pub message: String,
    pub author: String,
}

pub struct GitAnalyzer;

impl GitAnalyzer {
    pub fn analyze_all_projects(base_path: &str) -> Result<GitAnalysisReport, String> {
        let mut projects = Vec::new();
        let mut all_commits = Vec::new();

        // Scan meta-introspector directory
        let meta_path = Path::new(base_path);
        if !meta_path.exists() {
            return Err(format!("Base path does not exist: {}", base_path));
        }

        // Get 12 months ago date
        let twelve_months_ago = Utc::now() - Duration::days(365);
        let since_date = twelve_months_ago.format("%Y-%m-%d").to_string();

        for entry in std::fs::read_dir(meta_path).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();

            if path.is_dir() {
                if let Some(domain) = path.file_name().and_then(|n| n.to_str()) {
                    if domain.len() <= 3 && !domain.starts_with('.') {
                        Self::scan_domain_projects(
                            &path,
                            domain,
                            &since_date,
                            &mut projects,
                            &mut all_commits,
                        )?;
                    }
                }
            }
        }

        // Sort commits by date (most recent first)
        all_commits.sort_by(|a, b| b.date.cmp(&a.date));
        all_commits.truncate(100); // Keep top 100 recent commits

        let forks_count = projects.iter().filter(|p| p.is_fork).count();
        let active_projects = projects
            .iter()
            .filter(|p| p.commits_last_12_months > 0)
            .count();

        Ok(GitAnalysisReport {
            total_projects: projects.len(),
            forks_count,
            active_projects,
            projects,
            recent_commits: all_commits,
        })
    }

    fn scan_domain_projects(
        domain_path: &Path,
        _domain: &str,
        since_date: &str,
        projects: &mut Vec<GitProjectAnalysis>,
        all_commits: &mut Vec<CommitEntry>,
    ) -> Result<(), String> {
        for entry in std::fs::read_dir(domain_path).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();

            if path.is_dir() && path.join(".git").exists() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    match Self::analyze_git_repo(&path, name, since_date) {
                        Ok(analysis) => {
                            // Collect recent commits for this project
                            if let Ok(commits) = Self::get_recent_commits(&path, name, since_date) {
                                all_commits.extend(commits);
                            }
                            projects.push(analysis);
                        }
                        Err(e) => {
                            eprintln!("Warning: Failed to analyze {}: {}", path.display(), e);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn analyze_git_repo(
        repo_path: &Path,
        name: &str,
        since_date: &str,
    ) -> Result<GitProjectAnalysis, String> {
        let path_str = repo_path.to_string_lossy().to_string();

        // Get current branch
        let branch = Command::new("git")
            .args(&["branch", "--show-current"])
            .current_dir(repo_path)
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_else(|_| "unknown".to_string());

        // Get git status
        let git_status = Command::new("git")
            .args(&["status", "--porcelain"])
            .current_dir(repo_path)
            .output()
            .map(|o| {
                if o.stdout.is_empty() {
                    "clean".to_string()
                } else {
                    "dirty".to_string()
                }
            })
            .unwrap_or_else(|_| "unknown".to_string());

        // Get remote URL
        let remote_url = Command::new("git")
            .args(&["remote", "get-url", "origin"])
            .current_dir(repo_path)
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_else(|_| "no-remote".to_string());

        // Check if it's a fork (has upstream or is meta-introspector fork)
        let is_fork = remote_url.contains("github.com/meta-introspector/")
            || Command::new("git")
                .args(&["remote", "get-url", "upstream"])
                .current_dir(repo_path)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);

        // Get last commit info
        let last_commit_output = Command::new("git")
            .args(&["log", "-1", "--pretty=format:%H|%ci|%s"])
            .current_dir(repo_path)
            .output()
            .map_err(|e| format!("Failed to get last commit: {}", e))?;

        let commit_info = String::from_utf8_lossy(&last_commit_output.stdout);
        let parts: Vec<&str> = commit_info.split('|').collect();

        let (last_commit_hash, last_commit_date, last_commit_message) = if parts.len() >= 3 {
            (
                parts[0].to_string(),
                parts[1].to_string(),
                parts[2].to_string(),
            )
        } else {
            (
                "unknown".to_string(),
                "unknown".to_string(),
                "no commits".to_string(),
            )
        };

        // Count commits in last 12 months
        let commits_count = Command::new("git")
            .args(&["rev-list", "--count", "--since", since_date, "HEAD"])
            .current_dir(repo_path)
            .output()
            .map(|o| {
                String::from_utf8_lossy(&o.stdout)
                    .trim()
                    .parse::<usize>()
                    .unwrap_or(0)
            })
            .unwrap_or(0);

        Ok(GitProjectAnalysis {
            name: name.to_string(),
            path: path_str,
            is_fork,
            git_status,
            last_commit_date,
            last_commit_hash,
            last_commit_message,
            commits_last_12_months: commits_count,
            branch,
            remote_url,
        })
    }

    fn get_recent_commits(
        repo_path: &Path,
        project_name: &str,
        since_date: &str,
    ) -> Result<Vec<CommitEntry>, String> {
        let output = Command::new("git")
            .args(&[
                "log",
                "--since",
                since_date,
                "--pretty=format:%ci|%H|%s|%an",
                "--max-count=10",
            ])
            .current_dir(repo_path)
            .output()
            .map_err(|e| format!("Failed to get commits: {}", e))?;

        let commits_text = String::from_utf8_lossy(&output.stdout);
        let mut commits = Vec::new();

        for line in commits_text.lines() {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 4 {
                commits.push(CommitEntry {
                    project: project_name.to_string(),
                    date: parts[0].to_string(),
                    hash: parts[1][..8].to_string(), // Short hash
                    message: parts[2].to_string(),
                    author: parts[3].to_string(),
                });
            }
        }

        Ok(commits)
    }

    pub fn generate_summary_report(report: &GitAnalysisReport) -> String {
        format!(
            r#"
# Git Analysis Report - Last 12 Months

## Summary
- **Total Projects**: {}
- **Fork Projects**: {} ({:.1}%)
- **Active Projects**: {} ({:.1}%)
- **Recent Commits**: {}

## Fork Analysis
- **Forks**: {} repositories
- **Original**: {} repositories

## Activity Summary
- **Active projects** (with commits): {}
- **Inactive projects**: {}

## Recent Activity (Top 10 Commits)
{}

## Most Active Projects
{}
        "#,
            report.total_projects,
            report.forks_count,
            (report.forks_count as f64 / report.total_projects as f64) * 100.0,
            report.active_projects,
            (report.active_projects as f64 / report.total_projects as f64) * 100.0,
            report.recent_commits.len(),
            report.forks_count,
            report.total_projects - report.forks_count,
            report.active_projects,
            report.total_projects - report.active_projects,
            report
                .recent_commits
                .iter()
                .take(10)
                .map(|c| format!(
                    "- **{}** `{}` {} - {}",
                    c.project, c.hash, c.date, c.message
                ))
                .collect::<Vec<_>>()
                .join("\n"),
            {
                let mut active_projects: Vec<_> = report
                    .projects
                    .iter()
                    .filter(|p| p.commits_last_12_months > 0)
                    .collect();
                active_projects
                    .sort_by(|a, b| b.commits_last_12_months.cmp(&a.commits_last_12_months));
                active_projects
                    .iter()
                    .take(10)
                    .map(|p| format!("- **{}**: {} commits", p.name, p.commits_last_12_months))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        )
    }
}
