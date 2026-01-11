use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct KleeneMemoryNode {
    pub frequency_band: (f64, f64),
    pub resonance_level: usize,
    pub files: Vec<String>,
    pub memory_region: usize,
}

pub struct KleeneMemoryHierarchy {
    pub total_memory_gb: usize,
    pub nodes: Vec<Arc<Mutex<KleeneMemoryNode>>>,
    pub frequency_map: HashMap<String, usize>,
}

impl KleeneMemoryHierarchy {
    pub fn new() -> Self {
        let total_memory_gb = 29; // Available memory from free -h

        // Create Kleene hierarchy: 2^0, 2^1, 2^2, 2^3, 2^4 levels
        let mut nodes = Vec::new();
        let memory_per_node = total_memory_gb / 5;

        for level in 0..5 {
            let frequency = 2.0_f64.powi(level);
            let band_low = frequency * 0.75;
            let band_high = frequency * 1.25;

            nodes.push(Arc::new(Mutex::new(KleeneMemoryNode {
                frequency_band: (band_low, band_high),
                resonance_level: level as usize,
                files: Vec::with_capacity(300_000), // ~300k files per node
                memory_region: memory_per_node,
            })));
        }

        Self {
            total_memory_gb,
            nodes,
            frequency_map: HashMap::new(),
        }
    }

    pub fn calculate_resonance(&self, file_path: &str, kleene_score: f64) -> usize {
        // Find which memory node resonates with this file's frequency
        for (i, node) in self.nodes.iter().enumerate() {
            let node_guard = node.lock().unwrap();
            if kleene_score >= node_guard.frequency_band.0
                && kleene_score < node_guard.frequency_band.1
            {
                return i;
            }
        }
        0 // Default to lowest frequency node
    }

    pub fn allocate_file(&mut self, file_path: String, kleene_score: f64) {
        let node_idx = self.calculate_resonance(&file_path, kleene_score);

        if let Some(node) = self.nodes.get(node_idx) {
            let mut node_guard = node.lock().unwrap();
            node_guard.files.push(file_path.clone());
            self.frequency_map.insert(file_path, node_idx);
        }
    }

    pub fn get_memory_distribution(&self) -> Vec<(usize, usize, f64)> {
        self.nodes
            .iter()
            .enumerate()
            .map(|(i, node)| {
                let node_guard = node.lock().unwrap();
                (i, node_guard.files.len(), node_guard.frequency_band.0)
            })
            .collect()
    }

    pub fn optimize_memory_layout(&mut self) {
        println!(
            "🧠 Optimizing Kleene memory hierarchy across {}GB RAM",
            self.total_memory_gb
        );

        // Rebalance nodes based on actual file distribution
        let distribution = self.get_memory_distribution();

        for (node_idx, file_count, frequency) in distribution {
            let memory_usage_gb = (file_count * 1024) / (1024 * 1024 * 1024); // Rough estimate
            println!(
                "   Node {}: {} files, {:.1} frequency, ~{}GB",
                node_idx, file_count, frequency, memory_usage_gb
            );
        }
    }
}
