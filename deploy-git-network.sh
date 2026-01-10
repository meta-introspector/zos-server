#!/bin/bash
# ZOS Network Git Push Deployment
# Push updates to all nodes in the network

NODES=(
    "solana.solfunmeme.com:8080"
    "localhost:8081"  # ZOS2 instance
    # Add more nodes here
)

echo "🚀 ZOS Network Git Push Deployment"
echo "📡 Pushing to ${#NODES[@]} nodes..."
echo ""

# First, push to GitHub
echo "1️⃣ Pushing to GitHub..."
git push origin main
echo ""

# Wait a moment for GitHub to process
sleep 2

# Trigger updates on all nodes
for node in "${NODES[@]}"; do
    echo "📡 Updating node: $node"

    # Check if node is reachable
    if curl -s --connect-timeout 5 "http://$node/ping" >/dev/null; then
        echo "✅ Node reachable, triggering update..."

        # Trigger git poll with auto-deploy
        response=$(curl -s -X POST "http://$node/poll-git" \
          -H "Content-Type: application/json" \
          -d '{"auto_deploy": true, "branch": "main"}')

        status=$(echo "$response" | jq -r '.status // "unknown"')
        message=$(echo "$response" | jq -r '.message // "No message"')

        echo "   Status: $status"
        echo "   Message: $message"
    else
        echo "❌ Node unreachable: $node"
    fi
    echo ""
done

echo "🎉 Network deployment complete!"
echo ""
echo "🔍 Check status with:"
for node in "${NODES[@]}"; do
    echo "   curl -s http://$node/ping | jq ."
done
