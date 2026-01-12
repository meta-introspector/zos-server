use std::collections::HashMap;
use std::fs;
use std::process::Command;
use chrono::{DateTime, Utc};

fn main() {
    println!("📅 Creating Meta-Introspector Timeline");

    let mut timeline = TimelineBuilder::new();
    timeline.scan_repositories();
    timeline.identify_original_work();
    timeline.build_timeline();
    timeline.generate_report();
}

struct TimelineBuilder {
    repositories: Vec<RepoInfo>,
    timeline_entries: Vec<TimelineEntry>,
}

#[derive(Debug, Clone)]
struct RepoInfo {
    name: String,
    path: String,
    remote_url: String,
    is_fork: bool,
    is_original: bool,
    last_commit_date: Option<DateTime<Utc>>,
    commit_count: usize,
    rust_files: Vec<String>,
}

#[derive(Debug, Clone)]
struct TimelineEntry {
    date: DateTime<Utc>,
    repo_name: String,
    event_type: String, // "created", "major_commit", "rust_analysis", "fork"
    description: String,
    files_changed: Vec<String>,
}

impl TimelineBuilder {
    fn new() -> Self {
        Self {
            repositories: Vec::new(),
            timeline_entries: Vec::new(),
        }
    }

    fn scan_repositories(&mut self) {
        println!("🔍 Scanning repositories for timeline data...");

        // Scan home directory symlinks first
        self.scan_home_symlinks();

        // Scan major directories
        let scan_paths = vec![
            "/home/mdupont/nix",
            "/home/mdupont/terraform",
            "/home/mdupont/zombie-driver",
            "/mnt/data1/nix",
        ];

        for base_path in scan_paths {
            if std::path::Path::new(base_path).exists() {
                self.scan_directory_for_repos(base_path);
            }
        }

        println!("   Analyzed {} repositories", self.repositories.len());
    }

