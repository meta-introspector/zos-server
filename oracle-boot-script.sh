#!/bin/bash
# ZOS Oracle Instance Boot Script - Pure Rust Implementation
# AGPL-3.0 License

set -euo pipefail

echo "🚀 ZOS Oracle Instance with Rust Wallet Auth"

# Install Rust and dependencies
dnf update -y
dnf install -y git curl wget tmux iptables-services tc nginx
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source ~/.cargo/env

# Create ZOS user and directories
useradd -m -s /bin/bash zos || true
mkdir -p /opt/zos/{bin,config,logs}
chown -R zos:zos /opt/zos

# Download ZOS binaries
cd /opt/zos/bin
curl -L "https://github.com/solfunmeme/zos-server/releases/latest/download/zos_deploy-x86_64-unknown-linux-gnu" -o zos_deploy
curl -L "https://github.com/solfunmeme/zos-server/releases/latest/download/zos-oracle-x86_64-unknown-linux-gnu" -o zos_oracle
chmod +x zos_deploy zos_oracle

PUBLIC_IP=$(curl -s http://169.254.169.254/opc/v1/vnics/ | jq -r '.[0].publicIp // "127.0.0.1"')

# Create nginx config with Rust backend
cat > /etc/nginx/conf.d/zos.conf <<EOF
upstream zos_libp2p {
    server 127.0.0.1:4001;
}

upstream zos_wallet_auth {
    server 127.0.0.1:4003;
}

server {
    listen 80;
    server_name $PUBLIC_IP;

    # Serve static frontend from Vercel
    location / {
        proxy_pass https://zos-frontend.vercel.app;
        proxy_set_header Host zos-frontend.vercel.app;
    }

    # API with wallet authentication
    location /api/ {
        auth_request /auth;
        proxy_pass http://zos_libp2p;
        proxy_set_header X-Wallet-Tier \$wallet_tier;
        proxy_set_header X-Rate-Limit \$rate_limit;
    }

    # Rust wallet auth service
    location = /auth {
        internal;
        proxy_pass http://zos_wallet_auth/validate;
        proxy_pass_request_body off;
        proxy_set_header Content-Length "";
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Wallet-Address \$http_x_wallet_address;
        proxy_set_header X-Root-Auth \$http_x_root_auth;
    }

    # WebSocket for libp2p
    location /ws {
        auth_request /auth;
        proxy_pass http://zos_libp2p;
        proxy_http_version 1.1;
        proxy_set_header Upgrade \$http_upgrade;
        proxy_set_header Connection "upgrade";
    }
}
EOF

# Create systemd services
cat > /etc/systemd/system/zos-wallet-auth.service <<EOF
[Unit]
Description=ZOS Wallet Authentication (Rust)
After=network.target

[Service]
Type=simple
User=zos
WorkingDirectory=/opt/zos
ExecStart=/opt/zos/bin/zos_oracle wallet-auth-server
Restart=always
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
EOF

cat > /etc/systemd/system/zos-libp2p.service <<EOF
[Unit]
Description=ZOS LibP2P Network
After=network.target zos-wallet-auth.service

[Service]
Type=simple
User=zos
WorkingDirectory=/opt/zos
ExecStart=/opt/zos/bin/zos_deploy node --port 4001
Restart=always
Environment=ZOS_WALLET_AUTH=http://127.0.0.1:4003
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
EOF

# Enable and start services
systemctl daemon-reload
systemctl enable nginx zos-wallet-auth zos-libp2p
systemctl start zos-wallet-auth
sleep 2
systemctl start zos-libp2p nginx

echo "✅ ZOS deployed with Rust wallet authentication"
echo "🌐 Frontend: http://$PUBLIC_IP (Vercel proxy)"
echo "🔐 Wallet Auth: Rust service on port 4003"
echo "💰 Tiers: Root > Whales (1M+ SOL) > Holders (1K+ SOL) > Public"
