#!/bin/bash
# ZOS1 -> ZOS2 deployment via HTTP API

ZOS1_URL="http://localhost:8080"
ZOS2_PORT="8081"

echo "🔗 Using ZOS1 to deploy ZOS2 instance"

# Request ZOS1 to deploy ZOS2
curl -X POST "$ZOS1_URL/deploy" \
  -H "Content-Type: application/json" \
  -d "{
    \"target_port\": $ZOS2_PORT,
    \"instance_name\": \"zos2\",
    \"rebuild_self\": true,
    \"prepare_windows\": true
  }"

echo "✅ ZOS2 deployment initiated via ZOS1"
