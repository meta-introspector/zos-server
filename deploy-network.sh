#!/bin/bash
set -e

echo "🌐 ZOS Network Bootstrap Script"
echo "🚀 Deploying ZOS across multiple platforms..."

# Configuration
ZOS1_HOST="localhost:8080"
ORACLE_HOST="${ORACLE_HOST:-your-oracle-instance.com}"
ANDROID_HOST="${ANDROID_HOST:-192.168.1.100:8022}"  # SSH tunnel
WINDOWS_HOST="${WINDOWS_HOST:-192.168.1.101}"

# Check if ZOS1 is running
echo "🔍 Checking ZOS1 status..."
if ! curl -s http://$ZOS1_HOST/health >/dev/null; then
    echo "❌ ZOS1 not running on $ZOS1_HOST"
    echo "🚀 Starting ZOS1 locally..."
    cd zos-minimal-server
    cargo build --release
    sudo systemctl start zos-server.service || ./target/release/zos-minimal-server &
    sleep 5
    cd ..
fi

echo "✅ ZOS1 running on $ZOS1_HOST"

# Deploy to Oracle Cloud ARM64
echo ""
echo "☁️ Deploying to Oracle Cloud ARM64..."
ssh -o ConnectTimeout=10 oracle-user@$ORACLE_HOST << 'EOF'
    # Install Rust if needed
    if ! command -v cargo >/dev/null 2>&1; then
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source ~/.cargo/env
    fi

    # Clone and build ZOS
    if [ ! -d "zos-server" ]; then
        git clone https://github.com/your-repo/zos-server.git
    fi

    cd zos-server
    git pull
    cd zos-minimal-server
    cargo build --release

    # Start ZOS
    pkill -f zos-minimal-server || true
    nohup ./target/release/zos-minimal-server > zos.log 2>&1 &

    echo "✅ ZOS deployed on Oracle Cloud ARM64"
    echo "🌐 Access at: http://$(curl -s ifconfig.me):8080"
EOF

# Deploy to Android (via SSH tunnel)
echo ""
echo "📱 Deploying to Android ARM64..."
ssh -p 8022 -o ConnectTimeout=10 android-user@$ANDROID_HOST << 'EOF'
    # Use Nix4Droid environment
    nix-shell -p rustc cargo git pkg-config openssl --run '
        if [ ! -d "zos-server" ]; then
            git clone https://github.com/your-repo/zos-server.git
        fi

        cd zos-server
        git pull
        cd zos-minimal-server
        cargo build --release

        # Start ZOS
        pkill -f zos-minimal-server || true
        nohup ./target/release/zos-minimal-server > zos.log 2>&1 &

        echo "✅ ZOS deployed on Android ARM64"
    '
EOF

# Deploy to Windows
echo ""
echo "🪟 Deploying to Windows x64..."
ssh -o ConnectTimeout=10 windows-user@$WINDOWS_HOST << 'EOF'
    # Assume MinGW/MSYS2 environment with Rust
    if [ ! -d "zos-server" ]; then
        git clone https://github.com/your-repo/zos-server.git
    fi

    cd zos-server
    git pull
    cd zos-minimal-server
    cargo build --release

    # Start ZOS
    taskkill /F /IM zos-minimal-server.exe 2>/dev/null || true
    start /B target\\release\\zos-minimal-server.exe

    echo "✅ ZOS deployed on Windows x64"
EOF

echo ""
echo "🎉 ZOS Network Deployment Complete!"
echo ""
echo "🌐 Network Status:"
echo "  ZOS1 (Linux x64):    http://$ZOS1_HOST"
echo "  Oracle (ARM64):      http://$ORACLE_HOST:8080"
echo "  Android (ARM64):     http://$ANDROID_HOST:8080"
echo "  Windows (x64):       http://$WINDOWS_HOST:8080"
echo ""
echo "🔗 Test network:"
echo "  curl http://$ZOS1_HOST/health"
echo "  curl http://$ORACLE_HOST:8080/health"
echo ""
echo "🚀 ZOS Multi-Platform Network is LIVE!"
