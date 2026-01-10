#!/usr/bin/env nix-shell
#!nix-shell -i bash -p rustc cargo pkg-config openssl git

set -e

echo "🚀 Deploying ZOS with Nix environment"

# Stop existing systemd service if running
sudo systemctl stop zos-server.service 2>/dev/null || true

# Build with Nix
echo "📦 Building ZOS server with Nix..."
nix-build default.nix

# Create user and directories
echo "👤 Setting up ZOS user and directories..."
sudo useradd -r -s /bin/false -d /var/lib/zos -m zos 2>/dev/null || echo "User zos already exists"
sudo mkdir -p /var/lib/zos/{data,logs}
sudo chown -R zos:zos /var/lib/zos

# Install binary
echo "📋 Installing ZOS binary..."
sudo mkdir -p /opt/zos/bin
sudo cp result/bin/zos-minimal-server /opt/zos/bin/
sudo chmod +x /opt/zos/bin/zos-minimal-server

# Create systemd service with Nix environment
echo "🔧 Creating systemd service with Nix environment..."
sudo tee /etc/systemd/system/zos-server.service > /dev/null <<EOF
[Unit]
Description=ZOS Server - Zero Ontology System (Nix)
After=network.target
Wants=network.target

[Service]
Type=simple
User=zos
Group=zos
WorkingDirectory=/var/lib/zos
ExecStart=/opt/zos/bin/zos-minimal-server
Restart=always
RestartSec=5

# Environment with Nix paths
Environment=ZOS_HTTP_PORT=8080
Environment=ZOS_DATA_DIR=/var/lib/zos/data
Environment=ZOS_LOG_LEVEL=info
Environment=PATH=/run/current-system/sw/bin:/nix/var/nix/profiles/default/bin:\$PATH
Environment=NIX_PATH=nixpkgs=/nix/var/nix/profiles/per-user/root/channels/nixpkgs

# Security settings
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/zos

[Install]
WantedBy=multi-user.target
EOF

# Enable and start service
echo "⚙️ Enabling and starting ZOS service..."
sudo systemctl daemon-reload
sudo systemctl enable zos-server.service
sudo systemctl start zos-server.service

# Show status
echo "📊 Service status:"
sudo systemctl status zos-server.service --no-pager

echo "✅ ZOS Server deployed with Nix!"
echo "🔧 Cargo and Rust available for self-deployment"
echo "📝 Check logs: sudo journalctl -u zos-server.service -f"
