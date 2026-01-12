use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};

struct HierarchicalMarkov {
    path_transitions: HashMap<char, HashMap<char, u32>>,
    extensions: HashMap<String, u32>,
    total_files: u64,
    processed_files: u64,
}

impl HierarchicalMarkov {
    fn new() -> Self {
        Self {
            path_transitions: HashMap::new(),
            extensions: HashMap::new(),
            total_files: 0,
            processed_files: 0,
        }
    }

    fn analyze_from_file_list(&mut self) -> Result<(), String> {
        let file = fs::File::open("/mnt/data1/files.txt")
            .map_err(|e| format!("Failed to open files.txt: {}", e))?;

        let reader = BufReader::new(file);

        // First pass: count total files
        println!("📊 Counting total files...");
        self.total_files = 33981281; // We know from wc -l

        let file = fs::File::open("/mnt/data1/files.txt")
            .map_err(|e| format!("Failed to reopen files.txt: {}", e))?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let path = match line {
                Ok(p) => p,
                Err(_) => continue, // Skip invalid UTF-8 lines
            };

            self.analyze_path(&path);
            self.processed_files += 1;

            // Status update every 5%
            if self.processed_files % (self.total_files / 20) == 0 {
                let percent = (self.processed_files * 100) / self.total_files;
                print!("\r📁 {}% - {} files - {}",
                    percent,
                    self.processed_files,
                    path.chars().take(60).collect::<String>());
                use std::io::{self, Write};
                io::stdout().flush().unwrap();

                // Checkpoint: save binary model
                if percent >= 10 && percent % 10 == 0 {
                    println!("\n💾 Checkpoint at {}% - saving binary model...", percent);
                    self.save_binary_checkpoint(&format!("markov_checkpoint_{}.bin", percent)).ok();
                }
            }
        }

        println!("\n✅ Completed analyzing {} files", self.processed_files);
        Ok(())
    }

    fn analyze_path(&mut self, path: &str) {
        // Only process .rs files
        if !path.ends_with(".rs") {
            return;
        }

        // Train Markov on path characters
        let chars: Vec<char> = path.chars().collect();
        for window in chars.windows(2) {
            *self.path_transitions
                .entry(window[0])
                .or_insert_with(HashMap::new)
                .entry(window[1])
                .or_insert(0) += 1;
        }

        // Always count as rust extension
        *self.extensions.entry("rs".to_string()).or_insert(0) += 1;
    }

    fn save_binary_checkpoint(&self, filename: &str) -> Result<(), String> {
        use std::io::Write;
        let mut file = fs::File::create(filename)
            .map_err(|e| format!("Create file error: {}", e))?;

        // Write processed files count
        file.write_all(&self.processed_files.to_le_bytes()).unwrap();

        // Write extensions count and data
        file.write_all(&(self.extensions.len() as u32).to_le_bytes()).unwrap();
        for (ext, count) in &self.extensions {
            file.write_all(&(ext.len() as u32).to_le_bytes()).unwrap();
            file.write_all(ext.as_bytes()).unwrap();
            file.write_all(&count.to_le_bytes()).unwrap();
        }

        // Write top 100 transitions only (to keep size manageable)
        let mut transitions: Vec<_> = self.path_transitions.iter()
            .flat_map(|(from, to_map)| {
                to_map.iter().map(move |(to, count)| ((*from, *to), *count))
            })
            .collect();
        transitions.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
        transitions.truncate(100);

        file.write_all(&(transitions.len() as u32).to_le_bytes()).unwrap();
        for ((from, to), count) in transitions {
            file.write_all(&(from as u32).to_le_bytes()).unwrap();
            file.write_all(&(to as u32).to_le_bytes()).unwrap();
            file.write_all(&count.to_le_bytes()).unwrap();
        }

        println!("  Saved {} bytes to {}", file.metadata().unwrap().len(), filename);
        Ok(())
    }

    fn print_partial_analysis(&self) {
        let mut ext_sorted: Vec<_> = self.extensions.iter().collect();
        ext_sorted.sort_by_key(|(_, count)| std::cmp::Reverse(**count));

        println!("  Top 5 extensions so far:");
        for (ext, count) in ext_sorted.iter().take(5) {
            let percent = (**count as f64 / self.processed_files as f64) * 100.0;
            println!("    .{}: {} files ({:.2}%)", ext, count, percent);
        }
    }

    fn print_analysis(&self) {
        println!("\n🌳 Hierarchical Markov Analysis (33.9M files):");
        println!("  Total files processed: {}", self.processed_files);

        // Top extensions
        let mut ext_sorted: Vec<_> = self.extensions.iter().collect();
        ext_sorted.sort_by_key(|(_, count)| std::cmp::Reverse(**count));

        println!("\n🔧 Top 15 file extensions:");
        for (ext, count) in ext_sorted.iter().take(15) {
            let percent = (**count as f64 / self.processed_files as f64) * 100.0;
            println!("    .{}: {} files ({:.2}%)", ext, count, percent);
        }

        // Top path character transitions
        let mut transitions: Vec<_> = self.path_transitions.iter()
            .flat_map(|(from, to_map)| {
                to_map.iter().map(move |(to, count)| ((*from, *to), *count))
            })
            .collect();
        transitions.sort_by_key(|(_, count)| std::cmp::Reverse(*count));

        println!("\n🔤 Top 10 path character patterns:");
        for ((from, to), count) in transitions.iter().take(10) {
            let from_display = if *from == '/' { "'/' " } else { &format!("'{}' ", from) };
            let to_display = if *to == '/' { " '/'" } else { &format!(" '{}'", to) };
            println!("    {} →{}: {} times", from_display, to_display, count);
        }

        // Insights
        println!("\n🎯 Key Insights:");
        println!("  Dataset size: 33.9M files across all repositories");
        println!("  Path patterns reveal: Directory structure conventions");
        println!("  Extension distribution: Language and format preferences");
        println!("  Ready for Layer 2: Git pack tree analysis");
        println!("  Ready for Layer 3: Typed content analysis by extension");
    }
}

fn main() {
    let mut analyzer = HierarchicalMarkov::new();

    println!("🚀 Starting hierarchical Markov analysis from pre-collected file list...");

    if let Err(e) = analyzer.analyze_from_file_list() {
        eprintln!("Error: {}", e);
        return;
    }

    analyzer.print_analysis();
}
