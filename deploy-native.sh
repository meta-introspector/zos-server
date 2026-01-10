#!/bin/bash
set -e

echo "🚀 ZOS Native Build Deployment Script"
echo "📱 Platform: $(uname -m) $(uname -s)"

# Detect platform and set deployment strategy
PLATFORM=$(uname -s)
ARCH=$(uname -m)

case "$PLATFORM" in
    "Linux")
        if command -v nix >/dev/null 2>&1; then
            echo "🐧 Linux with Nix detected"
            DEPLOY_METHOD="nix"
        else
            echo "🐧 Standard Linux detected"
            DEPLOY_METHOD="cargo"
        fi
        ;;
    "Darwin")
        echo "🍎 macOS detected"
        DEPLOY_METHOD="cargo"
        ;;
    "MINGW"*|"MSYS"*|"CYGWIN"*)
        echo "🪟 Windows/MinGW detected"
        DEPLOY_METHOD="cargo"
        ;;
    *)
        echo "❓ Unknown platform: $PLATFORM"
        DEPLOY_METHOD="cargo"
        ;;
esac

# Install Rust if not present
if ! command -v cargo >/dev/null 2>&1; then
    echo "🦀 Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
fi

# Build ZOS natively
echo "🔨 Building ZOS natively for $ARCH..."
cd zos-minimal-server

case "$DEPLOY_METHOD" in
    "nix")
        nix-shell -p rustc cargo pkg-config openssl git --run "cargo build --release"
        ;;
    "cargo")
        cargo build --release
        ;;
esac

# Platform-specific deployment
case "$PLATFORM" in
    "Linux")
        echo "🐧 Deploying on Linux..."
        if [[ "$ARCH" == "aarch64" ]]; then
            echo "📱 ARM64 Linux deployment"
            # Oracle Cloud or Android deployment
            ./target/release/zos-minimal-server &
            ZOS_PID=$!
            echo "✅ ZOS running on PID $ZOS_PID"
            echo "🌐 Access at: http://$(hostname -I | awk '{print $1}'):8080"
        else
            echo "💻 x86_64 Linux deployment"
            # Use systemd if available
            if command -v systemctl >/dev/null 2>&1; then
                sudo ./deploy-local-systemd.sh
            else
                ./target/release/zos-minimal-server &
                ZOS_PID=$!
                echo "✅ ZOS running on PID $ZOS_PID"
            fi
        fi
        ;;
    "Darwin")
        echo "🍎 Deploying on macOS..."
        ./target/release/zos-minimal-server &
        ZOS_PID=$!
        echo "✅ ZOS running on PID $ZOS_PID"
        ;;
    "MINGW"*|"MSYS"*|"CYGWIN"*)
        echo "🪟 Deploying on Windows..."
        ./target/release/zos-minimal-server.exe &
        ZOS_PID=$!
        echo "✅ ZOS running on PID $ZOS_PID"
        ;;
esac

echo ""
echo "🎉 ZOS Native Deployment Complete!"
echo "📊 Platform: $PLATFORM $ARCH"
echo "🔧 Method: $DEPLOY_METHOD"
echo "🌐 Server: http://localhost:8080"
echo ""
echo "🔗 Test with: curl http://localhost:8080/health"
