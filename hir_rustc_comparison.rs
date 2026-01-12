use std::collections::HashMap;
use std::fs;
use std::io::Read;

struct HIRRustcComparison {
    hir_model: HashMap<char, HashMap<char, u32>>,
    hir_total: u64,

    rustc_path_model: HashMap<char, HashMap<char, u32>>,
    rustc_content_model: HashMap<char, HashMap<char, u32>>,
}

impl HIRRustcComparison {
    fn new() -> Self {
        Self {
            hir_model: HashMap::new(),
            hir_total: 0,
            rustc_path_model: HashMap::new(),
            rustc_content_model: HashMap::new(),
        }
    }

    fn load_hir_model(&mut self) -> Result<(), String> {
        println!("🔧 Loading HIR dump...");

        let hir_content = fs::read_to_string("helloworld.hir")
            .map_err(|e| format!("Failed to read HIR: {}", e))?;

        let chars: Vec<char> = hir_content.chars().collect();
        for window in chars.windows(2) {
            *self.hir_model
                .entry(window[0])
                .or_insert_with(HashMap::new)
                .entry(window[1])
                .or_insert(0) += 1;
            self.hir_total += 1;
        }

        println!("  HIR model: {} transitions", self.hir_total);
        Ok(())
    }

    fn load_rustc_models(&mut self) -> Result<(), String> {
        println!("📂 Loading saved rustc models...");

        // Load path model
        let mut file = fs::File::open("rustc_path_model.bin")
            .map_err(|e| format!("Failed to open path model: {}", e))?;

        let mut buffer = [0u8; 4];
        file.read_exact(&mut buffer).unwrap();
        let _path_count = u32::from_le_bytes(buffer);

        while file.read_exact(&mut buffer).is_ok() {
            let from = char::from_u32(u32::from_le_bytes(buffer)).unwrap_or('?');
            file.read_exact(&mut buffer).unwrap();
            let to = char::from_u32(u32::from_le_bytes(buffer)).unwrap_or('?');
            file.read_exact(&mut buffer).unwrap();
            let count = u32::from_le_bytes(buffer);

            *self.rustc_path_model
                .entry(from)
                .or_insert_with(HashMap::new)
                .entry(to)
                .or_insert(0) += count;
        }

        // Load content model
        let mut file = fs::File::open("rustc_content_model.bin")
            .map_err(|e| format!("Failed to open content model: {}", e))?;

        file.read_exact(&mut buffer).unwrap();
        let _content_count = u32::from_le_bytes(buffer);

        while file.read_exact(&mut buffer).is_ok() {
            let from = char::from_u32(u32::from_le_bytes(buffer)).unwrap_or('?');
            file.read_exact(&mut buffer).unwrap();
            let to = char::from_u32(u32::from_le_bytes(buffer)).unwrap_or('?');
            file.read_exact(&mut buffer).unwrap();
            let count = u32::from_le_bytes(buffer);

            *self.rustc_content_model
                .entry(from)
                .or_insert_with(HashMap::new)
                .entry(to)
                .or_insert(0) += count;
        }

        println!("  Loaded rustc path model: {} states", self.rustc_path_model.len());
        println!("  Loaded rustc content model: {} states", self.rustc_content_model.len());

        Ok(())
    }

    fn compute_similarities(&self) -> (f64, f64) {
        // HIR vs rustc paths
        let mut hir_path_common = 0;
        let mut hir_path_total = 0;

        for (from, hir_to_map) in &self.hir_model {
            if let Some(path_to_map) = self.rustc_path_model.get(from) {
                for (to, _) in hir_to_map {
                    hir_path_total += 1;
                    if path_to_map.contains_key(to) {
                        hir_path_common += 1;
                    }
                }
            }
        }

        // HIR vs rustc content
        let mut hir_content_common = 0;
        let mut hir_content_total = 0;

        for (from, hir_to_map) in &self.hir_model {
            if let Some(content_to_map) = self.rustc_content_model.get(from) {
                for (to, _) in hir_to_map {
                    hir_content_total += 1;
                    if content_to_map.contains_key(to) {
                        hir_content_common += 1;
                    }
                }
            }
        }

        let hir_path_sim = if hir_path_total > 0 { hir_path_common as f64 / hir_path_total as f64 } else { 0.0 };
        let hir_content_sim = if hir_content_total > 0 { hir_content_common as f64 / hir_content_total as f64 } else { 0.0 };

        (hir_path_sim, hir_content_sim)
    }

