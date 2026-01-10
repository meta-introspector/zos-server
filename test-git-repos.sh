#!/bin/bash
# ZOS Git Repository Management Test

ZOS_NODE="${ZOS_NODE:-localhost:8080}"

echo "🗂️  ZOS Git Repository Management"
echo "🌐 Node: $ZOS_NODE"
echo ""

# List existing repos
echo "1️⃣ Listing existing repositories..."
curl -s "http://$ZOS_NODE/git/repos" | jq .
echo ""

# Create a new repository
echo "2️⃣ Creating new repository 'test-repo'..."
curl -s -X POST "http://$ZOS_NODE/git/repos" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "test-repo",
    "description": "Test repository for ZOS git management"
  }' | jq .
echo ""

# Clone an existing repository
echo "3️⃣ Cloning zos-server repository..."
curl -s -X POST "http://$ZOS_NODE/git/repos/zos-server-clone/clone" \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://github.com/meta-introspector/zos-server.git",
    "branch": "main"
  }' | jq .
echo ""

# Get repository info
echo "4️⃣ Getting repository info..."
curl -s "http://$ZOS_NODE/git/repos/zos-server-clone" | jq .
echo ""

# List all repos again
echo "5️⃣ Listing all repositories..."
curl -s "http://$ZOS_NODE/git/repos" | jq .
echo ""

echo "🎯 Git Repository Management Endpoints:"
echo "   GET    /git/repos           - List all repositories"
echo "   POST   /git/repos           - Create new repository"
echo "   POST   /git/repos/:repo/clone - Clone external repository"
echo "   GET    /git/repos/:repo     - Get repository info"
echo "   DELETE /git/repos/:repo     - Delete repository"
