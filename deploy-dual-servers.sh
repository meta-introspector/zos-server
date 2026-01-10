#!/bin/bash
# ZOS Dual Server Manager - Dev and Production

set -e

REPO_DIR="/mnt/data1/nix/time/2024/12/10/swarms-terraform/services/submodules/zos-server"
DEV_BRANCH="main"
PROD_BRANCH="stable"

echo "🚀 ZOS Dual Server Deployment"
echo "📂 Repository: $REPO_DIR"
echo "🔧 Dev Branch: $DEV_BRANCH (port 8080)"
echo "🏭 Prod Branch: $PROD_BRANCH (port 8081)"
echo ""

# Function to deploy a specific branch
deploy_branch() {
    local branch="$1"
    local server_type="$2"
    local binary_name="$3"
    local service_name="$4"

    echo "📦 Deploying $server_type server from $branch branch..."

    # Create temporary directory for this deployment
    local temp_dir="/tmp/zos-deploy-$server_type"
    rm -rf "$temp_dir"

    # Clone specific branch
    git clone -b "$branch" "$REPO_DIR" "$temp_dir" 2>/dev/null || {
        echo "⚠️  Branch $branch not found, using main"
        git clone "$REPO_DIR" "$temp_dir"
        cd "$temp_dir"
        git checkout "$branch" 2>/dev/null || git checkout main
    }

    cd "$temp_dir"

    # Build the server
    echo "🔨 Building $server_type server..."
    cd zos-minimal-server
    cargo build --release

    # Stop service, deploy binary, start service
    echo "🔄 Deploying $server_type server..."
    sudo systemctl stop "$service_name" 2>/dev/null || true
    sudo cp "target/release/zos-minimal-server" "/opt/zos/bin/$binary_name"
    sudo chmod +x "/opt/zos/bin/$binary_name"
    sudo systemctl enable "$service_name" 2>/dev/null || true
    sudo systemctl start "$service_name"

    # Cleanup
    rm -rf "$temp_dir"

    echo "✅ $server_type server deployed successfully"
}

# Deploy development server (main branch, port 8080)
deploy_branch "$DEV_BRANCH" "development" "zos-minimal-server" "zos-server.service"

# Deploy production server (stable branch, port 8081)
deploy_branch "$PROD_BRANCH" "production" "zos-prod-server" "zos-prod-server.service"

echo ""
echo "🎯 Deployment Summary:"
echo "   🔧 Dev Server:  http://localhost:8080 ($DEV_BRANCH branch)"
echo "   🏭 Prod Server: http://localhost:8081 ($PROD_BRANCH branch)"
echo ""
echo "📊 Service Status:"
sudo systemctl status zos-server.service --no-pager -l | head -3
sudo systemctl status zos-prod-server.service --no-pager -l | head -3

echo ""
echo "🔗 Health Checks:"
sleep 2
curl -s http://localhost:8080/health | jq -r '"Dev: " + .status + " (" + .version + ")"' 2>/dev/null || echo "Dev: Not responding"
curl -s http://localhost:8081/health | jq -r '"Prod: " + .status + " (" + .version + ")"' 2>/dev/null || echo "Prod: Not responding"
