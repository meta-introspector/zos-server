#!/bin/bash
# ZOS Quick Install - Pull from running node
# Usage: curl -sSL https://raw.githubusercontent.com/your-repo/zos-server/main/quick-install.sh | bash
# Or: curl -sSL http://solana.solfunmeme.com:8080/install.sh | bash

set -e

ZOS_NODE="solana.solfunmeme.com:8080"

echo "🚀 ZOS Quick Installer"
echo "📡 Connecting to: $ZOS_NODE"
echo ""

# Test connection to ZOS node
echo "🔍 Testing connection..."
if ! curl -s --connect-timeout 10 "http://$ZOS_NODE/health" >/dev/null; then
    echo "❌ Cannot connect to ZOS node at $ZOS_NODE"
    echo "🔧 Please check:"
    echo "   - Network connectivity"
    echo "   - ZOS node is running"
    echo "   - Firewall settings"
    exit 1
fi

echo "✅ Connected to ZOS node"

# Get source information
echo "📋 Getting source information..."
SOURCE_INFO=$(curl -s "http://$ZOS_NODE/source")
echo "📦 $(echo "$SOURCE_INFO" | grep -o '"name":"[^"]*"' | cut -d'"' -f4)"
echo "🏷️  $(echo "$SOURCE_INFO" | grep -o '"version":"[^"]*"' | cut -d'"' -f4)"

# Download and run the installer
echo ""
echo "📥 Downloading installer from ZOS node..."
curl -sSL "http://$ZOS_NODE/install.sh" | bash

echo ""
echo "🎉 ZOS Installation from $ZOS_NODE complete!"
echo "🌐 Join the network: http://$ZOS_NODE"
