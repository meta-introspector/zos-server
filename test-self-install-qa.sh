#!/bin/bash
# Test ZOS Self-Installing QA Service

echo "🧪 ZOS Self-Installing QA Service Test"
echo "======================================"

# Start dev server
echo "1. Starting dev server..."
cargo run --bin zos-minimal-server serve 8080 &
DEV_PID=$!
sleep 3

# Show installer options
echo ""
echo "2. Installer Options Available:"
echo "   Basic install:     curl http://localhost:8080/install.sh | bash"
echo "   Install + QA:      curl http://localhost:8080/install.sh | bash -s qa"
echo ""

# Show current network status
echo "3. Current Network Status:"
cargo run --bin zos-minimal-server network-status

echo ""
echo "4. Ready for installation test!"
echo "   To test QA self-install: curl http://localhost:8080/install.sh | bash -s qa"
echo ""
echo "   This will:"
echo "   - Download and verify binary"
echo "   - Install to /usr/local/bin/zos-server"
echo "   - Deploy as systemd QA service on port 8082"
echo "   - Start the service automatically"

# Cleanup
kill $DEV_PID 2>/dev/null || true
echo ""
echo "✅ Test setup complete. Dev server stopped."
