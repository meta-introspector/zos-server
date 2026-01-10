#!/bin/bash
# ZOS CI/CD Pipeline Test - Dev -> QA -> Prod

set -e

echo "🚀 ZOS CI/CD Pipeline Test - $(date)"
echo "===================================="

# Configuration
DEV_PORT=8080
QA_PORT=8082
# PROD_PORT=8084  # Reserved for future production testing
TEST_BRANCH="qa"

# Function to wait for service
wait_for_service() {
    local port=$1
    local name=$2
    local max_attempts=30

    echo "⏳ Waiting for $name service on port $port..."
    for i in $(seq 1 $max_attempts); do
        if curl -s --max-time 2 "http://localhost:$port/health" >/dev/null 2>&1; then
            echo "✅ $name service is ready"
            return 0
        fi
        echo "   Attempt $i/$max_attempts..."
        sleep 2
    done
    echo "❌ $name service failed to start"
    return 1
}

# Function to test API endpoint
test_api() {
    local port=$1
    local endpoint=$2
    local name=$3

    echo "🧪 Testing $name API: $endpoint"
    if curl -s --max-time 5 "http://localhost:$port$endpoint" | jq . >/dev/null 2>&1; then
        echo "✅ $name API test passed"
        return 0
    else
        echo "❌ $name API test failed"
        return 1
    fi
}

# Stage 1: Start Dev Server
echo ""
echo "💻 Stage 1: Development Server"
echo "-----------------------------"

# Build dev server
cd zos-minimal-server
cargo build --release
cd ..

# Start dev server
echo "Starting dev server on port $DEV_PORT..."
ZOS_HTTP_PORT=$DEV_PORT ./zos-minimal-server/target/release/zos-minimal-server &
DEV_PID=$!
echo "Dev Server PID: $DEV_PID"

# Wait for dev server
wait_for_service $DEV_PORT "Dev"

# Stage 2: Install QA Service
echo ""
echo "🔧 Stage 2: QA Service Installation"
echo "----------------------------------"

echo "Installing QA service via dev server..."
curl -X POST "http://localhost:$DEV_PORT/install/qa-service" | jq .

# Wait for QA service to be ready
sleep 10
wait_for_service $QA_PORT "QA"

# Stage 3: Test QA Service
echo ""
echo "🧪 Stage 3: QA Service Testing"
echo "-----------------------------"

test_api $QA_PORT "/health" "QA Health"
test_api $QA_PORT "/api/status" "QA Status"

# Stage 4: Simulate Code Changes
echo ""
echo "📝 Stage 4: Code Change Simulation"
echo "---------------------------------"

# Make a test change
echo "// Test change - $(date)" >> zos-minimal-server/src/main.rs

# Commit changes
git add .
git commit -m "Test pipeline change - $(date +%s)" || echo "Nothing to commit"

# Push to QA branch (simulate)
echo "📤 Simulating push to $TEST_BRANCH branch..."

# Stage 5: Update QA from Git
echo ""
echo "🔄 Stage 5: QA Update from Git"
echo "-----------------------------"

echo "Triggering QA update via dev server..."
curl -X POST "http://localhost:$DEV_PORT/manage/qa/update" | jq .

# Wait for QA to restart
sleep 15
wait_for_service $QA_PORT "QA (after update)"

# Stage 6: QA Validation
echo ""
echo "✅ Stage 6: QA Validation"
echo "------------------------"

test_api $QA_PORT "/health" "QA Health (post-update)"

# Stage 7: Production Update
echo ""
echo "🏭 Stage 7: Production Update"
echo "----------------------------"

echo "Triggering production update via QA server..."
curl -X POST "http://localhost:$QA_PORT/manage/production/update" | jq . || echo "⚠️  Production update endpoint may not exist yet"

# Stage 8: Cleanup
echo ""
echo "🧹 Stage 8: Cleanup"
echo "------------------"

echo "Stopping dev server..."
kill $DEV_PID 2>/dev/null || true

echo "Stopping QA service..."
sudo systemctl stop zos-qa.service 2>/dev/null || echo "QA service not running"

echo ""
echo "🎉 Pipeline Test Summary"
echo "======================="
echo "✅ Dev server: STARTED"
echo "✅ QA installation: COMPLETED"
echo "✅ QA service: TESTED"
echo "✅ Code changes: SIMULATED"
echo "✅ QA update: TRIGGERED"
echo "✅ QA validation: PASSED"
echo "⚠️  Production update: SIMULATED"
echo ""
echo "Pipeline test completed successfully!"
