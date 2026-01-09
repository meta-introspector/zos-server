#!/bin/bash
# Move broken/experimental code to extras directory

set -euo pipefail

echo "🔧 Moving broken code to extras directory..."

# Create extras directory structure
mkdir -p src/extras/{broken,experimental,deprecated}

# List of potentially broken modules to move
BROKEN_MODULES=(
    "mini_sdf_server.rs"
    "blockchain_ingestor.rs"
    "universal_plugin_loader.rs"
)

EXPERIMENTAL_MODULES=(
    "feature_lattice.rs"
    "feature_tracer.rs"
    "execution_trace_analyzer.rs"
    "compiler_band_pass.rs"
    "compiler_polyfill_system.rs"
    "godel_emoji_tapestry.rs"
    "payment_intent_proof.rs"
)

# Move broken modules
echo "📦 Moving broken modules..."
for module in "${BROKEN_MODULES[@]}"; do
    if [ -f "src/${module}" ]; then
        echo "  Moving ${module} to extras/broken/"
        mv "src/${module}" "src/extras/broken/"
    fi
done

# Move experimental modules
echo "🧪 Moving experimental modules..."
for module in "${EXPERIMENTAL_MODULES[@]}"; do
    if [ -f "src/${module}" ]; then
        echo "  Moving ${module} to extras/experimental/"
        mv "src/${module}" "src/extras/experimental/"
    fi
done

# Update lib.rs to remove broken imports
echo "📝 Updating lib.rs..."
cp src/lib.rs src/lib.rs.backup

cat > src/lib.rs << 'EOF'
// ZOS Server - Zero Ontology System
// Core stable modules only

pub mod lmfdb_orbits;
#[macro_use]
pub mod orbit_macros;
pub mod zos_system;
pub mod automorphic_bootstrap;
pub mod rust_soul_eigenmatrix;

// Core plugins only
pub mod plugins;

// Optional modules for extended functionality
#[cfg(feature = "self-build")]
pub mod self_builder;
#[cfg(feature = "self-build")]
pub mod self_build_cli;

#[cfg(feature = "notebooklm")]
pub mod notebooklm_interface;
#[cfg(feature = "notebooklm")]
pub mod notebooklm_cli;

// Experimental modules (optional)
#[cfg(feature = "experimental")]
pub mod extras;

// Re-export orbit system
pub use lmfdb_orbits::*;
pub use orbit_macros::*;
EOF

# Create extras mod.rs
cat > src/extras/mod.rs << 'EOF'
// Experimental and broken modules
// Use at your own risk!

#[cfg(feature = "experimental-advanced")]
pub mod experimental {
    pub mod feature_lattice;
    pub mod feature_tracer;
    pub mod execution_trace_analyzer;
    pub mod compiler_band_pass;
    pub mod compiler_polyfill_system;
    pub mod godel_emoji_tapestry;
    pub mod payment_intent_proof;
}

#[cfg(feature = "broken-modules")]
pub mod broken {
    pub mod mini_sdf_server;
    pub mod blockchain_ingestor;
    pub mod universal_plugin_loader;
}
EOF

# Update Cargo.toml features
echo "⚙️ Updating Cargo.toml features..."
cp Cargo.toml Cargo.toml.backup

# Add experimental features to Cargo.toml
cat >> Cargo.toml << 'EOF'

[features]
default = []
self-build = ["uuid"]
notebooklm = ["reqwest"]
experimental = []
experimental-advanced = ["experimental", "nalgebra"]
broken-modules = ["experimental"]
all-features = ["self-build", "notebooklm", "experimental-advanced"]
EOF

echo "✅ Code reorganization complete!"
echo "📋 Summary:"
echo "  - Broken modules moved to src/extras/broken/"
echo "  - Experimental modules moved to src/extras/experimental/"
echo "  - lib.rs updated with stable core only"
echo "  - Cargo.toml updated with feature flags"
echo ""
echo "🔨 To build with experimental features:"
echo "  cargo build --features experimental-advanced"
echo ""
echo "🔨 To build core only (stable):"
echo "  cargo build --release"
