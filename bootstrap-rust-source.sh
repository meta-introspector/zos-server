#!/bin/bash
# ZOS Bootstrap Version 4: Git x.py (Rust from Source)
# Builds Rust compiler from source, then builds ZOS

set -e

echo "🔥 ZOS Bootstrap v4: Git x.py (Rust from Source)"
echo "================================================"

# Install system dependencies
echo "🛠️ Installing build dependencies..."
if command -v apt &> /dev/null; then
    sudo apt update
    sudo apt install -y git curl build-essential cmake python3 ninja-build
elif command -v yum &> /dev/null; then
    sudo yum install -y git curl gcc gcc-c++ cmake python3 ninja-build
elif command -v pacman &> /dev/null; then
    sudo pacman -S git curl base-devel cmake python ninja
fi

# Clone Rust source
echo "📥 Cloning Rust source code..."
if [ ! -d "rust" ]; then
    git clone https://github.com/rust-lang/rust.git
fi

cd rust

# Configure Rust build
echo "⚙️ Configuring Rust build..."
cat > config.toml << 'EOF'
[llvm]
download-ci-llvm = true

[build]
docs = false
compiler-docs = false
extended = true
tools = ["cargo", "clippy", "rustfmt"]

[install]
prefix = "/usr/local"

[rust]
channel = "stable"
debug = false
EOF

# Build Rust from source
echo "🔨 Building Rust from source (this takes a while)..."
python3 x.py build --stage 2

# Install Rust
echo "📦 Installing Rust..."
sudo python3 x.py install

# Update PATH
export PATH="/usr/local/bin:$PATH"

# Verify installation
echo "✅ Verifying Rust installation..."
rustc --version
cargo --version

# Go back to parent directory
cd ..

# Clone ZOS
echo "📥 Cloning ZOS repository..."
if [ ! -d "zos-server" ]; then
    git clone https://github.com/meta-introspector/zos-server.git
fi

cd zos-server

# Build ZOS with fresh Rust
echo "🔨 Building ZOS with source-built Rust..."
cargo build --release

# Run comprehensive tests
echo "🧪 Running comprehensive tests..."
cargo test --release --all-features

# Performance test
echo "📊 Running performance tests..."
time cargo build --release

# Memory usage test
echo "💾 Testing memory usage..."
valgrind --tool=memcheck --leak-check=full ./target/release/zos_server --help 2>&1 | head -20 || echo "Valgrind not available"

echo "✅ ZOS Bootstrap v4 Complete!"
echo "🔥 Built with Rust from source!"
echo "🚀 Run: ./target/release/zos_server"
echo "⚡ Maximum performance build achieved!"
