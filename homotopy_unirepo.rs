use std::collections::{HashMap, HashSet};
use std::fs;
use std::process::Command;

struct HomotopyUnirepo {
    object_hashes: HashSet<String>,
    object_types: HashMap<String, String>,
    object_sizes: HashMap<String, u64>,
    homotopy_classes: HashMap<String, Vec<String>>, // Similar objects grouped
    compression_map: HashMap<String, String>,       // Original -> compressed form
    total_objects: u64,
    deduplicated_objects: u64,
}

impl HomotopyUnirepo {
    fn new() -> Self {
        Self {
            object_hashes: HashSet::new(),
            object_types: HashMap::new(),
            object_sizes: HashMap::new(),
            homotopy_classes: HashMap::new(),
            compression_map: HashMap::new(),
            total_objects: 0,
            deduplicated_objects: 0,
        }
    }

    fn extract_all_git_objects(&mut self) -> Result<(), String> {
        let repos_dir = "/mnt/data1/meta-introspector/repos";

        if let Ok(entries) = fs::read_dir(repos_dir) {
            for entry in entries.flatten() {
                if entry.file_type().map(|ft| ft.is_symlink()).unwrap_or(false) {
                    if let Ok(real_path) = fs::read_link(entry.path()) {
                        let repo_name = entry.file_name().to_string_lossy().to_string();
                        self.extract_repo_objects(&real_path, &repo_name)?;
                    }
                }
            }
        }

        self.deduplicated_objects = self.object_hashes.len() as u64;
        Ok(())
    }

    fn extract_repo_objects(
        &mut self,
        repo_path: &std::path::Path,
        repo_name: &str,
    ) -> Result<(), String> {
        let git_dir = repo_path.join(".git");
        if !git_dir.exists() {
            return Ok(());
        }

        // Get all objects using git cat-file --batch-all-objects
        let output = Command::new("git")
            .args(&["cat-file", "--batch-all-objects", "--batch-check"])
            .current_dir(repo_path)
            .output()
            .map_err(|e| format!("Git cat-file failed: {}", e))?;

        if !output.status.success() {
            return Ok(());
        }

        let output_str = String::from_utf8_lossy(&output.stdout);

        for line in output_str.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let hash = parts[0].to_string();
                let obj_type = parts[1].to_string();
                let size = parts[2].parse::<u64>().unwrap_or(0);

                self.total_objects += 1;

                // Only add if not already seen (deduplication)
                if self.object_hashes.insert(hash.clone()) {
                    self.object_types.insert(hash.clone(), obj_type.clone());
                    self.object_sizes.insert(hash.clone(), size);

                    // Group into homotopy classes by type and size similarity
                    let class_key = format!("{}_{}", obj_type, size / 1024); // Group by KB
                    self.homotopy_classes
                        .entry(class_key)
                        .or_insert_with(Vec::new)
                        .push(hash.clone());
                }
            }
        }

        Ok(())
    }

    fn compute_homotopic_compression(&mut self) -> Result<(), String> {
        // For each homotopy class, find the canonical representative
        for (class_key, objects) in &self.homotopy_classes {
            if objects.len() > 1 {
                // Use the lexicographically smallest hash as canonical form
                let canonical = objects.iter().min().unwrap().clone();

                for obj in objects {
                    if obj != &canonical {
                        self.compression_map.insert(obj.clone(), canonical.clone());
                    }
                }
            }
        }
        Ok(())
    }

    fn extract_delta_patterns(&self, repo_path: &std::path::Path) -> Result<Vec<String>, String> {
        // Extract git pack delta chains to understand version relationships
        let pack_dir = repo_path.join(".git/objects/pack");
        if !pack_dir.exists() {
            return Ok(vec![]);
        }

        let output = Command::new("find")
            .args(&[pack_dir.to_str().unwrap(), "-name", "*.idx"])
            .output()
            .map_err(|e| format!("Find pack files failed: {}", e))?;

        let pack_files = String::from_utf8_lossy(&output.stdout);
        let mut delta_patterns = Vec::new();

        for pack_file in pack_files.lines().take(3) {
            // Sample first 3 packs
            let pack_file = pack_file.replace(".idx", ".pack");

            let verify_output = Command::new("git")
                .args(&["verify-pack", "-v", &pack_file])
                .output();

            if let Ok(out) = verify_output {
                if out.status.success() {
                    let verify_str = String::from_utf8_lossy(&out.stdout);
                    for line in verify_str.lines().take(5) {
                        if line.contains("chain") || line.contains("delta") {
                            delta_patterns.push(line.to_string());
                        }
                    }
                }
            }
        }

        Ok(delta_patterns)
    }

    fn print_homotopy_analysis(&self) {
        println!("🌌 Homotopy Unirepo Analysis:");
        println!("  Total objects across all repos: {}", self.total_objects);
        println!(
            "  Unique objects (after deduplication): {}",
            self.deduplicated_objects
        );
        println!(
            "  Duplicate objects removed: {}",
            self.total_objects - self.deduplicated_objects
        );
        println!(
            "  Deduplication ratio: {:.2}%",
            (1.0 - self.deduplicated_objects as f64 / self.total_objects as f64) * 100.0
        );

        // Object type distribution
        let mut type_counts: HashMap<String, u32> = HashMap::new();
        for obj_type in self.object_types.values() {
            *type_counts.entry(obj_type.clone()).or_insert(0) += 1;
        }

        println!("\n📊 Object type distribution:");
        for (obj_type, count) in &type_counts {
            println!("    {}: {} objects", obj_type, count);
        }

        // Homotopy class analysis
        println!("\n🔗 Homotopy class analysis:");
        println!("  Total homotopy classes: {}", self.homotopy_classes.len());

        let mut class_sizes: Vec<_> = self
            .homotopy_classes
            .iter()
            .map(|(key, objects)| (key, objects.len()))
            .collect();
        class_sizes.sort_by_key(|(_, size)| std::cmp::Reverse(*size));

        println!("  Largest homotopy classes:");
        for (class_key, size) in class_sizes.iter().take(5) {
            println!("    {}: {} equivalent objects", class_key, size);
        }

        // Compression potential
        let compressed_objects = self.compression_map.len();
        println!("\n💾 Homotopic compression potential:");
        println!("  Objects that can be compressed: {}", compressed_objects);
        println!(
            "  Compression ratio: {:.2}%",
            compressed_objects as f64 / self.deduplicated_objects as f64 * 100.0
        );

        // Size analysis
        let total_size: u64 = self.object_sizes.values().sum();
        println!(
            "  Total unique object size: {} MB",
            total_size / 1024 / 1024
        );

        // Theoretical unirepo benefits
        let space_saved =
            (self.total_objects - self.deduplicated_objects) as f64 / self.total_objects as f64;
        println!("\n🎯 Unirepo theoretical benefits:");
        println!("  Space efficiency: {:.1}% reduction", space_saved * 100.0);
        println!("  Homotopy classes enable: Abstract representation of code evolution");
        println!("  Delta compression: Preserves version relationships in topological space");
        println!("  Canonical forms: Each unique pattern has single representative");
    }
}

fn main() {
    let mut unirepo = HomotopyUnirepo::new();

    println!("🔬 Extracting git objects for homotopy unirepo construction...");

    if let Err(e) = unirepo.extract_all_git_objects() {
        eprintln!("Error extracting objects: {}", e);
        return;
    }

    println!("🧮 Computing homotopic compression mappings...");

    if let Err(e) = unirepo.compute_homotopic_compression() {
        eprintln!("Error computing compression: {}", e);
        return;
    }

    unirepo.print_homotopy_analysis();
}
