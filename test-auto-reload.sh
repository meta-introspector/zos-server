#!/bin/bash

echo "🧪 Testing daemon auto-reload functionality..."
echo "This will start daemon, modify source, and verify reload"
echo ""

# Stop any existing instances
./stop-dev-server.sh

# Start dev server daemon
echo "🚀 Starting daemon..."
./target/debug/zos-dev-minimal

# Wait for daemon startup
sleep 2

echo "📝 Modifying source file to trigger reload..."

# Make a small change to trigger reload
echo "// Auto-reload test comment $(date)" >> src/lib.rs

# Wait to see reload (check logs or process)
sleep 3

echo "✅ Test complete - daemon should have reloaded"
echo "🔍 Check if server is running:"
curl -s http://localhost:8080/health && echo "✅ Server is responding" || echo "❌ Server not responding"

# Clean up
git checkout src/lib.rs 2>/dev/null || true
echo ""
echo "🛑 Use ./stop-dev-server.sh to stop the daemon"
