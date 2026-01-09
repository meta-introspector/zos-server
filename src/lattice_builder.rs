// GitHub Action Lattice Builder - Multi-Layer Security Compilation
use std::collections::HashMap;
use std::process::Command;

/// GitHub Action lattice builder
pub struct LatticeBuilder {
    dependency_graph: DependencyGraph,
    ast_filters: HashMap<SecurityLayer, ASTFilter>,
    bandwidth_limits: HashMap<SecurityLayer, BandwidthLimit>,
}

#[derive(Debug, Clone)]
pub struct DependencyGraph {
    pub our_crates: Vec<CrateInfo>,
    pub rustc_deps: Vec<CrateInfo>,
    pub external_deps: Vec<CrateInfo>,
}

#[derive(Debug, Clone)]
pub struct CrateInfo {
    pub name: String,
    pub version: String,
    pub source_path: String,
    pub target_layer: SecurityLayer,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SecurityLayer {
    L0Public,    // Public WASM layer
    L1Gateway,   // Auth/routing layer
    L2Service,   // Business logic layer
    L3Core,      // Admin operations layer
    L4Kernel,    // Root/syscall layer
}

#[derive(Debug, Clone)]
pub struct ASTFilter {
    pub allowed_nodes: Vec<ASTNodeType>,
    pub blocked_patterns: Vec<String>,
    pub complexity_limit: u32,
}

#[derive(Debug, Clone)]
pub enum ASTNodeType {
    Function,
    Struct,
    Enum,
    Impl,
    Trait,
    Const,
    Static,
    Macro,
    UnsafeBlock,
    ExternBlock,
}

#[derive(Debug, Clone)]
pub struct BandwidthLimit {
    pub max_functions: u32,
    pub max_complexity: u32,
    pub max_dependencies: u32,
    pub max_binary_size: u64,
}

impl LatticeBuilder {
    pub fn new() -> Self {
        let mut builder = Self {
            dependency_graph: DependencyGraph {
                our_crates: Vec::new(),
                rustc_deps: Vec::new(),
                external_deps: Vec::new(),
            },
            ast_filters: HashMap::new(),
            bandwidth_limits: HashMap::new(),
        };
        builder.setup_layer_filters();
        builder
    }

    fn setup_layer_filters(&mut self) {
        // L0 Public - Most restrictive
        self.ast_filters.insert(SecurityLayer::L0Public, ASTFilter {
            allowed_nodes: vec![
                ASTNodeType::Function,
                ASTNodeType::Struct,
                ASTNodeType::Enum,
                ASTNodeType::Const,
            ],
            blocked_patterns: vec![
                "unsafe".to_string(),
                "libc::".to_string(),
                "std::process".to_string(),
                "std::fs::remove".to_string(),
            ],
            complexity_limit: 10,
        });

        self.bandwidth_limits.insert(SecurityLayer::L0Public, BandwidthLimit {
            max_functions: 50,
            max_complexity: 100,
            max_dependencies: 10,
            max_binary_size: 1024 * 1024, // 1MB
        });

        // L1 Gateway - Auth operations
        self.ast_filters.insert(SecurityLayer::L1Gateway, ASTFilter {
            allowed_nodes: vec![
                ASTNodeType::Function,
                ASTNodeType::Struct,
                ASTNodeType::Enum,
                ASTNodeType::Impl,
                ASTNodeType::Trait,
                ASTNodeType::Const,
            ],
            blocked_patterns: vec![
                "libc::execve".to_string(),
                "std::process::Command".to_string(),
            ],
            complexity_limit: 50,
        });

        self.bandwidth_limits.insert(SecurityLayer::L1Gateway, BandwidthLimit {
            max_functions: 200,
            max_complexity: 500,
            max_dependencies: 50,
            max_binary_size: 10 * 1024 * 1024, // 10MB
        });

        // L4 Kernel - Least restrictive
        self.ast_filters.insert(SecurityLayer::L4Kernel, ASTFilter {
            allowed_nodes: vec![
                ASTNodeType::Function,
                ASTNodeType::Struct,
                ASTNodeType::Enum,
                ASTNodeType::Impl,
                ASTNodeType::Trait,
                ASTNodeType::Const,
                ASTNodeType::Static,
                ASTNodeType::Macro,
                ASTNodeType::UnsafeBlock,
                ASTNodeType::ExternBlock,
            ],
            blocked_patterns: vec![], // No restrictions
            complexity_limit: 1000,
        });

        self.bandwidth_limits.insert(SecurityLayer::L4Kernel, BandwidthLimit {
            max_functions: 10000,
            max_complexity: 50000,
            max_dependencies: 1000,
            max_binary_size: 100 * 1024 * 1024, // 100MB
        });
    }

