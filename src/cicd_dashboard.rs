use crate::meta_introspector::RepoOntology;
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectOwnership {
    pub root_owner: String,
    pub organization: String,
    pub is_fork: bool,
    pub upstream_owner: Option<String>,
    pub upstream_repo: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectStatus {
    pub name: String,
    pub path: String,
    pub git_branch: String,
    pub git_status: String,     // "clean", "dirty", "ahead", "behind"
    pub lattice_status: String, // "not_indexed", "indexing", "indexed", "failed"
    pub rustc_status: String,   // "not_compiled", "compiling", "compiled", "failed"
    pub last_commit: String,
    pub file_count: usize,
    pub ownership: ProjectOwnership,
}

pub struct CICDDashboard {
    projects: Vec<ProjectStatus>,
}

impl CICDDashboard {
    pub fn new() -> Self {
        Self {
            projects: Vec::new(),
        }
    }

    pub fn scan_projects(&mut self, repos: &[RepoOntology]) -> Result<(), String> {
        self.projects.clear();

        for repo in repos {
            let project = self.analyze_project(repo)?;
            self.projects.push(project);
        }

        Ok(())
    }

    fn analyze_project(&self, repo: &RepoOntology) -> Result<ProjectStatus, String> {
        let path = &repo.path;

        // Get git branch
        let git_branch = Command::new("git")
            .args(&["branch", "--show-current"])
            .current_dir(path)
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_else(|_| "unknown".to_string());

        // Get git status
        let git_status = Command::new("git")
            .args(&["status", "--porcelain"])
            .current_dir(path)
            .output()
            .map(|o| {
                if o.stdout.is_empty() {
                    "clean".to_string()
                } else {
                    "dirty".to_string()
                }
            })
            .unwrap_or_else(|_| "unknown".to_string());

        // Get last commit
        let last_commit = Command::new("git")
            .args(&["log", "-1", "--pretty=format:%h %s"])
            .current_dir(path)
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_else(|_| "no commits".to_string());

        // Analyze ownership
        let ownership = self.analyze_ownership(path, &repo.name)?;

        // Check lattice status (simplified)
        let lattice_status = if std::path::Path::new(path)
            .join("lattice_index.json")
            .exists()
        {
            "indexed".to_string()
        } else {
            "not_indexed".to_string()
        };

        // Check rustc status
        let rustc_status = if std::path::Path::new(path).join("Cargo.toml").exists() {
            if std::path::Path::new(path).join("target").exists() {
                "compiled".to_string()
            } else {
                "not_compiled".to_string()
            }
        } else {
            "not_rust".to_string()
        };

        Ok(ProjectStatus {
            name: repo.name.clone(),
            path: path.clone(),
            git_branch,
            git_status,
            lattice_status,
            rustc_status,
            last_commit,
            file_count: repo.file_count,
            ownership,
        })
    }

    fn analyze_ownership(&self, path: &str, _repo_name: &str) -> Result<ProjectOwnership, String> {
        // Get remote origin URL
        let _remote_url = Command::new("git")
            .args(&["remote", "get-url", "origin"])
            .current_dir(path)
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_else(|_| String::new());

        // Parse ownership from path structure: /domain/org/repo
        let path_parts: Vec<&str> = path.split('/').collect();
        let (root_owner, organization) = if path_parts.len() >= 3 {
            let domain_idx = path_parts
                .iter()
                .position(|&p| p.len() <= 3 && p != "")
                .unwrap_or(0);
            if domain_idx + 2 < path_parts.len() {
                (
                    path_parts[domain_idx + 1].to_string(),
                    path_parts[domain_idx + 1].to_string(),
                )
            } else {
                ("unknown".to_string(), "unknown".to_string())
            }
        } else {
            ("unknown".to_string(), "unknown".to_string())
        };

        // Check if it's a fork by looking for upstream remote
        let upstream_info = Command::new("git")
            .args(&["remote", "get-url", "upstream"])
            .current_dir(path)
            .output()
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    let upstream_url = String::from_utf8_lossy(&o.stdout).trim().to_string();
                    self.parse_git_url(&upstream_url)
                } else {
                    None
                }
            });

        let (is_fork, upstream_owner, upstream_repo) = if let Some((owner, repo)) = upstream_info {
            (true, Some(owner), Some(repo))
        } else {
            (false, None, None)
        };

        Ok(ProjectOwnership {
            root_owner,
            organization,
            is_fork,
            upstream_owner,
            upstream_repo,
        })
    }

    fn parse_git_url(&self, url: &str) -> Option<(String, String)> {
        // Parse GitHub/GitLab style URLs: git@github.com:owner/repo.git or https://github.com/owner/repo
        if url.contains("github.com") || url.contains("gitlab.com") {
            if let Some(parts) = url.split('/').last() {
                let repo = parts.replace(".git", "");
                if let Some(owner_part) = url.split('/').nth_back(1) {
                    return Some((owner_part.to_string(), repo));
                }
            }
        }
        None
    }

    pub fn generate_dashboard_html(&self) -> String {
        let projects_html = self
            .projects
            .iter()
            .map(|p| self.generate_project_row(p))
            .collect::<Vec<_>>()
            .join("");

        format!(
            r#"
<!DOCTYPE html>
<html>
<head>
    <title>ZOS CI/CD Dashboard</title>
    <style>
        body {{ font-family: monospace; background: #0a0a0a; color: #00ff00; padding: 20px; }}
        table {{ width: 100%; border-collapse: collapse; }}
        th, td {{ padding: 8px; text-align: left; border: 1px solid #004400; }}
        th {{ background: #002200; }}
        .status-clean {{ color: #00ff00; }}
        .status-dirty {{ color: #ffaa00; }}
        .status-indexed {{ color: #00ff00; }}
        .status-not_indexed {{ color: #ff4400; }}
        .status-compiled {{ color: #00ff00; }}
        .status-not_compiled {{ color: #ffaa00; }}
        .status-not_rust {{ color: #666666; }}
        .actions {{ white-space: nowrap; }}
        button {{ background: #004400; color: #00ff00; border: 1px solid #00ff00; padding: 4px 8px; margin: 2px; }}
    </style>
</head>
<body>
    <h1>🚀 ZOS CI/CD Dashboard</h1>
    <p>Projects: {} | Indexed: {} | Compiled: {}</p>

    <table>
        <thead>
            <tr>
                <th>Project</th>
                <th>Branch</th>
                <th>Git Status</th>
                <th>Lattice</th>
                <th>Rustc</th>
                <th>Last Commit</th>
                <th>Files</th>
                <th>Actions</th>
            </tr>
        </thead>
        <tbody>
            {}
        </tbody>
    </table>
</body>
</html>
        "#,
            self.projects.len(),
            self.projects
                .iter()
                .filter(|p| p.lattice_status == "indexed")
                .count(),
            self.projects
                .iter()
                .filter(|p| p.rustc_status == "compiled")
                .count(),
            projects_html
        )
    }

    fn generate_project_row(&self, project: &ProjectStatus) -> String {
        format!(
            r#"
            <tr>
                <td><strong>{}</strong></td>
                <td>{}</td>
                <td class="status-{}">{}</td>
                <td class="status-{}">{}</td>
                <td class="status-{}">{}</td>
                <td>{}</td>
                <td>{}</td>
                <td class="actions">
                    <button onclick="indexProject('{}')">Index</button>
                    <button onclick="compileProject('{}')">Compile</button>
                    <button onclick="viewProject('{}')">View</button>
                </td>
            </tr>
        "#,
            project.name,
            project.git_branch,
            project.git_status,
            project.git_status,
            project.lattice_status,
            project.lattice_status,
            project.rustc_status,
            project.rustc_status,
            project.last_commit,
            project.file_count,
            project.name,
            project.name,
            project.name
        )
    }

    pub fn get_projects(&self) -> &[ProjectStatus] {
        &self.projects
    }
}
