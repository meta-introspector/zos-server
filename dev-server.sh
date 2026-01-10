#!/bin/bash

# Development server with hot reload
echo "🔥 Starting ZOS Dev Server with Hot Reload"

# Function to build and restart server
restart_server() {
    echo "🔄 Rebuilding..."
    if cargo build --bin zos_server; then
        echo "✅ Build successful, restarting server..."
        pkill -f "zos_server serve" 2>/dev/null
        sleep 1
        nohup ./target/debug/zos_server serve > server.log 2>&1 &
        echo "🚀 Server restarted (PID: $!)"
    else
        echo "❌ Build failed, keeping old server running"
    fi
}

# Initial build and start
restart_server

# Watch for file changes
cargo watch -q -w src -x 'build --bin zos_server' -s 'pkill -f "zos_server serve"; sleep 1; nohup ./target/debug/zos_server serve > server.log 2>&1 &'
