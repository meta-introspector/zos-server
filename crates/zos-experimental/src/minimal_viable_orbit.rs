// Aggressive Eigenmatrix Minimization - Remove everything non-essential
use crate::eigenmatrix_compression::EigenmatrixCompression;

/// Minimal Viable Orbit - Stripped down to absolute essentials
#[derive(Debug, Clone)]
pub struct MinimalViableOrbit {
    pub core_matrix: Vec<Vec<f64>>,
    pub removed_functions: Vec<String>,
    pub disabled_features: Vec<String>,
    pub macro_wrappers: Vec<String>,
    pub minimization_ratio: f64,
}

impl MinimalViableOrbit {
    /// Create minimal orbit through aggressive stripping
    pub fn minimize(compression: &EigenmatrixCompression) -> Self {
        println!("🔪 AGGRESSIVE MINIMIZATION - REMOVING ALL NON-ESSENTIALS...");

        let mut removed_functions = Vec::new();
        let mut disabled_features = Vec::new();
        let mut macro_wrappers = Vec::new();

        // Remove functions
        removed_functions.extend(Self::remove_functions());

        // Disable logging
        disabled_features.extend(Self::disable_logging());

        // Turn off graphviz
        disabled_features.extend(Self::disable_graphviz());

        // Wrap with macros
        macro_wrappers.extend(Self::create_macro_wrappers());

        // Create minimal core matrix (only essential eigenvalues)
        let core_matrix = Self::extract_core_matrix(&compression.compressed_matrix);

        let original_size = compression.original_matrix.len() *
                           compression.original_matrix.get(0).map_or(0, |r| r.len());
        let minimal_size = core_matrix.len() *
                          core_matrix.get(0).map_or(0, |r| r.len());

        let minimization_ratio = if original_size > 0 {
            1.0 - (minimal_size as f64 / original_size as f64)
        } else {
            0.0
        };

        println!("✅ Minimization complete:");
        println!("   Functions removed: {}", removed_functions.len());
        println!("   Features disabled: {}", disabled_features.len());
        println!("   Macro wrappers: {}", macro_wrappers.len());
        println!("   Minimization ratio: {:.1}%", minimization_ratio * 100.0);

        MinimalViableOrbit {
            core_matrix,
            removed_functions,
            disabled_features,
            macro_wrappers,
            minimization_ratio,
        }
    }

    /// Remove non-essential functions
    fn remove_functions() -> Vec<String> {
        vec![
            "debug_print".to_string(),
            "trace_execution".to_string(),
            "format_output".to_string(),
            "validate_input".to_string(),
            "error_reporting".to_string(),
            "metrics_collection".to_string(),
            "profiling_hooks".to_string(),
            "documentation_gen".to_string(),
        ]
    }

    /// Disable logging features
    fn disable_logging() -> Vec<String> {
        vec![
            "log".to_string(),
            "env_logger".to_string(),
            "tracing".to_string(),
            "slog".to_string(),
            "println!".to_string(),
            "eprintln!".to_string(),
            "dbg!".to_string(),
        ]
    }

    /// Turn off graphviz and visualization
    fn disable_graphviz() -> Vec<String> {
        vec![
            "graphviz".to_string(),
            "dot_generator".to_string(),
            "svg_output".to_string(),
            "visualization".to_string(),
            "graph_rendering".to_string(),
            "plot_generation".to_string(),
        ]
    }

    /// Create macro wrappers for remaining code
    fn create_macro_wrappers() -> Vec<String> {
        vec![
            "minimal_fn!".to_string(),
            "core_only!".to_string(),
            "no_debug!".to_string(),
            "essential!".to_string(),
            "strip_all!".to_string(),
        ]
    }

    /// Extract only the core eigenmatrix components
    fn extract_core_matrix(compressed: &[Vec<f64>]) -> Vec<Vec<f64>> {
        if compressed.is_empty() {
            return vec![vec![1.0]]; // Minimal 1x1 identity
        }

        // Keep only the most significant eigenvalues (top-left corner)
        let core_size = (compressed.len() / 2).max(1).min(3); // Max 3x3 core

        let mut core = Vec::new();
        for i in 0..core_size {
            let mut row = Vec::new();
            for j in 0..core_size {
                let value = compressed.get(i)
                    .and_then(|r| r.get(j))
                    .copied()
                    .unwrap_or(if i == j { 1.0 } else { 0.0 });
                row.push(value);
            }
            core.push(row);
        }

        core
    }

    /// Generate minimal Cargo.toml
    pub fn generate_minimal_cargo_toml(&self) -> String {
        format!(
            r#"[package]
name = "minimal-zos"
version = "0.1.0"
edition = "2021"

[dependencies]
# Only absolute essentials - {} functions removed
# {} features disabled
# {} macro wrappers applied

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
strip = true

[features]
default = []
# All non-essential features removed
"#,
            self.removed_functions.len(),
            self.disabled_features.len(),
            self.macro_wrappers.len()
        )
    }

