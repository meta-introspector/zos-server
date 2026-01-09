#!/bin/bash
# ZOS Cross-Compilation Script
# AGPL-3.0 License

set -euo pipefail

# Target architectures for our node inventory
TARGETS=(
    "x86_64-unknown-linux-gnu"      # Linux server
    "x86_64-pc-windows-gnu"         # Windows laptops
    "aarch64-linux-android"         # Android phones
    "aarch64-unknown-linux-gnu"     # ARM64 Linux
)

# Build directory
BUILD_DIR="target/cross-compiled"
mkdir -p "$BUILD_DIR"

echo "🔨 ZOS Cross-Compilation for Multi-Platform Deployment"
echo "======================================================"

# Install cross-compilation targets
echo "📦 Installing Rust targets..."
for target in "${TARGETS[@]}"; do
    echo "  Adding target: $target"
    rustup target add "$target" || echo "  ⚠️  Target $target may already be installed"
done

echo ""
echo "🏗️  Cross-compiling hwinfo plugin..."

cd zos-deploy

for target in "${TARGETS[@]}"; do
    echo "  Building for $target..."

    # Determine output extension
    if [[ "$target" == *"windows"* ]]; then
        ext=".exe"
    else
        ext=""
    fi

    # Cross-compile
    if cargo build --target "$target" --release; then
        # Copy to deployment directory
        cp "target/$target/release/zos_deploy$ext" "../$BUILD_DIR/zos_deploy-$target$ext"
        echo "  ✅ Success: zos_deploy-$target$ext"
    else
        echo "  ❌ Failed: $target"
    fi
done

cd ..

echo ""
echo "📋 Cross-compilation Summary:"
ls -la "$BUILD_DIR/"

echo ""
echo "🚀 Deployment binaries ready:"
echo "  - Linux server: zos_deploy-x86_64-unknown-linux-gnu"
echo "  - Windows laptops: zos_deploy-x86_64-pc-windows-gnu.exe"
echo "  - Android phones: zos_deploy-aarch64-linux-android"
echo "  - ARM64 Linux: zos_deploy-aarch64-unknown-linux-gnu"

echo ""
echo "📡 Next: Deploy to nodes with:"
echo "  scp $BUILD_DIR/zos_deploy-<target> user@node:/usr/local/bin/zos_deploy"
