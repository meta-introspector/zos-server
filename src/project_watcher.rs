use notify::{Event, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use tokio::sync::mpsc;

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
    _watchers: Vec<Box<dyn notify::Watcher + Send>>, // Keep watchers alive
}

impl ProjectWatcher {
    pub fn new() -> (Self, mpsc::UnboundedReceiver<FileChangeEvent>) {
        let (tx, rx) = mpsc::unbounded_channel();

        (
            Self {
                watched_dirs: Vec::new(),
                change_sender: tx,
                _watchers: Vec::new(),
            },
            rx,
        )
    }

    pub fn add_project_dir(&mut self, path: PathBuf) -> Result<(), String> {
        let sender = self.change_sender.clone();
        let path_clone = path.clone();

        let mut watcher =
            notify::recommended_watcher(move |res: Result<Event, notify::Error>| match res {
                Ok(event) => {
                    if let Some(event_path) = event.paths.first() {
                        if Self::is_relevant_file(event_path) {
                            let change_event = FileChangeEvent {
                                path: event_path.clone(),
                                event_type: format!("{:?}", event.kind),
                                timestamp: SystemTime::now(),
                                project_root: Self::find_project_root(event_path),
                            };

                            if let Err(e) = sender.send(change_event) {
                                eprintln!("Failed to send file change event: {}", e);
                            }
                        }
                    }
                }
                Err(e) => eprintln!("Watch error: {:?}", e),
            })
            .map_err(|e| format!("Failed to create watcher: {}", e))?;

        watcher
            .watch(&path_clone, RecursiveMode::Recursive)
            .map_err(|e| format!("Failed to watch directory: {}", e))?;

        self.watched_dirs.push(path);
        self._watchers.push(Box::new(watcher));
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

    pub async fn start_watching(&mut self) {
        // Watch the standardized symlink directory
        let repos_dir = PathBuf::from("/mnt/data1/meta-introspector/repos");

        if repos_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&repos_dir) {
                for entry in entries.flatten() {
                    if entry.file_type().map(|ft| ft.is_symlink()).unwrap_or(false) {
                        // Follow the symlink to get the real path
                        if let Ok(real_path) = std::fs::read_link(entry.path()) {
                            if real_path.exists() {
                                if let Err(e) = self.add_project_dir(real_path.clone()) {
                                    eprintln!("Failed to watch {}: {}", real_path.display(), e);
                                } else {
                                    println!(
                                        "👁️  Watching: {} -> {}",
                                        entry.file_name().to_string_lossy(),
                                        real_path.display()
                                    );
                                }
                            }
                        }
                    }
                }
            }
        } else {
            eprintln!("❌ Repos directory not found: {}", repos_dir.display());
            eprintln!("💡 Run ./create-repo-symlinks.sh to create it");
        }
    }
}

trait PathExpand {
    fn expand(&self) -> PathBuf;
}

impl PathExpand for PathBuf {
    fn expand(&self) -> PathBuf {
        let binding = self.to_string_lossy();
        let expanded = binding.replace("~", &std::env::var("HOME").unwrap_or_default());
        PathBuf::from(expanded.to_string())
    }
}
