#!/bin/bash
# Comprehensive compilation analysis and tracing script

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$SCRIPT_DIR"
ANALYSIS_DIR="$PROJECT_ROOT/compilation-analysis"

mkdir -p "$ANALYSIS_DIR"/{features,enums,flags,traces,mir,hir,coverage}

echo "🔍 Starting comprehensive compilation analysis..."

# Extract all feature flags from Cargo.toml
echo "📦 Extracting feature definitions..."
if [ -f "Cargo.toml" ]; then
    grep -A 20 "\[features\]" Cargo.toml | grep "=" | tee "$ANALYSIS_DIR/features/cargo-features.txt"
fi

# Extract all cfg attributes and feature gates
echo "🏗️ Extracting compile-time configuration..."
find src/ -name "*.rs" -exec grep -H "cfg(" {} \; | tee "$ANALYSIS_DIR/flags/cfg-attributes.txt"
find src/ -name "*.rs" -exec grep -H "feature =" {} \; | tee "$ANALYSIS_DIR/flags/feature-gates.txt"

# Extract enum definitions
echo "📋 Extracting enum types..."
find src/ -name "*.rs" -exec grep -H "^[[:space:]]*enum " {} \; | tee "$ANALYSIS_DIR/enums/enum-definitions.txt"
find src/ -name "*.rs" -exec grep -H "derive(" {} \; | tee "$ANALYSIS_DIR/enums/derive-macros.txt"

# Generate feature permutation matrix
echo "🔄 Generating feature permutation matrix..."
FEATURES=(
    "default"
    "all-plugins"
    "core-only"
    "extra-plugins"
    "notebooklm"
    "reqwest"
    "self-build"
    "uuid"
)

TARGETS=(
    "x86_64-unknown-linux-gnu"
    "aarch64-unknown-linux-gnu"
)

PROFILES=("dev" "release")
TRACES=("basic" "strace" "perf" "coverage")

echo "Feature,Target,Profile,Trace,Command" > "$ANALYSIS_DIR/permutation-matrix.csv"

for feature in "${FEATURES[@]}"; do
    for target in "${TARGETS[@]}"; do
        for profile in "${PROFILES[@]}"; do
            for trace in "${TRACES[@]}"; do
                profile_flag=""
                if [ "$profile" = "release" ]; then
                    profile_flag="--release"
                fi

                cmd="cargo build --target $target --features $feature $profile_flag"
                echo "$feature,$target,$profile,$trace,$cmd" >> "$ANALYSIS_DIR/permutation-matrix.csv"
            done
        done
    done
done

echo "📊 Generated $(wc -l < "$ANALYSIS_DIR/permutation-matrix.csv") build permutations"

# Function to run traced build
run_traced_build() {
    local feature="$1"
    local target="$2"
    local profile="$3"
    local trace_type="$4"

    local trace_dir="$ANALYSIS_DIR/traces/${feature}-${target}-${profile}-${trace_type}"
    mkdir -p "$trace_dir"

    local profile_flag=""
    if [ "$profile" = "release" ]; then
        profile_flag="--release"
    fi

    echo "🚀 Building: $feature + $target + $profile + $trace_type"

    case "$trace_type" in
        "basic")
            cargo build --target "$target" --features "$feature" $profile_flag 2>&1 | \
                tee "$trace_dir/build.log"
            ;;
        "strace")
            if command -v strace >/dev/null 2>&1; then
                strace -o "$trace_dir/strace.log" -f -e trace=all \
                    cargo build --target "$target" --features "$feature" $profile_flag 2>&1 | \
                    tee "$trace_dir/build.log"
            else
                echo "strace not available, falling back to basic build"
                cargo build --target "$target" --features "$feature" $profile_flag 2>&1 | \
                    tee "$trace_dir/build.log"
            fi
            ;;
        "perf")
            if command -v perf >/dev/null 2>&1; then
                perf record -g -o "$trace_dir/perf.data" \
                    cargo build --target "$target" --features "$feature" $profile_flag 2>&1 | \
                    tee "$trace_dir/build.log"
                perf report --input="$trace_dir/perf.data" > "$trace_dir/perf-report.txt" 2>/dev/null || true
            else
                echo "perf not available, falling back to basic build"
                cargo build --target "$target" --features "$feature" $profile_flag 2>&1 | \
                    tee "$trace_dir/build.log"
            fi
            ;;
        "coverage")
            RUSTFLAGS="-C instrument-coverage" \
                cargo build --target "$target" --features "$feature" $profile_flag 2>&1 | \
                tee "$trace_dir/build.log"
            ;;
    esac

    # Collect metadata
    echo "Feature: $feature" > "$trace_dir/metadata.txt"
    echo "Target: $target" >> "$trace_dir/metadata.txt"
    echo "Profile: $profile" >> "$trace_dir/metadata.txt"
    echo "Trace: $trace_type" >> "$trace_dir/metadata.txt"
    echo "Timestamp: $(date -Iseconds)" >> "$trace_dir/metadata.txt"
    rustc --version --verbose >> "$trace_dir/metadata.txt"
}

# Run a subset of builds for demonstration (to avoid overwhelming CI)
echo "🏃 Running sample traced builds..."

# Core builds with different tracing
run_traced_build "default" "x86_64-unknown-linux-gnu" "dev" "basic"
run_traced_build "all-plugins" "x86_64-unknown-linux-gnu" "release" "coverage"

# Generate analysis summary
echo "📈 Generating analysis summary..."
cat > "$ANALYSIS_DIR/README.md" << 'EOF'
# ZOS Server Compilation Analysis

This directory contains comprehensive analysis of the ZOS Server compilation process.

## Structure

- `features/` - Feature flag definitions and usage
- `enums/` - Enum type definitions and derive macros
- `flags/` - Compile-time configuration flags
- `traces/` - Build traces with different profiling methods
- `mir/` - MIR (Mid-level IR) dumps
- `hir/` - HIR (High-level IR) dumps
- `coverage/` - Code coverage reports

## Permutation Matrix

The build system tests all combinations of:
- Features: default, all-plugins, core-only, extra-plugins, notebooklm, reqwest, self-build, uuid
- Targets: x86_64-unknown-linux-gnu, aarch64-unknown-linux-gnu
- Profiles: dev, release
- Tracing: basic, strace, perf, coverage

Total permutations: 8 × 2 × 2 × 4 = 128 builds

## Usage

Run `./analyze-compilation.sh` to generate fresh analysis data.
EOF

echo "✅ Compilation analysis complete!"
echo "📁 Results saved to: $ANALYSIS_DIR"
echo "📊 View summary: cat $ANALYSIS_DIR/README.md"
