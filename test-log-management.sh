#!/bin/bash
# ZOS Log Management Test

ZOS_NODE="${ZOS_NODE:-localhost:8080}"

echo "📋 ZOS Log Management Test"
echo "🌐 Node: $ZOS_NODE"
echo ""

# Send test install log
echo "1️⃣ Sending test install log..."
curl -s -X POST "http://$ZOS_NODE/logs/install" \
  -H "Content-Type: application/json" \
  -d '{
    "host": "test-host.local",
    "platform": "Linux x86_64",
    "status": "completed",
    "message": "Test installation successful",
    "duration_seconds": 120
  }' | jq .
echo ""

# Send test build log
echo "2️⃣ Sending test build log..."
curl -s -X POST "http://$ZOS_NODE/logs/build" \
  -H "Content-Type: application/json" \
  -d '{
    "host": "test-host.local",
    "platform": "Linux x86_64",
    "status": "success",
    "build_output": "Finished release [optimized] target(s) in 45.2s",
    "duration_seconds": 45
  }' | jq .
echo ""

# List all logs
echo "3️⃣ Listing all logs..."
curl -s "http://$ZOS_NODE/logs" | jq .
echo ""

# Get logs for specific host
echo "4️⃣ Getting logs for test-host.local..."
curl -s "http://$ZOS_NODE/logs/test-host.local" | jq .
echo ""

echo "🎯 Log Management Endpoints:"
echo "   POST /logs/install    - Receive install feedback"
echo "   POST /logs/build      - Receive build logs"
echo "   GET  /logs            - List all log files"
echo "   GET  /logs/:host      - Get logs for specific host"
