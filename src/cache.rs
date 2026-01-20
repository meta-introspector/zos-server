use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;
use std::process::Command;

pub struct CacheStore {
    cache_dir: String,
}

impl CacheStore {
    pub fn new() -> Self {
        let cache_dir = "/tmp/zos_cache".to_string();
        fs::create_dir_all(&cache_dir).ok();
        Self { cache_dir }
    }

    fn hash_key(&self, key: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        format!("{:x}", hasher.finalize())[..16].to_string()
    }

    pub fn get_or_compute<F>(&self, key: &str, deps: &[&str], compute: F) -> Option<String>
    where
        F: FnOnce() -> Option<String>,
    {
        let hash = self.hash_key(key);
        let cache_path = format!("{}/{}", self.cache_dir, hash);
        let deps_path = format!("{}/{}.deps", self.cache_dir, hash);

        // Check if cache exists and deps haven't changed
        if Path::new(&cache_path).exists() {
            if let Ok(cached_deps) = fs::read_to_string(&deps_path) {
                let current_deps = self.compute_deps_hash(deps);
                if cached_deps.trim() == current_deps {
                    if let Ok(result) = fs::read_to_string(&cache_path) {
                        return Some(result);
                    }
                }
            }
        }

        // Cache miss or stale, recompute
        if let Some(result) = compute() {
            fs::write(&cache_path, &result).ok();
            fs::write(&deps_path, self.compute_deps_hash(deps)).ok();
            Some(result)
        } else {
            None
        }
    }

    fn compute_deps_hash(&self, deps: &[&str]) -> String {
        let mut hasher = Sha256::new();
        for dep in deps {
            if let Ok(metadata) = fs::metadata(dep) {
                if let Ok(modified) = metadata.modified() {
                    hasher.update(format!("{}:{:?}", dep, modified).as_bytes());
                }
            }
        }
        format!("{:x}", hasher.finalize())[..16].to_string()
    }
}

// Cached computations
impl CacheStore {
    pub fn file_count(&self) -> String {
        self.get_or_compute("files_txt_count", &["/mnt/data1/files.txt"], || {
            Command::new("wc")
                .arg("-l")
                .arg("/mnt/data1/files.txt")
                .output()
                .ok()
                .and_then(|output| {
                    let count_str = String::from_utf8_lossy(&output.stdout)
                        .split_whitespace()
                        .next()?
                        .to_string();
                    let count: u64 = count_str.parse().ok()?;
                    Some(if count > 1_000_000 {
                        format!("{:.1}M", count as f64 / 1_000_000.0)
                    } else if count > 1_000 {
                        format!("{:.1}K", count as f64 / 1_000.0)
                    } else {
                        count.to_string()
                    })
                })
        })
        .unwrap_or_else(|| "unknown".to_string())
    }

    pub fn lattice_stats(&self) -> Vec<(String, usize)> {
        self.get_or_compute(
            "lattice_structure",
            &["/mnt/data1/meta-introspector/value-lattice"],
            || {
                let mut stats = Vec::new();
                if let Ok(entries) = fs::read_dir("/mnt/data1/meta-introspector/value-lattice") {
                    for entry in entries.filter_map(|e| e.ok()) {
                        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                            let dir_name = entry.file_name().to_string_lossy().to_string();
                            let count = fs::read_dir(entry.path())
                                .map(|entries| entries.count())
                                .unwrap_or(0);
                            stats.push((dir_name, count));
                        }
                    }
                }
                stats.sort_by(|a, b| {
                    let a_num: u32 =
                        a.0.strip_prefix("length-")
                            .unwrap_or("0")
                            .parse()
                            .unwrap_or(0);
                    let b_num: u32 =
                        b.0.strip_prefix("length-")
                            .unwrap_or("0")
                            .parse()
                            .unwrap_or(0);
                    a_num.cmp(&b_num)
                });
                Some(serde_json::to_string(&stats).ok()?)
            },
        )
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or_default()
    }

    pub fn memory_info(&self) -> String {
        // Memory changes frequently, cache for 30 seconds only
        let key = format!(
            "memory_info_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                / 30
        );
        self.get_or_compute(
            &key,
            &[], // No file deps for memory
            || {
                Command::new("free")
                    .arg("-h")
                    .output()
                    .ok()
                    .map(|output| String::from_utf8_lossy(&output.stdout).to_string())
            },
        )
        .unwrap_or_else(|| "Memory info unavailable".to_string())
    }
}
