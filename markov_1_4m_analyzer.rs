use rayon::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
struct MarkovFingerprint {
    file_path: String,
    char_transitions: HashMap<(char, char), f64>,
    entropy: f64,
    signature: u64,
}

fn analyze_file_markov(path: &str) -> Option<MarkovFingerprint> {
    let content = fs::read_to_string(path).ok()?;
    let chars: Vec<char> = content.chars().collect();

    let mut transitions = HashMap::new();
    let mut total = 0u64;

    for window in chars.windows(2) {
        let pair = (window[0], window[1]);
        *transitions.entry(pair).or_insert(0u64) += 1;
        total += 1;
    }

    let char_transitions: HashMap<(char, char), f64> = transitions
        .into_iter()
        .map(|(k, v)| (k, v as f64 / total as f64))
        .collect();

    let entropy = -char_transitions.values().map(|p| p * p.log2()).sum::<f64>();

    let signature = char_transitions
        .keys()
        .map(|(a, b)| (*a as u64) ^ (*b as u64))
        .fold(0, |acc, x| acc ^ x);

    Some(MarkovFingerprint {
        file_path: path.to_string(),
        char_transitions,
        entropy,
        signature,
    })
}

fn main() {
    println!("🧬 Markov Chain Analysis on 1.4M Rust Files");
    println!("=".repeat(50));

    // Find all Rust files
    let rust_files: Vec<String> = walkdir::WalkDir::new("/")
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
        .map(|e| e.path().to_string_lossy().to_string())
        .take(1_400_000)
        .collect();

    println!("📊 Found {} Rust files", rust_files.len());

    // Parallel Markov analysis
    let fingerprints: Vec<MarkovFingerprint> = rust_files
        .par_iter()
        .filter_map(|path| analyze_file_markov(path))
        .collect();

    println!("🔬 Generated {} fingerprints", fingerprints.len());

    // Find similarities
    let mut similarity_groups = HashMap::new();
    for fp in &fingerprints {
        similarity_groups
            .entry(fp.signature)
            .or_insert(Vec::new())
            .push(fp);
    }

    println!("🎯 Similarity groups: {}", similarity_groups.len());

    // Show conformal folding
    for (sig, group) in similarity_groups.iter().take(5) {
        if group.len() > 1 {
            println!("📁 Group {}: {} files", sig, group.len());
            println!(
                "   Entropy range: {:.3}-{:.3}",
                group
                    .iter()
                    .map(|f| f.entropy)
                    .fold(f64::INFINITY, f64::min),
                group
                    .iter()
                    .map(|f| f.entropy)
                    .fold(f64::NEG_INFINITY, f64::max)
            );
        }
    }
}