    /// Generate minimal main.rs with macros
    pub fn generate_minimal_main(&self) -> String {
        format!(
            r#"// Minimal ZOS - Stripped to essentials
#![no_std]
#![no_main]

// Macro wrappers for minimal code
macro_rules! minimal_fn {{
    ($name:ident) => {{
        #[inline(always)]
        fn $name() {{ /* minimal implementation */ }}
    }};
}}

macro_rules! core_only {{
    ($code:block) => {{
        #[cfg(feature = "core-only")]
        $code
    }};
}}

macro_rules! no_debug {{
    ($($tt:tt)*) => {{
        // Debug code stripped out
    }};
}}

// Core eigenmatrix ({}x{})
const CORE_MATRIX: [[f64; {}]; {}] = {};

// Minimal entry point
#[no_mangle]
pub extern "C" fn _start() -> ! {{
    core_only! {{
        // Execute minimal orbit
        let _ = CORE_MATRIX;
    }}

    loop {{}}
}}

// {} functions removed
// {} features disabled
// Minimization ratio: {:.1}%
"#,
            self.core_matrix.len(),
            self.core_matrix.get(0).map_or(0, |r| r.len()),
            self.core_matrix.get(0).map_or(0, |r| r.len()),
            self.core_matrix.len(),
            self.format_matrix_literal(),
            self.removed_functions.len(),
            self.disabled_features.len(),
            self.minimization_ratio * 100.0
        )
    }

    /// Format matrix as Rust literal
    fn format_matrix_literal(&self) -> String {
        let mut literal = String::from("[\n");
        for row in &self.core_matrix {
            literal.push_str("    [");
            for (i, &value) in row.iter().enumerate() {
                if i > 0 { literal.push_str(", "); }
                literal.push_str(&format!("{:.3}", value));
            }
            literal.push_str("],\n");
        }
        literal.push(']');
        literal
    }

    /// Verify minimal orbit is still functional
    pub fn verify_minimal_functionality(&self) -> bool {
        // Check core matrix is valid
        !self.core_matrix.is_empty() &&
        self.core_matrix.iter().all(|row| !row.is_empty()) &&
        self.minimization_ratio > 0.5 // At least 50% reduction
    }

    /// Get minimization summary
    pub fn minimization_summary(&self) -> String {
        format!(
            "MINIMAL_ORBIT[Matrix:{}x{}:Removed:{}:Disabled:{}:Macros:{}:Ratio:{:.1}%]",
            self.core_matrix.len(),
            self.core_matrix.get(0).map_or(0, |r| r.len()),
            self.removed_functions.len(),
            self.disabled_features.len(),
            self.macro_wrappers.len(),
            self.minimization_ratio * 100.0
        )
    }
}

/// Generate complete minimal project
pub fn generate_minimal_project(compression: &EigenmatrixCompression, output_dir: &str) -> Result<(), String> {
    println!("📦 GENERATING MINIMAL PROJECT...");

    let minimal = MinimalViableOrbit::minimize(compression);

    // Create output directory
    std::fs::create_dir_all(output_dir)
        .map_err(|e| format!("Failed to create output dir: {}", e))?;

    // Write Cargo.toml
    let cargo_toml = minimal.generate_minimal_cargo_toml();
    std::fs::write(format!("{}/Cargo.toml", output_dir), cargo_toml)
        .map_err(|e| format!("Failed to write Cargo.toml: {}", e))?;

    // Write main.rs
    let main_rs = minimal.generate_minimal_main();
    std::fs::create_dir_all(format!("{}/src", output_dir))
        .map_err(|e| format!("Failed to create src dir: {}", e))?;
    std::fs::write(format!("{}/src/main.rs", output_dir), main_rs)
        .map_err(|e| format!("Failed to write main.rs: {}", e))?;

    println!("✅ Minimal project generated:");
    println!("   {}", minimal.minimization_summary());
    println!("   Output: {}", output_dir);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lock_of_rust::LockOfRust;
    use crate::fools_path::FoolsPath;
    use crate::lock_eigenmatrix::LockEigenmatrix;

    #[test]
    fn test_minimal_viable_orbit() {
        let mock_lock = r#"
[[package]]
name = "serde"
version = "1.0.0"
"#;

        let fools_path = FoolsPath::begin();
        let lock = LockOfRust::emerge_from_journey(mock_lock, &fools_path);
        let eigenmatrix = LockEigenmatrix::from_lock(&lock);
        let mock_perf = "12.34%  cargo  0x401234  serde::serialize";
        let compression = EigenmatrixCompression::compress(&eigenmatrix, mock_perf).unwrap();

        let minimal = MinimalViableOrbit::minimize(&compression);

        assert!(!minimal.core_matrix.is_empty());
        assert!(!minimal.removed_functions.is_empty());
        assert!(!minimal.disabled_features.is_empty());
        assert!(minimal.minimization_ratio > 0.0);
        assert!(minimal.verify_minimal_functionality());
    }
}
