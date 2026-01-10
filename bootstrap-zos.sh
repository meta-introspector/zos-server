#!/bin/bash
# ZOS Bootstrap - Self-deploying server that manages its own git state

set -e

BOOTSTRAP_DIR="/opt/zos-bootstrap"
SERVICE_NAME="zos-bootstrap"
REPO_URL="https://github.com/meta-introspector/zos-server.git"
BRANCH="main"

echo "🚀 ZOS Bootstrap Deployment"
echo "📂 Bootstrap dir: $BOOTSTRAP_DIR"
echo "🌿 Branch: $BRANCH"

# Create bootstrap directory
sudo mkdir -p "$BOOTSTRAP_DIR"
sudo chown -R "$USER:$USER" "$BOOTSTRAP_DIR"

# Clone or update repository
if [ -d "$BOOTSTRAP_DIR/.git" ]; then
    echo "🔄 Updating existing repository..."
    cd "$BOOTSTRAP_DIR"
    git fetch origin
    git checkout "$BRANCH"
    git pull origin "$BRANCH"
else
    echo "📥 Cloning repository..."
    git clone -b "$BRANCH" "$REPO_URL" "$BOOTSTRAP_DIR"
    cd "$BOOTSTRAP_DIR"
fi

# Build the server
echo "🔨 Building ZOS server..."
cd zos-minimal-server
cargo build --release

# Create systemd service
echo "📋 Creating systemd service..."
sudo tee /etc/systemd/system/$SERVICE_NAME.service > /dev/null << EOF
[Unit]
Description=ZOS Bootstrap Server - Self-Maintaining
After=network.target
Wants=network.target

[Service]
Type=simple
User=$USER
Group=$USER
WorkingDirectory=$BOOTSTRAP_DIR
ExecStart=$BOOTSTRAP_DIR/target/release/zos-minimal-server
Restart=always
RestartSec=5

Environment=ZOS_HTTP_PORT=8080
Environment=ZOS_DATA_DIR=$BOOTSTRAP_DIR/data
Environment=ZOS_LOG_LEVEL=info

[Install]
WantedBy=multi-user.target
EOF

# Enable and start service
sudo systemctl daemon-reload
sudo systemctl enable $SERVICE_NAME
sudo systemctl start $SERVICE_NAME

echo "✅ ZOS Bootstrap deployed successfully!"
echo "🌐 Server: http://localhost:8080"
echo "🔧 Manage: sudo systemctl {start|stop|restart} $SERVICE_NAME"
