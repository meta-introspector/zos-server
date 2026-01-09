#!/bin/bash
# Auto-generated join script for solfunmeme network instance 1
# AGPL-3.0 License

COORDINATOR="solana.solfunmeme.com:4001"
PEER_ID="12D3KooW8f630cc23eb3b786ea5aabcbeca3c38c5b1f2018"
ACCOUNT="solfunmeme"
INSTANCE="1"

echo "🔗 Joining ZOS Network"
echo "======================"
echo "Account: $ACCOUNT"
echo "Instance: $INSTANCE"
echo "Coordinator: $COORDINATOR"
echo "Bootstrap Peer: $PEER_ID"

# Detect architecture
ARCH=$(uname -m)
case $ARCH in
    x86_64) TARGET="x86_64-unknown-linux-gnu" ;;
    aarch64) TARGET="aarch64-unknown-linux-gnu" ;;
    armv7l) TARGET="armv7-unknown-linux-gnueabihf" ;;
    *) echo "❌ Unsupported architecture: $ARCH"; exit 1 ;;
esac

# Download appropriate binary
BINARY_URL="https://github.com/solfunmeme/zos-server/releases/latest/download/zos_deploy-$TARGET"
echo "📦 Downloading binary for $TARGET..."
curl -L "$BINARY_URL" -o zos_deploy
chmod +x zos_deploy

# Join network
echo "🚀 Joining network..."
./zos_deploy join \
    --coordinator "$COORDINATOR" \
    --peer-id "$PEER_ID" \
    --account "$ACCOUNT" \
    --instance "$INSTANCE"
