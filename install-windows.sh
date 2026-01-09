#!/bin/bash
# Quick installer script for Windows laptops
# Downloads and installs ZOS Deploy from GitHub releases

ACCOUNT="solfunmeme"
REPO="zos-server"
INSTALLER_URL="https://github.com/$ACCOUNT/$REPO/releases/latest/download/zos-deploy-installer.exe"

echo "🪟 ZOS Deploy Windows Installer"
echo "==============================="

# Download installer
echo "📦 Downloading installer..."
curl -L "$INSTALLER_URL" -o zos-deploy-installer.exe

# Run installer
echo "🚀 Running installer..."
./zos-deploy-installer.exe

echo "✅ Installation complete!"
echo "💡 Run 'zos_deploy' from any command prompt to start"
