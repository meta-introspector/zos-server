#!/bin/bash
# ZOS Instance Manager - Create systemd-managed instances with git branches

set -e

INSTANCE_NAME="$1"
BRANCH="$2"
PORT="$3"

if [ -z "$INSTANCE_NAME" ] || [ -z "$BRANCH" ] || [ -z "$PORT" ]; then
    echo "Usage: $0 <instance_name> <branch> <port>"
    echo "Example: $0 qa main 8080"
    exit 1
fi

INSTANCE_DIR="/opt/zos/instance/$INSTANCE_NAME"
SERVICE_NAME="zos-$INSTANCE_NAME"

echo "🚀 Creating ZOS instance: $INSTANCE_NAME"
echo "📂 Directory: $INSTANCE_DIR"
echo "🌿 Branch: $BRANCH"
echo "🔌 Port: $PORT"

# Create instance directory
sudo mkdir -p "$INSTANCE_DIR"
sudo chown -R "$USER:$USER" "$INSTANCE_DIR"

# Clone repository to instance directory
if [ -d "$INSTANCE_DIR/.git" ]; then
    echo "🔄 Updating existing repository..."
    cd "$INSTANCE_DIR"
    git fetch origin
    git checkout "$BRANCH"
    git pull origin "$BRANCH"
else
    echo "📥 Cloning repository..."
    git clone -b "$BRANCH" https://github.com/meta-introspector/zos-server.git "$INSTANCE_DIR"
    cd "$INSTANCE_DIR"
fi

# Build the server
echo "🔨 Building ZOS server..."
cd zos-minimal-server
cargo build --release

# Create systemd service
echo "📋 Creating systemd service: $SERVICE_NAME"
sudo tee "/etc/systemd/system/$SERVICE_NAME.service" > /dev/null << EOF
[Unit]
Description=ZOS $INSTANCE_NAME Server - Branch $BRANCH
After=network.target
Wants=network.target

[Service]
Type=simple
User=$USER
Group=$USER
WorkingDirectory=$INSTANCE_DIR
ExecStart=$INSTANCE_DIR/target/release/zos-minimal-server
Restart=always
RestartSec=5

Environment=ZOS_HTTP_PORT=$PORT
Environment=ZOS_DATA_DIR=$INSTANCE_DIR/data
Environment=ZOS_LOG_LEVEL=info

[Install]
WantedBy=multi-user.target
EOF

# Enable and start service
sudo systemctl daemon-reload
sudo systemctl enable "$SERVICE_NAME"
sudo systemctl start "$SERVICE_NAME"

echo "✅ ZOS instance '$INSTANCE_NAME' created successfully!"
echo "🌐 Server: http://localhost:$PORT"
echo "🔧 Manage: sudo systemctl {start|stop|restart} $SERVICE_NAME"
echo "📂 Directory: $INSTANCE_DIR"
