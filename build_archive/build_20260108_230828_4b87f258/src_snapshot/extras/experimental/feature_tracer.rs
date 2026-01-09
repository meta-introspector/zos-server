use crate::feature_lattice::{FeatureLattice, OrbitWarp};
use std::process::Command;

pub struct FeatureTracer {
    lattice: FeatureLattice,
}

impl FeatureTracer {
    pub fn new() -> Self {
        let mut lattice = FeatureLattice::new();
        lattice.build_lattice();
        Self { lattice }
    }

    pub fn trace_all_features(&mut self) -> Result<String, String> {
        println!("🔬 Tracing execution of all feature flags...");

        // Create tapestry weaver for Gödel emoji encoding
        let weaver = crate::godel_emoji_tapestry::ExecutionTapestryWeaver::new();

        let mut results = String::new();
        results.push_str("# Feature Flag Orbit Warp Analysis\n\n");

        let mut all_functions = Vec::new();

        for feature in self.lattice.features.clone() {
            println!("📡 Tracing feature: {}", feature);

            // Build with feature enabled
            let build_result = self.build_with_feature(&feature)?;
            results.push_str(&format!("## Feature: {}\n", feature));
            results.push_str(&format!("Build result: {}\n", build_result));

            // Add to function trace for tapestry
            all_functions.push(format!("feature_{}", feature));

            // Trace orbit warp
            match self.lattice.trace_feature_execution(&feature) {
                Ok(warp) => {
                    results.push_str(&self.format_warp_analysis(&warp));
                },
                Err(e) => {
                    results.push_str(&format!("Error tracing {}: {}\n", feature, e));
                }
            }
            results.push_str("\n");
        }

        // Generate Gödel emoji tapestry
        let tapestry = weaver.weave_execution_tapestry(&all_functions);
        let tapestry_story = weaver.read_tapestry_story(&tapestry);
        let compressed = weaver.compress_tapestry(&tapestry);

        results.push_str("# 🎭 Gödel Emoji Tapestry\n\n");
        results.push_str(&format!("**Compressed Tapestry**: {}\n\n", compressed));
        results.push_str(&tapestry_story);
        results.push_str("\n");

        // Generate mathematical proof
        results.push_str(&self.lattice.prove_orbit_warping());

        Ok(results)
    }

    fn build_with_feature(&self, feature: &str) -> Result<String, String> {
        let output = Command::new("cargo")
            .args(&["build", "--features", feature, "--quiet"])
            .output()
            .map_err(|e| format!("Failed to run cargo: {}", e))?;

        if output.status.success() {
            Ok("✅ Build successful".to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Ok(format!("❌ Build failed: {}", stderr.lines().next().unwrap_or("Unknown error")))
        }
    }

    fn format_warp_analysis(&self, warp: &OrbitWarp) -> String {
        let mut analysis = String::new();

        analysis.push_str("### Orbit Warp Analysis\n");
        analysis.push_str(&format!("- **Warp Magnitude**: {:.6}\n", warp.warp_magnitude));
        analysis.push_str(&format!("- **Pre-orbit Norm**: {:.6}\n", warp.pre_orbit.norm()));
        analysis.push_str(&format!("- **Post-orbit Norm**: {:.6}\n", warp.post_orbit.norm()));

        let change_percent = (warp.warp_magnitude / warp.pre_orbit.norm()) * 100.0;
        analysis.push_str(&format!("- **Orbit Change**: {:.2}%\n", change_percent));

        analysis.push_str("\n### Execution Trace\n");
        for (i, trace) in warp.execution_trace.iter().enumerate() {
            analysis.push_str(&format!("{}. {}\n", i + 1, trace));
        }

        // Mathematical classification
        analysis.push_str("\n### Mathematical Classification\n");
        if change_percent > 50.0 {
            analysis.push_str("- **Type**: Major orbit transformation\n");
            analysis.push_str("- **Effect**: Fundamental space warping\n");
        } else if change_percent > 10.0 {
            analysis.push_str("- **Type**: Moderate orbit perturbation\n");
            analysis.push_str("- **Effect**: Significant local warping\n");
        } else {
            analysis.push_str("- **Type**: Minor orbit adjustment\n");
            analysis.push_str("- **Effect**: Subtle space curvature\n");
        }

        analysis
    }

    pub fn generate_lattice_matrix(&self) -> String {
        let mut output = String::new();
        output.push_str("# Feature Lattice Adjacency Matrix\n\n");

        // Header
        output.push_str("```\n");
        output.push_str("     ");
        for (i, feature) in self.lattice.features.iter().enumerate() {
            output.push_str(&format!("{:>3}", i));
        }
        output.push_str("\n");

        // Matrix rows
        for (i, feature) in self.lattice.features.iter().enumerate() {
            output.push_str(&format!("{:>3}: ", i));
            for j in 0..self.lattice.features.len() {
                let val = self.lattice.adjacency_matrix[(i, j)];
                output.push_str(&format!("{:>3}", if val > 0.0 { "1" } else { "0" }));
            }
            output.push_str(&format!("  # {}\n", feature));
        }
        output.push_str("```\n\n");

        // Orbit vector
        output.push_str("## Current Orbit Vector\n\n");
        output.push_str("```\n");
        for (i, feature) in self.lattice.features.iter().enumerate() {
            output.push_str(&format!("{}: {:>8.3}  # {}\n", i, self.lattice.orbit_vector[i], feature));
        }
        output.push_str("```\n");

        output
    }
}
