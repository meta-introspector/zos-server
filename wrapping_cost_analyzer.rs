use std::process::Command;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

struct GitObjectWrapper {
    object_id: String,
    object_type: String,
    content_hash: String,
    required_macros: HashSet<String>,
    required_types: HashSet<String>,
    required_imports: HashSet<String>,
    compilation_deps: HashSet<String>,
    wrapping_cost: u32,
}

struct WrappingCostAnalyzer {
    wrapped_objects: HashMap<String, GitObjectWrapper>,
    macro_frequency: HashMap<String, u32>,
    type_frequency: HashMap<String, u32>,
    total_wrapping_cost: u64,
}

impl WrappingCostAnalyzer {
    fn new() -> Self {
        Self {
            wrapped_objects: HashMap::new(),
            macro_frequency: HashMap::new(),
            type_frequency: HashMap::new(),
            total_wrapping_cost: 0,
        }
    }

    fn analyze_git_objects(&mut self) -> Result<(), String> {
        let repos_dir = "/mnt/data1/meta-introspector/repos";

        if let Ok(entries) = fs::read_dir(repos_dir) {
            for entry in entries.flatten() {
                if entry.file_type().map(|ft| ft.is_symlink()).unwrap_or(false) {
                    if let Ok(real_path) = fs::read_link(entry.path()) {
                        self.analyze_repo_objects(&real_path)?;
                    }
                }
            }
        }
        Ok(())
    }

    fn analyze_repo_objects(&mut self, repo_path: &std::path::Path) -> Result<(), String> {
        let git_dir = repo_path.join(".git");
        if !git_dir.exists() {
            return Ok(());
        }

        // Get blob objects (source code files)
        let output = Command::new("git")
            .args(&["cat-file", "--batch-all-objects", "--batch-check"])
            .current_dir(repo_path)
            .output()
            .map_err(|e| format!("Git cat-file failed: {}", e))?;

        if !output.status.success() {
            return Ok(());
        }

        let output_str = String::from_utf8_lossy(&output.stdout);

        for line in output_str.lines().take(100) { // Sample first 100 objects
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 && parts[1] == "blob" {
                let hash = parts[0];
                let size = parts[2].parse::<u64>().unwrap_or(0);

                if size > 0 && size < 100000 { // Reasonable file size
                    if let Ok(wrapper) = self.create_object_wrapper(repo_path, hash) {
                        self.wrapped_objects.insert(hash.to_string(), wrapper);
                    }
                }
            }
        }

        Ok(())
    }

    fn create_object_wrapper(&mut self, repo_path: &std::path::Path, object_hash: &str) -> Result<GitObjectWrapper, String> {
        // Get object content
        let output = Command::new("git")
            .args(&["cat-file", "blob", object_hash])
            .current_dir(repo_path)
            .output()
            .map_err(|e| format!("Git cat-file blob failed: {}", e))?;

        if !output.status.success() {
            return Err("Failed to get blob content".to_string());
        }

        let content = String::from_utf8_lossy(&output.stdout);

        // Analyze content for wrapping requirements
        let mut required_macros = HashSet::new();
        let mut required_types = HashSet::new();
        let mut required_imports = HashSet::new();
        let mut compilation_deps = HashSet::new();

        // Extract macros (lines starting with #)
        for line in content.lines() {
            let trimmed = line.trim();

            // Macros
            if trimmed.starts_with("#[") || trimmed.starts_with("#!") {
                required_macros.insert(trimmed.to_string());
            }

            // Use statements (imports)
            if trimmed.starts_with("use ") {
                required_imports.insert(trimmed.to_string());
            }

            // Type definitions
            if trimmed.starts_with("struct ") || trimmed.starts_with("enum ") ||
               trimmed.starts_with("trait ") || trimmed.starts_with("type ") {
                if let Some(type_name) = trimmed.split_whitespace().nth(1) {
                    required_types.insert(type_name.to_string());
                }
            }

            // External crate dependencies
            if trimmed.contains("extern crate") || trimmed.contains("::") {
                compilation_deps.insert(trimmed.to_string());
            }
        }

        // Calculate wrapping cost c(w(d))
        let wrapping_cost = self.calculate_wrapping_cost(
            &required_macros,
            &required_types,
            &required_imports,
            &compilation_deps
        );

        // Update frequency maps
        for macro_def in &required_macros {
            *self.macro_frequency.entry(macro_def.clone()).or_insert(0) += 1;
        }
        for type_def in &required_types {
            *self.type_frequency.entry(type_def.clone()).or_insert(0) += 1;
        }

        self.total_wrapping_cost += wrapping_cost as u64;

        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        let content_hash = format!("{:x}", hasher.finish());

        Ok(GitObjectWrapper {
            object_id: object_hash.to_string(),
            object_type: "blob".to_string(),
            content_hash,
            required_macros,
            required_types,
            required_imports,
            compilation_deps,
            wrapping_cost,
        })
    }