    /// Build complete lattice from dependencies
    pub fn build_lattice(&mut self, workspace_path: &str) -> Result<LatticeOutput, String> {
        println!("🔧 Building security lattice from workspace: {}", workspace_path);

        // Step 1: Discover all dependencies
        self.discover_dependencies(workspace_path)?;

        // Step 2: Classify crates by security layer
        self.classify_crates()?;

        // Step 3: Apply AST filters per layer
        let filtered_crates = self.apply_ast_filters()?;

        // Step 4: Check bandwidth limits
        self.check_bandwidth_limits(&filtered_crates)?;

        // Step 5: Generate layer-specific binaries
        let layer_binaries = self.generate_layer_binaries(&filtered_crates)?;

        Ok(LatticeOutput {
            layers: layer_binaries,
            dependency_count: self.dependency_graph.rustc_deps.len() +
                             self.dependency_graph.external_deps.len(),
            total_crates: filtered_crates.len(),
        })
    }

    fn discover_dependencies(&mut self, workspace_path: &str) -> Result<(), String> {
        // Get our crates
        self.discover_our_crates(workspace_path)?;

        // Get rustc dependencies
        self.discover_rustc_dependencies()?;

        // Get external dependencies
        self.discover_external_dependencies(workspace_path)?;

        println!("📊 Discovered dependencies:");
        println!("  Our crates: {}", self.dependency_graph.our_crates.len());
        println!("  Rustc deps: {}", self.dependency_graph.rustc_deps.len());
        println!("  External deps: {}", self.dependency_graph.external_deps.len());

        Ok(())
    }

    fn discover_our_crates(&mut self, workspace_path: &str) -> Result<(), String> {
        let output = Command::new("find")
            .args(&[workspace_path, "-name", "Cargo.toml"])
            .output()
            .map_err(|e| format!("Failed to find Cargo.toml files: {}", e))?;

        let paths = String::from_utf8_lossy(&output.stdout);
        for path in paths.lines() {
            if let Ok(content) = std::fs::read_to_string(path) {
                if let Some(name) = self.extract_crate_name(&content) {
                    self.dependency_graph.our_crates.push(CrateInfo {
                        name: name.clone(),
                        version: "0.1.0".to_string(),
                        source_path: path.replace("/Cargo.toml", ""),
                        target_layer: self.determine_target_layer(&name),
                    });
                }
            }
        }

        Ok(())
    }

    fn discover_rustc_dependencies(&mut self) -> Result<(), String> {
        // Get rustc source dependencies
        let rustc_deps = vec![
            "rustc_driver", "rustc_interface", "rustc_session", "rustc_ast",
            "rustc_parse", "rustc_hir", "rustc_middle", "rustc_codegen_ssa",
        ];

        for dep in rustc_deps {
            self.dependency_graph.rustc_deps.push(CrateInfo {
                name: dep.to_string(),
                version: "1.0.0".to_string(),
                source_path: format!("/rustc/compiler/{}", dep),
                target_layer: SecurityLayer::L4Kernel, // Rustc goes to kernel layer
            });
        }

        Ok(())
    }

