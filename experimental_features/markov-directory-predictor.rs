use rand::Rng;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

struct MarkovChain {
    chain: HashMap<String, Vec<String>>,
    order: usize,
}

impl MarkovChain {
    fn new(order: usize) -> Self {
        MarkovChain {
            chain: HashMap::new(),
            order,
        }
    }

    fn train(&mut self, text: &str) {
        let words: Vec<&str> = text.split_whitespace().collect();

        if words.len() <= self.order {
            return;
        }

        for i in 0..words.len() - self.order {
            let state = words[i..i + self.order].join(" ");
            let next = words[i + self.order].to_string();

            self.chain.entry(state).or_insert_with(Vec::new).push(next);
        }
    }

    fn generate(&self, max_words: usize) -> String {
        if self.chain.is_empty() {
            return String::new();
        }

        let mut rng = rand::thread_rng();

        let keys: Vec<_> = self.chain.keys().collect();
        let mut current = keys[rng.gen_range(0..keys.len())].clone();
        let mut result = current.clone();

        for _ in 0..max_words {
            if let Some(next_words) = self.chain.get(&current) {
                let next = &next_words[rng.gen_range(0..next_words.len())];
                result.push(' ');
                result.push_str(next);

                let mut words: Vec<&str> = current.split_whitespace().collect();
                words.remove(0);
                words.push(next);
                current = words.join(" ");
            } else {
                break;
            }
        }

        result
    }
}

struct DirectoryMarkovPredictor {
    filename_chain: MarkovChain,
    path_chain: MarkovChain,
}

impl DirectoryMarkovPredictor {
    fn new() -> Self {
        Self {
            filename_chain: MarkovChain::new(2),
            path_chain: MarkovChain::new(3),
        }
    }

    fn train_on_directory(&mut self, dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut filenames = Vec::new();
        let mut paths = Vec::new();

        self.collect_files(dir, &mut filenames, &mut paths)?;

        // Train on filename sequences
        let filename_text = filenames.join(" ");
        self.filename_chain.train(&filename_text);

        // Train on path components
        let path_text = paths.join(" ");
        self.path_chain.train(&path_text);

        println!(
            "Trained on {} files, {} paths",
            filenames.len(),
            paths.len()
        );
        Ok(())
    }

    fn collect_files(
        &self,
        dir: &Path,
        filenames: &mut Vec<String>,
        paths: &mut Vec<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    filenames.push(name.to_string());
                }
                paths.push(path.to_string_lossy().to_string());
            } else if path.is_dir() {
                self.collect_files(&path, filenames, paths)?;
            }
        }
        Ok(())
    }

    fn predict_next_files(&self, count: usize) -> Vec<String> {
        (0..count)
            .map(|_| self.filename_chain.generate(5))
            .collect()
    }

    fn predict_similar_paths(&self, count: usize) -> Vec<String> {
        (0..count).map(|_| self.path_chain.generate(8)).collect()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧠 MARKOV DIRECTORY PREDICTOR");
    println!("=" * 35);

    let mut predictor = DirectoryMarkovPredictor::new();

    // Train on nix index
    let nix_index = Path::new(&std::env::var("HOME")?).join("nix/index");
    if nix_index.exists() {
        println!("📚 Training on ~/nix/index...");
        predictor.train_on_directory(&nix_index)?;
    }

    // Train on current directory
    println!("📚 Training on current directory...");
    predictor.train_on_directory(&std::env::current_dir()?)?;

    println!("\n🎯 PREDICTED FILENAMES:");
    for (i, prediction) in predictor.predict_next_files(5).iter().enumerate() {
        println!("{}. {}", i + 1, prediction);
    }

    println!("\n🗂️  PREDICTED PATHS:");
    for (i, prediction) in predictor.predict_similar_paths(3).iter().enumerate() {
        println!("{}. {}", i + 1, prediction);
    }

    Ok(())
}
