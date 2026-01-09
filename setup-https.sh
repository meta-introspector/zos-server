#!/bin/bash

# ZOS Server HTTPS Setup Script
# Sets up Let's Encrypt SSL certificates and dynamic DNS

set -e

DOMAIN=${1:-"node1.solfunmeme.com"}
EMAIL=${2:-"admin@solfunmeme.com"}
HTTP_PORT=8080
HTTPS_PORT=8443

echo "🔐 Setting up HTTPS for ZOS Server"
echo "Domain: $DOMAIN"
echo "HTTP Port: $HTTP_PORT"
echo "HTTPS Port: $HTTPS_PORT"

# Install dependencies
echo "📦 Installing dependencies..."
sudo apt update
sudo apt install -y certbot nginx curl dnsutils

# Setup dynamic DNS (Namecheap)
echo "🌐 Setting up Namecheap Dynamic DNS..."
read -p "Enter your domain (e.g., solfunmeme.com): " DOMAIN
read -p "Enter subdomain/host (e.g., node1 or @ for root): " HOST
read -s -p "Enter Namecheap DDNS password: " DDNS_PASSWORD
echo

if [ ! -z "$DDNS_PASSWORD" ]; then
    # Install Python DDNS client
    sudo cp namecheap_ddns.py /usr/local/bin/
    sudo chmod +x /usr/local/bin/namecheap_ddns.py

    # Create environment file
    sudo tee /etc/default/namecheap-ddns > /dev/null <<EOF
NAMECHEAP_DOMAIN=$DOMAIN
NAMECHEAP_HOST=$HOST
NAMECHEAP_PASSWORD=$DDNS_PASSWORD
DDNS_DAEMON=true
EOF

    # Create systemd service for DDNS
    sudo tee /etc/systemd/system/namecheap-ddns.service > /dev/null <<EOF
[Unit]
Description=Namecheap Dynamic DNS Client
After=network.target
Wants=network.target

[Service]
Type=simple
User=nobody
Group=nogroup
EnvironmentFile=/etc/default/namecheap-ddns
ExecStart=/usr/bin/python3 /usr/local/bin/namecheap_ddns.py --daemon
Restart=always
RestartSec=60

[Install]
WantedBy=multi-user.target
EOF

    # Enable and start DDNS service
    sudo systemctl daemon-reload
    sudo systemctl enable namecheap-ddns
    sudo systemctl start namecheap-ddns

    # Test initial update
    echo "🧪 Testing initial DNS update..."
    sudo -u nobody python3 /usr/local/bin/namecheap_ddns.py $DOMAIN $DDNS_PASSWORD $HOST

    echo "✅ Namecheap Dynamic DNS configured"
    echo "   Domain: $HOST.$DOMAIN"
    echo "   Service: systemctl status namecheap-ddns"
    echo "   Logs: journalctl -u namecheap-ddns -f"
else
    echo "⚠️  Skipping dynamic DNS setup"
fi

# Create nginx configuration for ACME challenge
echo "⚙️  Configuring nginx for Let's Encrypt..."
sudo tee /etc/nginx/sites-available/zos-server > /dev/null <<EOF
server {
    listen 80;
    server_name $DOMAIN;

    # ACME challenge location
    location /.well-known/acme-challenge/ {
        root /var/www/html;
    }

    # Redirect everything else to HTTPS
    location / {
        return 301 https://\$server_name\$request_uri;
    }
}

server {
    listen 443 ssl http2;
    server_name $DOMAIN;

    # SSL configuration (will be updated after cert generation)
    ssl_certificate /etc/letsencrypt/live/$DOMAIN/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/$DOMAIN/privkey.pem;

    # Modern SSL configuration
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-RSA-AES256-GCM-SHA512:DHE-RSA-AES256-GCM-SHA512:ECDHE-RSA-AES256-GCM-SHA384:DHE-RSA-AES256-GCM-SHA384;
    ssl_prefer_server_ciphers off;
    ssl_session_cache shared:SSL:10m;

    # Security headers
    add_header Strict-Transport-Security "max-age=63072000" always;
    add_header X-Content-Type-Options nosniff;
    add_header X-Frame-Options DENY;
    add_header X-XSS-Protection "1; mode=block";

    # Proxy to ZOS server
    location / {
        proxy_pass http://127.0.0.1:$HTTP_PORT;
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;

        # WebSocket support
        proxy_http_version 1.1;
        proxy_set_header Upgrade \$http_upgrade;
        proxy_set_header Connection "upgrade";
    }

    # LibP2P WebSocket endpoint
    location /libp2p {
        proxy_pass http://127.0.0.1:4001;
        proxy_http_version 1.1;
        proxy_set_header Upgrade \$http_upgrade;
        proxy_set_header Connection "upgrade";
    }
}
EOF

