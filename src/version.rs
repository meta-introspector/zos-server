// Version information module
use chrono::{DateTime, Utc};
use serde_json::Value;
use std::process::Command;

#[derive(Clone)]
pub struct GitStatus {
    pub dirty_files: Vec<String>,
    pub change_count: usize,
    pub latest_change: Option<DateTime<Utc>>,
}

pub struct VersionInfo {
    pub version: String,
    pub git_commit: String,
    pub git_status: GitStatus,
    pub build_time: String,
    pub binary_age: String,
}

impl GitStatus {
    pub fn get() -> Self {
        let mut dirty_files = Vec::new();

        // Get modified files
        if let Ok(output) = Command::new("git").args(&["diff", "--name-only"]).output() {
            let files: Vec<String> = String::from_utf8_lossy(&output.stdout)
                .lines()
                .map(|s| s.to_string())
                .collect();
            dirty_files.extend(files);
        }

        // Get staged files
        if let Ok(output) = Command::new("git")
            .args(&["diff", "--cached", "--name-only"])
            .output()
        {
            let files: Vec<String> = String::from_utf8_lossy(&output.stdout)
                .lines()
                .map(|s| s.to_string())
                .collect();
            dirty_files.extend(files);
        }

        // Get latest change time from the most recently modified file
        let latest_change = dirty_files
            .iter()
            .filter_map(|file| std::fs::metadata(file).ok())
            .filter_map(|meta| meta.modified().ok())
            .filter_map(|time| {
                DateTime::from_timestamp(
                    time.duration_since(std::time::UNIX_EPOCH).ok()?.as_secs() as i64,
                    0,
                )
            })
            .max();

        Self {
            change_count: dirty_files.len(),
            dirty_files,
            latest_change,
        }
    }
}

impl VersionInfo {
    pub fn get() -> Self {
        let version = env!("CARGO_PKG_VERSION").to_string();

        let git_commit = Command::new("git")
            .args(&["rev-parse", "--short", "HEAD"])
            .output()
            .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
            .unwrap_or_else(|_| "unknown".to_string());

        let git_status = GitStatus::get();
        let build_time = chrono::Utc::now()
            .format("%Y-%m-%d %H:%M:%S UTC")
            .to_string();
        let binary_age = Self::get_binary_age();

        Self {
            version,
            git_commit,
            git_status,
            build_time,
            binary_age,
        }
    }

    fn get_binary_age() -> String {
        if let Ok(metadata) = std::fs::metadata(std::env::current_exe().unwrap_or_default()) {
            if let Ok(modified) = metadata.modified() {
                if let Ok(duration) = modified.elapsed() {
                    let seconds = duration.as_secs();
                    if seconds < 60 {
                        return format!("{}s", seconds);
                    } else if seconds < 3600 {
                        return format!("{}m", seconds / 60);
                    } else if seconds < 86400 {
                        return format!("{}h", seconds / 3600);
                    } else {
                        return format!("{}d", seconds / 86400);
                    }
                }
            }
        }
        "unknown".to_string()
    }

    pub fn to_json(&self) -> Value {
        serde_json::json!({
            "version": self.version,
            "git_commit": if self.git_status.change_count > 0 {
                format!("{}-dirty", self.git_commit)
            } else {
                self.git_commit.clone()
            },
            "dirty_files": self.git_status.dirty_files,
            "change_count": self.git_status.change_count,
            "latest_change": self.git_status.latest_change,
            "build_time": self.build_time,
            "binary_age": self.binary_age
        })
    }
}
