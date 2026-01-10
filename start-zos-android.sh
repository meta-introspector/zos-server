#!/bin/bash
# Start ZOS on Android after installation

echo "🚀 Starting ZOS on Android..."

# Check if ZOS is installed
if [ -f "$HOME/.zos/bin/zos-minimal-server" ]; then
    echo "✅ Found ZOS at $HOME/.zos/bin/zos-minimal-server"
    cd "$HOME/.zos/zos-server/zos-minimal-server"

    # Start ZOS in background
    echo "🌐 Starting ZOS server on port 8080..."
    nohup "$HOME/.zos/bin/zos-minimal-server" > "$HOME/.zos/zos.log" 2>&1 &

    sleep 3

    # Test if it's running
    if curl -s http://localhost:8080/health >/dev/null 2>&1; then
        echo "✅ ZOS is running!"
        echo "🌐 Access at: http://localhost:8080"
        echo "📱 Or from network: http://$(hostname -I | awk '{print $1}'):8080"
    else
        echo "❌ ZOS failed to start. Check logs:"
        echo "📋 tail $HOME/.zos/zos.log"
    fi
else
    echo "❌ ZOS not found. Install first with:"
    echo "curl -sSL http://solana.solfunmeme.com:8080/install.sh | bash"
fi
