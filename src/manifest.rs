// Manifest API
use axum::{routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct Manifest {
    pub version: String,
    pub files: HashMap<String, FileInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileInfo {
    pub hash: String,
    pub size: u64,
    pub modified: String,
}

pub fn create_manifest_routes() -> Router {
    Router::new().route("/api/manifest", get(manifest_handler))
}

async fn manifest_handler() -> Json<Manifest> {
    let mut files = HashMap::new();

    // Get server's store path hash
    let store_hash = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().and_then(|p| p.parent()).map(|p| p.to_path_buf()))
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        .and_then(|n| n.split('-').next().map(|s| s.to_string()))
        .unwrap_or_else(|| "unknown".to_string());

    // Scan www directory
    if let Ok(entries) = fs::read_dir("www") {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    let path = entry.path();
                    let filename = path.file_name().unwrap().to_string_lossy().to_string();

                    // Calculate hash
                    if let Ok(content) = fs::read(&path) {
                        let mut hasher = Sha256::new();
                        hasher.update(&content);
                        let hash = format!("{:x}", hasher.finalize());

                        files.insert(
                            filename,
                            FileInfo {
                                hash: hash[..8].to_string(),
                                size: metadata.len(),
                                modified: format!("{:?}", metadata.modified().ok()),
                            },
                        );
                    }
                }
            }
        }
    }

    Json(Manifest {
        version: format!("{}-{}", env!("GIT_HASH"), store_hash),
        files,
    })
}
