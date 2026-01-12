use std::collections::HashMap;
use std::fs;

pub struct SimpleMarkovModel {
    transitions: HashMap<char, HashMap<char, u32>>,
    total_chars: u64,
}

impl SimpleMarkovModel {
    pub fn new() -> Self {
        Self {
            transitions: HashMap::new(),
            total_chars: 0,
        }
    }

    pub fn train_from_repos(&mut self) -> Result<(), String> {
        let repos_dir = "/mnt/data1/meta-introspector/repos";

        if let Ok(entries) = fs::read_dir(repos_dir) {
            for entry in entries.flatten() {
                if entry.file_type().map(|ft| ft.is_symlink()).unwrap_or(false) {
                    if let Ok(real_path) = fs::read_link(entry.path()) {
                        self.scan_repo(&real_path)?;
                    }
                }
            }
        }

        println!("📊 Markov model trained on {} characters", self.total_chars);
        Ok(())
    }

    fn scan_repo(&mut self, repo_path: &std::path::Path) -> Result<(), String> {
        if let Ok(entries) = fs::read_dir(repo_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "rs") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        self.train_on_text(&content);
                    }
                }
            }
        }
        Ok(())
    }

    fn train_on_text(&mut self, text: &str) {
        let chars: Vec<char> = text.chars().collect();

        for window in chars.windows(2) {
            let from = window[0];
            let to = window[1];

            *self.transitions
                .entry(from)
                .or_insert_with(HashMap::new)
                .entry(to)
                .or_insert(0) += 1;

            self.total_chars += 1;
        }
    }

    pub fn find_enum_stats(&self) -> (u32, Vec<String>) {
        let mut enum_count = 0;
        let mut following_chars = HashMap::new();

        // Look for 'enum' pattern: e->n->u->m
        for (e_char, e_map) in &self.transitions {
            if *e_char == 'e' {
                if let Some(&n_count) = e_map.get(&'n') {
                    if let Some(n_map) = self.transitions.get(&'n') {
                        if let Some(&u_count) = n_map.get(&'u') {
                            if let Some(u_map) = self.transitions.get(&'u') {
                                if let Some(&m_count) = u_map.get(&'m') {
                                    enum_count += std::cmp::min(std::cmp::min(n_count, u_count), m_count);

                                    // Find what follows 'm' in enum context
                                    if let Some(m_map) = self.transitions.get(&'m') {
                                        for (next_char, count) in m_map {
                                            *following_chars.entry(*next_char).or_insert(0) += count;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let mut sorted_following: Vec<_> = following_chars.into_iter()
            .map(|(c, count)| format!("'{}': {} times", c, count))
            .collect();
        sorted_following.sort();

        (enum_count, sorted_following.into_iter().take(10).collect())
    }

    pub fn print_stats(&self) {
        println!("🔍 Markov Model Statistics:");
        println!("  Total characters processed: {}", self.total_chars);
        println!("  Unique characters: {}", self.transitions.len());

        // Show top transitions
        let mut all_transitions: Vec<_> = self.transitions.iter()
            .flat_map(|(from, to_map)| {
                to_map.iter().map(move |(to, count)| ((*from, *to), *count))
            })
            .collect();
        all_transitions.sort_by_key(|(_, count)| std::cmp::Reverse(*count));

        println!("  Top 10 character transitions:");
        for ((from, to), count) in all_transitions.iter().take(10) {
            println!("    '{}' → '{}': {} times", from, to, count);
        }
    }
}

fn main() {
    let mut model = SimpleMarkovModel::new();

    println!("🚀 Building Markov model from all repositories...");

    if let Err(e) = model.train_from_repos() {
        eprintln!("Error: {}", e);
        return;
    }

    model.print_stats();

    // Find 'enum' statistics
    let (enum_count, enum_patterns) = model.find_enum_stats();
    println!("\n🔍 'enum' Statistics:");
    println!("  'enum' pattern strength: {}", enum_count);
    println!("  Characters following 'm' in enum context:");
    for pattern in enum_patterns {
        println!("    {}", pattern);
    }
}
