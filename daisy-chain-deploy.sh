#!/bin/bash
# ZOS Daisy Chain Test & Deploy - Each environment tests and promotes the next

set -e

echo "🔗 ZOS Daisy Chain Test & Deploy"
echo "   Dev → QA → Prod (with testing at each stage)"
echo ""

# Test function
test_server() {
    local port=$1
    local name=$2

    echo "🧪 Testing $name server (port $port)..."

    # Health check
    local health=$(curl -s "http://localhost:$port/health" | jq -r .status 2>/dev/null || echo "failed")
    if [ "$health" != "healthy" ]; then
        echo "❌ $name health check failed"
        return 1
    fi

    # Installer test
    if ! curl -s "http://localhost:$port/install.sh" | head -5 | grep -q "ZOS Universal Installer"; then
        echo "❌ $name installer test failed"
        return 1
    fi

    # Git info test (if available)
    local commit=$(curl -s "http://localhost:$port/health" | jq -r .git.commit_short 2>/dev/null || echo "")
    if [ -n "$commit" ] && [ "$commit" != "null" ] && [ "$commit" != '""' ]; then
        echo "✅ $name tests passed (commit: $commit)"
    else
        echo "✅ $name tests passed (no git info)"
    fi

    return 0
}

# Deploy and test function
deploy_and_test() {
    local endpoint=$1
    local target_port=$2
    local target_name=$3

    echo "📦 Deploying to $target_name..."
    curl -X POST "http://localhost:8080$endpoint"

    echo "⏳ Waiting for deployment..."
    sleep 10

    if test_server "$target_port" "$target_name"; then
        echo "✅ $target_name deployment successful"
        return 0
    else
        echo "❌ $target_name deployment failed"
        return 1
    fi
}

# Step 1: Test dev server
echo "1️⃣ Testing dev server..."
if ! test_server 8080 "Dev"; then
    echo "❌ Dev server failed tests - aborting pipeline"
    exit 1
fi

# Step 2: Deploy to QA and test
echo "2️⃣ Dev → QA deployment..."
if ! deploy_and_test "/deploy/dev-to-staging" 8080 "QA"; then
    echo "❌ QA deployment failed - aborting pipeline"
    exit 1
fi

# Step 3: Deploy to Prod and test
echo "3️⃣ QA → Prod deployment..."
if ! deploy_and_test "/deploy/staging-to-prod" 8081 "Prod"; then
    echo "❌ Prod deployment failed - aborting pipeline"
    exit 1
fi

# Step 4: Client rollout
echo "4️⃣ Client rollout..."
curl -X POST "http://localhost:8080/deploy/rollout"

echo ""
echo "🎉 Daisy Chain Deployment Complete!"
echo "   ✅ Dev tested and working"
echo "   ✅ QA deployed and tested"
echo "   ✅ Prod deployed and tested"
echo "   ✅ Clients updated via stable branch"
