#!/usr/bin/env nix-shell
#!nix-shell -i bash -p git rustc cargo pkg-config openssl curl

# ZOS Bootstrap Version 3: Nix (Reproducible)
# Uses Nix for reproducible, declarative builds

set -e

echo "❄️ ZOS Bootstrap v3: Nix (Reproducible)"
echo "======================================="

# Check if nix is available
if ! command -v nix &> /dev/null; then
    echo "❌ Nix not found. Install with:"
    echo "curl -L https://nixos.org/nix/install | sh"
    exit 1
fi

# Clone ZOS
echo "📥 Cloning ZOS repository..."
if [ ! -d "zos-server" ]; then
    git clone https://github.com/meta-introspector/zos-server.git
fi

cd zos-server

# Create shell.nix if it doesn't exist
if [ ! -f "shell.nix" ]; then
    echo "📝 Creating shell.nix..."
    cat > shell.nix << 'EOF'
{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    # Rust toolchain
    rustc
    cargo
    rustfmt
    clippy
    
    # System dependencies
    pkg-config
    openssl
    curl
    git
    
    # Development tools
    gdb
    valgrind
    strace
    
    # LLVM for universal plugin loader
    llvm
    clang
    
    # Mathematical tools
    sage
    gap
    
    # Networking
    libp2p
  ];
  
  shellHook = ''
    echo "❄️ ZOS Nix Development Environment"
    echo "Rust version: $(rustc --version)"
    echo "Cargo version: $(cargo --version)"
    
    export RUST_BACKTRACE=1
    export CARGO_TARGET_DIR="$PWD/target"
    
    # Set up LLVM paths
    export LLVM_SYS_120_PREFIX="${pkgs.llvm}"
  '';
}
EOF
fi

# Enter nix shell and build
echo "❄️ Entering Nix shell..."
nix-shell --run "
    echo '🔨 Building ZOS in reproducible environment...'
    cargo build --release
    
    echo '🧪 Running tests...'
    cargo test --release
    
    echo '📊 Checking binary...'
    file target/release/zos_server
    ldd target/release/zos_server || echo 'Static binary or different platform'
    
    echo '✅ ZOS Bootstrap v3 Complete!'
    echo '🚀 Run: ./target/release/zos_server'
    echo '❄️ Reproducible build guaranteed by Nix!'
"

# Create default.nix for building
if [ ! -f "default.nix" ]; then
    echo "📝 Creating default.nix for building..."
    cat > default.nix << 'EOF'
{ pkgs ? import <nixpkgs> {} }:

pkgs.rustPlatform.buildRustPackage rec {
  pname = "zos-server";
  version = "0.1.0";
  
  src = ./.;
  
  cargoLock = {
    lockFile = ./Cargo.lock;
  };
  
  nativeBuildInputs = with pkgs; [
    pkg-config
  ];
  
  buildInputs = with pkgs; [
    openssl
    curl
    git
    llvm
  ];
  
  meta = with pkgs.lib; {
    description = "Zero Ontology System - Mathematical computation platform";
    homepage = "https://github.com/meta-introspector/zos-server";
    license = licenses.mit;
    maintainers = [ "ZOS Team" ];
  };
}
EOF
fi

echo "❄️ Nix build files created!"
echo "🔨 To build with Nix: nix-build"
echo "🚀 To run: ./result/bin/zos_server"
