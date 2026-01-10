#!/usr/bin/env nix-shell
#!nix-shell shell.nix -i bash

set -e

# Accept Android SDK license
export NIXPKGS_ACCEPT_ANDROID_SDK_LICENSE=1

echo "🔧 Setting up ZOS cross-compilation environment"

# Check available targets
echo "🎯 Available Rust targets:"
rustc --print target-list | grep -E "(windows|android|aarch64)" | head -10

# Check cross-compilation toolchains
echo "🛠️  Cross-compilation toolchains:"
echo "Windows (MinGW): $(which x86_64-w64-mingw32-gcc 2>/dev/null || echo 'Not found')"
echo "ARM64 Linux: $(which aarch64-unknown-linux-gnu-gcc 2>/dev/null || echo 'Not found')"

# Test build for current platform
echo "🧪 Testing build for current platform..."
cd zos-minimal-server
cargo build --release
cd ..

# Test Windows cross-compilation
echo "🪟 Testing Windows cross-compilation..."
cd zos-minimal-server
RUSTFLAGS="-C target-feature=+crt-static" cargo build --release --target x86_64-pc-windows-gnu || echo "❌ Windows build failed"
cd ..

# Test ARM64 Linux cross-compilation (for OCI)
echo "🦾 Testing ARM64 Linux cross-compilation (OCI)..."
cd zos-minimal-server
cargo build --release --target aarch64-unknown-linux-gnu || echo "❌ ARM64 Linux build failed"
cd ..

# Test Android cross-compilation
echo "🤖 Testing Android cross-compilation..."
cd zos-minimal-server
cargo build --release --target aarch64-linux-android || echo "❌ Android ARM64 build failed"
cargo build --release --target armv7-linux-androideabi || echo "❌ Android ARM32 build failed"
cd ..

echo "✅ Cross-compilation environment ready!"
echo "🚀 Run ./build-cross-platform.sh to build for all targets"
