#!/bin/bash
# ZOS Production Update Script - Run as root via sudo
# This script is called by QA server to trigger prod updates

set -e

PROD_USER="zos-prod"
PROD_DIR="/opt/zos-production"
BRANCH="${1:-stable}"

echo "🏭 Production update triggered by QA server"
echo "📋 Branch: $BRANCH"
echo "👤 Running as: $(whoami)"

# Switch to prod user and update
sudo -u $PROD_USER bash -c "
cd $PROD_DIR
echo '🔄 Fetching latest code...'
git fetch origin
git checkout '$BRANCH'
git pull origin '$BRANCH'

echo '🔨 Building production server...'
cd zos-minimal-server
cargo build --release

echo '✅ Production build complete'
"

# Restart production service
echo "🔄 Restarting production service..."
systemctl restart zos-production.service

echo "✅ Production update complete"
systemctl status zos-production.service --no-pager -l
