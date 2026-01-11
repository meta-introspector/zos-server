#!/bin/bash
# ZOS Server Bootstrap Script for Oracle Cloud
# Installs and configures ZOS server with HTTPS

set -e

ZOS_DOMAIN="${ZOS_DOMAIN:-example.com}"
export ZOS_VERSION="1.0.0"

echo "🚀 ZOS Server Bootstrap Starting..."
echo "Domain: $ZOS_DOMAIN"

# Update system
dnf update -y

# Install dependencies
dnf install -y \
    git \
    curl \
    wget \
    unzip \
    nginx \
    certbot \
    python3-certbot-nginx \
    firewalld

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
# shellcheck source=/dev/null
source ~/.cargo/env

# Configure firewall
systemctl enable firewalld
systemctl start firewalld
firewall-cmd --permanent --add-service=http
firewall-cmd --permanent --add-service=https
firewall-cmd --permanent --add-port=8080/tcp
firewall-cmd --permanent --add-port=4001/tcp
firewall-cmd --permanent --add-port=4001/udp
firewall-cmd --reload

# Create ZOS user
useradd -r -s /bin/false -d /opt/zos zos
mkdir -p /opt/zos/{bin,data,logs}
chown -R zos:zos /opt/zos

# Download ZOS server binary (placeholder - would download from releases)
cat > /opt/zos/bin/zos-minimal-server << 'EOF'
#!/bin/bash
echo "ZOS Server placeholder - replace with actual binary"
sleep infinity
EOF
chmod +x /opt/zos/bin/zos-minimal-server

# Create ZOS configuration
cat > /opt/zos/zos-config.toml << EOF
[server]
http_port = 8080
domain = "$ZOS_DOMAIN"

[ddns]
enabled = false

[services]
max_concurrent_users = 50
block_duration_ms = 400
free_tier_credits = 100
EOF

chown zos:zos /opt/zos/zos-config.toml

# Create systemd service
cat > /etc/systemd/system/zos-server.service << EOF
[Unit]
Description=ZOS Server - Zero Ontology System
After=network.target
Wants=network.target

[Service]
Type=simple
User=zos
Group=zos
WorkingDirectory=/opt/zos
ExecStart=/opt/zos/bin/zos-minimal-server
Restart=always
RestartSec=5
Environment=ZOS_HTTP_PORT=8080
Environment=ZOS_DOMAIN=$ZOS_DOMAIN

# Security settings
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/opt/zos/data

[Install]
WantedBy=multi-user.target
EOF

# Configure nginx reverse proxy
cat > /etc/nginx/conf.d/zos.conf << EOF
server {
    listen 80;
    server_name $ZOS_DOMAIN;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;

        # WebSocket support
        proxy_http_version 1.1;
        proxy_set_header Upgrade \$http_upgrade;
        proxy_set_header Connection "upgrade";
    }
}
EOF

# Enable services
systemctl daemon-reload
systemctl enable nginx
systemctl enable zos-server

# Start nginx
systemctl start nginx

# Start ZOS server
systemctl start zos-server

# Setup Let's Encrypt (will fail initially until DNS is configured)
# certbot --nginx -d $ZOS_DOMAIN --non-interactive --agree-tos --email admin@$ZOS_DOMAIN || true

echo "✅ ZOS Server bootstrap complete!"
echo ""
echo "🌐 Server accessible at:"
echo "   HTTP:  http://$ZOS_DOMAIN"
echo "   HTTPS: https://$ZOS_DOMAIN (after DNS + SSL setup)"
echo ""
echo "📋 Next steps:"
echo "1. Configure DNS: $ZOS_DOMAIN -> $(curl -s ifconfig.me)"
echo "2. Run: certbot --nginx -d $ZOS_DOMAIN"
echo "3. Upload ZOS binary to /opt/zos/bin/zos-minimal-server"
echo "4. Restart: systemctl restart zos-server"
