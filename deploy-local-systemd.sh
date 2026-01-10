#!/bin/bash
set -e

echo "🚀 Deploying ZOS Server locally with systemd"

# Build the server
echo "📦 Building ZOS server..."
cargo build --release

# Create zos user
echo "👤 Creating zos user..."
sudo useradd -r -s /bin/false -d /opt/zos -m zos 2>/dev/null || echo "User zos already exists"

# Create directories
echo "📁 Setting up directories..."
sudo mkdir -p /opt/zos/{bin,data,config,logs}
sudo chown -R zos:zos /opt/zos

# Copy binary
echo "📋 Installing binary..."
sudo cp target/release/zos_server /opt/zos/bin/
sudo chmod +x /opt/zos/bin/zos_server

# Create systemd service
echo "🔧 Creating systemd service..."
sudo tee /etc/systemd/system/zos-server.service > /dev/null <<EOF
[Unit]
Description=ZOS Server - Zero Ontology System
After=network.target
Wants=network.target

[Service]
Type=simple
User=zos
Group=zos
WorkingDirectory=/opt/zos
ExecStart=/opt/zos/bin/zos_server
Restart=always
RestartSec=5
Environment=ZOS_HTTP_PORT=8080
Environment=ZOS_DATA_DIR=/opt/zos/data
Environment=ZOS_LOG_LEVEL=info

# Security settings
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/opt/zos/data /opt/zos/logs

[Install]
WantedBy=multi-user.target
EOF

# Reload systemd and enable service
echo "⚙️ Enabling service..."
sudo systemctl daemon-reload
sudo systemctl enable zos-server.service

# Start service
echo "🎯 Starting ZOS server..."
sudo systemctl start zos-server.service

# Show status
echo "📊 Service status:"
sudo systemctl status zos-server.service --no-pager

echo "✅ ZOS Server deployed successfully!"
echo "📝 Check logs: sudo journalctl -u zos-server.service -f"
echo "🔧 Control: sudo systemctl {start|stop|restart|status} zos-server.service"
