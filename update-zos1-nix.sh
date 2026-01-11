#!/usr/bin/env bash
#!nix-shell -i bash -p rustc cargo pkg-config openssl git systemd

set -e

echo "🚀 Deploying ZOS1 with Nix shell environment"

# Stop existing service
sudo systemctl stop zos-server.service 2>/dev/null || true

# Build in Nix shell
echo "📦 Building ZOS server in Nix shell..."
cd zos-minimal-server
rm -f Cargo.lock
cargo build --release
cd ..

# Update service with Nix environment paths
echo "🔧 Updating systemd service with Nix paths..."
sudo tee /etc/systemd/system/zos-server.service > /dev/null <<EOF
[Unit]
Description=ZOS1 Server - Zero Ontology System (Nix Shell)
After=network.target
Wants=network.target

[Service]
Type=simple
User=zos
Group=zos
WorkingDirectory=/var/lib/zos
ExecStart=/usr/bin/env nix-shell --run '/opt/zos/bin/zos-minimal-server' -p rustc cargo pkg-config openssl git
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

# Restart service
echo "⚙️ Restarting ZOS1 service..."
sudo systemctl daemon-reload
sudo systemctl start zos-server.service

echo "📊 Service status:"
sudo systemctl status zos-server.service --no-pager

echo "✅ ZOS1 updated with Nix environment!"
echo "🔧 Now has cargo/rust available for ZOS2 deployment"
