#!/bin/bash
# Quick CI/CD Pipeline Test - Manual QA Setup

set -e

echo "🧪 Quick ZOS CI/CD Test - $(date)"
echo "================================"

DEV_PORT=8080
QA_PORT=8082

# Function to wait for service
wait_for_service() {
    local port=$1
    local name=$2

    echo "⏳ Waiting for $name service on port $port..."
    for i in {1..10}; do
        if curl -s --max-time 2 "http://localhost:$port/health" >/dev/null 2>&1; then
            echo "✅ $name service is ready"
            return 0
        fi
        echo "   Attempt $i/10..."
        sleep 2
    done
    echo "❌ $name service failed to start"
    return 1
}

# Stage 1: Start Dev Server
echo ""
echo "💻 Stage 1: Development Server"
echo "-----------------------------"

cd zos-minimal-server
if ! cargo build; then
    echo "❌ Dev server build failed"
    exit 1
fi
cd ..

echo "Starting dev server..."
cd zos-minimal-server
ZOS_HTTP_PORT=$DEV_PORT cargo run &
DEV_PID=$!
cd ..

if ! wait_for_service $DEV_PORT "Dev"; then
    echo "❌ Dev server failed"
    kill $DEV_PID 2>/dev/null || true
    exit 1
fi

# Stage 2: Manual QA Setup
echo ""
echo "🔧 Stage 2: Manual QA Setup"
echo "---------------------------"

# Start QA service manually
echo "Starting QA service manually..."
cd zos-minimal-server
ZOS_HTTP_PORT=$QA_PORT cargo run &
QA_PID=$!
cd ..

if ! wait_for_service $QA_PORT "QA"; then
    echo "❌ QA service failed"
    kill $DEV_PID $QA_PID 2>/dev/null || true
    exit 1
fi

# Stage 3: Test Both Services
echo ""
echo "🧪 Stage 3: Test Services"
echo "------------------------"

echo "Testing dev health..."
curl -s "http://localhost:$DEV_PORT/health" | jq .

echo "Testing QA health..."
curl -s "http://localhost:$QA_PORT/health" | jq .

# Stage 4: Cleanup
echo ""
echo "🧹 Stage 4: Cleanup"
echo "------------------"

kill $DEV_PID $QA_PID 2>/dev/null || true
echo "Services stopped"

echo ""
echo "✅ Quick pipeline test completed!"
