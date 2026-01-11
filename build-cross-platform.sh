#!/usr/bin/env bash
#!nix-shell shell.nix -i bash

set -e

# Accept Android SDK license
export NIXPKGS_ACCEPT_ANDROID_SDK_LICENSE=1

echo "🌍 ZOS Cross-Platform Build System"

# Available targets
TARGETS=(
    "x86_64-unknown-linux-gnu"      # Linux x64 (host)
    "x86_64-pc-windows-gnu"         # Windows x64
    "aarch64-unknown-linux-gnu"     # ARM64 Linux (OCI)
    "aarch64-linux-android"         # Android ARM64
    "armv7-linux-androideabi"       # Android ARM32
)

# Build function
build_target() {
    local target=$1
    local output_dir="target/cross-builds/$target"

    echo "🔨 Building for $target..."

    cd zos-minimal-server

    case $target in
        "x86_64-pc-windows-gnu")
            # Windows build with static linking
            RUSTFLAGS="-C target-feature=+crt-static" \
            cargo build --release --target $target
            ;;
        "aarch64-unknown-linux-gnu")
            # ARM64 Linux build for OCI
            cargo build --release --target $target
            ;;
        "aarch64-linux-android"|"armv7-linux-androideabi")
            # Android builds
            echo "🤖 Building for Android ($target)..."
            cargo build --release --target $target
            ;;
        *)
            # Default build
            cargo build --release --target $target
            ;;
    esac

    # Create output directory
    mkdir -p "../$output_dir"

    # Copy binary with appropriate extension
    if [[ $target == *"windows"* ]]; then
        cp "target/$target/release/zos-minimal-server.exe" "../$output_dir/"
        echo "✅ Windows binary: $output_dir/zos-minimal-server.exe"
    else
        cp "target/$target/release/zos-minimal-server" "../$output_dir/"
        echo "✅ Binary: $output_dir/zos-minimal-server"
    fi

    cd ..
}

# Build all targets or specific target
if [ $# -eq 0 ]; then
    echo "🚀 Building for all supported targets..."
    for target in "${TARGETS[@]}"; do
        build_target "$target" || echo "❌ Failed to build for $target"
    done
else
    echo "🎯 Building for specific target: $1"
    build_target "$1"
fi

echo "📦 Cross-compilation complete!"
echo "📁 Binaries available in target/cross-builds/"

# Create deployment package
echo "📦 Creating deployment package..."
cd target/cross-builds
tar -czf "../zos-cross-platform-$(date +%Y%m%d-%H%M%S).tar.gz" */
echo "✅ Deployment package created: target/zos-cross-platform-*.tar.gz"
