#!/bin/bash
# ZOS Instance Generator - Creates instance-specific configs
# AGPL-3.0 License

set -euo pipefail

ACCOUNT="solfunmeme"
INSTANCE_ID="1"
COORDINATOR_HOST="solana.solfunmeme.com"
LIBP2P_PORT="4001"
TIMESTAMP=$(date +%s)

echo "🏭 ZOS Instance Generator"
echo "========================"
echo "Account: $ACCOUNT"
echo "Instance: $INSTANCE_ID"
echo "Coordinator: $COORDINATOR_HOST"

# Generate unique peer ID for this instance
PEER_ID="12D3KooW$(openssl rand -hex 20)"
PRIVATE_KEY=$(openssl rand -base64 32)

# Create instance directory
INSTANCE_DIR="accounts/$ACCOUNT/instances/instance-$INSTANCE_ID"
mkdir -p "$INSTANCE_DIR"

# Generate instance-specific network config
cat > "$INSTANCE_DIR/network.toml" <<EOF
# ZOS Network Configuration - Instance $INSTANCE_ID
# Generated: $(date -Iseconds)
# AGPL-3.0 License

[instance]
id = "$INSTANCE_ID"
account = "$ACCOUNT"
role = "coordinator"
created = $TIMESTAMP
coordinator_host = "$COORDINATOR_HOST"

[libp2p]
peer_id = "$PEER_ID"
listen_addresses = [
    "/ip4/0.0.0.0/tcp/$LIBP2P_PORT",
    "/ip4/0.0.0.0/udp/$LIBP2P_PORT/quic-v1"
]
external_address = "/dns4/$COORDINATOR_HOST/tcp/$LIBP2P_PORT"
bootstrap_peers = []  # This IS the bootstrap peer

[network]
name = "$ACCOUNT-net"
bootstrap_url = "https://$COORDINATOR_HOST:$LIBP2P_PORT/p2p/$PEER_ID"
discovery_url = "https://github.com/$ACCOUNT/zos-bootstrap/raw/main/peers.json"
fallback_urls = [
    "https://$ACCOUNT.github.io/zos-bootstrap/",
    "https://zos-bootstrap.$ACCOUNT.workers.dev/"
]

[deployment]
binary_url = "https://github.com/$ACCOUNT/zos-server/releases/latest/download/"
config_url = "https://raw.githubusercontent.com/$ACCOUNT/zos-server/main/$INSTANCE_DIR/"
update_channel = "stable"

[security]
instance_key = "$PRIVATE_KEY"
require_auth = false
encryption_required = true
EOF

# Generate peer identity file
cat > "$INSTANCE_DIR/peer_identity.json" <<EOF
{
  "peer_id": "$PEER_ID",
  "private_key": "$PRIVATE_KEY",
  "public_key": "$(echo $PRIVATE_KEY | base64 -d | openssl dgst -sha256 -binary | base64)",
  "instance_id": "$INSTANCE_ID",
  "account": "$ACCOUNT",
  "created": $TIMESTAMP,
  "coordinator": true
}
EOF

# Generate bootstrap script for other nodes
cat > "$INSTANCE_DIR/join-network.sh" <<EOF
#!/bin/bash
# Auto-generated join script for $ACCOUNT network instance $INSTANCE_ID
# AGPL-3.0 License

COORDINATOR="$COORDINATOR_HOST:$LIBP2P_PORT"
PEER_ID="$PEER_ID"
ACCOUNT="$ACCOUNT"
INSTANCE="$INSTANCE_ID"

echo "🔗 Joining ZOS Network"
echo "======================"
echo "Account: \$ACCOUNT"
echo "Instance: \$INSTANCE"
echo "Coordinator: \$COORDINATOR"
echo "Bootstrap Peer: \$PEER_ID"

# Detect architecture
ARCH=\$(uname -m)
case \$ARCH in
    x86_64) TARGET="x86_64-unknown-linux-gnu" ;;
    aarch64) TARGET="aarch64-unknown-linux-gnu" ;;
    armv7l) TARGET="armv7-unknown-linux-gnueabihf" ;;
    *) echo "❌ Unsupported architecture: \$ARCH"; exit 1 ;;
esac

# Download appropriate binary
BINARY_URL="https://github.com/$ACCOUNT/zos-server/releases/latest/download/zos_deploy-\$TARGET"
echo "📦 Downloading binary for \$TARGET..."
curl -L "\$BINARY_URL" -o zos_deploy
chmod +x zos_deploy

# Join network
echo "🚀 Joining network..."
./zos_deploy join \\
    --coordinator "\$COORDINATOR" \\
    --peer-id "\$PEER_ID" \\
    --account "\$ACCOUNT" \\
    --instance "\$INSTANCE"
EOF

chmod +x "$INSTANCE_DIR/join-network.sh"

# Generate deployment manifest
cat > "$INSTANCE_DIR/deployment.json" <<EOF
{
  "account": "$ACCOUNT",
  "instance_id": "$INSTANCE_ID",
  "coordinator": {
    "host": "$COORDINATOR_HOST",
    "port": $LIBP2P_PORT,
    "peer_id": "$PEER_ID",
    "bootstrap_url": "https://$COORDINATOR_HOST:$LIBP2P_PORT/p2p/$PEER_ID"
  },
  "deployment": {
    "created": $TIMESTAMP,
    "version": "0.1.0",
    "config_url": "https://raw.githubusercontent.com/$ACCOUNT/zos-server/main/$INSTANCE_DIR/network.toml",
    "join_script": "https://raw.githubusercontent.com/$ACCOUNT/zos-server/main/$INSTANCE_DIR/join-network.sh"
  },
  "targets": {
    "linux_x86_64": "zos_deploy-x86_64-unknown-linux-gnu",
    "linux_aarch64": "zos_deploy-aarch64-unknown-linux-gnu",
    "android_aarch64": "zos_deploy-aarch64-linux-android",
    "windows_x86_64": "zos_deploy-x86_64-pc-windows-gnu.exe"
  }
}
EOF

echo ""
echo "✅ Instance $INSTANCE_ID generated successfully!"
echo ""
echo "📁 Files created:"
echo "  - $INSTANCE_DIR/network.toml"
echo "  - $INSTANCE_DIR/peer_identity.json"
echo "  - $INSTANCE_DIR/join-network.sh"
echo "  - $INSTANCE_DIR/deployment.json"
echo ""
echo "🌐 Network URLs:"
echo "  Bootstrap: https://$COORDINATOR_HOST:$LIBP2P_PORT/p2p/$PEER_ID"
echo "  Join Script: curl -sSL https://raw.githubusercontent.com/$ACCOUNT/zos-server/main/$INSTANCE_DIR/join-network.sh | bash"
echo ""
echo "🔑 Peer ID: $PEER_ID"
echo ""
echo "🚀 Next: Run bootstrap-network.sh to start the coordinator"
