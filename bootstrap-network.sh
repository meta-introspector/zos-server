#!/bin/bash
# ZOS Bootstrap Script for solfunmeme network
# AGPL-3.0 License

set -euo pipefail

ACCOUNT="solfunmeme"
COORDINATOR="solana.solfunmeme.com"
LIBP2P_PORT="4001"
CONFIG_DIR="/etc/zos"
LOG_DIR="/var/log/zos"

echo "🚀 ZOS Network Bootstrap for $ACCOUNT"
echo "====================================="

# Create directories
sudo mkdir -p "$CONFIG_DIR" "$LOG_DIR"
sudo chown $USER:$USER "$LOG_DIR"

# Copy network configuration
echo "📋 Installing network configuration..."
sudo cp "accounts/$ACCOUNT/network.toml" "$CONFIG_DIR/"

# Generate libp2p identity if not exists
IDENTITY_FILE="$CONFIG_DIR/peer_identity.json"
if [[ ! -f "$IDENTITY_FILE" ]]; then
    echo "🔑 Generating libp2p peer identity..."
    # For now, create a placeholder - will be replaced with actual libp2p key generation
    sudo tee "$IDENTITY_FILE" > /dev/null <<EOF
{
  "peer_id": "12D3KooW$(openssl rand -hex 20)",
  "private_key": "$(openssl rand -base64 32)",
  "created": "$(date -Iseconds)"
}
EOF
    sudo chmod 600 "$IDENTITY_FILE"
fi

# Start ZOS node as systemd service
echo "🔧 Creating systemd service..."
sudo tee "/etc/systemd/system/zos-node.service" > /dev/null <<EOF
[Unit]
Description=ZOS Network Node
After=network.target
Wants=network.target

[Service]
Type=simple
User=$USER
WorkingDirectory=/home/$USER
ExecStart=/usr/local/bin/zos_deploy node --config $CONFIG_DIR/network.toml
Restart=always
RestartSec=5
Environment=ZOS_ACCOUNT=$ACCOUNT
Environment=ZOS_ROLE=coordinator
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
EOF

# Install the hwinfo binary
echo "📦 Installing ZOS deployment binary..."
sudo cp "target/debug/zos_deploy" "/usr/local/bin/"
sudo chmod +x "/usr/local/bin/zos_deploy"

# Enable and start service
echo "🎯 Starting ZOS node service..."
sudo systemctl daemon-reload
sudo systemctl enable zos-node
sudo systemctl start zos-node

# Wait for service to start
sleep 3

# Check status
echo ""
echo "📊 Service Status:"
sudo systemctl status zos-node --no-pager -l

# Show network info
echo ""
echo "🌐 Network Information:"
echo "  Coordinator: $COORDINATOR:$LIBP2P_PORT"
echo "  Peer ID: $(sudo cat $IDENTITY_FILE | grep peer_id | cut -d'"' -f4)"
echo "  Config: $CONFIG_DIR/network.toml"
echo "  Logs: journalctl -u zos-node -f"

# Create bootstrap info for other nodes
BOOTSTRAP_INFO="accounts/$ACCOUNT/bootstrap.sh"
cat > "$BOOTSTRAP_INFO" <<EOF
#!/bin/bash
# Auto-generated bootstrap script for $ACCOUNT network
# Run this on other nodes to join the network

COORDINATOR="$COORDINATOR:$LIBP2P_PORT"
PEER_ID="\$(sudo cat $IDENTITY_FILE | grep peer_id | cut -d'"' -f4)"

echo "Joining ZOS network..."
echo "Coordinator: \$COORDINATOR"
echo "Bootstrap Peer: \$PEER_ID"

# Download and run hwinfo
curl -L "https://github.com/solfunmeme/zos-server/releases/latest/download/zos_deploy-\$(uname -m)" -o zos_deploy
chmod +x zos_deploy
./zos_deploy join --coordinator "\$COORDINATOR" --peer-id "\$PEER_ID"
EOF

chmod +x "$BOOTSTRAP_INFO"

echo ""
echo "✅ Bootstrap complete!"
echo "📤 Share this with other nodes:"
echo "   curl -sSL https://raw.githubusercontent.com/solfunmeme/zos-server/main/$BOOTSTRAP_INFO | bash"
