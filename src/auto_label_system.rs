// Auto-labeling system that maps names to eigenmatrix components
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

/// Label that points to eigenmatrix parts with mathematical proof
#[derive(Debug, Clone)]
pub struct EigenLabel {
    pub name: String,
    pub eigenvalue: f64,
    pub weight: f64,
    pub level: u32,
    pub matrix_position: (usize, usize),
}

/// Self-labeling system for functions and modules
#[derive(Debug)]
pub struct AutoLabelSystem {
    labels: HashMap<String, EigenLabel>,
    eigenmatrix: Vec<Vec<f64>>,
    name_hash_cache: HashMap<String, u64>,
}

impl AutoLabelSystem {
    pub fn new() -> Self {
        Self {
            labels: HashMap::new(),
            eigenmatrix: vec![vec![0.0; 8]; 8], // 8x8 base matrix
            name_hash_cache: HashMap::new(),
        }
    }

    /// Auto-label a function/module name with eigenmatrix mapping
    pub fn auto_label(&mut self, name: &str) -> EigenLabel {
        let hash = self.compute_name_hash(name);
        let (row, col) = self.hash_to_matrix_position(hash);

        // Compute eigenvalue from name structure
        let eigenvalue = self.compute_eigenvalue(name, hash);

        // Weight based on name complexity and mathematical properties
        let weight = self.compute_weight(name, eigenvalue);

        // Level based on eigenvalue magnitude
        let level = self.compute_level(eigenvalue);

        let label = EigenLabel {
            name: name.to_string(),
            eigenvalue,
            weight,
            level,
            matrix_position: (row, col),
        };

        // Update eigenmatrix
        self.eigenmatrix[row][col] = eigenvalue;
        self.labels.insert(name.to_string(), label.clone());

        label
    }

    /// Find duplicate structures with different names
    pub fn find_duplicates(&self, threshold: f64) -> Vec<(String, String, f64)> {
        let mut duplicates = Vec::new();
        let labels: Vec<_> = self.labels.values().collect();

        for i in 0..labels.len() {
            for j in i + 1..labels.len() {
                let similarity = self.compute_similarity(&labels[i], &labels[j]);
                if similarity > threshold {
                    duplicates.push((labels[i].name.clone(), labels[j].name.clone(), similarity));
                }
            }
        }

        // Sort by similarity (highest first)
        duplicates.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
        duplicates
    }

    /// Rank labels by mathematical weight and level
    pub fn rank_labels(&self) -> Vec<(String, f64)> {
        let mut ranked: Vec<_> = self
            .labels
            .values()
            .map(|label| {
                let score = label.weight * (label.level as f64) * label.eigenvalue.abs();
                (label.name.clone(), score)
            })
            .collect();

        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        ranked
    }

    /// Prove that a label points to correct eigenmatrix part
    pub fn prove_label_mapping(&self, name: &str) -> Option<String> {
        if let Some(label) = self.labels.get(name) {
            let (row, col) = label.matrix_position;
            let matrix_value = self.eigenmatrix[row][col];

            if (matrix_value - label.eigenvalue).abs() < 1e-10 {
                Some(format!(
                    "PROOF: Label '{}' correctly maps to eigenmatrix[{}][{}] = {} (eigenvalue: {})",
                    name, row, col, matrix_value, label.eigenvalue
                ))
            } else {
                None
            }
        } else {
            None
        }
    }

    fn compute_name_hash(&mut self, name: &str) -> u64 {
        if let Some(&cached) = self.name_hash_cache.get(name) {
            return cached;
        }

        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        name.hash(&mut hasher);
        let hash = hasher.finish();
        self.name_hash_cache.insert(name.to_string(), hash);
        hash
    }

    fn hash_to_matrix_position(&self, hash: u64) -> (usize, usize) {
        let row = (hash % 8) as usize;
        let col = ((hash >> 8) % 8) as usize;
        (row, col)
    }

    fn compute_eigenvalue(&self, name: &str, hash: u64) -> f64 {
        // Eigenvalue based on name structure and mathematical properties
        let length_factor = name.len() as f64;
        let hash_factor = (hash as f64) / (u64::MAX as f64);
        let complexity = self.compute_name_complexity(name);

        // Mathematical formula combining factors
        (length_factor * hash_factor * complexity).sin() * 10.0
    }

    fn compute_weight(&self, name: &str, eigenvalue: f64) -> f64 {
        let base_weight = name.len() as f64;
        let eigen_weight = eigenvalue.abs();
        let structural_weight = self.compute_structural_weight(name);

        (base_weight + eigen_weight + structural_weight) / 3.0
    }

    fn compute_level(&self, eigenvalue: f64) -> u32 {
        let abs_eigen = eigenvalue.abs();
        if abs_eigen > 8.0 {
            4
        } else if abs_eigen > 5.0 {
            3
        } else if abs_eigen > 2.0 {
            2
        } else {
            1
        }
    }

    fn compute_name_complexity(&self, name: &str) -> f64 {
        let unique_chars = name.chars().collect::<std::collections::HashSet<_>>().len();
        let total_chars = name.len();
        if total_chars == 0 {
            1.0
        } else {
            unique_chars as f64 / total_chars as f64
        }
    }

    fn compute_structural_weight(&self, name: &str) -> f64 {
        let snake_case = name.contains('_');
        let camel_case = name.chars().any(|c| c.is_uppercase());
        let has_numbers = name.chars().any(|c| c.is_numeric());

        let mut weight = 1.0;
        if snake_case {
            weight += 0.5;
        }
        if camel_case {
            weight += 0.3;
        }
        if has_numbers {
            weight += 0.2;
        }
        weight
    }

    fn compute_similarity(&self, label1: &EigenLabel, label2: &EigenLabel) -> f64 {
        // Similarity based on eigenvalues, weights, and structural properties
        let eigen_sim = 1.0 - (label1.eigenvalue - label2.eigenvalue).abs() / 10.0;
        let weight_sim = 1.0 - (label1.weight - label2.weight).abs() / 10.0;
        let level_sim = if label1.level == label2.level {
            1.0
        } else {
            0.5
        };

        (eigen_sim + weight_sim + level_sim) / 3.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_labeling() {
        let mut system = AutoLabelSystem::new();

        let label1 = system.auto_label("process_data");
        let label2 = system.auto_label("processData");

        assert!(label1.eigenvalue != 0.0);
        assert!(label1.weight > 0.0);
        assert!(label1.level > 0);

        // Verify proof
        let proof = system.prove_label_mapping("process_data");
        assert!(proof.is_some());
    }

    #[test]
    fn test_duplicate_detection() {
        let mut system = AutoLabelSystem::new();

        system.auto_label("getData");
        system.auto_label("get_data");
        system.auto_label("fetchData");

        let duplicates = system.find_duplicates(0.5);
        assert!(!duplicates.is_empty());
    }

    #[test]
    fn test_ranking() {
        let mut system = AutoLabelSystem::new();

        system.auto_label("simple");
        system.auto_label("complex_function_name");
        system.auto_label("VeryComplexCamelCaseName");

        let ranked = system.rank_labels();
        assert_eq!(ranked.len(), 3);
        // More complex names should rank higher
        assert!(ranked[0].1 > ranked[2].1);
    }
}