    fn scan_home_symlinks(&mut self) {
        println!("   Scanning home directory symlinks...");

        if let Ok(entries) = fs::read_dir("/home/mdupont") {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_symlink() {
                    if let Ok(target) = fs::read_link(&path) {
                        if target.to_string_lossy().contains(".git") ||
                           std::path::Path::new(&target).join(".git").exists() {
                            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                                self.analyze_git_repo(name, &target.to_string_lossy(), true);
                            }
                        }
                    }
                }
            }
        }
    }

    fn scan_directory_for_repos(&mut self, base_path: &str) {
        println!("   Scanning {}...", base_path);

        let output = Command::new("find")
            .args(&[base_path, "-name", ".git", "-type", "d"])
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                let git_dirs = String::from_utf8_lossy(&output.stdout);
                for git_dir in git_dirs.lines() {
                    if let Some(repo_path) = git_dir.strip_suffix("/.git") {
                        if let Some(repo_name) = std::path::Path::new(repo_path).file_name() {
                            if let Some(name_str) = repo_name.to_str() {
                                self.analyze_git_repo(name_str, repo_path, false);
                            }
                        }
                    }
                }
            }
        }
    }

    fn analyze_git_repo(&mut self, name: &str, path: &str, is_symlink: bool) {
        // Get remote URL
        let remote_url = self.get_git_remote(path).unwrap_or_else(|| "local".to_string());

        // Get commit activity in last year
        let recent_commits = self.get_recent_commit_activity(path);

        // Check for our authorship
        let our_commits = self.get_our_commit_count(path);

        // Determine if this is original work
        let is_original = our_commits > 5 ||
                         remote_url.contains("meta-introspector") ||
                         self.has_our_commit_patterns(path);

        let repo_info = RepoInfo {
            name: name.to_string(),
            path: path.to_string(),
            remote_url: remote_url.clone(),
            is_fork: remote_url.contains("meta-introspector") && !is_original,
            is_original,
            last_commit_date: self.get_last_commit_date(path),
            commit_count: self.get_commit_count(path),
            rust_files: self.find_rust_files(path),
        };

        if is_symlink || is_original || repo_info.commit_count > 0 {
            self.repositories.push(repo_info);
        }
    }

    fn get_recent_commit_activity(&self, path: &str) -> usize {
        let output = Command::new("git")
            .args(&["-C", path, "log", "--oneline", "--since=2024-01-01"])
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                String::from_utf8_lossy(&output.stdout).lines().count()
            } else { 0 }
        } else { 0 }
    }

    fn get_our_commit_count(&self, path: &str) -> usize {
        let output = Command::new("git")
            .args(&["-C", path, "log", "--oneline", "--author=mdupont", "--since=2024-01-01"])
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                String::from_utf8_lossy(&output.stdout).lines().count()
            } else { 0 }
        } else { 0 }
    }

    fn has_our_commit_patterns(&self, path: &str) -> bool {
        let output = Command::new("git")
            .args(&["-C", path, "log", "--oneline", "--grep=CRQ-016", "--grep=nixify", "--grep=introspector"])
            .output();

        if let Ok(output) = output {
            output.status.success() && !output.stdout.is_empty()
        } else {
            false
        }
    }

    fn analyze_repository(&self, name: &str, path: &str, remote_url: &str) -> Option<RepoInfo> {
        // Check if repository exists
        if !std::path::Path::new(path).exists() {
            return None;
        }

        // Determine if it's a fork or original work
        let is_fork = remote_url.contains("meta-introspector");
        let is_original = self.is_original_work(path, remote_url);

        // Get last commit date
        let last_commit_date = self.get_last_commit_date(path);

        // Count commits
        let commit_count = self.get_commit_count(path);

        // Find Rust files
        let rust_files = self.find_rust_files(path);

        Some(RepoInfo {
            name: name.to_string(),
            path: path.to_string(),
            remote_url: remote_url.to_string(),
            is_fork,
            is_original,
            last_commit_date,
            commit_count,
            rust_files,
        })
    }

    fn is_original_work(&self, path: &str, remote_url: &str) -> bool {
        // Check for indicators of original work
        if !remote_url.contains("meta-introspector") {
            return false;
        }

        // Check for recent commits with our patterns
        let output = Command::new("git")
            .args(&["-C", path, "log", "--oneline", "--since=2024-01-01", "--author=mdupont"])
            .output();

        if let Ok(output) = output {
            let log = String::from_utf8_lossy(&output.stdout);
            // Look for our commit patterns
            log.contains("CRQ-016") ||
            log.contains("nixify") ||
            log.contains("introspector") ||
            log.contains("meta-") ||
            log.lines().count() > 10 // Significant commit activity
        } else {
            false
        }
    }

    fn get_last_commit_date(&self, path: &str) -> Option<DateTime<Utc>> {
        let output = Command::new("git")
            .args(&["-C", path, "log", "-1", "--format=%cI"])
            .output()
            .ok()?;

        if output.status.success() {
            let date_str = String::from_utf8(output.stdout).ok()?;
            DateTime::parse_from_rfc3339(date_str.trim()).ok()?.with_timezone(&Utc).into()
        } else {
            None
        }
    }

    fn get_commit_count(&self, path: &str) -> usize {
        let output = Command::new("git")
            .args(&["-C", path, "rev-list", "--count", "HEAD"])
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                let count_str = String::from_utf8_lossy(&output.stdout);
                count_str.trim().parse().unwrap_or(0)
            } else {
                0
            }
        } else {
            0
        }
    }

    fn find_rust_files(&self, path: &str) -> Vec<String> {
        let output = Command::new("find")
            .args(&[path, "-name", "*.rs", "-type", "f"])
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                let files = String::from_utf8_lossy(&output.stdout);
                files.lines().map(|s| s.to_string()).collect()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    }

    fn identify_original_work(&mut self) {
        println!("🔬 Identifying original work vs forks...");

        let mut original_count = 0;
        let mut fork_count = 0;

        for repo in &self.repositories {
            if repo.is_original {
                original_count += 1;
                println!("   ✨ Original: {} ({} commits, {} Rust files)",
                    repo.name, repo.commit_count, repo.rust_files.len());
            } else if repo.is_fork {
                fork_count += 1;
            }
        }

        println!("   Found {} original repositories, {} forks", original_count, fork_count);
    }

    fn build_timeline(&mut self) {
        println!("📅 Building chronological timeline...");

        for repo in &self.repositories {
            if let Some(date) = repo.last_commit_date {
                let event_type = if repo.is_original {
                    "original_work"
                } else if repo.is_fork {
                    "fork"
                } else {
                    "external"
                };

                let description = format!(
                    "{} - {} commits, {} Rust files",
                    repo.name, repo.commit_count, repo.rust_files.len()
                );

                self.timeline_entries.push(TimelineEntry {
                    date,
                    repo_name: repo.name.clone(),
                    event_type: event_type.to_string(),
                    description,
                    files_changed: repo.rust_files.clone(),
                });
            }
        }

        // Sort by date (most recent first)
        self.timeline_entries.sort_by(|a, b| b.date.cmp(&a.date));

        println!("   Created {} timeline entries", self.timeline_entries.len());
    }

    fn generate_report(&self) {
        println!("📊 Generating timeline report...");

        let mut report = String::new();
        report.push_str("# Meta-Introspector Development Timeline\n\n");
        report.push_str(&format!("Generated: {}\n\n", Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));

        // Summary statistics
        let original_repos: Vec<_> = self.repositories.iter().filter(|r| r.is_original).collect();
        let total_rust_files: usize = original_repos.iter().map(|r| r.rust_files.len()).sum();
        let total_commits: usize = original_repos.iter().map(|r| r.commit_count).sum();

        report.push_str("## Summary\n\n");
        report.push_str(&format!("- **Original Repositories**: {}\n", original_repos.len()));
        report.push_str(&format!("- **Total Rust Files**: {}\n", total_rust_files));
        report.push_str(&format!("- **Total Commits**: {}\n", total_commits));
        report.push_str(&format!("- **Forks Managed**: {}\n", self.repositories.iter().filter(|r| r.is_fork).count()));

        // Timeline entries
        report.push_str("\n## Timeline (Reverse Chronological)\n\n");

        for entry in &self.timeline_entries {
            let icon = match entry.event_type.as_str() {
                "original_work" => "✨",
                "fork" => "🍴",
                _ => "📦"
            };

            report.push_str(&format!(
                "### {} {} - {}\n",
                icon,
                entry.date.format("%Y-%m-%d"),
                entry.repo_name
            ));
            report.push_str(&format!("**Type**: {}\n", entry.event_type));
            report.push_str(&format!("**Description**: {}\n", entry.description));

            if !entry.files_changed.is_empty() && entry.files_changed.len() <= 10 {
                report.push_str("**Key Files**:\n");
                for file in &entry.files_changed {
                    if let Some(filename) = file.split('/').last() {
                        report.push_str(&format!("- `{}`\n", filename));
                    }
                }
            }
            report.push_str("\n");
        }

        // Original work focus
        report.push_str("## Original Work Repositories\n\n");
        for repo in &original_repos {
            report.push_str(&format!("### {}\n", repo.name));
            report.push_str(&format!("- **Path**: `{}`\n", repo.path));
            report.push_str(&format!("- **Commits**: {}\n", repo.commit_count));
            report.push_str(&format!("- **Rust Files**: {}\n", repo.rust_files.len()));
            if let Some(date) = repo.last_commit_date {
                report.push_str(&format!("- **Last Activity**: {}\n", date.format("%Y-%m-%d")));
            }
            report.push_str("\n");
        }

        // Save report
        let _ = fs::write("introspector_timeline.md", &report);

        // Also save JSON data
        let json_data = self.generate_json();
        let _ = fs::write("introspector_timeline.json", &json_data);

        println!("   ✅ Timeline saved to introspector_timeline.md and .json");
        println!("   📈 {} original repositories identified", original_repos.len());
    }

    fn generate_json(&self) -> String {
        // Simple JSON generation for the timeline data
        let mut json = String::from("{\n");
        json.push_str(&format!("  \"generated_at\": \"{}\",\n", Utc::now().to_rfc3339()));
        json.push_str("  \"repositories\": [\n");

        for (i, repo) in self.repositories.iter().enumerate() {
            if i > 0 { json.push_str(",\n"); }
            json.push_str(&format!(
                "    {{\n      \"name\": \"{}\",\n      \"is_original\": {},\n      \"commit_count\": {},\n      \"rust_files\": {}\n    }}",
                repo.name, repo.is_original, repo.commit_count, repo.rust_files.len()
            ));
        }

        json.push_str("\n  ]\n}");
        json
    }

    fn get_git_remote(&self, path: &str) -> Option<String> {
        let output = Command::new("git")
            .args(&["-C", path, "remote", "get-url", "origin"])
            .output()
            .ok()?;

        if output.status.success() {
            Some(String::from_utf8(output.stdout).ok()?.trim().to_string())
        } else {
            None
        }
    }
}
