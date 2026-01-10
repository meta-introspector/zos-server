#!/bin/bash
# ZOS Boot Script - Initial setup of QA and Prod instances

set -e

echo "🚀 ZOS Multi-Instance Boot Script"
echo "   Setting up QA and Prod instances"
echo ""

# Ensure we're running as root
if [ "$EUID" -ne 0 ]; then
    echo "❌ This script must be run as root/sudo"
    exit 1
fi

USER="${1:-mdupont}"

echo "👤 Setting up instances for user: $USER"
echo ""

# Step 1: Create QA instance (main branch, port 8082)
echo "1️⃣ Creating QA instance..."
./setup-zos-instance.sh qa main 8082 "$USER"

echo ""

# Step 2: Create Prod instance (stable branch, port 8081)
echo "2️⃣ Creating Prod instance..."
./setup-zos-instance.sh prod stable 8081 "$USER"

echo ""

# Step 3: Wait for both services to start
echo "3️⃣ Waiting for services to start..."
sleep 10

# Step 4: Verify both instances
echo "4️⃣ Verifying instances..."
QA_STATUS=$(curl -s http://localhost:8082/health | jq -r .status 2>/dev/null || echo "failed")
PROD_STATUS=$(curl -s http://localhost:8081/health | jq -r .status 2>/dev/null || echo "failed")

echo "   QA Server (8082):   $QA_STATUS"
echo "   Prod Server (8081): $PROD_STATUS"

if [ "$QA_STATUS" = "healthy" ] && [ "$PROD_STATUS" = "healthy" ]; then
    echo ""
    echo "🎉 ZOS Multi-Instance Setup Complete!"
    echo "   🔧 QA Server:   http://localhost:8082 (main branch)"
    echo "   🏭 Prod Server: http://localhost:8081 (stable branch)"
    echo ""
    echo "📋 Management:"
    echo "   systemctl status zos-qa"
    echo "   systemctl status zos-prod"
    echo ""
    echo "🔄 QA can now manage Prod via:"
    echo "   curl -X POST http://localhost:8082/deploy/staging-to-prod"
else
    echo "❌ Some instances failed to start properly"
    exit 1
fi
