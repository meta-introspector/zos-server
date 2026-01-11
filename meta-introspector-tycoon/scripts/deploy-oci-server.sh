#!/bin/bash
# Deploy to Oracle OCI ARM64 Free Tier

echo "🏗️ Deploying to Oracle OCI ARM64..."

# Update system
sudo apt update && sudo apt upgrade -y

# Install WireGuard
sudo apt install -y wireguard

# Generate keys
wg genkey | tee server_private.key | wg pubkey > server_public.key

# Configure WireGuard
sudo cp wg0-server.conf /etc/wireguard/wg0.conf
sudo systemctl enable wg-quick@wg0
sudo systemctl start wg-quick@wg0

# Install Docker for containerized services
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh

# Open firewall
sudo ufw allow 51820/udp
sudo ufw allow 8080/tcp

echo "✅ Oracle OCI ARM64 server deployed"
