use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Compressed repository graph with hierarchical numeric labels
#[derive(Debug)]
struct RepoGraph {
    /// String to numeric ID mapping
    label_to_id: HashMap<String, u32>,
    /// Numeric ID to string mapping
    id_to_label: Vec<String>,
    /// Hierarchical categories (lang:rust -> 1000, user:meta-introspector -> 2000, etc)
    category_ranges: HashMap<String, (u32, u32)>,
    /// Compressed adjacency matrix (sparse)
    adjacency: HashMap<u32, HashSet<u32>>,
    /// Node weights (stars, forks, commits)
    weights: Vec<f32>,
    /// Next available ID
    next_id: u32,
}

impl RepoGraph {
    fn new() -> Self {
        Self {
            label_to_id: HashMap::new(),
            id_to_label: Vec::new(),
            category_ranges: HashMap::new(),
            adjacency: HashMap::new(),
            weights: Vec::new(),
            next_id: 0,
        }
    }

    /// Get or create numeric ID for a label with category hierarchy
    fn get_or_create_id(&mut self, label: &str) -> u32 {
        if let Some(&id) = self.label_to_id.get(label) {
            return id;
        }

        // Determine category and assign ID range
        let category = self.categorize_label(label);
        let id = self.allocate_id_for_category(&category);

        self.label_to_id.insert(label.to_string(), id);

        // Extend vectors if needed
        while self.id_to_label.len() <= id as usize {
            self.id_to_label.push(String::new());
            self.weights.push(0.0);
        }

        self.id_to_label[id as usize] = label.to_string();
        id
    }

    fn categorize_label(&self, label: &str) -> String {
        if label.starts_with("lang:") {
            "language".to_string()
        } else if label.starts_with("user:") {
            "user".to_string()
        } else if label.starts_with("org:") {
            "organization".to_string()
        } else if label.contains('/') {
            "repository".to_string()
        } else if ["rustc", "gcc", "bash", "git", "curl", "openssl", "nix"].contains(&label) {
            "core".to_string()
        } else {
            "other".to_string()
        }
    }

    fn allocate_id_for_category(&mut self, category: &str) -> u32 {
        let base_id = match category {
            "core" => 0,              // 0-999: Core infrastructure
            "language" => 1000,       // 1000-9999: Languages
            "user" => 10000,          // 10000-99999: Users
            "organization" => 100000, // 100000-999999: Organizations
            "repository" => 1000000,  // 1000000+: Repositories
            _ => 10000000,            // 10M+: Other
        };

        let range = self
            .category_ranges
            .entry(category.to_string())
            .or_insert((base_id, base_id));

        let id = range.1;
        range.1 += 1;
        id
    }

    /// Add edge between two nodes
    fn add_edge(&mut self, from: &str, to: &str, weight: f32) {
        let from_id = self.get_or_create_id(from);
        let to_id = self.get_or_create_id(to);

        self.adjacency.entry(from_id).or_default().insert(to_id);

        // Update weights
        if (from_id as usize) < self.weights.len() {
            self.weights[from_id as usize] += weight;
        }
        if (to_id as usize) < self.weights.len() {
            self.weights[to_id as usize] += weight;
        }
    }

    /// Load from GitHub JSON data
    fn load_from_github_data(&mut self, json_data: &str) -> Result<(), Box<dyn std::error::Error>> {
        let data: serde_json::Value = serde_json::from_str(json_data)?;

        if let Some(repos) = data.as_array() {
            for repo in repos {
                if let Some(full_name) = repo["full_name"].as_str() {
                    let stars = repo["stargazers_count"].as_f64().unwrap_or(0.0) as f32;
                    let forks = repo["forks_count"].as_f64().unwrap_or(0.0) as f32;

                    // Add repository node
                    let repo_id = self.get_or_create_id(full_name);
                    if (repo_id as usize) < self.weights.len() {
                        self.weights[repo_id as usize] = stars + forks;
                    }

                    // Add language connection
                    if let Some(language) = repo["language"].as_str() {
                        let lang_label = format!("lang:{}", language);
                        self.add_edge(&lang_label, full_name, 1.0);
                    }

                    // Add owner connection
                    if let Some(owner) = repo["owner"]["login"].as_str() {
                        let owner_type = repo["owner"]["type"].as_str().unwrap_or("User");
                        let owner_label = if owner_type == "Organization" {
                            format!("org:{}", owner)
                        } else {
                            format!("user:{}", owner)
                        };
                        self.add_edge(&owner_label, full_name, 1.0);
                    }

                    // Add fork relationship
                    if let Some(parent) = repo["parent"]["full_name"].as_str() {
                        self.add_edge(parent, full_name, 2.0); // Forks have higher weight
                    }
                }
            }
        }

        Ok(())
    }

