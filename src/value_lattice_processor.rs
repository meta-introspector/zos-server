use crate::project_watcher::FileChangeEvent;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{error, info};

#[derive(Debug, Serialize, Deserialize)]
pub struct ValueLatticeEntry {
    pub value: String,
    pub godel_number: u64,
    pub usage_count: u32,
    pub first_seen: std::time::SystemTime,
    pub last_updated: std::time::SystemTime,
    pub file_locations: Vec<String>,
}

pub struct ValueLatticeProcessor {
    lattice: HashMap<String, ValueLatticeEntry>,
    next_godel_number: u64,
}

impl ValueLatticeProcessor {
    pub fn new() -> Self {
        Self {
            lattice: HashMap::new(),
            next_godel_number: 1,
        }
    }

    pub fn process_file_change(&mut self, event: &FileChangeEvent) -> Result<(), String> {
        if !Self::is_processable_file(&event.path) {
            return Ok(());
        }

        info!("🔬 Processing file change: {}", event.path.display());

        // Read file and extract values
        let content = std::fs::read_to_string(&event.path)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        self.extract_and_index_values(&content, &event.path)?;

        info!(
            "📊 Lattice now contains {} unique values",
            self.lattice.len()
        );
        Ok(())
    }

    fn extract_and_index_values(
        &mut self,
        content: &str,
        file_path: &PathBuf,
    ) -> Result<(), String> {
        let file_path_str = file_path.to_string_lossy().to_string();

        // Markov character graph approach - scan for numeric sequences
        let chars: Vec<char> = content.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            if chars[i].is_ascii_digit() || chars[i] == '-' {
                // Found start of number, collect the full sequence
                let start = i;

                // Handle negative sign
                if chars[i] == '-' {
                    i += 1;
                }

                // Collect digits
                while i < chars.len() && chars[i].is_ascii_digit() {
                    i += 1;
                }

                // Extract the number
                let number_str: String = chars[start..i].iter().collect();
                if let Ok(_) = number_str.parse::<i64>() {
                    self.index_value(&number_str, &file_path_str);
                }
            } else {
                i += 1;
            }
        }

        Ok(())
    }

    fn index_value(&mut self, value: &str, file_path: &str) {
        let now = std::time::SystemTime::now();

        match self.lattice.get_mut(value) {
            Some(entry) => {
                // Update existing entry
                entry.usage_count += 1;
                entry.last_updated = now;
                if !entry.file_locations.contains(&file_path.to_string()) {
                    entry.file_locations.push(file_path.to_string());
                }
            }
            None => {
                // Create new entry
                let godel_number = self.next_godel_number;
                self.next_godel_number += 1;

                self.lattice.insert(
                    value.to_string(),
                    ValueLatticeEntry {
                        value: value.to_string(),
                        godel_number,
                        usage_count: 1,
                        first_seen: now,
                        last_updated: now,
                        file_locations: vec![file_path.to_string()],
                    },
                );

                info!("✨ New value indexed: {} (Gödel: {})", value, godel_number);
            }
        }
    }

    fn is_processable_file(path: &PathBuf) -> bool {
        if let Some(ext) = path.extension() {
            matches!(ext.to_str(), Some("rs" | "toml" | "json" | "md"))
        } else {
            false
        }
    }

    pub fn get_lattice_stats(&self) -> HashMap<String, u32> {
        let mut stats = HashMap::new();
        stats.insert("total_values".to_string(), self.lattice.len() as u32);
        stats.insert("next_godel".to_string(), self.next_godel_number as u32);

        // Count by usage frequency
        let high_usage = self.lattice.values().filter(|e| e.usage_count > 10).count();
        stats.insert("high_usage_values".to_string(), high_usage as u32);

        stats
    }

    pub fn get_top_values(&self, limit: usize) -> Vec<&ValueLatticeEntry> {
        let mut values: Vec<&ValueLatticeEntry> = self.lattice.values().collect();
        values.sort_by(|a, b| b.usage_count.cmp(&a.usage_count));
        values.into_iter().take(limit).collect()
    }
}
