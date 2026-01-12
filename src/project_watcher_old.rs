use std::path::PathBuf;
use std::time::SystemTime;
use notify::{Watcher, RecursiveMode, Event, EventKind};
use tokio::sync::mpsc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct FileChangeEvent {
    pub path: PathBuf,
    pub event_type: String,
    pub timestamp: SystemTime,
    pub project_root: PathBuf,
}

pub struct ProjectWatcher {
    watched_dirs: Vec<PathBuf>,
    change_sender: mpsc::UnboundedSender<FileChangeEvent>,
}

impl ProjectWatcher {
    pub fn new() -> (Self, mpsc::UnboundedReceiver<FileChangeEvent>) {
        let (tx, rx) = mpsc::unbounded_channel();

        (Self {
            watched_dirs: Vec::new(),
            change_sender: tx,
        }, rx)
    }

    pub fn add_project_dir(&mut self, path: PathBuf) -> Result<(), String> {
        // Watch for Rust files, Cargo.toml, and other relevant changes
        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            match res {
                Ok(event) => {
                    if let Some(path) = event.paths.first() {
                        // Filter for relevant files
                        if Self::is_relevant_file(path) {
                            let change_event = FileChangeEvent {
                                path: path.clone(),
                                event_type: format!("{:?}", event.kind),
                                timestamp: SystemTime::now(),
                                project_root: Self::find_project_root(path),
                            };

                            // Send to server for processing
                            if let Err(e) = self.change_sender.send(change_event) {
                                eprintln!("Failed to send file change event: {}", e);
                            }
                        }
                    }
                }
                Err(e) => eprintln!("Watch error: {:?}", e),
            }
        }).map_err(|e| format!("Failed to create watcher: {}", e))?;

        watcher.watch(&path, RecursiveMode::Recursive)
            .map_err(|e| format!("Failed to watch directory: {}", e))?;

        self.watched_dirs.push(path);
        Ok(())
    }

    fn is_relevant_file(path: &PathBuf) -> bool {
        if let Some(ext) = path.extension() {
            matches!(ext.to_str(), Some("rs" | "toml" | "md" | "json"))
        } else {
            false
        }
    }

    fn find_project_root(path: &PathBuf) -> PathBuf {
        let mut current = path.clone();
        while let Some(parent) = current.parent() {
            if parent.join("Cargo.toml").exists() {
                return parent.to_path_buf();
            }
            current = parent.to_path_buf();
        }
        path.clone()
    }

    async fn start_watching(&mut self) {
        // Load repos from meta-introspector repos.txt
        if let Ok(content) = std::fs::read_to_string("/mnt/data1/meta-introspector/repos.txt") {
            for line in content.lines() {
                let path = PathBuf::from(line.trim().replace("~", &std::env::var("HOME").unwrap_or_default()));
                if path.exists() {
                    if let Err(e) = self.add_project_dir(path.clone()) {
                        eprintln!("Failed to watch {}: {}", path.display(), e);
                    } else {
                        println!("👁️  Watching: {}", path.display());
                    }
                }
            }
        }

        // Add meta-introspector canonical directories
        if let Ok(entries) = std::fs::read_dir("/mnt/data1/meta-introspector/canonical") {
            for entry in entries.flatten() {
                let sources_path = entry.path().join("sources/v1");
                if sources_path.exists() {
                    if let Ok(real_path) = std::fs::read_link(&sources_path) {
                        if let Err(e) = self.add_project_dir(real_path.clone()) {
                            eprintln!("Failed to watch canonical {}: {}", real_path.display(), e);
                        } else {
                            println!("👁️  Watching canonical: {}", real_path.display());
                        }
                    }
                }
            }
        }
    }
}

trait PathExpand {
    fn expand(&self) -> PathBuf;
}

impl PathExpand for PathBuf {
    fn expand(&self) -> PathBuf {
        if let Ok(home) = std::env::var("HOME") {
            let expanded = self.to_string_lossy().replace("~", &home);
            PathBuf::from(expanded.to_string())
        } else {
            self.clone()
        }
    }
}
