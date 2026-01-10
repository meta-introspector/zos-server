#!/bin/bash
# ZOS Pipeline Test - Proper CI/CD Flow

set -e

echo "🧪 ZOS CI/CD Pipeline Test - $(date)"
echo "===================================="

# Configuration
DEV_PORT=8080
QA_PORT=8082
PROD_PORT=8084

# Function to wait for service
wait_for_service() {
    local port=$1
    local name=$2
    local max_attempts=15

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

# Function to test API
test_api() {
    local port=$1
    local endpoint=$2
    local name=$3

    echo "🧪 Testing $name: $endpoint"
    local response=$(curl -s --max-time 5 "http://localhost:$port$endpoint" || echo "FAILED")
    if [[ "$response" != "FAILED" ]] && echo "$response" | jq . >/dev/null 2>&1; then
        echo "✅ $name test passed"
        return 0
    else
        echo "❌ $name test failed"
        return 1
    fi
}

# Stage 1: Start Development Server
echo ""
echo "💻 Stage 1: Development Server"
echo "-----------------------------"

# Build and start dev server
cd zos-minimal-server
cargo build --release
cd ..

echo "Starting dev server on port $DEV_PORT..."
ZOS_HTTP_PORT=$DEV_PORT ./zos-minimal-server/target/release/zos-minimal-server &
DEV_PID=$!
echo "Dev Server PID: $DEV_PID"

# Wait for dev server
wait_for_service $DEV_PORT "Dev"

# Stage 2: Install QA Service
echo ""
echo "🔧 Stage 2: Install QA Service"
echo "------------------------------"

echo "Installing QA service..."
curl -X POST "http://localhost:$DEV_PORT/install/qa-service" | jq . || echo "QA installation triggered"

# Wait for QA service
sleep 15
wait_for_service $QA_PORT "QA"

# Stage 3: Test Current State
echo ""
echo "🧪 Stage 3: Test Current State"
echo "-----------------------------"

test_api $DEV_PORT "/health" "Dev Health"
test_api $QA_PORT "/health" "QA Health"

# Stage 4: Simulate Code Change and Update
echo ""
echo "📝 Stage 4: Code Change & QA Update"
echo "----------------------------------"

# Make a test change
echo "// Pipeline test change - $(date +%s)" >> zos-minimal-server/src/main.rs

# Commit changes
git add .
git commit -m "Pipeline test change - $(date +%s)" || echo "Nothing new to commit"

# Trigger QA update
echo "Triggering QA update..."
curl -X POST "http://localhost:$DEV_PORT/manage/qa/update" | jq . || echo "QA update triggered"

# Wait for QA to restart
sleep 20
wait_for_service $QA_PORT "QA (after update)"

# Stage 5: Final Validation
echo ""
echo "✅ Stage 5: Final Validation"
echo "---------------------------"

test_api $DEV_PORT "/health" "Dev Health (final)"
test_api $QA_PORT "/health" "QA Health (final)"

# Stage 6: Cleanup
echo ""
echo "🧹 Stage 6: Cleanup"
echo "------------------"

echo "Stopping dev server..."
kill $DEV_PID 2>/dev/null || true

echo "Stopping QA service..."
sudo systemctl stop zos-qa.service 2>/dev/null || echo "QA service already stopped"

# Revert test change
git reset --hard HEAD~1 2>/dev/null || echo "No commits to revert"

echo ""
echo "🎉 Pipeline Test Summary"
echo "======================="
echo "✅ Dev server: STARTED & TESTED"
echo "✅ QA service: INSTALLED & TESTED"
echo "✅ Code change: SIMULATED"
echo "✅ QA update: TRIGGERED & VALIDATED"
echo "✅ Cleanup: COMPLETED"
echo ""
echo "Pipeline test completed successfully!"
echo "Ready for production deployment testing."
