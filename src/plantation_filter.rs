use rayon::prelude::*;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct SecurityFrequency {
    pub level: String,
    pub frequency: f64,
    pub band_low: f64,
    pub band_high: f64,
}

pub struct PlantationFilter {
    pub security_bands: Vec<SecurityFrequency>,
    pub source_index: String,
}

impl PlantationFilter {
    pub fn new(index_path: &str) -> Self {
        Self {
            security_bands: vec![
                SecurityFrequency {
                    level: "Public".to_string(),
                    frequency: 1.0,
                    band_low: 0.5,
                    band_high: 1.5,
                },
                SecurityFrequency {
                    level: "Guest".to_string(),
                    frequency: 2.0,
                    band_low: 1.5,
                    band_high: 3.0,
                },
                SecurityFrequency {
                    level: "User".to_string(),
                    frequency: 4.0,
                    band_low: 3.0,
                    band_high: 6.0,
                },
                SecurityFrequency {
                    level: "Admin".to_string(),
                    frequency: 8.0,
                    band_low: 6.0,
                    band_high: 12.0,
                },
                SecurityFrequency {
                    level: "SuperAdmin".to_string(),
                    frequency: 16.0,
                    band_low: 12.0,
                    band_high: 24.0,
                },
            ],
            source_index: index_path.to_string(),
        }
    }

    pub fn harvest_plantation(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(&self.source_index)?;
        Ok(content.lines().map(|s| s.to_string()).collect())
    }

    pub fn analyze_security_frequency(&self, file_path: &str) -> f64 {
        // Harmonic analysis based on path depth and security indicators
        let depth = file_path.matches('/').count() as f64;
        let security_score = if file_path.contains("auth") || file_path.contains("admin") {
            8.0
        } else if file_path.contains("user") || file_path.contains("session") {
            4.0
        } else if file_path.contains("guest") {
            2.0
        } else {
            1.0
        };

        security_score * (1.0 + depth * 0.1)
    }

    pub fn filter_by_band(&self, files: &[String], target_band: &SecurityFrequency) -> Vec<String> {
        files
            .par_iter()
            .filter(|file| {
                let freq = self.analyze_security_frequency(file);
                freq >= target_band.band_low && freq < target_band.band_high
            })
            .cloned()
            .collect()
    }

    pub fn generate_security_lattice(&self) -> Result<(), Box<dyn std::error::Error>> {
        let all_files = self.harvest_plantation()?;
        println!(
            "🌱 Harvested {} Rust files from plantation",
            all_files.len()
        );

        for band in &self.security_bands {
            let filtered_files = self.filter_by_band(&all_files, band);
            let output_dir = format!("target/security_contexts/{}", band.level.to_lowercase());
            fs::create_dir_all(&output_dir)?;

            let manifest_path = format!("{}/filtered_files.txt", output_dir);
            fs::write(&manifest_path, filtered_files.join("\n"))?;

            println!(
                "🔧 {} context: {} files (freq {:.1})",
                band.level,
                filtered_files.len(),
                band.frequency
            );
        }

        Ok(())
    }
}
