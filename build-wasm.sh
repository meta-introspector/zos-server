#!/bin/bash
set -e

echo "🦀 Building ZOS Dashboard WASM..."

cd zos-dashboard-wasm

# Install wasm-pack if not available
if ! command -v wasm-pack &> /dev/null; then
    echo "Installing wasm-pack..."
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
fi

# Build WASM module
wasm-pack build --target web --out-dir ../static

echo "✅ WASM build complete!"
echo "📁 Files generated in ./static/"
ls -la ../static/
