use std::collections::{HashMap, HashSet};

fn main() {
    println!("Loading repository graph into memory...");

    let mut graph = RepoGraph::new();

    // Load GitHub data
    if let Ok(github_data) = std::fs::read_to_string("/mnt/data1/nix/index/github_meta-introspector_repos.json") {
        match graph.load_from_github_data(&github_data) {
            Ok(_) => println!("✓ Loaded GitHub data"),
            Err(e) => println!("✗ Error loading GitHub data: {}", e),
        }
    }

    // Load starred data
    if let Ok(starred_data) = std::fs::read_to_string("/mnt/data1/nix/index/starred.json") {
        match graph.load_from_github_data(&starred_data) {
            Ok(_) => println!("✓ Loaded starred data"),
            Err(e) => println!("✗ Error loading starred data: {}", e),
        }
    }

    println!("\nGraph loaded successfully!");
    println!("Nodes: {}", graph.id_to_label.len());
    println!("Edges: {}", graph.adjacency.values().map(|s| s.len()).sum::<usize>());
    println!("Memory: {:.1} MB", graph.memory_usage() as f64 / 1024.0 / 1024.0);
}

struct RepoGraph {
    label_to_id: HashMap<String, u32>,
    id_to_label: Vec<String>,
    adjacency: HashMap<u32, HashSet<u32>>,
    weights: Vec<f32>,
    next_id: u32,
}

impl RepoGraph {
    fn new() -> Self {
        Self {
            label_to_id: HashMap::new(),
            id_to_label: Vec::new(),
            adjacency: HashMap::new(),
            weights: Vec::new(),
            next_id: 0,
        }
    }

    fn get_or_create_id(&mut self, label: &str) -> u32 {
        if let Some(&id) = self.label_to_id.get(label) {
            return id;
        }

        let id = self.next_id;
        self.next_id += 1;

        self.label_to_id.insert(label.to_string(), id);
        self.id_to_label.push(label.to_string());
        self.weights.push(0.0);

        id
    }

    fn add_edge(&mut self, from: &str, to: &str, weight: f32) {
        let from_id = self.get_or_create_id(from);
        let to_id = self.get_or_create_id(to);

        self.adjacency.entry(from_id).or_default().insert(to_id);
        self.weights[from_id as usize] += weight;
    }

    fn load_from_github_data(&mut self, json_data: &str) -> Result<(), Box<dyn std::error::Error>> {
        let data: serde_json::Value = serde_json::from_str(json_data)?;

        if let Some(repos) = data.as_array() {
            for repo in repos {
                if let Some(full_name) = repo["full_name"].as_str() {
                    let stars = repo["stargazers_count"].as_f64().unwrap_or(0.0) as f32;

                    let repo_id = self.get_or_create_id(full_name);
                    self.weights[repo_id as usize] = stars;

                    if let Some(language) = repo["language"].as_str() {
                        let lang_label = format!("lang:{}", language);
                        self.add_edge(&lang_label, full_name, 1.0);
                    }

                    if let Some(owner) = repo["owner"]["login"].as_str() {
                        let owner_label = format!("user:{}", owner);
                        self.add_edge(&owner_label, full_name, 1.0);
                    }

                    if let Some(parent) = repo["parent"]["full_name"].as_str() {
                        self.add_edge(parent, full_name, 2.0);
                    }
                }
            }
        }

        Ok(())
    }

    fn memory_usage(&self) -> usize {
        self.label_to_id.len() * 64 +
        self.adjacency.values().map(|set| set.len() * 4).sum::<usize>() +
        self.weights.len() * 4
    }
}
