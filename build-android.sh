#!/usr/bin/env bash
#!nix-shell shell.nix -i bash

set -e

# Accept Android SDK license
export NIXPKGS_ACCEPT_ANDROID_SDK_LICENSE=1

echo "🤖 Building ZOS for Android"

cd zos-minimal-server

# Build for Android ARM64 (modern devices)
echo "📱 Building for Android ARM64..."
cargo build --release --target aarch64-linux-android

# Build for Android ARM32 (older devices)
echo "📱 Building for Android ARM32..."
cargo build --release --target armv7-linux-androideabi

cd ..

# Create Android deployment directory
mkdir -p target/android-builds
cp target/aarch64-linux-android/release/zos-minimal-server target/android-builds/zos-minimal-server-arm64
cp target/armv7-linux-androideabi/release/zos-minimal-server target/android-builds/zos-minimal-server-arm32

echo "✅ Android builds complete!"
echo "📁 ARM64: target/android-builds/zos-minimal-server-arm64"
echo "📁 ARM32: target/android-builds/zos-minimal-server-arm32"
