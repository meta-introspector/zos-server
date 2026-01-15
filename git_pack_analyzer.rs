use std::collections::HashMap;
use std::fs;
use std::process::Command;

struct GitPackAnalyzer {
    pack_stats: HashMap<String, PackInfo>,
    total_objects: u64,
    total_compressed_size: u64,
    total_uncompressed_size: u64,
}

#[derive(Debug)]
struct PackInfo {
    objects: u32,
    compressed_size: u64,
    uncompressed_size: u64,
    compression_ratio: f64,
}

impl GitPackAnalyzer {
    fn new() -> Self {
        Self {
            pack_stats: HashMap::new(),
            total_objects: 0,
            total_compressed_size: 0,
            total_uncompressed_size: 0,
        }
    }

    fn analyze_all_repos(&mut self) -> Result<(), String> {
        let repos_dir = "/mnt/data1/meta-introspector/repos";

        if let Ok(entries) = fs::read_dir(repos_dir) {
            for entry in entries.flatten() {
                if entry.file_type().map(|ft| ft.is_symlink()).unwrap_or(false) {
                    if let Ok(real_path) = fs::read_link(entry.path()) {
                        let repo_name = entry.file_name().to_string_lossy().to_string();
                        self.analyze_repo(&real_path, &repo_name)?;
                    }
                }
            }
        }
        Ok(())
    }

    fn analyze_repo(&mut self, repo_path: &std::path::Path, repo_name: &str) -> Result<(), String> {
        let git_dir = repo_path.join(".git");
        if !git_dir.exists() {
            return Ok(()); // Skip non-git repos
        }

        // Get pack file info using git count-objects
        let output = Command::new("git")
            .args(&["count-objects", "-v"])
            .current_dir(repo_path)
            .output()
            .map_err(|e| format!("Git command failed: {}", e))?;

        if !output.status.success() {
            return Ok(()); // Skip repos with git issues
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut objects = 0;
        let mut size_pack = 0;
        let mut size = 0;

        for line in output_str.lines() {
            if line.starts_with("count ") {
                objects += line
                    .split_whitespace()
                    .nth(1)
                    .and_then(|s| s.parse::<u32>().ok())
                    .unwrap_or(0);
            } else if line.starts_with("in-pack ") {
                objects += line
                    .split_whitespace()
                    .nth(1)
                    .and_then(|s| s.parse::<u32>().ok())
                    .unwrap_or(0);
            } else if line.starts_with("size-pack ") {
                size_pack = line
                    .split_whitespace()
                    .nth(1)
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(0);
            } else if line.starts_with("size ") {
                size += line
                    .split_whitespace()
                    .nth(1)
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(0);
            }
        }

        // Get actual uncompressed size using git cat-file
        let uncompressed = self.estimate_uncompressed_size(repo_path)?;

        let compression_ratio = if uncompressed > 0 {
            size_pack as f64 / uncompressed as f64
        } else {
            1.0
        };

        let pack_info = PackInfo {
            objects,
            compressed_size: size_pack,
            uncompressed_size: uncompressed,
            compression_ratio,
        };

        self.pack_stats.insert(repo_name.to_string(), pack_info);
        self.total_objects += objects as u64;
        self.total_compressed_size += size_pack;
        self.total_uncompressed_size += uncompressed;

        Ok(())
    }

    fn estimate_uncompressed_size(&self, repo_path: &std::path::Path) -> Result<u64, String> {
        // Use git ls-tree to get blob sizes
        let output = Command::new("git")
            .args(&["ls-tree", "-r", "-l", "HEAD"])
            .current_dir(repo_path)
            .output()
            .map_err(|e| format!("Git ls-tree failed: {}", e))?;

        if !output.status.success() {
            return Ok(0);
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut total_size = 0;

        for line in output_str.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                if let Ok(size) = parts[3].parse::<u64>() {
                    total_size += size;
                }
            }
        }

        Ok(total_size)
    }

    fn extract_delta_patterns(&self, repo_path: &std::path::Path) -> Result<Vec<String>, String> {
        // Get pack file delta information
        let output = Command::new("git")
            .args(&["verify-pack", "-v"])
            .current_dir(repo_path.join(".git/objects/pack"))
            .output();

        match output {
            Ok(out) if out.status.success() => {
                let output_str = String::from_utf8_lossy(&out.stdout);
                let mut patterns = Vec::new();

                for line in output_str.lines().take(10) {
                    // Sample first 10
                    if line.contains("chain") {
                        patterns.push(line.to_string());
                    }
                }
                Ok(patterns)
            }
            _ => Ok(vec![]),
        }
    }

    fn print_analysis(&self) {
        println!("📦 Git Pack Compression Analysis:");
        println!("  Total repositories analyzed: {}", self.pack_stats.len());
        println!("  Total objects: {}", self.total_objects);
        println!(
            "  Total compressed size: {} MB",
            self.total_compressed_size / 1024 / 1024
        );
        println!(
            "  Total uncompressed size: {} MB",
            self.total_uncompressed_size / 1024 / 1024
        );

        let overall_ratio = if self.total_uncompressed_size > 0 {
            self.total_compressed_size as f64 / self.total_uncompressed_size as f64
        } else {
            1.0
        };

        println!("  Overall compression ratio: {:.3}", overall_ratio);
        println!(
            "  Space saved: {} MB ({:.1}%)",
            (self.total_uncompressed_size - self.total_compressed_size) / 1024 / 1024,
            (1.0 - overall_ratio) * 100.0
        );

        // Top compressed repos
        let mut repos: Vec<_> = self.pack_stats.iter().collect();
        repos.sort_by(|a, b| {
            a.1.compression_ratio
                .partial_cmp(&b.1.compression_ratio)
                .unwrap()
        });

        println!("\n🏆 Best compression ratios:");
        for (name, info) in repos.iter().take(5) {
            println!(
                "    {}: {:.3} ({} objects, {} KB compressed)",
                name,
                info.compression_ratio,
                info.objects,
                info.compressed_size / 1024
            );
        }

        println!("\n📈 Largest repositories by compressed size:");
        repos.sort_by_key(|(_, info)| std::cmp::Reverse(info.compressed_size));
        for (name, info) in repos.iter().take(5) {
            println!(
                "    {}: {} MB compressed ({} objects)",
                name,
                info.compressed_size / 1024 / 1024,
                info.objects
            );
        }

        // Estimate indexing value from compression
        let compression_efficiency = 1.0 - overall_ratio;
        let indexing_value = compression_efficiency * self.total_objects as f64;

        println!("\n💡 Compression-based indexing insights:");
        println!(
            "    Compression efficiency: {:.1}%",
            compression_efficiency * 100.0
        );
        println!("    Estimated unique patterns: {:.0}", indexing_value);
        println!("    Delta compression potential: High (git packs already optimized)");
    }
}

fn main() {
    let mut analyzer = GitPackAnalyzer::new();

    println!("🔍 Analyzing git pack compression across all repositories...");

    if let Err(e) = analyzer.analyze_all_repos() {
        eprintln!("Error: {}", e);
        return;
    }

    analyzer.print_analysis();
}
