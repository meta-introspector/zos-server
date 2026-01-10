#!/bin/bash
# Test ZOS source distribution endpoints

ZOS_HOST="${ZOS_HOST:-localhost:8080}"

echo "🧪 Testing ZOS Source Distribution Endpoints"
echo "🌐 Host: $ZOS_HOST"
echo ""

# Test health endpoint
echo "1️⃣ Testing health endpoint..."
curl -s "http://$ZOS_HOST/health" | jq . || echo "❌ Health check failed"
echo ""

# Test source info endpoint
echo "2️⃣ Testing source info endpoint..."
curl -s "http://$ZOS_HOST/source" | jq . || echo "❌ Source info failed"
echo ""

# Test installer script endpoint
echo "3️⃣ Testing installer script endpoint..."
INSTALLER_SIZE=$(curl -s "http://$ZOS_HOST/install.sh" | wc -c)
if [ "$INSTALLER_SIZE" -gt 100 ]; then
    echo "✅ Installer script: $INSTALLER_SIZE bytes"
else
    echo "❌ Installer script too small or failed"
fi
echo ""

# Test tarball endpoint
echo "4️⃣ Testing tarball endpoint..."
TARBALL_SIZE=$(curl -s -I "http://$ZOS_HOST/tarball" | grep -i content-length | awk '{print $2}' | tr -d '\r')
if [ -n "$TARBALL_SIZE" ] && [ "$TARBALL_SIZE" -gt 1000 ]; then
    echo "✅ Tarball available: $TARBALL_SIZE bytes"
else
    echo "❌ Tarball not available or too small"
fi
echo ""

echo "🎯 Quick install command:"
echo "   curl -sSL http://$ZOS_HOST/install.sh | bash"
echo ""
echo "🌐 Source info:"
echo "   curl -s http://$ZOS_HOST/source | jq ."
