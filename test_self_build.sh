#!/bin/bash

echo "🧪 Testing Self-Build System"
echo "=============================="

# Test current build status
echo "1. Testing current build status..."
./zos-server self-build test

echo ""
echo "2. Starting self-build process..."
./zos-server self-build build

echo ""
echo "3. Final build verification..."
cargo build --release 2>&1 | head -10