    fn discover_external_dependencies(&mut self, workspace_path: &str) -> Result<(), String> {
        let output = Command::new("cargo")
            .args(&["tree", "--format", "{p}"])
            .current_dir(workspace_path)
            .output()
            .map_err(|e| format!("Failed to get dependency tree: {}", e))?;

        let deps = String::from_utf8_lossy(&output.stdout);
        for line in deps.lines() {
            if !line.trim().is_empty() && !line.contains("zos-") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if let Some(name_version) = parts.first() {
                    let name = name_version.split('@').next().unwrap_or(name_version);
                    self.dependency_graph.external_deps.push(CrateInfo {
                        name: name.to_string(),
                        version: "unknown".to_string(),
                        source_path: format!("~/.cargo/registry/src/*/{}", name),
                        target_layer: self.determine_target_layer(name),
                    });
                }
            }
        }

        Ok(())
    }

    fn extract_crate_name(&self, cargo_content: &str) -> Option<String> {
        cargo_content.lines()
            .find(|line| line.starts_with("name"))
            .and_then(|line| line.split('=').nth(1))
            .map(|s| s.trim().trim_matches('"').to_string())
    }

    fn determine_target_layer(&self, crate_name: &str) -> SecurityLayer {
        match crate_name {
            name if name.contains("public") || name.contains("wasm") => SecurityLayer::L0Public,
            name if name.contains("auth") || name.contains("gateway") => SecurityLayer::L1Gateway,
            name if name.contains("service") || name.contains("api") => SecurityLayer::L2Service,
            name if name.contains("core") || name.contains("admin") => SecurityLayer::L3Core,
            name if name.contains("kernel") || name.contains("rustc") => SecurityLayer::L4Kernel,
            _ => SecurityLayer::L2Service, // Default to service layer
        }
    }

    fn classify_crates(&mut self) -> Result<(), String> {
        let mut layer_counts = HashMap::new();

        // Count crates per layer
        for crates in [&self.dependency_graph.our_crates,
                      &self.dependency_graph.rustc_deps,
                      &self.dependency_graph.external_deps] {
            for crate_info in crates {
                *layer_counts.entry(crate_info.target_layer.clone()).or_insert(0) += 1;
            }
        }

        println!("📊 Crate classification:");
        for (layer, count) in layer_counts {
            println!("  {:?}: {} crates", layer, count);
        }

        Ok(())
    }

    fn apply_ast_filters(&self) -> Result<Vec<FilteredCrate>, String> {
        let mut filtered_crates = Vec::new();

        for crates in [&self.dependency_graph.our_crates,
                      &self.dependency_graph.rustc_deps,
                      &self.dependency_graph.external_deps] {
            for crate_info in crates {
                let filter = self.ast_filters.get(&crate_info.target_layer)
                    .ok_or("No filter for layer")?;

                let filtered = self.filter_crate_ast(crate_info, filter)?;
                filtered_crates.push(filtered);
            }
        }

        Ok(filtered_crates)
    }

    fn filter_crate_ast(&self, crate_info: &CrateInfo, filter: &ASTFilter) -> Result<FilteredCrate, String> {
        // Simulate AST filtering
        let mut allowed_functions = 0;
        let mut blocked_patterns = 0;

        // In real implementation, would parse AST and filter nodes
        for pattern in &filter.blocked_patterns {
            if crate_info.name.contains(pattern) {
                blocked_patterns += 1;
            }
        }

        allowed_functions = filter.complexity_limit.min(100); // Simulate filtering

        Ok(FilteredCrate {
            name: crate_info.name.clone(),
            layer: crate_info.target_layer.clone(),
            functions_kept: allowed_functions,
            patterns_blocked: blocked_patterns,
            ast_nodes_filtered: filter.allowed_nodes.len() as u32,
        })
    }

    fn check_bandwidth_limits(&self, filtered_crates: &[FilteredCrate]) -> Result<(), String> {
        let mut layer_stats = HashMap::new();

        for crate_info in filtered_crates {
            let stats = layer_stats.entry(crate_info.layer.clone())
                .or_insert(LayerStats::default());

            stats.total_functions += crate_info.functions_kept;
            stats.total_crates += 1;
        }

        // Check limits
        for (layer, stats) in &layer_stats {
            if let Some(limit) = self.bandwidth_limits.get(layer) {
                if stats.total_functions > limit.max_functions {
                    return Err(format!("Layer {:?} exceeds function limit: {} > {}",
                                     layer, stats.total_functions, limit.max_functions));
                }
            }
        }

        println!("✅ All bandwidth limits satisfied");
        Ok(())
    }

