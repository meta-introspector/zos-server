use std::fs;
use std::io::Read;
use std::collections::HashMap;

struct ModelSimilarityAnalyzer {
    models: Vec<ModelProfile>,
}

#[derive(Debug, Clone)]
struct ModelProfile {
    filename: String,
    transition_signature: HashMap<(char, char), u32>,
    total_transitions: u32,
}

impl ModelSimilarityAnalyzer {
    fn new() -> Self {
        Self { models: Vec::new() }
    }

    fn load_model_profiles(&mut self) -> Result<(), String> {
        println!("🔍 Loading model profiles...");

        let entries = fs::read_dir(".")
            .map_err(|e| format!("Cannot read directory: {}", e))?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "bin") {
                let filename = path.file_name().unwrap().to_string_lossy().to_string();

                if let Ok(profile) = self.create_model_profile(&filename) {
                    self.models.push(profile);

                    if self.models.len() % 1000 == 0 {
                        print!("\r  Loaded {} models", self.models.len());
                    }
                }
            }
        }

        println!("\n✅ Loaded {} model profiles", self.models.len());
        Ok(())
    }

    fn create_model_profile(&self, filename: &str) -> Result<ModelProfile, String> {
        let mut file = fs::File::open(filename)
            .map_err(|e| format!("Cannot open {}: {}", filename, e))?;

        let mut buffer = [0u8; 4];
        file.read_exact(&mut buffer).map_err(|_| "Cannot read count")?;
        let total_transitions = u32::from_le_bytes(buffer);

        let mut signature = HashMap::new();

        // Read up to 100 transitions to create signature
        for _ in 0..std::cmp::min(total_transitions, 100) {
            if file.read_exact(&mut buffer).is_ok() {
                let from = u32::from_le_bytes(buffer);
                if file.read_exact(&mut buffer).is_ok() {
                    let to = u32::from_le_bytes(buffer);
                    if file.read_exact(&mut buffer).is_ok() {
                        let count = u32::from_le_bytes(buffer);

                        if from <= 127 && to <= 127 {
                            let from_char = char::from(from as u8);
                            let to_char = char::from(to as u8);
                            signature.insert((from_char, to_char), count);
                        }
                    }
                }
            }
        }

        Ok(ModelProfile {
            filename: filename.to_string(),
            transition_signature: signature,
            total_transitions,
        })
    }

    fn compute_similarity(&self, model1: &ModelProfile, model2: &ModelProfile) -> f64 {
        let mut common_transitions = 0;
        let mut total_unique_transitions = 0;

        // Get all unique transitions from both models
        let mut all_transitions = std::collections::HashSet::new();
        for key in model1.transition_signature.keys() {
            all_transitions.insert(*key);
        }
        for key in model2.transition_signature.keys() {
            all_transitions.insert(*key);
        }

        total_unique_transitions = all_transitions.len();

        // Count common transitions
        for transition in &all_transitions {
            if model1.transition_signature.contains_key(transition) &&
               model2.transition_signature.contains_key(transition) {
                common_transitions += 1;
            }
        }

        if total_unique_transitions > 0 {
            common_transitions as f64 / total_unique_transitions as f64
        } else {
            0.0
        }
    }

    fn find_most_similar_pairs(&self) -> Vec<(String, String, f64)> {
        println!("🔍 Computing similarity matrix...");
        let mut similarities = Vec::new();

        // Compare first 100 models to prevent explosion
        let sample_size = std::cmp::min(self.models.len(), 100);

        for i in 0..sample_size {
            for j in i+1..sample_size {
                let similarity = self.compute_similarity(&self.models[i], &self.models[j]);
                if similarity > 0.1 { // Only keep significant similarities
                    similarities.push((
                        self.models[i].filename.clone(),
                        self.models[j].filename.clone(),
                        similarity
                    ));
                }
            }

            if i % 10 == 0 {
                print!("\r  Compared {} models", i);
            }
        }

        similarities.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
        println!("\n✅ Found {} similar pairs", similarities.len());

        similarities
    }

    fn find_model_clusters(&self) -> HashMap<String, Vec<String>> {
        println!("🔍 Finding model clusters...");
        let mut clusters: HashMap<String, Vec<String>> = HashMap::new();

        // Simple clustering based on filename patterns
        for model in &self.models {
            let cluster_key = if model.filename.contains("forward") {
                "forward_models".to_string()
            } else if model.filename.contains("reverse") {
                "reverse_models".to_string()
            } else if model.filename.contains("rustc") {
                "compiler_models".to_string()
            } else if model.filename.chars().all(|c| c.is_numeric() || c == '_' || c == '.') {
                "numeric_models".to_string()
            } else {
                "other_models".to_string()
            };

            clusters.entry(cluster_key).or_insert_with(Vec::new).push(model.filename.clone());
        }

        clusters
    }

    fn analyze_cluster_similarities(&self, clusters: &HashMap<String, Vec<String>>) {
        println!("\n🔍 Cluster Similarity Analysis:");

        for (cluster_name, filenames) in clusters {
            if filenames.len() > 1 {
                println!("  📂 {}: {} models", cluster_name, filenames.len());

                // Sample similarity within cluster
                if filenames.len() >= 2 {
                    let model1 = self.models.iter().find(|m| m.filename == filenames[0]);
                    let model2 = self.models.iter().find(|m| m.filename == filenames[1]);

                    if let (Some(m1), Some(m2)) = (model1, model2) {
                        let similarity = self.compute_similarity(m1, m2);
                        println!("    Sample intra-cluster similarity: {:.3}", similarity);
                    }
                }
            }
        }
    }

    fn print_similarity_analysis(&self) {
        let similar_pairs = self.find_most_similar_pairs();

        println!("\n🎯 Top 10 Most Similar Model Pairs:");
        for (i, (model1, model2, similarity)) in similar_pairs.iter().take(10).enumerate() {
            println!("  {}. {} ↔ {}: {:.3} similarity",
                i + 1,
                &model1[..std::cmp::min(20, model1.len())],
                &model2[..std::cmp::min(20, model2.len())],
                similarity
            );
        }

        let clusters = self.find_model_clusters();
        self.analyze_cluster_similarities(&clusters);

        println!("\n📊 Similarity Statistics:");
        println!("  High similarity pairs (>0.8): {}",
            similar_pairs.iter().filter(|(_, _, s)| *s > 0.8).count());
        println!("  Medium similarity pairs (0.5-0.8): {}",
            similar_pairs.iter().filter(|(_, _, s)| *s > 0.5 && *s <= 0.8).count());
        println!("  Low similarity pairs (0.1-0.5): {}",
            similar_pairs.iter().filter(|(_, _, s)| *s > 0.1 && *s <= 0.5).count());

        if let Some((_, _, max_sim)) = similar_pairs.first() {
            println!("  Maximum similarity found: {:.3}", max_sim);
        }
    }
}

fn main() {
    let mut analyzer = ModelSimilarityAnalyzer::new();

    println!("🚀 Model Similarity Analysis");

    if let Err(e) = analyzer.load_model_profiles() {
        eprintln!("Loading failed: {}", e);
        return;
    }

    analyzer.print_similarity_analysis();
}