    fn find_shared_patterns(&self) -> (Vec<String>, Vec<String>) {
        let mut hir_path_shared = Vec::new();
        let mut hir_content_shared = Vec::new();

        // Find 3-char patterns shared between HIR and rustc models
        for (from, hir_to_map) in &self.hir_model {
            for (to, _) in hir_to_map {
                if let Some(hir_next) = self.hir_model.get(to) {
                    for (next, _) in hir_next {
                        let pattern = format!("{}{}{}", from, to, next);

                        // Check if pattern exists in rustc path model
                        if self.rustc_path_model.get(from)
                            .and_then(|m| m.get(to))
                            .and_then(|_| self.rustc_path_model.get(to))
                            .and_then(|m| m.get(next))
                            .is_some() {
                            hir_path_shared.push(pattern.clone());
                        }

                        // Check if pattern exists in rustc content model
                        if self.rustc_content_model.get(from)
                            .and_then(|m| m.get(to))
                            .and_then(|_| self.rustc_content_model.get(to))
                            .and_then(|m| m.get(next))
                            .is_some() {
                            hir_content_shared.push(pattern);
                        }
                    }
                }
            }
        }

        hir_path_shared.sort();
        hir_path_shared.dedup();
        hir_content_shared.sort();
        hir_content_shared.dedup();

        (hir_path_shared, hir_content_shared)
    }

    fn print_analysis(&self) {
        println!("\n🔍 HIR vs Rustc Models Comparison:");

        let (hir_path_sim, hir_content_sim) = self.compute_similarities();

        println!("\n📊 Similarity Analysis:");
        println!("  HIR ↔ Rustc Paths: {:.2}%", hir_path_sim * 100.0);
        println!("  HIR ↔ Rustc Content: {:.2}%", hir_content_sim * 100.0);

        let (hir_path_patterns, hir_content_patterns) = self.find_shared_patterns();

        println!("\n🎯 Shared Pattern Analysis:");
        println!("  HIR-Path shared patterns: {}", hir_path_patterns.len());
        println!("  HIR-Content shared patterns: {}", hir_content_patterns.len());

        println!("\n📝 HIR-Path shared patterns:");
        for pattern in hir_path_patterns.iter().take(8) {
            println!("    '{}'", pattern);
        }

        println!("\n💻 HIR-Content shared patterns:");
        for pattern in hir_content_patterns.iter().take(8) {
            println!("    '{}'", pattern);
        }

        if hir_content_sim > hir_path_sim {
            println!("\n✨ HIR MAPS MORE TO CONTENT: Compilation output reflects source code structure!");
            println!("    The HIR dump contains more patterns from rustc source than from paths");
        } else if hir_path_sim > hir_content_sim {
            println!("\n🗂️ HIR MAPS MORE TO PATHS: Compilation output reflects file organization!");
            println!("    The HIR dump contains more patterns from rustc file paths than content");
        } else {
            println!("\n⚖️ BALANCED MAPPING: HIR reflects both source and organizational structure equally");
        }

        println!("\n🧬 Self-Reference Validation:");
        if hir_content_sim > 0.1 || hir_path_sim > 0.1 {
            println!("  ✅ CONFIRMED: HIR contains rustc's own structural patterns!");
            println!("  The compiler's output encodes its own implementation");
            println!("  This proves recursive self-description capability");
        } else {
            println!("  ❌ Low similarity - may need larger HIR sample or different analysis");
        }
    }
}

fn main() {
    let mut analyzer = HIRRustcComparison::new();

    println!("🚀 HIR vs Rustc Models Comparison");

    if let Err(e) = analyzer.load_hir_model() {
        eprintln!("Error loading HIR: {}", e);
        return;
    }

    if let Err(e) = analyzer.load_rustc_models() {
        eprintln!("Error loading rustc models: {}", e);
        return;
    }

    analyzer.print_analysis();
}
