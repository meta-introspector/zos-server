#!/bin/bash

# ZOS Development Server Launch Script
# Usage: ./launch-dev-server.sh [owner_address]

set -e

OWNER_ADDRESS=${1:-"dev_0x1234567890abcdef"}
SERVER_PORT=${2:-3000}

echo "🚀 Launching ZOS Development Server"
echo "Owner: $OWNER_ADDRESS"
echo "Port: $SERVER_PORT"

# Create dev environment
mkdir -p ~/.zos/dev/$OWNER_ADDRESS/{logs,data,audit,proposals}

# Launch server with eigenmatrix bootstrap
echo "📋 Loading eigenmatrix v1.0.0..."
cargo run --bin zos-dev-server -- \
    --owner "$OWNER_ADDRESS" \
    --port "$SERVER_PORT" \
    --eigenmatrix "eigenmatrix_v1.lock" \
    --dev-mode

echo ""
echo "✅ Development server launched!"
echo ""
echo "🌐 Access URLs:"
echo "  Dashboard: http://localhost:$SERVER_PORT/dev/$OWNER_ADDRESS"
echo "  Audit UI:  http://localhost:$SERVER_PORT/audit/$OWNER_ADDRESS"
echo "  Proposals: http://localhost:$SERVER_PORT/proposals/$OWNER_ADDRESS"
echo ""
echo "👥 Onboard stakeholders:"
echo "  curl -X POST http://localhost:$SERVER_PORT/onboard \\"
echo "    -d '{\"address\":\"0xSTAKEHOLDER\", \"role\":\"Auditor\"}'"
echo ""
echo "🚩 Flag components:"
echo "  curl -X POST http://localhost:$SERVER_PORT/flag \\"
echo "    -d '{\"component\":\"governance\", \"description\":\"Security concern\", \"severity\":\"High\"}'"
echo ""
echo "📋 Submit proposals:"
echo "  curl -X POST http://localhost:$SERVER_PORT/propose \\"
echo "    -d '{\"title\":\"Update rustc lock\", \"description\":\"...\", \"changes\":[...]}'"
echo ""
echo "📊 View status:"
echo "  curl http://localhost:$SERVER_PORT/status/$OWNER_ADDRESS"
