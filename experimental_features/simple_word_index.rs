use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug)]
struct FileTypeStats {
    word_counts: HashMap<String, u32>,
    file_count: u32,
    total_words: u32,
}

impl FileTypeStats {
    fn new() -> Self {
        Self {
            word_counts: HashMap::new(),
            file_count: 0,
            total_words: 0,
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <repo1_path> <repo2_path>", args[0]);
        std::process::exit(1);
    }

    let repo1 = &args[1];
    let repo2 = &args[2];

    println!("Scanning {} and {}", repo1, repo2);

    let stats1 = scan_repo(repo1);
    let stats2 = scan_repo(repo2);

    println!("\n=== REPO 1: {} ===", repo1);
    print_repo_stats(&stats1);

    println!("\n=== REPO 2: {} ===", repo2);
    print_repo_stats(&stats2);

    println!("\n=== COMPARISON ===");
    compare_repos(&stats1, &stats2);
}

fn scan_repo(path: &str) -> HashMap<String, FileTypeStats> {
    let mut stats = HashMap::new();
    scan_dir(Path::new(path), &mut stats);
    stats
}

fn scan_dir(dir: &Path, stats: &mut HashMap<String, FileTypeStats>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().unwrap().to_string_lossy();
                if !matches!(name.as_ref(), ".git" | "target" | "node_modules" | ".lake" | "build") {
                    scan_dir(&path, stats);
                }
            } else {
                let file_type = get_file_type(&path);
                scan_file(&path, stats.entry(file_type).or_insert_with(FileTypeStats::new));
            }
        }
    }
}

fn get_file_type(path: &Path) -> String {
    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        match name {
            "Cargo.toml" => "cargo_toml".to_string(),
            "Cargo.lock" => "cargo_lock".to_string(),
            _ => {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    match ext {
                        "rs" => "rust".to_string(),
                        "py" => "python".to_string(),
                        "js" => "javascript".to_string(),
                        "ts" => "typescript".to_string(),
                        "md" => "markdown".to_string(),
                        "txt" => "text".to_string(),
                        "toml" => "toml".to_string(),
                        "yaml" | "yml" => "yaml".to_string(),
                        "json" => "json".to_string(),
                        "sh" => "shell".to_string(),
                        "lean" => "lean".to_string(),
                        _ => format!("other_{}", ext),
                    }
                } else {
                    "no_extension".to_string()
                }
            }
        }
    } else {
        "unknown".to_string()
    }
}

fn scan_file(path: &Path, stats: &mut FileTypeStats) {
    if let Ok(content) = fs::read_to_string(path) {
        stats.file_count += 1;
        for word in content
            .split_whitespace()
            .flat_map(|w| w.split(|c: char| !c.is_alphanumeric() && c != '_'))
            .filter(|w| !w.is_empty() && w.len() > 2)
            .map(|w| w.to_lowercase())
        {
            *stats.word_counts.entry(word).or_insert(0) += 1;
            stats.total_words += 1;
        }
    }
}

fn print_repo_stats(stats: &HashMap<String, FileTypeStats>) {
    let mut file_types: Vec<_> = stats.iter().collect();
    file_types.sort_by(|a, b| b.1.total_words.cmp(&a.1.total_words));

    for (file_type, stat) in file_types {
        println!("\n{}: {} files, {} words, {} unique",
                 file_type, stat.file_count, stat.total_words, stat.word_counts.len());

        let mut top_words: Vec<_> = stat.word_counts.iter().collect();
        top_words.sort_by(|a, b| b.1.cmp(a.1));

        print!("  Top words: ");
        for (word, count) in top_words.iter().take(10) {
            print!("{}({}) ", word, count);
        }
        println!();
    }
}

fn compare_repos(stats1: &HashMap<String, FileTypeStats>, stats2: &HashMap<String, FileTypeStats>) {
    let mut all_types: std::collections::HashSet<String> = stats1.keys().cloned().collect();
    all_types.extend(stats2.keys().cloned());

    for file_type in all_types {
        let stat1 = stats1.get(&file_type);
        let stat2 = stats2.get(&file_type);

        match (stat1, stat2) {
            (Some(s1), Some(s2)) => {
                println!("\n{}: {} vs {} files, {} vs {} words",
                         file_type, s1.file_count, s2.file_count, s1.total_words, s2.total_words);

                let common_words = find_common_words(&s1.word_counts, &s2.word_counts);
                if !common_words.is_empty() {
                    print!("  Common: ");
                    for (word, c1, c2) in common_words.iter().take(5) {
                        print!("{}({},{}) ", word, c1, c2);
                    }
                    println!();
                }
            }
            (Some(s1), None) => println!("{}: only in repo1 ({} files)", file_type, s1.file_count),
            (None, Some(s2)) => println!("{}: only in repo2 ({} files)", file_type, s2.file_count),
            (None, None) => unreachable!(),
        }
    }
}

fn find_common_words(words1: &HashMap<String, u32>, words2: &HashMap<String, u32>) -> Vec<(String, u32, u32)> {
    let mut common: Vec<_> = words1
        .iter()
        .filter_map(|(word, count1)| {
            words2.get(word).map(|count2| (word.clone(), *count1, *count2))
        })
        .collect();

    common.sort_by(|a, b| (a.1 + a.2).cmp(&(b.1 + b.2)).reverse());
    common
}
