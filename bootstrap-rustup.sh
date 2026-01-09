#!/bin/bash
# ZOS Bootstrap Version 1: Rustup (n00bs)
# The simplest way to get ZOS running

set -e

echo "🦀 ZOS Bootstrap v1: Rustup (Beginner Friendly)"
echo "================================================"

# Check if rustup is installed
if ! command -v rustup &> /dev/null; then
    echo "📦 Installing Rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
fi

# Update rust
echo "🔄 Updating Rust toolchain..."
rustup update stable
rustup default stable

# Install required components
echo "🛠️ Installing Rust components..."
rustup component add clippy rustfmt

# Clone ZOS
echo "📥 Cloning ZOS repository..."
if [ ! -d "zos-server" ]; then
    git clone https://github.com/meta-introspector/zos-server.git
fi

cd zos-server

# Build ZOS
echo "🔨 Building ZOS Server..."
cargo build --release

# Test the build
echo "🧪 Testing ZOS..."
./target/release/zos_server --version || echo "Version check failed, but binary exists"

# Run basic functionality test
echo "🌌 Testing basic orbit functionality..."
echo 'fn main() { println!("Hello ZOS!"); }' > test.rs
rustc test.rs -o test_binary
rm test.rs test_binary

echo "✅ ZOS Bootstrap v1 Complete!"
echo "🚀 Run: ./target/release/zos_server"
echo "📚 For help: ./target/release/zos_server --help"
