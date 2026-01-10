#!/bin/bash
# ZOS Universal Installer - Install from any running ZOS node
# Usage: curl -sSL http://solana.solfunmeme.com:8080/install.sh | bash

set -e

echo "🚀 ZOS Universal Installer"
echo "📡 Installing from: solana.solfunmeme.com:8080"
echo ""

# Detect platform
PLATFORM=$(uname -s)
ARCH=$(uname -m)
echo "🖥️  Platform: $PLATFORM $ARCH"

# Install dependencies based on platform
case "$PLATFORM" in
    "Linux")
        echo "🐧 Linux detected"
        if command -v nix >/dev/null 2>&1; then
            echo "❄️  Nix detected - using Nix environment"
            INSTALL_METHOD="nix"
        elif command -v apt >/dev/null 2>&1; then
            echo "📦 APT detected - installing dependencies"
            sudo apt update
            sudo apt install -y curl git build-essential pkg-config libssl-dev
            INSTALL_METHOD="cargo"
        elif command -v yum >/dev/null 2>&1; then
            echo "📦 YUM detected - installing dependencies"
            sudo yum install -y curl git gcc pkg-config openssl-devel
            INSTALL_METHOD="cargo"
        else
            echo "⚠️  Unknown package manager - assuming dependencies exist"
            INSTALL_METHOD="cargo"
        fi
        ;;
    "Darwin")
        echo "🍎 macOS detected"
        if ! command -v brew >/dev/null 2>&1; then
            echo "🍺 Installing Homebrew..."
            /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
        fi
        brew install pkg-config openssl
        INSTALL_METHOD="cargo"
        ;;
    "MINGW"*|"MSYS"*|"CYGWIN"*)
        echo "🪟 Windows/MinGW detected"
        INSTALL_METHOD="cargo"
        ;;
    *)
        echo "❓ Unknown platform - attempting generic install"
        INSTALL_METHOD="cargo"
        ;;
esac

# Install Rust if not present
if ! command -v cargo >/dev/null 2>&1; then
    echo "🦀 Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
    export PATH="$HOME/.cargo/bin:$PATH"
fi

# Download and extract ZOS source
echo "📥 Downloading ZOS source..."
if command -v git >/dev/null 2>&1; then
    echo "📂 Cloning from Git..."
    git clone https://github.com/meta-introspector/zos-server.git
    cd zos-server
else
    echo "📦 Downloading tarball..."
    curl -L http://solana.solfunmeme.com:8080/tarball -o zos-server.tar.gz
    tar -xzf zos-server.tar.gz
    cd zos-server
fi

# Build ZOS
echo "🔨 Building ZOS..."
cd zos-minimal-server

case "$INSTALL_METHOD" in
    "nix")
        nix-shell -p rustc cargo pkg-config openssl git --run "cargo build --release"
        ;;
    "cargo")
        cargo build --release
        ;;
esac

# Install ZOS
echo "📦 Installing ZOS..."
INSTALL_DIR="$HOME/.local/bin"
mkdir -p "$INSTALL_DIR"
cp target/release/zos-minimal-server "$INSTALL_DIR/"

# Add to PATH if not already there
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo "export PATH=\"$INSTALL_DIR:\$PATH\"" >> ~/.bashrc
    echo "export PATH=\"$INSTALL_DIR:\$PATH\"" >> ~/.profile 2>/dev/null || true
    export PATH="$INSTALL_DIR:$PATH"
fi

echo ""
echo "🎉 ZOS Installation Complete!"
echo ""
echo "🚀 Start ZOS with: zos-minimal-server"
echo "🌐 Or run directly: $INSTALL_DIR/zos-minimal-server"
echo "🔗 Test with: curl http://localhost:8080/health"
echo ""
echo "📚 Documentation: http://solana.solfunmeme.com:8080/source"
echo "🌐 Network: http://solana.solfunmeme.com:8080"