    /// Get memory usage estimate
    fn memory_usage(&self) -> usize {
        let labels_size = self.label_to_id.len() * 64 + self.id_to_label.len() * 64;
        let adjacency_size = self.adjacency.len() * 32
            + self
                .adjacency
                .values()
                .map(|set| set.len() * 4)
                .sum::<usize>();
        let weights_size = self.weights.len() * 4;

        labels_size + adjacency_size + weights_size
    }

    /// Find shortest path between two nodes
    fn shortest_path(&self, from: &str, to: &str) -> Option<Vec<String>> {
        let from_id = self.label_to_id.get(from)?;
        let to_id = self.label_to_id.get(to)?;

        let mut queue = std::collections::VecDeque::new();
        let mut visited = HashSet::new();
        let mut parent = HashMap::new();

        queue.push_back(*from_id);
        visited.insert(*from_id);

        while let Some(current) = queue.pop_front() {
            if current == *to_id {
                // Reconstruct path
                let mut path = Vec::new();
                let mut node = current;

                while let Some(&p) = parent.get(&node) {
                    path.push(self.id_to_label[node as usize].clone());
                    node = p;
                }
                path.push(self.id_to_label[*from_id as usize].clone());
                path.reverse();
                return Some(path);
            }

            if let Some(neighbors) = self.adjacency.get(&current) {
                for &neighbor in neighbors {
                    if !visited.contains(&neighbor) {
                        visited.insert(neighbor);
                        parent.insert(neighbor, current);
                        queue.push_back(neighbor);
                    }
                }
            }
        }

        None
    }
}

fn main() {
    demo!("loading entire repository graph into 40GB RAM");

    let mut graph = RepoGraph::new();

    // Load GitHub data
    if let Ok(github_data) =
        std::fs::read_to_string("~/nix/index/github_meta-introspector_repos.json")
    {
        match graph.load_from_github_data(&github_data) {
            Ok(_) => println!("Loaded GitHub data successfully"),
            Err(e) => println!("Error loading GitHub data: {}", e),
        }
    }

    // Load starred data
    if let Ok(starred_data) = std::fs::read_to_string("~/nix/index/starred.json") {
        match graph.load_from_github_data(&starred_data) {
            Ok(_) => println!("Loaded starred data successfully"),
            Err(e) => println!("Error loading starred data: {}", e),
        }
    }

    println!("\nGraph Statistics:");
    println!("Total nodes: {}", graph.id_to_label.len());
    println!(
        "Total edges: {}",
        graph.adjacency.values().map(|s| s.len()).sum::<usize>()
    );
    println!(
        "Memory usage: {:.2} MB",
        graph.memory_usage() as f64 / 1024.0 / 1024.0
    );

    println!("\nCategory ranges:");
    for (category, (start, end)) in &graph.category_ranges {
        println!("{}: {} - {} ({} nodes)", category, start, end, end - start);
    }

    // Test shortest path between key nodes
    if let Some(path) = graph.shortest_path("rustc", "meta-introspector/zos-server") {
        println!("\nPath from rustc to zos-server:");
        for (i, node) in path.iter().enumerate() {
            if i > 0 {
                print!(" -> ");
            }
            print!("{}", node);
        }
        println!();
    }

    production!();
}
