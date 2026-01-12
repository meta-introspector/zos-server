use std::collections::HashMap;
use std::fs;
use std::process::Command;

struct HierarchicalMarkov {
    // Layer 1: File path patterns
    path_transitions: HashMap<char, HashMap<char, u32>>,
    path_stats: PathStats,

    // Layer 2: Git pack tree structures
    tree_transitions: HashMap<String, HashMap<String, u32>>,
    pack_stats: PackStats,

    // Layer 3: Typed content analysis
    content_by_type: HashMap<String, HashMap<char, HashMap<char, u32>>>,
    type_stats: HashMap<String, TypeStats>,
}

#[derive(Default)]
struct PathStats {
    total_files: u64,
    total_path_chars: u64,
    extensions: HashMap<String, u32>,
}

#[derive(Default)]
struct PackStats {
    total_objects: u64,
    object_types: HashMap<String, u32>,
    tree_depth: HashMap<u32, u32>,
}

#[derive(Default)]
struct TypeStats {
    file_count: u32,
    total_chars: u64,
    avg_file_size: f64,
}

impl HierarchicalMarkov {
    fn new() -> Self {
        Self {
            path_transitions: HashMap::new(),
            path_stats: PathStats::default(),
            tree_transitions: HashMap::new(),
            pack_stats: PackStats::default(),
            content_by_type: HashMap::new(),
            type_stats: HashMap::new(),
        }
    }

    fn analyze_all_layers(&mut self) -> Result<(), String> {
        println!("🔍 Layer 1: Analyzing file paths...");
        self.analyze_file_paths()?;

        println!("🔍 Layer 2: Analyzing git pack trees...");
        self.analyze_git_trees()?;

        println!("🔍 Layer 3: Analyzing typed file contents...");
        self.analyze_typed_contents()?;

        Ok(())
    }

    // Layer 1: File path Markov analysis
    fn analyze_file_paths(&mut self) -> Result<(), String> {
        let repos_dir = "/mnt/data1/meta-introspector/repos";

        if let Ok(entries) = fs::read_dir(repos_dir) {
            for entry in entries.flatten() {
                if entry.file_type().map(|ft| ft.is_symlink()).unwrap_or(false) {
                    if let Ok(real_path) = fs::read_link(entry.path()) {
                        self.walk_paths(&real_path, 0)?;
                    }
                }
            }
        }
        Ok(())
    }

