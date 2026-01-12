use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileIndex {
    pub path: PathBuf,
    pub word_counts: HashMap<String, u32>,
    pub total_words: u32,
    pub line_count: u32,
    pub byte_size: u64,
    pub manifold_coords: [f64; 8], // 8D manifold coordinates
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifoldDimensions {
    pub complexity: f64,   // Code complexity metric
    pub level: f64,        // Directory depth level
    pub weight: f64,       // File importance weight
    pub domain: f64,       // Domain classification (0-1)
    pub entropy: f64,      // Information entropy
    pub connectivity: f64, // Cross-reference density
    pub temporal: f64,     // Temporal relevance
    pub semantic: f64,     // Semantic clustering
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RepoIndex {
    pub repo_path: PathBuf,
    pub files: Vec<FileIndex>,
    pub global_word_counts: HashMap<String, u32>,
    pub total_files: usize,
    pub total_words: u64,
    pub manifold_bounds: [(f64, f64); 8], // Min/max for each dimension
}

impl RepoIndex {
    pub fn new(repo_path: PathBuf) -> Self {
        Self {
            repo_path,
            files: Vec::new(),
            global_word_counts: HashMap::new(),
            total_files: 0,
            total_words: 0,
            manifold_bounds: [(0.0, 1.0); 8],
        }
    }

    pub fn scan_repository(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Scanning repository: {:?}", self.repo_path);

        let files = self.collect_files(&self.repo_path)?;
        println!("Found {} files to process", files.len());

        // Process files in parallel
        let file_indices: Vec<FileIndex> = files
            .par_iter()
            .filter_map(|path| self.process_file(path).ok())
            .collect();

        self.files = file_indices;
        self.total_files = self.files.len();

        // Build global word counts
        self.build_global_index();

        // Calculate manifold coordinates
        self.calculate_manifold_coordinates();

        Ok(())
    }

    fn collect_files(&self, dir: &Path) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
        let mut files = Vec::new();

        fn visit_dir(
            dir: &Path,
            files: &mut Vec<PathBuf>,
        ) -> Result<(), Box<dyn std::error::Error>> {
            if dir.is_dir() {
                for entry in fs::read_dir(dir)? {
                    let entry = entry?;
                    let path = entry.path();

                    if path.is_dir() {
                        // Skip common ignore patterns
                        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                            if matches!(
                                name,
                                ".git" | "target" | "node_modules" | ".lake" | "build"
                            ) {
                                continue;
                            }
                        }
                        visit_dir(&path, files)?;
                    } else if Self::is_text_file(&path) {
                        files.push(path);
                    }
                }
            }
            Ok(())
        }

        visit_dir(dir, &mut files)?;
        Ok(files)
    }

    fn is_text_file(path: &Path) -> bool {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            matches!(
                ext,
                "rs" | "py"
                    | "js"
                    | "ts"
                    | "md"
                    | "txt"
                    | "toml"
                    | "yaml"
                    | "yml"
                    | "json"
                    | "sh"
                    | "lean"
                    | "c"
                    | "cpp"
                    | "h"
                    | "hpp"
                    | "go"
                    | "java"
                    | "scala"
                    | "hs"
                    | "ml"
                    | "elm"
                    | "clj"
                    | "lisp"
                    | "pl"
                    | "rb"
                    | "php"
                    | "css"
                    | "html"
                    | "xml"
                    | "sql"
                    | "r"
                    | "m"
                    | "swift"
                    | "kt"
                    | "dart"
            )
        } else {
            false
        }
    }

    fn process_file(&self, path: &Path) -> Result<FileIndex, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let metadata = fs::metadata(path)?;

        let mut word_counts = HashMap::new();
        let mut total_words = 0u32;
        let line_count = content.lines().count() as u32;

        // Tokenize and count words
        for word in content
            .split_whitespace()
            .flat_map(|w| w.split(|c: char| !c.is_alphanumeric() && c != '_'))
            .filter(|w| !w.is_empty() && w.len() > 1)
            .map(|w| w.to_lowercase())
        {
            *word_counts.entry(word).or_insert(0) += 1;
            total_words += 1;
        }

        Ok(FileIndex {
            path: path.to_path_buf(),
            word_counts,
            total_words,
            line_count,
            byte_size: metadata.len(),
            manifold_coords: [0.0; 8], // Will be calculated later
        })
    }

    fn build_global_index(&mut self) {
        self.global_word_counts.clear();
        self.total_words = 0;

        for file_index in &self.files {
            self.total_words += file_index.total_words as u64;
            for (word, count) in &file_index.word_counts {
                *self.global_word_counts.entry(word.clone()).or_insert(0) += count;
            }
        }
    }

    fn calculate_manifold_coordinates(&mut self) {
        let max_depth = self
            .files
            .iter()
            .map(|f| f.path.components().count())
            .max()
            .unwrap_or(1) as f64;

        let max_size = self.files.iter().map(|f| f.byte_size).max().unwrap_or(1) as f64;

        let max_words = self.files.iter().map(|f| f.total_words).max().unwrap_or(1) as f64;

        for file_index in &mut self.files {
            let dims =
                self.calculate_file_manifold_dims(file_index, max_depth, max_size, max_words);
            file_index.manifold_coords = [
                dims.complexity,
                dims.level,
                dims.weight,
                dims.domain,
                dims.entropy,
                dims.connectivity,
                dims.temporal,
                dims.semantic,
            ];
        }

        // Update manifold bounds
        for i in 0..8 {
            let values: Vec<f64> = self.files.iter().map(|f| f.manifold_coords[i]).collect();
            if let (Some(&min), Some(&max)) = (
                values.iter().min_by(|a, b| a.partial_cmp(b).unwrap()),
                values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()),
            ) {
                self.manifold_bounds[i] = (min, max);
            }
        }
    }

    fn calculate_file_manifold_dims(
        &self,
        file_index: &FileIndex,
        max_depth: f64,
        max_size: f64,
        max_words: f64,
    ) -> ManifoldDimensions {
        // Complexity: Based on unique words, file size, and code patterns
        let unique_words = file_index.word_counts.len() as f64;
        let complexity = (unique_words / max_words.max(1.0)
            + file_index.byte_size as f64 / max_size.max(1.0))
            / 2.0;

        // Level: Directory depth normalized
        let level = file_index.path.components().count() as f64 / max_depth;

        // Weight: Based on file size and word count
        let weight = (file_index.byte_size as f64 / max_size.max(1.0)
            + file_index.total_words as f64 / max_words.max(1.0))
            / 2.0;

        // Domain: File type classification
        let domain = self.classify_domain(&file_index.path);

        // Entropy: Information entropy of word distribution
        let entropy = self.calculate_entropy(&file_index.word_counts);

        // Connectivity: How many words are shared with other files
        let connectivity = self.calculate_connectivity(file_index);

        // Temporal: Based on file modification patterns (simplified)
        let temporal = 0.5; // Placeholder - would need git history

        // Semantic: Clustering based on word similarity
        let semantic = self.calculate_semantic_similarity(file_index);

        ManifoldDimensions {
            complexity,
            level,
            weight,
            domain,
            entropy,
            connectivity,
            temporal,
            semantic,
        }
    }

    fn classify_domain(&self, path: &Path) -> f64 {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            match ext {
                "rs" => 0.1,                             // Rust
                "py" => 0.2,                             // Python
                "js" | "ts" => 0.3,                      // JavaScript/TypeScript
                "md" => 0.4,                             // Documentation
                "toml" | "yaml" | "yml" | "json" => 0.5, // Config
                "sh" => 0.6,                             // Scripts
                "lean" => 0.7,                           // Lean4
                "c" | "cpp" | "h" | "hpp" => 0.8,        // C/C++
                _ => 0.9,
            }
        } else {
            1.0
        }
    }

    fn calculate_entropy(&self, word_counts: &HashMap<String, u32>) -> f64 {
        let total: u32 = word_counts.values().sum();
        if total == 0 {
            return 0.0;
        }

        let mut entropy = 0.0;
        for &count in word_counts.values() {
            let p = count as f64 / total as f64;
            if p > 0.0 {
                entropy -= p * p.log2();
            }
        }
        entropy / 10.0 // Normalize roughly
    }

    fn calculate_connectivity(&self, file_index: &FileIndex) -> f64 {
        let mut shared_words = 0;
        let mut total_comparisons = 0;

        for word in file_index.word_counts.keys() {
            if let Some(&global_count) = self.global_word_counts.get(word) {
                if global_count > 1 {
                    // Word appears in multiple files
                    shared_words += 1;
                }
            }
            total_comparisons += 1;
        }

        if total_comparisons > 0 {
            shared_words as f64 / total_comparisons as f64
        } else {
            0.0
        }
    }

    fn calculate_semantic_similarity(&self, file_index: &FileIndex) -> f64 {
        // Simplified semantic similarity based on common programming terms
        let semantic_keywords = [
            "function", "struct", "impl", "trait", "enum", "mod", "use", "pub", "fn", "let", "mut",
            "const", "static", "async", "await", "match", "if", "else", "for", "while", "loop",
            "return", "break", "continue",
        ];

        let semantic_count = file_index
            .word_counts
            .keys()
            .filter(|word| semantic_keywords.contains(&word.as_str()))
            .count();

        semantic_count as f64 / semantic_keywords.len() as f64
    }

    pub fn save_compressed(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let serialized = bincode::serialize(self)?;
        let file = fs::File::create(path)?;
        let mut encoder = GzEncoder::new(file, Compression::best());
        encoder.write_all(&serialized)?;
        encoder.finish()?;
        Ok(())
    }

    pub fn load_compressed(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let file = fs::File::open(path)?;
        let mut decoder = GzDecoder::new(file);
        let mut buffer = Vec::new();
        decoder.read_to_end(&mut buffer)?;
        let index = bincode::deserialize(&buffer)?;
        Ok(index)
    }

    pub fn find_duplicates(&self, other: &RepoIndex) -> Vec<(PathBuf, PathBuf, f64)> {
        let mut duplicates = Vec::new();

        for file1 in &self.files {
            for file2 in &other.files {
                let similarity = self.calculate_file_similarity(file1, file2);
                if similarity > 0.8 {
                    // High similarity threshold
                    duplicates.push((file1.path.clone(), file2.path.clone(), similarity));
                }
            }
        }

        duplicates.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
        duplicates
    }

    fn calculate_file_similarity(&self, file1: &FileIndex, file2: &FileIndex) -> f64 {
        // Jaccard similarity for word sets
        let words1: std::collections::HashSet<_> = file1.word_counts.keys().collect();
        let words2: std::collections::HashSet<_> = file2.word_counts.keys().collect();

        let intersection = words1.intersection(&words2).count();
        let union = words1.union(&words2).count();

        if union > 0 {
            intersection as f64 / union as f64
        } else {
            0.0
        }
    }

    pub fn print_stats(&self) {
        println!("\n=== Repository Index Stats ===");
        println!("Repository: {:?}", self.repo_path);
        println!("Total files: {}", self.total_files);
        println!("Total words: {}", self.total_words);
        println!("Unique words: {}", self.global_word_counts.len());

        println!("\nTop 20 most common words:");
        let mut word_vec: Vec<_> = self.global_word_counts.iter().collect();
        word_vec.sort_by(|a, b| b.1.cmp(a.1));
        for (word, count) in word_vec.iter().take(20) {
            println!("  {}: {}", word, count);
        }

        println!("\nManifold bounds:");
        let dims = [
            "complexity",
            "level",
            "weight",
            "domain",
            "entropy",
            "connectivity",
            "temporal",
            "semantic",
        ];
        for (i, (min, max)) in self.manifold_bounds.iter().enumerate() {
            println!("  {}: [{:.3}, {:.3}]", dims[i], min, max);
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <repo1_path> [repo2_path]", args[0]);
        std::process::exit(1);
    }

    // Create index directory
    fs::create_dir_all("~/nix/index")?;

    // Index first repository
    let repo1_path = PathBuf::from(&args[1]);
    let mut index1 = RepoIndex::new(repo1_path.clone());
    index1.scan_repository()?;

    let index1_file = Path::new("~/nix/index").join(format!(
        "{}_index.gz",
        repo1_path.file_name().unwrap().to_string_lossy()
    ));
    index1.save_compressed(&index1_file)?;
    index1.print_stats();

    // Index second repository if provided
    if args.len() > 2 {
        let repo2_path = PathBuf::from(&args[2]);
        let mut index2 = RepoIndex::new(repo2_path.clone());
        index2.scan_repository()?;

        let index2_file = Path::new("~/nix/index").join(format!(
            "{}_index.gz",
            repo2_path.file_name().unwrap().to_string_lossy()
        ));
        index2.save_compressed(&index2_file)?;
        index2.print_stats();

        // Find duplicates
        println!("\n=== Duplicate Analysis ===");
        let duplicates = index1.find_duplicates(&index2);
        println!("Found {} potential duplicates:", duplicates.len());

        for (path1, path2, similarity) in duplicates.iter().take(10) {
            println!("  {:.3}: {:?} <-> {:?}", similarity, path1, path2);
        }
    }

    Ok(())
}
