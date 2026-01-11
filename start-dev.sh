#!/bin/bash

# Kill any existing server processes
echo "🛑 Stopping existing server processes..."
pkill -f "zos_server" || true
pkill -f "zos-dev-launch" || true
lsof -ti:8080 | xargs kill -9 2>/dev/null || true

# Build the dev launcher
echo "🔨 Building zos-dev-launch..."
if ! cargo build --bin zos-dev-launch; then
    echo "❌ Build failed!"
    exit 1
fi

# Start the dev launcher with file watching
echo "🚀 Starting ZOS development server with hot reload..."
./target/debug/zos-dev-launch > devserver.log 2>&1 &
SERVER_PID=$!

# Wait a moment for startup
sleep 2

# Check if server is running
if ! kill -0 $SERVER_PID 2>/dev/null; then
    echo "❌ Server failed to start!"
    echo "📋 Error logs:"
    cat devserver.log
    exit 1
fi

# Test server response
if curl -s http://localhost:8080/health > /dev/null; then
    echo "✅ ZOS dev server running successfully"
    echo "🌐 Server: http://localhost:8080"
    echo "📋 Logs: tail -f devserver.log"
    echo "🛑 Stop: pkill -f 'zos-dev-launch'"
else
    echo "⚠️  Server started but not responding on port 8080"
fi

# Show recent logs
echo "📋 Recent logs:"
tail -10 devserver.log
