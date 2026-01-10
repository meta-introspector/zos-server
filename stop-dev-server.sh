#!/bin/bash
echo "🛑 Stopping ZOS Dev Server daemon..."

# Kill by PID file
if [ -f "/tmp/zos-dev.pid" ]; then
    PID=$(cat /tmp/zos-dev.pid)
    if kill "$PID" 2>/dev/null; then
        echo "✅ Stopped daemon (PID: $PID)"
    fi
    rm -f /tmp/zos-dev.pid
fi

# Kill by process name
pkill -f "zos-dev-minimal --daemon" 2>/dev/null
pkill -f "zos_server serve" 2>/dev/null

echo "✅ All ZOS dev processes stopped"
