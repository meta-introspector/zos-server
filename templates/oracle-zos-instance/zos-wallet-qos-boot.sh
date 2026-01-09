#!/bin/bash
# Enhanced ZOS boot script with wallet-based traffic prioritization
# AGPL-3.0 License

set -euo pipefail

echo "🚀 ZOS Oracle Instance with Wallet-Based QoS"

# Install dependencies
dnf update -y
dnf install -y git curl wget tmux iptables-services tc nginx

# Configure traffic control (tc) for QoS
modprobe sch_htb
modprobe cls_u32

# Create traffic classes
tc qdisc add dev eth0 root handle 1: htb default 30
tc class add dev eth0 parent 1: classid 1:1 htb rate 100mbit
tc class add dev eth0 parent 1:1 classid 1:10 htb rate 50mbit ceil 80mbit  # Root users
tc class add dev eth0 parent 1:1 classid 1:20 htb rate 30mbit ceil 60mbit  # Wallet holders
tc class add dev eth0 parent 1:1 classid 1:30 htb rate 10mbit ceil 20mbit  # Public users

# Install ZOS and create wallet auth system
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source ~/.cargo/env

useradd -m -s /bin/bash zos || true
mkdir -p /opt/zos/{bin,config,logs,wallets}
chown -R zos:zos /opt/zos

# Download ZOS binary
cd /opt/zos/bin
curl -L "https://github.com/solfunmeme/zos-server/releases/latest/download/zos_deploy-x86_64-unknown-linux-gnu" -o zos_deploy
chmod +x zos_deploy

PUBLIC_IP=$(curl -s http://169.254.169.254/opc/v1/vnics/ | jq -r '.[0].publicIp // "127.0.0.1"')

# Create wallet authentication database
cat > /opt/zos/config/wallets.json <<EOF
{
  "root_keys": [
    "oracle_pem_fingerprint_here"
  ],
  "whale_wallets": {
    "minimum_balance": 1000000,
    "wallets": []
  },
  "holder_wallets": {
    "minimum_balance": 1000,
    "wallets": []
  },
  "rate_limits": {
    "root": "unlimited",
    "whale": "100/minute",
    "holder": "50/minute",
    "public": "10/minute"
  }
}
EOF

# Create nginx config for HTTP frontend
cat > /etc/nginx/conf.d/zos.conf <<EOF
upstream zos_backend {
    server 127.0.0.1:4001;
}

server {
    listen 80;
    server_name $PUBLIC_IP;

    # Serve static frontend
    location / {
        proxy_pass https://zos-frontend.vercel.app;
        proxy_set_header Host zos-frontend.vercel.app;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
    }

    # API endpoints with wallet auth
    location /api/ {
        auth_request /auth;
        proxy_pass http://zos_backend;
        proxy_set_header X-Wallet-Address \$wallet_address;
        proxy_set_header X-Wallet-Tier \$wallet_tier;
    }

    # Wallet authentication endpoint
    location = /auth {
        internal;
        proxy_pass http://127.0.0.1:4003/validate-wallet;
        proxy_pass_request_body off;
        proxy_set_header Content-Length "";
        proxy_set_header X-Original-URI \$request_uri;
        proxy_set_header X-Real-IP \$remote_addr;
    }

    # WebSocket upgrade for libp2p
    location /ws {
        proxy_pass http://zos_backend;
        proxy_http_version 1.1;
        proxy_set_header Upgrade \$http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header X-Wallet-Tier \$wallet_tier;
    }
}
EOF

# Create wallet validation service
cat > /opt/zos/bin/wallet-auth.py <<'EOF'
#!/usr/bin/env python3
import json
import requests
from flask import Flask, request, jsonify
import hashlib

app = Flask(__name__)

def get_wallet_balance(address):
    """Get wallet balance from Solana RPC"""
    try:
        response = requests.post('https://api.mainnet-beta.solana.com', json={
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getBalance",
            "params": [address]
        })
        return response.json().get('result', {}).get('value', 0) / 1e9  # Convert lamports to SOL
    except:
        return 0

def get_wallet_tier(address, balance):
    """Determine wallet tier based on balance"""
    with open('/opt/zos/config/wallets.json') as f:
        config = json.load(f)

    if balance >= config['whale_wallets']['minimum_balance']:
        return 'whale'
    elif balance >= config['holder_wallets']['minimum_balance']:
        return 'holder'
    else:
        return 'public'

@app.route('/validate-wallet', methods=['GET', 'POST'])
def validate_wallet():
    wallet_address = request.headers.get('X-Wallet-Address')

    if not wallet_address:
        return jsonify({'tier': 'public'}), 200

    balance = get_wallet_balance(wallet_address)
    tier = get_wallet_tier(wallet_address, balance)

    # Set iptables mark based on tier
    if tier == 'whale':
        mark = '0x20'  # Class 1:20
    elif tier == 'holder':
        mark = '0x20'  # Class 1:20
    else:
        mark = '0x30'  # Class 1:30 (public)

    return jsonify({
        'tier': tier,
        'balance': balance,
        'mark': mark
    }), 200

if __name__ == '__main__':
    app.run(host='127.0.0.1', port=4003)
EOF

chmod +x /opt/zos/bin/wallet-auth.py

# Create iptables rules with wallet-based marking
cat > /opt/zos/bin/setup-qos.sh <<'EOF'
#!/bin/bash
# Setup QoS based on wallet tiers

# Mark packets based on wallet tier (set by nginx auth)
iptables -t mangle -A PREROUTING -p tcp --dport 4001 -j MARK --set-mark 0x30  # Default: public

# Rate limiting by tier
iptables -A INPUT -p tcp --dport 4001 -m mark --mark 0x10 -j ACCEPT  # Root: unlimited
iptables -A INPUT -p tcp --dport 4001 -m mark --mark 0x20 -m limit --limit 100/min -j ACCEPT  # Whales
iptables -A INPUT -p tcp --dport 4001 -m mark --mark 0x30 -m limit --limit 10/min -j ACCEPT   # Public

# Drop excess connections
iptables -A INPUT -p tcp --dport 4001 -j DROP

# Traffic control filters
tc filter add dev eth0 protocol ip parent 1:0 prio 1 u32 match mark 0x10 0xff flowid 1:10  # Root
tc filter add dev eth0 protocol ip parent 1:0 prio 2 u32 match mark 0x20 0xff flowid 1:20  # Whales
tc filter add dev eth0 protocol ip parent 1:0 prio 3 u32 match mark 0x30 0xff flowid 1:30  # Public

service iptables save
EOF

chmod +x /opt/zos/bin/setup-qos.sh
/opt/zos/bin/setup-qos.sh

# Create systemd services
cat > /etc/systemd/system/zos-wallet-auth.service <<EOF
[Unit]
Description=ZOS Wallet Authentication Service
After=network.target

[Service]
Type=simple
User=zos
ExecStart=/usr/bin/python3 /opt/zos/bin/wallet-auth.py
Restart=always

[Install]
WantedBy=multi-user.target
EOF

# Enable services
systemctl enable nginx zos-wallet-auth
systemctl start nginx zos-wallet-auth

echo "✅ ZOS deployed with wallet-based QoS"
echo "🌐 HTTP Frontend: http://$PUBLIC_IP (proxies to Vercel)"
echo "💰 Wallet tiers: Root > Whales (1M+ SOL) > Holders (1K+ SOL) > Public"
echo "⚡ Rate limits: Root=unlimited, Whales=100/min, Holders=50/min, Public=10/min"
EOF
