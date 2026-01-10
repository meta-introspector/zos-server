#!/bin/bash

# Kill any existing server on port 8080
echo "🛑 Stopping existing server on port 8080..."
pkill -f "zos_server serve" || true
pkill -f "cargo watch" || true
lsof -ti:8080 | xargs kill -9 2>/dev/null || true

# Start dev server with auto-reload and log capture
echo "🚀 Starting ZOS dev server with auto-reload..."
echo "📝 Logs will be captured in watch.log"
cargo watch -x 'run --bin zos_server serve' > watch.log 2>&1 &

echo "✅ Dev server started in background"
echo "📋 View logs: tail -f watch.log"
echo "🛑 Stop server: pkill -f 'cargo watch'"
