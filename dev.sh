#!/bin/bash

echo "🔥 ZOS Dev Server - One Command Setup"

# Kill any existing processes
echo "🧹 Cleaning up existing processes..."
pkill -f zos-dev-server 2>/dev/null
pkill -f zos_server 2>/dev/null
sleep 1

# Build the dev server
echo "🔨 Building dev server..."
if ! cargo build --bin zos-dev-server --quiet; then
    echo "❌ Build failed!"
    exit 1
fi

# Start dev server in background
echo "🚀 Starting dev server daemon..."
nohup ./target/debug/zos-dev-server > /tmp/zos-dev.log 2>&1 &
DEV_PID=$!

# Wait a moment for startup
sleep 3

# Check if it's working
if curl -s http://localhost:8080/health >/dev/null 2>&1; then
    echo "✅ Dev server is running (PID: $DEV_PID)"
    echo "📝 Logs: tail -f /tmp/zos-dev.log"

    # Get login URL
    if ./target/debug/zos_server login alice 2>/dev/null | grep -q "Dashboard URL"; then
        URL=$(./target/debug/zos_server login alice 2>/dev/null | grep "Dashboard URL" | cut -d' ' -f3)
        echo "🌐 Dashboard: $URL"
    fi

    echo ""
    echo "🔥 Dev server is active with:"
    echo "   • Auto-rebuild on file changes"
    echo "   • Health monitoring & restart"
    echo "   • Error reporting to dashboard"
    echo "   • Background daemon process"
    echo ""
    echo "Edit files in src/ to see auto-reload in action!"
else
    echo "❌ Dev server failed to start"
    echo "📝 Check logs: tail /tmp/zos-dev.log"
    exit 1
fi