    fn walk_paths(&mut self, dir: &std::path::Path, depth: u32) -> Result<(), String> {
        if depth > 15 { return Ok(()); }

        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let path_str = path.to_string_lossy();

                // Train on path characters
                self.train_path_transitions(&path_str);

                // Track extension
                if let Some(ext) = path.extension() {
                    let ext_str = ext.to_string_lossy().to_lowercase();
                    *self.path_stats.extensions.entry(ext_str).or_insert(0) += 1;
                }

                self.path_stats.total_files += 1;

                // Status update every 5%
                if self.path_stats.total_files % 1000 == 0 {
                    print!("\r📁 Layer 1: {} files processed - {}",
                        self.path_stats.total_files,
                        path_str.chars().take(50).collect::<String>());
                    use std::io::{self, Write};
                    io::stdout().flush().unwrap();
                }

                if path.is_dir() && !path.file_name().unwrap_or_default()
                    .to_string_lossy().starts_with('.') {
                    self.walk_paths(&path, depth + 1)?;
                }
            }
        }
        Ok(())
    }

    fn train_path_transitions(&mut self, path: &str) {
        let chars: Vec<char> = path.chars().collect();
        self.path_stats.total_path_chars += chars.len() as u64;

        for window in chars.windows(2) {
            *self.path_transitions
                .entry(window[0])
                .or_insert_with(HashMap::new)
                .entry(window[1])
                .or_insert(0) += 1;
        }
    }

    // Layer 2: Git pack tree analysis
    fn analyze_git_trees(&mut self) -> Result<(), String> {
        let repos_dir = "/mnt/data1/meta-introspector/repos";

        if let Ok(entries) = fs::read_dir(repos_dir) {
            for entry in entries.flatten().take(10) { // Sample 10 repos
                if entry.file_type().map(|ft| ft.is_symlink()).unwrap_or(false) {
                    if let Ok(real_path) = fs::read_link(entry.path()) {
                        self.analyze_repo_trees(&real_path)?;
                    }
                }
            }
        }
        Ok(())
    }

    fn analyze_repo_trees(&mut self, repo_path: &std::path::Path) -> Result<(), String> {
        let git_dir = repo_path.join(".git");
        if !git_dir.exists() { return Ok(()); }

        // Get tree objects and their structure
        let output = Command::new("git")
            .args(&["cat-file", "--batch-all-objects", "--batch-check"])
            .current_dir(repo_path)
            .output()
            .map_err(|e| format!("Git command failed: {}", e))?;

        if !output.status.success() { return Ok(()); }

        let output_str = String::from_utf8_lossy(&output.stdout);

        for line in output_str.lines().take(50) { // Sample first 50 objects
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let obj_type = parts[1];
                *self.pack_stats.object_types.entry(obj_type.to_string()).or_insert(0) += 1;
                self.pack_stats.total_objects += 1;

                // For tree objects, analyze structure
                if obj_type == "tree" {
                    self.analyze_tree_structure(repo_path, parts[0])?;
                }
            }
        }
        Ok(())
    }

    fn analyze_tree_structure(&mut self, repo_path: &std::path::Path, tree_hash: &str) -> Result<(), String> {
        let output = Command::new("git")
            .args(&["cat-file", "-p", tree_hash])
            .current_dir(repo_path)
            .output()
            .map_err(|e| format!("Git cat-file failed: {}", e))?;

        if !output.status.success() { return Ok(()); }

        let tree_content = String::from_utf8_lossy(&output.stdout);
        let mut prev_entry = String::new();

        for line in tree_content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let entry_type = parts[1];

                if !prev_entry.is_empty() {
                    *self.tree_transitions
                        .entry(prev_entry.clone())
                        .or_insert_with(HashMap::new)
                        .entry(entry_type.to_string())
                        .or_insert(0) += 1;
                }
                prev_entry = entry_type.to_string();
            }
        }
        Ok(())
    }

    // Layer 3: Typed content analysis
    fn analyze_typed_contents(&mut self) -> Result<(), String> {
        let repos_dir = "/mnt/data1/meta-introspector/repos";

        if let Ok(entries) = fs::read_dir(repos_dir) {
            for entry in entries.flatten().take(5) { // Sample 5 repos for content
                if entry.file_type().map(|ft| ft.is_symlink()).unwrap_or(false) {
                    if let Ok(real_path) = fs::read_link(entry.path()) {
                        self.analyze_repo_contents(&real_path)?;
                    }
                }
            }
        }
        Ok(())
    }

    fn analyze_repo_contents(&mut self, repo_path: &std::path::Path) -> Result<(), String> {
        self.walk_content_files(repo_path, 0)
    }

    fn walk_content_files(&mut self, dir: &std::path::Path, depth: u32) -> Result<(), String> {
        if depth > 10 { return Ok(()); }

        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten().take(20) { // Sample 20 files per dir
                let path = entry.path();

                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        let ext_str = ext.to_string_lossy().to_lowercase();

                        // Only analyze common code file types
                        if matches!(ext_str.as_str(), "rs" | "py" | "js" | "ts" | "go" | "c" | "cpp" | "h" | "java" | "md" | "txt" | "json" | "toml" | "yaml") {
                            if let Ok(content) = fs::read_to_string(&path) {
                                if content.len() < 50000 { // Reasonable file size
                                    self.train_typed_content(&ext_str, &content);

                                    // Status update
                                    let total_files: u32 = self.type_stats.values().map(|s| s.file_count).sum();
                                    if total_files % 50 == 0 {
                                        print!("\r📝 Layer 3: {} files - {}",
                                            total_files,
                                            path.file_name().unwrap_or_default().to_string_lossy());
                                        use std::io::{self, Write};
                                        io::stdout().flush().unwrap();
                                    }
                                }
                            }
                        }
                    }
                } else if path.is_dir() && !path.file_name().unwrap_or_default()
                    .to_string_lossy().starts_with('.') {
                    self.walk_content_files(&path, depth + 1)?;
                }
            }
        }
        Ok(())
    }

    fn train_typed_content(&mut self, file_type: &str, content: &str) {
        let chars: Vec<char> = content.chars().collect();

        // Initialize type stats
        let type_stats = self.type_stats.entry(file_type.to_string()).or_insert_with(TypeStats::default);
        type_stats.file_count += 1;
        type_stats.total_chars += chars.len() as u64;
        type_stats.avg_file_size = type_stats.total_chars as f64 / type_stats.file_count as f64;

        // Train content transitions for this type
        let content_transitions = self.content_by_type
            .entry(file_type.to_string())
            .or_insert_with(HashMap::new);

        for window in chars.windows(2) {
            *content_transitions
                .entry(window[0])
                .or_insert_with(HashMap::new)
                .entry(window[1])
                .or_insert(0) += 1;
        }
    }

    fn print_hierarchical_analysis(&self) {
        println!("\n🌳 Hierarchical Markov Analysis Results:");

        // Layer 1: Path analysis
        println!("\n📁 Layer 1 - File Path Patterns:");
        println!("  Total files: {}", self.path_stats.total_files);
        println!("  Total path characters: {}", self.path_stats.total_path_chars);
        println!("  Average path length: {:.1}",
            self.path_stats.total_path_chars as f64 / self.path_stats.total_files as f64);

        let mut ext_sorted: Vec<_> = self.path_stats.extensions.iter().collect();
        ext_sorted.sort_by_key(|(_, count)| std::cmp::Reverse(**count));
        println!("  Top extensions:");
        for (ext, count) in ext_sorted.iter().take(5) {
            println!("    .{}: {} files", ext, count);
        }

        // Layer 2: Git tree analysis
        println!("\n🌲 Layer 2 - Git Pack Tree Structure:");
        println!("  Total git objects: {}", self.pack_stats.total_objects);
        println!("  Object type distribution:");
        for (obj_type, count) in &self.pack_stats.object_types {
            println!("    {}: {} objects", obj_type, count);
        }

        println!("  Tree transition patterns:");
        for (from, to_map) in self.tree_transitions.iter().take(3) {
            for (to, count) in to_map.iter().take(2) {
                println!("    {} → {}: {} times", from, to, count);
            }
        }

        // Layer 3: Typed content analysis
        println!("\n📝 Layer 3 - Typed Content Patterns:");
        for (file_type, stats) in &self.type_stats {
            println!("  .{} files:", file_type);
            println!("    Count: {} files", stats.file_count);
            println!("    Total chars: {}", stats.total_chars);
            println!("    Avg size: {:.0} chars", stats.avg_file_size);

            // Show top transitions for this type
            if let Some(transitions) = self.content_by_type.get(file_type) {
                let mut trans_vec: Vec<_> = transitions.iter()
                    .flat_map(|(from, to_map)| {
                        to_map.iter().map(move |(to, count)| ((*from, *to), *count))
                    })
                    .collect();
                trans_vec.sort_by_key(|(_, count)| std::cmp::Reverse(*count));

                println!("    Top transitions:");
                for ((from, to), count) in trans_vec.iter().take(3) {
                    println!("      '{}' → '{}': {} times", from, to, count);
                }
            }
        }

        // Summary insights
        println!("\n🎯 Hierarchical Insights:");
        println!("  Layer 1 captures: File organization patterns");
        println!("  Layer 2 captures: Version control structure");
        println!("  Layer 3 captures: Language-specific syntax patterns");
        println!("  Combined model enables: Intelligent file type prediction and content generation");
    }
}

fn main() {
    let mut analyzer = HierarchicalMarkov::new();

    println!("🚀 Starting hierarchical Markov analysis...");

    if let Err(e) = analyzer.analyze_all_layers() {
        eprintln!("Error: {}", e);
        return;
    }

    analyzer.print_hierarchical_analysis();
}