    fn calculate_wrapping_cost(&self, macros: &HashSet<String>, types: &HashSet<String>,
                              imports: &HashSet<String>, deps: &HashSet<String>) -> u32 {
        // Cost function c(w(d)) = base_cost + macro_cost + type_cost + import_cost + dep_cost
        let base_cost = 1; // Every object has base wrapping cost
        let macro_cost = macros.len() as u32 * 2; // Macros are expensive
        let type_cost = types.len() as u32 * 3; // Types require more context
        let import_cost = imports.len() as u32 * 1; // Imports are moderate cost
        let dep_cost = deps.len() as u32 * 4; // External deps are most expensive

        base_cost + macro_cost + type_cost + import_cost + dep_cost
    }

    fn find_optimal_wrapping_groups(&self) -> HashMap<String, Vec<String>> {
        // Group objects by similar wrapping requirements to minimize total cost
        let mut groups: HashMap<String, Vec<String>> = HashMap::new();

        for (obj_id, wrapper) in &self.wrapped_objects {
            // Create signature based on required macros and types
            let mut signature_parts = Vec::new();

            // Add most common macros to signature
            for macro_def in &wrapper.required_macros {
                if self.macro_frequency.get(macro_def).unwrap_or(&0) > &2 {
                    signature_parts.push(format!("M:{}", macro_def));
                }
            }

            // Add most common types to signature
            for type_def in &wrapper.required_types {
                if self.type_frequency.get(type_def).unwrap_or(&0) > &2 {
                    signature_parts.push(format!("T:{}", type_def));
                }
            }

            signature_parts.sort();
            let signature = signature_parts.join("|");

            groups.entry(signature).or_insert_with(Vec::new).push(obj_id.clone());
        }

        groups
    }

    fn print_wrapping_analysis(&self) {
        println!("🎁 Git Object Wrapping Cost Analysis c(w(d)):");
        println!("  Total wrapped objects: {}", self.wrapped_objects.len());
        println!("  Total wrapping cost: {}", self.total_wrapping_cost);
        println!("  Average wrapping cost per object: {:.2}",
            self.total_wrapping_cost as f64 / self.wrapped_objects.len() as f64);

        // Most expensive objects to wrap
        let mut cost_sorted: Vec<_> = self.wrapped_objects.iter().collect();
        cost_sorted.sort_by_key(|(_, wrapper)| std::cmp::Reverse(wrapper.wrapping_cost));

        println!("\n💰 Most expensive objects to wrap:");
        for (obj_id, wrapper) in cost_sorted.iter().take(5) {
            println!("    {}: cost {} (macros: {}, types: {}, imports: {})",
                &obj_id[..8], wrapper.wrapping_cost,
                wrapper.required_macros.len(), wrapper.required_types.len(),
                wrapper.required_imports.len());
        }

        // Most common macros (highest reuse potential)
        let mut macro_sorted: Vec<_> = self.macro_frequency.iter().collect();
        macro_sorted.sort_by_key(|(_, count)| std::cmp::Reverse(**count));

        println!("\n🔧 Most reusable macros (optimization targets):");
        for (macro_def, count) in macro_sorted.iter().take(5) {
            println!("    {}: used {} times", macro_def, count);
        }

        // Most common types
        let mut type_sorted: Vec<_> = self.type_frequency.iter().collect();
        type_sorted.sort_by_key(|(_, count)| std::cmp::Reverse(**count));

        println!("\n📝 Most reusable types:");
        for (type_def, count) in type_sorted.iter().take(5) {
            println!("    {}: used {} times", type_def, count);
        }

        // Optimal grouping analysis
        let groups = self.find_optimal_wrapping_groups();
        println!("\n🎯 Optimal wrapping groups (shared environments):");
        println!("  Total groups: {}", groups.len());

        let mut group_sizes: Vec<_> = groups.iter()
            .map(|(sig, objects)| (sig, objects.len()))
            .collect();
        group_sizes.sort_by_key(|(_, size)| std::cmp::Reverse(*size));

        for (signature, size) in group_sizes.iter().take(3) {
            println!("    Group with {} objects: {}", size,
                if signature.len() > 50 { &signature[..50] } else { signature });
        }

        // Cost optimization potential
        let single_env_cost = self.total_wrapping_cost;
        let grouped_env_cost = groups.len() as u64 * 10; // Assume 10 cost per shared environment
        let savings = single_env_cost.saturating_sub(grouped_env_cost);

        println!("\n📊 Cost optimization potential:");
        println!("  Individual wrapping cost: {}", single_env_cost);
        println!("  Shared environment cost: {}", grouped_env_cost);
        println!("  Potential savings: {} ({:.1}%)", savings,
            savings as f64 / single_env_cost as f64 * 100.0);
    }
}

fn main() {
    let mut analyzer = WrappingCostAnalyzer::new();

    println!("🔍 Analyzing git object wrapping costs c(w(d))...");

    if let Err(e) = analyzer.analyze_git_objects() {
        eprintln!("Error: {}", e);
        return;
    }

    analyzer.print_wrapping_analysis();
}
