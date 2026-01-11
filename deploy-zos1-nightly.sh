#!/usr/bin/env bash
#!nix-shell shell.nix -i bash

set -e

# Accept Android SDK license
export NIXPKGS_ACCEPT_ANDROID_SDK_LICENSE=1

echo "🚀 Deploying ZOS1 with Nix nightly Rust"

# Check Rust version
echo "🦀 Rust version:"
rustc --version

# Stop existing service
sudo systemctl stop zos-server.service 2>/dev/null || true

# Build with nightly Rust
echo "📦 Building ZOS server with nightly Rust..."
cd zos-minimal-server
rm -f Cargo.lock
cargo build --release
cd ..

# Update service to use nix-shell
echo "🔧 Updating systemd service..."
sudo tee /etc/systemd/system/zos-server.service > /dev/null <<EOF
[Unit]
Description=ZOS1 Server - Zero Ontology System (Nix Nightly)
After=network.target
Wants=network.target

[Service]
Type=simple
User=zos
Group=zos
WorkingDirectory=/var/lib/zos
ExecStart=/usr/bin/env nix-shell $(pwd)/shell.nix --run '/opt/zos/bin/zos-minimal-server'
Restart=always
RestartSec=5

Environment=ZOS_HTTP_PORT=8080
Environment=ZOS_DATA_DIR=/var/lib/zos/data
Environment=ZOS_LOG_LEVEL=info

# Security settings
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/zos

[Install]
WantedBy=multi-user.target
EOF

# Copy new binary
echo "📋 Installing updated binary..."
sudo cp target/release/zos-minimal-server /opt/zos/bin/
sudo chmod +x /opt/zos/bin/zos-minimal-server

# Make shell.nix accessible to zos user
sudo cp shell.nix /var/lib/zos/
sudo chown zos:zos /var/lib/zos/shell.nix

# Update service to use local shell.nix
sudo sed -i 's|$(pwd)/shell.nix|/var/lib/zos/shell.nix|g' /etc/systemd/system/zos-server.service

# Restart service
echo "⚙️ Restarting ZOS1 service..."
sudo systemctl daemon-reload
sudo systemctl start zos-server.service

echo "📊 Service status:"
sudo systemctl status zos-server.service --no-pager

echo "✅ ZOS1 updated with nightly Rust!"
echo "🔧 Now has cargo/nightly rust for ZOS2 deployment"
