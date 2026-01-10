#!/bin/bash
# ZOS Instance Setup - Run as root/sudo to create instance directories and services

set -e

INSTANCE_NAME="$1"
BRANCH="$2"
PORT="$3"
USER="${4:-mdupont}"

if [ -z "$INSTANCE_NAME" ] || [ -z "$BRANCH" ] || [ -z "$PORT" ]; then
    echo "Usage: sudo $0 <instance_name> <branch> <port> [user]"
    echo "Example: sudo $0 qa main 8082 mdupont"
    exit 1
fi

INSTANCE_DIR="/opt/zos/instance/$INSTANCE_NAME"
SERVICE_NAME="zos-$INSTANCE_NAME"

echo "🔧 Setting up ZOS instance: $INSTANCE_NAME"
echo "📂 Directory: $INSTANCE_DIR"
echo "🌿 Branch: $BRANCH"
echo "🔌 Port: $PORT"
echo "👤 User: $USER"

# Create instance directory with proper ownership
mkdir -p "/opt/zos/instance"
mkdir -p "$INSTANCE_DIR"
chown -R "$USER:$USER" "/opt/zos/instance"

# Clone repository as user
cd "/opt/zos/instance"
sudo -u "$USER" git clone -b "$BRANCH" https://github.com/meta-introspector/zos-server.git "$INSTANCE_NAME"

# Build as user
cd "$INSTANCE_DIR/zos-minimal-server"
sudo -u "$USER" cargo build --release

# Create systemd service (simple, secure)
cat > "/etc/systemd/system/$SERVICE_NAME.service" << EOF
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
systemctl daemon-reload
systemctl enable "$SERVICE_NAME"
systemctl start "$SERVICE_NAME"

echo "✅ ZOS instance '$INSTANCE_NAME' created successfully!"
echo "🌐 Server: http://localhost:$PORT"
echo "🔧 Manage: systemctl {start|stop|restart} $SERVICE_NAME"
