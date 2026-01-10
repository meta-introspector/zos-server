#!/bin/bash
# ZOS Git CI/CD Test Script

ZOS_NODE="${ZOS_NODE:-localhost:8080}"

echo "🔧 ZOS Git CI/CD Testing"
echo "🌐 Node: $ZOS_NODE"
echo ""

# Test ping endpoint
echo "1️⃣ Testing ping endpoint..."
curl -s "http://$ZOS_NODE/ping" | jq .
echo ""

# Test git polling (check only)
echo "2️⃣ Testing git polling (check only)..."
curl -s -X POST "http://$ZOS_NODE/poll-git" \
  -H "Content-Type: application/json" \
  -d '{"auto_deploy": false, "branch": "main"}' | jq .
echo ""

# Test git polling with auto-deploy
echo "3️⃣ Testing git polling with auto-deploy..."
read -p "Deploy updates if available? (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    curl -s -X POST "http://$ZOS_NODE/poll-git" \
      -H "Content-Type: application/json" \
      -d '{"auto_deploy": true, "branch": "main"}' | jq .
else
    echo "Skipped auto-deploy test"
fi
echo ""

# Test webhook endpoint (simulate GitHub webhook)
echo "4️⃣ Testing webhook endpoint..."
curl -s -X POST "http://$ZOS_NODE/webhook/git" \
  -H "Content-Type: application/json" \
  -d '{
    "ref": "refs/heads/main",
    "repository": {
      "name": "zos-server",
      "clone_url": "https://github.com/meta-introspector/zos-server.git"
    },
    "head_commit": {
      "id": "abc123def456",
      "message": "Test webhook deployment",
      "author": {
        "name": "CI/CD Test"
      }
    }
  }' | jq .
echo ""

echo "🎯 CI/CD Endpoints:"
echo "   GET  /ping           - Node status and git info"
echo "   POST /poll-git       - Poll for updates"
echo "   POST /webhook/git    - Git webhook (GitHub/GitLab)"
echo "   POST /update-self    - Manual self-update"
echo ""
echo "🔗 GitHub Webhook URL: http://$ZOS_NODE/webhook/git"
