#!/bin/bash
# Deploy to Linux Server (12GB GPU)

echo "🖥️ Deploying to Linux Server..."

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source ~/.cargo/env

# Install WireGuard
sudo apt install -y wireguard

# Build tycoon system
cargo build --release --bin tycoon-server
cargo build --release --bin gpu-dashboard

# Install systemd services
sudo tee /etc/systemd/system/tycoon-server.service << 'SERVICE'
[Unit]
Description=Meta-Introspector Tycoon Server
After=network.target

[Service]
Type=simple
User=tycoon
ExecStart=/home/tycoon/meta-introspector-tycoon/target/release/tycoon-server
Restart=always

[Install]
WantedBy=multi-user.target
SERVICE

sudo systemctl enable tycoon-server
sudo systemctl start tycoon-server

echo "✅ Linux server deployed"
