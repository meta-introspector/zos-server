use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

fn main() {
    println!("Loading 40k+ repositories from ~/nix/index...");

    let mut graph = RepoGraph::new();
    let index_dir = "/mnt/data1/nix/index";

    // Load all JSON files from index directory
    if let Ok(entries) = fs::read_dir(index_dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "json") {
                    println!("Loading: {}", path.display());

                    if let Ok(data) = fs::read_to_string(&path) {
                        match graph.load_from_github_data(&data) {
                            Ok(_) => {},
                            Err(e) => println!("Error loading {}: {}", path.display(), e),
                        }
                    }
                }
            }
        }
    }

    // Also load from allrs.txt if it exists
    let allrs_path = format!("{}/allrs.txt", index_dir);
    if Path::new(&allrs_path).exists() {
        println!("Loading repository paths from allrs.txt...");
        if let Ok(content) = fs::read_to_string(&allrs_path) {
            for line in content.lines() {
                if !line.trim().is_empty() {
                    // Extract repo name from path
                    if let Some(repo_name) = extract_repo_name(line) {
                        let repo_id = graph.get_or_create_id(&repo_name);
                        graph.weights[repo_id as usize] = 1.0; // Base weight for discovered repos
                    }
                }
            }
        }
    }

    println!("\n🚀 Graph loaded into memory!");
    println!("📊 Nodes: {}", graph.id_to_label.len());
    println!("🔗 Edges: {}", graph.adjacency.values().map(|s| s.len()).sum::<usize>());
    println!("💾 Memory: {:.1} MB", graph.memory_usage() as f64 / 1024.0 / 1024.0);

    // Show top repositories by connections
    let mut connections: Vec<(String, usize)> = graph.adjacency
        .iter()
        .map(|(id, edges)| (graph.id_to_label[*id as usize].clone(), edges.len()))
        .collect();
    connections.sort_by(|a, b| b.1.cmp(&a.1));

    println!("\n🔥 Most connected repositories:");
    for (repo, count) in connections.iter().take(10) {
        println!("  {} -> {} connections", repo, count);
    }

    // Calculate eigenvector centrality
    println!("\n🎯 Calculating eigenvector centrality...");
    let centrality = graph.eigenvector_centrality(50);

    let mut central_repos: Vec<(String, f32)> = centrality
        .iter()
        .enumerate()
        .map(|(i, &score)| (graph.id_to_label[i].clone(), score))
        .collect();
    central_repos.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    println!("🌟 Most central repositories:");
    for (repo, score) in central_repos.iter().take(10) {
        println!("  {} -> {:.6}", repo, score);
    }

    // Find clusters
    println!("\n🔍 Finding repository clusters...");
    let clusters = graph.find_clusters();

    println!("📦 Found {} clusters:", clusters.len());
    for (i, cluster) in clusters.iter().take(5).enumerate() {
        println!("  Cluster {}: {} repositories", i+1, cluster.len());
        for &node_id in cluster.iter().take(3) {
            println!("    - {}", graph.id_to_label[node_id as usize]);
        }
        if cluster.len() > 3 {
            println!("    ... and {} more", cluster.len() - 3);
        }
    }
}

fn extract_repo_name(path: &str) -> Option<String> {
    // Extract repo name from paths like /path/to/repo/file.rs
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() > 3 {
        // Look for patterns like github.com/user/repo or just user/repo
        for i in 0..parts.len()-1 {
            if parts[i].contains("github") || parts[i] == "repos" {
                if i + 2 < parts.len() {
                    return Some(format!("{}/{}", parts[i+1], parts[i+2]));
                }
            }
        }
        // Fallback: use last two meaningful directory names
        if parts.len() >= 2 {
            let user = parts[parts.len()-3];
            let repo = parts[parts.len()-2];
            if !user.is_empty() && !repo.is_empty() && user != "src" && repo != "src" {
                return Some(format!("{}/{}", user, repo));
            }
        }
    }
    None
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

    /// Calculate eigenvector centrality (simplified power iteration)
    fn eigenvector_centrality(&self, iterations: usize) -> Vec<f32> {
        let n = self.id_to_label.len();
        let mut centrality = vec![1.0; n];

        for _ in 0..iterations {
            let mut new_centrality = vec![0.0; n];

            for (&from_id, neighbors) in &self.adjacency {
                let from_score = centrality[from_id as usize];
                for &to_id in neighbors {
                    new_centrality[to_id as usize] += from_score / neighbors.len() as f32;
                }
            }

            // Normalize
            let sum: f32 = new_centrality.iter().sum();
            if sum > 0.0 {
                for score in &mut new_centrality {
                    *score /= sum;
                }
            }

            centrality = new_centrality;
        }

        centrality
    }

    /// Find clusters using simple connected components
    fn find_clusters(&self) -> Vec<Vec<u32>> {
        let mut visited = HashSet::new();
        let mut clusters = Vec::new();

        for &node_id in self.label_to_id.values() {
            if !visited.contains(&node_id) {
                let cluster = self.dfs_cluster(node_id, &mut visited);
                if cluster.len() > 1 {
                    clusters.push(cluster);
                }
            }
        }

        clusters.sort_by(|a, b| b.len().cmp(&a.len()));
        clusters
    }

    fn dfs_cluster(&self, start: u32, visited: &mut HashSet<u32>) -> Vec<u32> {
        let mut cluster = Vec::new();
        let mut stack = vec![start];

        while let Some(node) = stack.pop() {
            if visited.insert(node) {
                cluster.push(node);

                if let Some(neighbors) = self.adjacency.get(&node) {
                    for &neighbor in neighbors {
                        if !visited.contains(&neighbor) {
                            stack.push(neighbor);
                        }
                    }
                }
            }
        }

        cluster
    }
}
