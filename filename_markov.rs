use std::collections::HashMap;
use std::fs;
use std::path::Path;

struct FilenameMarkov {
    path_transitions: HashMap<char, HashMap<char, u32>>,
    extension_counts: HashMap<String, u32>,
    depth_distribution: HashMap<u32, u32>,
    total_files: u64,
    total_path_chars: u64,
}

impl FilenameMarkov {
    fn new() -> Self {
        Self {
            path_transitions: HashMap::new(),
            extension_counts: HashMap::new(),
            depth_distribution: HashMap::new(),
            total_files: 0,
            total_path_chars: 0,
        }
    }

    fn scan_all_repos(&mut self) -> Result<(), String> {
        let repos_dir = "/mnt/data1/meta-introspector/repos";

        if let Ok(entries) = fs::read_dir(repos_dir) {
            for entry in entries.flatten() {
                if entry.file_type().map(|ft| ft.is_symlink()).unwrap_or(false) {
                    if let Ok(real_path) = fs::read_link(entry.path()) {
                        self.walk_directory(&real_path, 0)?;
                    }
                }
            }
        }
        Ok(())
    }

    fn walk_directory(&mut self, dir: &Path, depth: u32) -> Result<(), String> {
        if depth > 20 {
            return Ok(());
        } // Prevent infinite recursion

        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let path_str = path.to_string_lossy();

                // Train on full path
                self.train_on_path(&path_str);

                // Count extension
                if let Some(ext) = path.extension() {
                    let ext_str = ext.to_string_lossy().to_lowercase();
                    *self.extension_counts.entry(ext_str).or_insert(0) += 1;
                }

                // Track depth
                *self.depth_distribution.entry(depth).or_insert(0) += 1;
                self.total_files += 1;

                // Recurse into directories
                if path.is_dir()
                    && !path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .starts_with('.')
                {
                    self.walk_directory(&path, depth + 1)?;
                }
            }
        }
        Ok(())
    }

    fn train_on_path(&mut self, path: &str) {
        let chars: Vec<char> = path.chars().collect();
        self.total_path_chars += chars.len() as u64;

        for window in chars.windows(2) {
            let from = window[0];
            let to = window[1];

            *self
                .path_transitions
                .entry(from)
                .or_insert_with(HashMap::new)
                .entry(to)
                .or_insert(0) += 1;
        }
    }

    fn estimate_indexing_cost(&self) -> (f64, f64, f64) {
        // AWS pricing estimates (per 1000 operations)
        let s3_list_cost = 0.0005; // $0.0005 per 1000 LIST requests
        let lambda_cost = 0.0000002; // $0.0000002 per request (128MB, 100ms avg)
        let dynamodb_write_cost = 0.00125; // $0.00125 per 1000 write units

        let files_in_thousands = self.total_files as f64 / 1000.0;

        let s3_cost = files_in_thousands * s3_list_cost;
        let processing_cost = (self.total_files as f64) * lambda_cost;
        let storage_cost = files_in_thousands * dynamodb_write_cost;

        (s3_cost, processing_cost, storage_cost)
    }

    fn print_analysis(&self) {
        println!("📁 Filename Markov Analysis:");
        println!("  Total files: {}", self.total_files);
        println!("  Total path characters: {}", self.total_path_chars);
        println!(
            "  Average path length: {:.1}",
            self.total_path_chars as f64 / self.total_files as f64
        );

        // Top extensions
        let mut ext_vec: Vec<_> = self.extension_counts.iter().collect();
        ext_vec.sort_by_key(|(_, count)| std::cmp::Reverse(**count));
        println!("\n🔧 Top 10 file extensions:");
        for (ext, count) in ext_vec.iter().take(10) {
            println!("    .{}: {} files", ext, count);
        }

        // Depth distribution
        println!("\n📊 Directory depth distribution:");
        let mut depth_vec: Vec<_> = self.depth_distribution.iter().collect();
        depth_vec.sort_by_key(|(depth, _)| **depth);
        for (depth, count) in depth_vec.iter().take(10) {
            println!("    Depth {}: {} files", depth, count);
        }

        // Path character patterns
        let mut path_transitions: Vec<_> = self
            .path_transitions
            .iter()
            .flat_map(|(from, to_map)| to_map.iter().map(move |(to, count)| ((*from, *to), *count)))
            .collect();
        path_transitions.sort_by_key(|(_, count)| std::cmp::Reverse(*count));

        println!("\n🔤 Top 10 path character transitions:");
        for ((from, to), count) in path_transitions.iter().take(10) {
            let from_display = if *from == '/' {
                "'/'"
            } else {
                &format!("'{}'", from)
            };
            let to_display = if *to == '/' {
                "'/'"
            } else {
                &format!("'{}'", to)
            };
            println!("    {} → {}: {} times", from_display, to_display, count);
        }

        // Cost estimation
        let (s3_cost, processing_cost, storage_cost) = self.estimate_indexing_cost();
        let total_cost = s3_cost + processing_cost + storage_cost;

        println!("\n💰 Estimated AWS indexing costs:");
        println!("    S3 LIST operations: ${:.4}", s3_cost);
        println!("    Lambda processing: ${:.4}", processing_cost);
        println!("    DynamoDB storage: ${:.4}", storage_cost);
        println!("    Total estimated cost: ${:.4}", total_cost);
    }
}

fn main() {
    let mut model = FilenameMarkov::new();

    println!("🚀 Scanning all repositories for filename patterns...");

    if let Err(e) = model.scan_all_repos() {
        eprintln!("Error: {}", e);
        return;
    }

    model.print_analysis();
}
