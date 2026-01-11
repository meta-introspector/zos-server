// Context-aware dead code elimination
use std::collections::HashSet;

pub struct ContextualUsageAnalyzer {
    used_items: HashSet<String>,
    current_context: CompileContext,
}

#[derive(Debug)]
struct CompileContext {
    features: Vec<String>,
    target: String,
    cfg_flags: Vec<String>,
}

impl ContextualUsageAnalyzer {
    pub fn analyze_actual_usage(&mut self, source: &str) -> Vec<UnusedItem> {
        let mut unused = Vec::new();

        // Parse for conditional compilation
        for line in source.lines() {
            if line.contains("#[cfg(") && !self.is_cfg_active(line) {
                unused.push(UnusedItem {
                    item_type: "cfg_disabled".to_string(),
                    name: extract_item_name(line),
                    reason: "Disabled by current cfg".to_string(),
                });
            }

            if line.contains("#[cfg(feature =") && !self.is_feature_enabled(line) {
                unused.push(UnusedItem {
                    item_type: "feature_disabled".to_string(),
                    name: extract_item_name(line),
                    reason: "Feature not enabled in current build".to_string(),
                });
            }
        }

        unused
    }

    fn is_cfg_active(&self, line: &str) -> bool {
        // Check if cfg condition matches current context
        self.current_context.cfg_flags.iter()
            .any(|flag| line.contains(flag))
    }

    fn is_feature_enabled(&self, line: &str) -> bool {
        // Extract feature name and check if enabled
        if let Some(feature) = extract_feature_name(line) {
            self.current_context.features.contains(&feature)
        } else {
            false
        }
    }
}

#[derive(Debug)]
struct UnusedItem {
    item_type: String,
    name: String,
    reason: String,
}

fn extract_item_name(line: &str) -> String {
    // Extract function/struct name from line
    line.split_whitespace()
        .find(|word| word.starts_with("fn ") || word.starts_with("struct "))
        .unwrap_or("unknown")
        .to_string()
}

fn extract_feature_name(line: &str) -> Option<String> {
    // Extract feature name from #[cfg(feature = "name")]
    if let Some(start) = line.find("feature = \"") {
        let start = start + 11;
        if let Some(end) = line[start..].find('"') {
            return Some(line[start..start + end].to_string());
        }
    }
    None
}

fn main() {
    println!("🧹 Contextual Dead Code Eliminator");
    println!("Removes code unused in current compile context");
}
