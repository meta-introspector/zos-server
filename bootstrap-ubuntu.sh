#!/bin/bash
# ZOS Bootstrap Version 2: Ubuntu (Package Manager)
# Uses Ubuntu's package manager for dependencies

set -e

echo "🐧 ZOS Bootstrap v2: Ubuntu (Package Manager)"
echo "=============================================="

# Update package lists
echo "📦 Updating package lists..."
sudo apt update

# Install system dependencies
echo "🛠️ Installing system dependencies..."
sudo apt install -y \
    curl \
    git \
    build-essential \
    pkg-config \
    libssl-dev \
    libcurl4-openssl-dev \
    libgit2-dev \
    cmake \
    clang \
    llvm-dev

# Install Rust via package manager (if available) or rustup
if apt list --installed 2>/dev/null | grep -q "^rustc/"; then
    echo "🦀 Using system Rust..."
    sudo apt install -y rustc cargo
else
    echo "🦀 Installing Rust via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    # shellcheck source=/dev/null
    source ~/.cargo/env
fi

# Install additional tools
echo "🔧 Installing additional tools..."
sudo apt install -y \
    strace \
    perf-tools-unstable \
    valgrind \
    gdb

# Clone ZOS
echo "📥 Cloning ZOS repository..."
if [ ! -d "zos-server" ]; then
    git clone https://github.com/meta-introspector/zos-server.git
fi

cd zos-server

# Set up environment
echo "🌍 Setting up environment..."
export RUST_BACKTRACE=1
export CARGO_TARGET_DIR="target"

# Build with all features
echo "🔨 Building ZOS Server with all features..."
cargo build --release --all-features

# Install system-wide (optional)
echo "📦 Installing ZOS system-wide..."
sudo cp target/release/zos_server /usr/local/bin/
sudo chmod +x /usr/local/bin/zos_server

# Create systemd service (optional)
echo "🔧 Creating systemd service..."
sudo tee /etc/systemd/system/zos-server.service > /dev/null <<EOF
[Unit]
Description=ZOS Server - Zero Ontology System
After=network.target

[Service]
Type=simple
User=zos
WorkingDirectory=/opt/zos-server
ExecStart=/usr/local/bin/zos_server
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF

echo "✅ ZOS Bootstrap v2 Complete!"
echo "🚀 Run: zos_server (system-wide)"
echo "🔧 Or: sudo systemctl start zos-server"
echo "📊 Status: sudo systemctl status zos-server"