    fn generate_layer_binaries(&self, filtered_crates: &[FilteredCrate]) -> Result<HashMap<SecurityLayer, LayerBinary>, String> {
        let mut layer_binaries = HashMap::new();

        for layer in [SecurityLayer::L0Public, SecurityLayer::L1Gateway,
                     SecurityLayer::L2Service, SecurityLayer::L3Core, SecurityLayer::L4Kernel] {
            let layer_crates: Vec<_> = filtered_crates.iter()
                .filter(|c| c.layer == layer)
                .collect();

            if !layer_crates.is_empty() {
                let binary = LayerBinary {
                    layer: layer.clone(),
                    crate_count: layer_crates.len(),
                    total_functions: layer_crates.iter().map(|c| c.functions_kept).sum(),
                    binary_path: format!("/usr/bin/zos-{:?}", layer).to_lowercase(),
                    security_proof: format!("Layer {:?} contains {} filtered crates", layer, layer_crates.len()),
                };

                layer_binaries.insert(layer, binary);
            }
        }

        Ok(layer_binaries)
    }
}

#[derive(Debug, Clone)]
pub struct FilteredCrate {
    pub name: String,
    pub layer: SecurityLayer,
    pub functions_kept: u32,
    pub patterns_blocked: u32,
    pub ast_nodes_filtered: u32,
}

#[derive(Debug, Clone, Default)]
pub struct LayerStats {
    pub total_functions: u32,
    pub total_crates: u32,
}

#[derive(Debug, Clone)]
pub struct LayerBinary {
    pub layer: SecurityLayer,
    pub crate_count: usize,
    pub total_functions: u32,
    pub binary_path: String,
    pub security_proof: String,
}

#[derive(Debug, Clone)]
pub struct LatticeOutput {
    pub layers: HashMap<SecurityLayer, LayerBinary>,
    pub dependency_count: usize,
    pub total_crates: usize,
}

/// Generate GitHub Action workflow
pub fn generate_github_action() -> String {
    r#"
name: ZOS Security Lattice Builder

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  build-lattice:
    runs-on: ubuntu-latest

    steps:
    - uses: actions/checkout@v3

    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        target: wasm32-unknown-unknown

    - name: Install rustc source
      run: rustup component add rust-src

    - name: Build Security Lattice
      run: |
        cargo run --bin lattice-builder -- \
          --workspace . \
          --include-rustc-deps \
          --output-dir ./lattice-output

    - name: Verify Layer Isolation
      run: |
        ./scripts/verify-layer-isolation.sh ./lattice-output

    - name: Generate WASM Modules
      run: |
        cargo run --bin wasm-compiler -- \
          --input ./lattice-output/l0-public \
          --output ./web-modules

    - name: Deploy to GitHub Pages
      uses: peaceiris/actions-gh-pages@v3
      with:
        github_token: ${{ secrets.GITHUB_TOKEN }}
        publish_dir: ./web-modules

    - name: Upload Lattice Artifacts
      uses: actions/upload-artifact@v3
      with:
        name: security-lattice
        path: ./lattice-output/
"#.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_classification() {
        let builder = LatticeBuilder::new();

        assert_eq!(builder.determine_target_layer("zos-public-api"), SecurityLayer::L0Public);
        assert_eq!(builder.determine_target_layer("zos-auth-gateway"), SecurityLayer::L1Gateway);
        assert_eq!(builder.determine_target_layer("rustc_driver"), SecurityLayer::L4Kernel);
    }

    #[test]
    fn test_ast_filtering() {
        let builder = LatticeBuilder::new();
        let filter = builder.ast_filters.get(&SecurityLayer::L0Public).unwrap();

        assert!(filter.blocked_patterns.contains(&"unsafe".to_string()));
        assert_eq!(filter.complexity_limit, 10);
    }
}
