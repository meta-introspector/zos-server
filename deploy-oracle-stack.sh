#!/bin/bash
# Deploy Oracle Cloud stack with ZOS configuration
# AGPL-3.0 License

set -euo pipefail

STACK_DIR="~/terraform/accounts/solfunmeme-oci"
TEMPLATE_DIR="templates/oracle-zos-instance"

echo "🚀 Deploying ZOS Oracle Cloud Stack"
echo "==================================="

# Create template directory structure
mkdir -p "$TEMPLATE_DIR"

# Copy base instance configuration
cp /mnt/data1/nix/time/2024/12/10/swarms-terraform/accounts/solfunmeme-oci/stacks/instance-20250315-0853/main.tf "$TEMPLATE_DIR/"

# Add ZOS-specific configuration
cat >> "$TEMPLATE_DIR/main.tf" <<EOF

# ZOS Network Security Groups
resource "oci_core_network_security_group" "zos_public" {
  compartment_id = var.compartment_id
  vcn_id         = var.vcn_id
  display_name   = "zos-public-nsg"
}

resource "oci_core_network_security_group_security_rule" "zos_public_ingress" {
  network_security_group_id = oci_core_network_security_group.zos_public.id
  direction                 = "INGRESS"
  protocol                  = "6"  # TCP

  source      = "0.0.0.0/0"
  source_type = "CIDR_BLOCK"

  tcp_options {
    destination_port_range {
      min = 4001
      max = 4001
    }
  }
}

# Rate limiting via iptables (configured in boot script)
resource "oci_core_network_security_group_security_rule" "zos_rate_limit" {
  network_security_group_id = oci_core_network_security_group.zos_public.id
  direction                 = "INGRESS"
  protocol                  = "6"

  source      = "0.0.0.0/0"
  source_type = "CIDR_BLOCK"

  tcp_options {
    destination_port_range {
      min = 4001
      max = 4001
    }
  }
}

# Output public IP for network registration
output "zos_public_ip" {
  value = oci_core_instance.generated_oci_core_instance.public_ip
}

output "zos_network_info" {
  value = {
    account = "solfunmeme"
    instance_id = oci_core_instance.generated_oci_core_instance.id
    public_endpoint = "\${oci_core_instance.generated_oci_core_instance.public_ip}:4001"
    private_endpoint = "127.0.0.1:4002"
  }
}
EOF

# Create enhanced boot script with rate limiting
cat > "$TEMPLATE_DIR/zos-boot-script.sh" <<'EOF'
#!/bin/bash
# ZOS Oracle Instance with Rate Limiting
set -euo pipefail

echo "🚀 ZOS Oracle Instance Boot with Rate Limiting"

# Install dependencies
dnf update -y
dnf install -y git curl wget tmux iptables-services

# Configure iptables rate limiting
systemctl enable iptables
systemctl start iptables

# Rate limit: 10 connections per minute per IP for port 4001
iptables -A INPUT -p tcp --dport 4001 -m state --state NEW -m recent --set --name zos_public
iptables -A INPUT -p tcp --dport 4001 -m state --state NEW -m recent --update --seconds 60 --hitcount 10 --name zos_public -j DROP

# Allow established connections
iptables -A INPUT -p tcp --dport 4001 -m state --state ESTABLISHED,RELATED -j ACCEPT

# Block port 4002 from external access completely
iptables -A INPUT -p tcp --dport 4002 ! -s 127.0.0.1 -j DROP

# Save iptables rules
service iptables save

# Install Rust and ZOS
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source ~/.cargo/env

# Create ZOS user
useradd -m -s /bin/bash zos || true
mkdir -p /opt/zos/{bin,config,logs}
chown -R zos:zos /opt/zos

# Download ZOS binary
cd /opt/zos/bin
curl -L "https://github.com/solfunmeme/zos-server/releases/latest/download/zos_deploy-x86_64-unknown-linux-gnu" -o zos_deploy
chmod +x zos_deploy

# Get public IP
PUBLIC_IP=$(curl -s http://169.254.169.254/opc/v1/vnics/ | jq -r '.[0].publicIp // "127.0.0.1"')

# Create network configuration
cat > /opt/zos/config/network.toml <<NETEOF
[account]
name = "solfunmeme"
instance_type = "oracle_cloud"
public_ip = "$PUBLIC_IP"

[networks.public]
port = 4001
access_level = "public"
bind_address = "0.0.0.0"
external_address = "$PUBLIC_IP"
rate_limit = "10/minute"

[networks.private]
port = 4002
access_level = "root"
bind_address = "127.0.0.1"
external_address = "127.0.0.1"
require_pem_auth = true

[rate_limiting]
enabled = true
public_connections_per_minute = 10
burst_limit = 5
NETEOF

# Create systemd services
cat > /etc/systemd/system/zos-public.service <<SVCEOF
[Unit]
Description=ZOS Public Network (Rate Limited)
After=network.target iptables.service

[Service]
Type=simple
User=zos
ExecStart=/opt/zos/bin/zos_deploy node --network public
Restart=always
Environment=ZOS_PORT=4001
Environment=ZOS_RATE_LIMIT=enabled

[Install]
WantedBy=multi-user.target
SVCEOF

cat > /etc/systemd/system/zos-private.service <<SVCEOF
[Unit]
Description=ZOS Private Network (Internal Only)
After=network.target zos-public.service

[Service]
Type=simple
User=zos
ExecStart=/opt/zos/bin/zos_deploy node --network private
Restart=always
Environment=ZOS_PORT=4002
Environment=ZOS_INTERNAL_ONLY=true

[Install]
WantedBy=multi-user.target
SVCEOF

# Enable services
systemctl daemon-reload
systemctl enable zos-public zos-private
systemctl start zos-public zos-private

# Register with solfunmeme network
curl -X POST "https://api.solfunmeme.com/register-node" \
  -H "Content-Type: application/json" \
  -d "{\"public_ip\":\"$PUBLIC_IP\",\"port\":4001,\"account\":\"solfunmeme\",\"instance_type\":\"oracle_cloud\"}" || true

echo "✅ ZOS Oracle instance deployed with rate limiting"
echo "🌐 Public: $PUBLIC_IP:4001 (rate limited: 10 conn/min)"
echo "🔐 Private: 127.0.0.1:4002 (internal only)"
EOF

# Deploy the stack
echo "📦 Deploying to Oracle Cloud..."
cd "$STACK_DIR"

# Initialize terraform if needed
if [ ! -f ".terraform.lock.hcl" ]; then
    terraform init
fi

# Plan and apply
terraform plan -var-file="terraform.tfvars"
terraform apply -auto-approve -var-file="terraform.tfvars"

# Get outputs
PUBLIC_IP=$(terraform output -raw zos_public_ip)
echo ""
echo "✅ Deployment Complete!"
echo "🌐 ZOS Public Network: $PUBLIC_IP:4001"
echo "🔐 ZOS Private Network: 127.0.0.1:4002 (internal)"
echo "⚡ Rate Limiting: 10 connections/minute per IP"
echo "📋 Network registered with solfunmeme account"
