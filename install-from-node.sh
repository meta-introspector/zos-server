#!/bin/bash
# ZOS Universal Installer - Install from any running ZOS node
# Usage: curl -sSL http://NODE:PORT/install.sh | bash

set -e

# Extract ZOS server from script URL or use default
ZOS_SERVER="${ZOS_SERVER:-solana.solfunmeme.com:8080}"
ZOS_BRANCH="${ZOS_BRANCH:-main}"
if [[ -n "$BASH_SOURCE" ]]; then
    # Try to extract from the URL this script was downloaded from
    SCRIPT_URL=$(ps -o args= -p $PPID | grep -o 'http://[^/]*' || echo "")
    if [[ -n "$SCRIPT_URL" ]]; then
        ZOS_SERVER=$(echo "$SCRIPT_URL" | sed 's|http://||')
    fi
fi

echo "🚀 ZOS Universal Installer"
echo "📡 Installing from: $ZOS_SERVER"
echo "🌿 Branch: $ZOS_BRANCH"
echo ""

# Function to send install feedback
send_install_feedback() {
    local status="$1"
    local message="$2"
    local error="$3"
    local duration="${4:-0}"

    curl -s -X POST "http://$ZOS_SERVER/logs/install" \
        -H "Content-Type: application/json" \
        -d "{
            \"host\": \"$(hostname)\",
            \"platform\": \"$PLATFORM $ARCH\",
            \"status\": \"$status\",
            \"message\": \"$message\",
            \"error\": $(if [[ -n "$error" ]]; then echo "\"$error\""; else echo "null"; fi),
            \"duration_seconds\": $duration
        }" >/dev/null 2>&1 || true
}

# Error handler
handle_error() {
    local exit_code=$?
    local line_number=$1
    END_TIME=$(date +%s)
    DURATION=$((END_TIME - START_TIME))
    send_install_feedback "failed" "Installation failed at line $line_number" "Exit code: $exit_code" "$DURATION"
    echo "❌ Installation failed at line $line_number with exit code $exit_code"
    exit $exit_code
}

# Set error trap
trap 'handle_error $LINENO' ERR

# Start timer
START_TIME=$(date +%s)

# Detect platform
PLATFORM=$(uname -s)
ARCH=$(uname -m)
echo "🖥️  Platform: $PLATFORM $ARCH"

# Send start feedback
send_install_feedback "started" "Installation started" ""

# Install dependencies based on platform
case "$PLATFORM" in
    "Linux")
        echo "🐧 Linux detected"
        if command -v nix >/dev/null 2>&1; then
            echo "❄️  Nix detected - using Nix environment"
            # Check if we're in Termux/Android
            if [[ "$PREFIX" == *"termux"* ]] || [[ "$PWD" == *"termux"* ]]; then
                echo "📱 Android/Termux detected - using Termux packages"
                INSTALL_METHOD="termux"
            else
                INSTALL_METHOD="nix"
            fi
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
send_install_feedback "downloading" "Downloading source code" ""
if command -v git >/dev/null 2>&1; then
    echo "📂 Cloning from Git..."
    git clone -b "$ZOS_BRANCH" https://github.com/meta-introspector/zos-server.git 2>/dev/null || {
        echo "⚠️  Branch $ZOS_BRANCH not found, using main"
        git clone https://github.com/meta-introspector/zos-server.git
        cd zos-server
        git checkout "$ZOS_BRANCH" 2>/dev/null || git checkout main
        cd ..
    }
    cd zos-server
else
    echo "📦 Downloading tarball..."
    curl -L http://$ZOS_SERVER/tarball -o zos-server.tar.gz
    tar -xzf zos-server.tar.gz
    cd zos-server
fi

# Build ZOS
echo "🔨 Building ZOS..."
send_install_feedback "building" "Compiling source code" ""

# Report git version info
echo "📋 Repository Information:"
GIT_COMMIT=$(git rev-parse HEAD 2>/dev/null || echo "unknown")
GIT_BRANCH=$(git branch --show-current 2>/dev/null || echo "unknown")
GIT_REMOTE=$(git remote get-url origin 2>/dev/null || echo "unknown")
echo "   🌿 Branch: $GIT_BRANCH"
echo "   📝 Commit: ${GIT_COMMIT:0:8}"
echo "   🔗 Remote: $GIT_REMOTE"

# Send git info feedback
send_install_feedback "git_info" "Git info: branch=$GIT_BRANCH, commit=${GIT_COMMIT:0:8}" ""

cd zos-minimal-server

case "$INSTALL_METHOD" in
    "nix")
        BUILD_OUTPUT=$(nix-shell -p rustc cargo pkg-config openssl git --run "cargo build --release" 2>&1)
        BUILD_STATUS=$?
        ;;
    "termux")
        echo "📱 Building with Termux packages..."
        pkg install -y rust binutils || true
        BUILD_OUTPUT=$(cargo build --release 2>&1)
        BUILD_STATUS=$?
        ;;
    "cargo")
        BUILD_OUTPUT=$(cargo build --release 2>&1)
        BUILD_STATUS=$?
        ;;
esac

# Check for specific Cargo.lock version error
if echo "$BUILD_OUTPUT" | grep -q "lock file version.*requires.*lockfile-bump"; then
    echo "🔧 Fixing Cargo.lock version compatibility..."
    rm -f Cargo.lock
    BUILD_OUTPUT=$(cargo generate-lockfile && cargo build --release 2>&1)
    BUILD_STATUS=$?
fi

if [ $BUILD_STATUS -eq 0 ]; then
    echo "✅ Build successful"
    send_build_feedback "success" "$BUILD_OUTPUT" ""
else
    echo "❌ Build failed"
    echo "📋 Build output:"
    echo "$BUILD_OUTPUT" | tail -20
    send_build_feedback "failed" "$BUILD_OUTPUT" "Build command failed with status $BUILD_STATUS"
    send_install_feedback "failed" "Build failed" "Build command failed with status $BUILD_STATUS"
    exit 1
fi

# Install ZOS
echo "📦 Installing ZOS..."
send_install_feedback "installing" "Installing binary" ""
INSTALL_DIR="$HOME/.local/bin"
mkdir -p "$INSTALL_DIR"
cp target/release/zos-minimal-server "$INSTALL_DIR/"

# Add to PATH if not already there
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo "export PATH=\"$INSTALL_DIR:\$PATH\"" >> ~/.bashrc
    echo "export PATH=\"$INSTALL_DIR:\$PATH\"" >> ~/.profile 2>/dev/null || true
    export PATH="$INSTALL_DIR:$PATH"
fi

# Calculate duration
END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))

echo ""
echo "🎉 ZOS Installation Complete!"
echo ""
echo "📁 Installed to: $INSTALL_DIR"
echo "🚀 Binary at: $INSTALL_DIR/zos-minimal-server"
echo ""
echo "▶️  Start ZOS with: zos-minimal-server"
echo "🔗 Test with: curl http://localhost:8080/health"
echo ""
echo "📚 Documentation: http://$ZOS_SERVER/source"
echo "🌐 Join network: http://$ZOS_SERVER"

# Send success feedback
send_install_feedback "completed" "Installation completed successfully" "" "$DURATION"