# Enable the site
sudo ln -sf /etc/nginx/sites-available/zos-server /etc/nginx/sites-enabled/
sudo rm -f /etc/nginx/sites-enabled/default

# Test nginx configuration
sudo nginx -t

# Start nginx
sudo systemctl enable nginx
sudo systemctl restart nginx

# Wait for DNS propagation
echo "⏳ Waiting for DNS propagation..."
sleep 10

# Check if domain resolves
if ! nslookup $DOMAIN > /dev/null 2>&1; then
    echo "⚠️  Warning: $DOMAIN does not resolve yet. You may need to wait for DNS propagation."
    echo "   You can continue once DNS is working."
    read -p "Press Enter when DNS is ready..."
fi

# Obtain Let's Encrypt certificate
echo "🔐 Obtaining SSL certificate..."
sudo certbot certonly \
    --webroot \
    --webroot-path=/var/www/html \
    --email $EMAIL \
    --agree-tos \
    --no-eff-email \
    --domains $DOMAIN

# Setup auto-renewal
echo "🔄 Setting up certificate auto-renewal..."
sudo systemctl enable certbot.timer
sudo systemctl start certbot.timer

# Test renewal
sudo certbot renew --dry-run

# Reload nginx with new certificates
sudo systemctl reload nginx

# Create ZOS server systemd service
echo "🚀 Creating ZOS server service..."
sudo tee /etc/systemd/system/zos-server.service > /dev/null <<EOF
[Unit]
Description=ZOS Server - Zero Ontology System
After=network.target
Wants=network.target

[Service]
Type=simple
User=zos
Group=zos
WorkingDirectory=/opt/zos
ExecStart=/opt/zos/zos-stage1-server
Restart=always
RestartSec=5
Environment=ZOS_HTTP_PORT=$HTTP_PORT
Environment=ZOS_HTTPS_PORT=$HTTPS_PORT
Environment=ZOS_DOMAIN=$DOMAIN

# Security settings
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/opt/zos/data

[Install]
WantedBy=multi-user.target
EOF

# Create zos user
sudo useradd -r -s /bin/false -d /opt/zos zos || true
sudo mkdir -p /opt/zos/data
sudo chown -R zos:zos /opt/zos

# Create ZOS configuration
sudo tee /opt/zos/zos-config.toml > /dev/null <<EOF
[server]
http_port = $HTTP_PORT
https_port = $HTTPS_PORT
domain = "$DOMAIN"
libp2p_port = 4001

[ssl]
cert_path = "/etc/letsencrypt/live/$DOMAIN/fullchain.pem"
key_path = "/etc/letsencrypt/live/$DOMAIN/privkey.pem"
auto_reload = true

[blockchain]
solana_rpc = "https://api.mainnet-beta.solana.com"
solfunmeme_mint = "SoLFuNMeMeTokenAddress123456789"

[services]
max_concurrent_users = 50
block_duration_ms = 400
free_tier_credits = 100

[payments]
usdc_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
swap_fee_percentage = 0.3
commission_rates = { swap = 20.0, referral = 10.0, service = 5.0 }
EOF

sudo chown zos:zos /opt/zos/zos-config.toml

echo ""
echo "✅ HTTPS setup complete!"
echo ""
echo "🌐 Your ZOS server will be available at:"
echo "   https://$DOMAIN"
echo ""
echo "📋 Next steps:"
echo "1. Copy your ZOS server binary to /opt/zos/zos-stage1-server"
echo "2. sudo systemctl enable zos-server"
echo "3. sudo systemctl start zos-server"
echo ""
echo "🔧 Useful commands:"
echo "   sudo systemctl status zos-server    # Check server status"
echo "   sudo journalctl -u zos-server -f    # View server logs"
echo "   sudo certbot certificates           # Check SSL certificates"
echo "   sudo nginx -t && sudo systemctl reload nginx  # Reload nginx config"
echo ""
echo "🔐 SSL certificate will auto-renew via systemd timer"
echo "🌐 Dynamic DNS will update every 5 minutes via cron"
