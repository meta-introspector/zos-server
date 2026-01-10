#!/bin/bash
set -e

echo "🔄 Testing hot swap functionality..."

# Get current server info
echo "📋 Current server status:"
curl -s http://localhost:8080/health | jq .git

# Check if socat is available
if ! command -v socat &> /dev/null; then
    echo "⚠️  Installing socat for port forwarding..."
    sudo apt-get update && sudo apt-get install -y socat
fi

# Find current server process and port
CURRENT_PID=$(ps aux | grep zos-minimal-server | grep -v grep | head -1 | awk '{print $2}')
CURRENT_PORT=8080
NEW_PORT=9080

echo "🔌 Current PID: $CURRENT_PID, Port: $CURRENT_PORT -> New Port: $NEW_PORT"

# Start new server on different port
echo "🚀 Starting new server on port $NEW_PORT..."
ZOS_HTTP_PORT=$NEW_PORT ./target/release/zos-minimal-server &
NEW_PID=$!

sleep 3

# Test new server
if curl -s http://localhost:$NEW_PORT/health > /dev/null; then
    echo "✅ New server healthy on port $NEW_PORT"

    # Show both servers
    echo "📊 Server comparison:"
    echo "Old server (port $CURRENT_PORT):"
    curl -s http://localhost:$CURRENT_PORT/health | jq .git
    echo "New server (port $NEW_PORT):"
    curl -s http://localhost:$NEW_PORT/health | jq .git

    echo "🔄 Hot swap ready - new server PID: $NEW_PID"
else
    echo "❌ New server failed health check"
    kill $NEW_PID 2>/dev/null || true
    exit 1
fi
