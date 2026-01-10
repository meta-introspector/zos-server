#!/bin/bash
# Deploy ZOS Server to Oracle Cloud using Rust OCI client
# Creates stack via OCI Resource Manager API

set -e

COMPARTMENT_ID=${1:-"ocid1.tenancy.oc1..aaaaaaaapxfkcjaczqslvnbekbqq2eefxgwx7kqbakvddhzaaiym62vmt5la"}
ZOS_DOMAIN=${2:-"node1.solfunmeme.com"}
SSH_PUBLIC_KEY_PATH=${3:-"~/.ssh/id_rsa.pub"}
OCI_CONFIG_PATH=${4:-"~/.solfunmeme-keys/oci_config"}

echo "🚀 Deploying ZOS Server to Oracle Cloud"
echo "Compartment: $COMPARTMENT_ID"
echo "Domain: $ZOS_DOMAIN"
echo "SSH Key: $SSH_PUBLIC_KEY_PATH"
echo "OCI Config: $OCI_CONFIG_PATH"

# Check prerequisites
if [ ! -f "$OCI_CONFIG_PATH" ]; then
    echo "❌ OCI config not found at $OCI_CONFIG_PATH"
    echo "Expected format:"
    echo "user=ocid1.user.oc1..your-user-id"
    echo "tenancy=ocid1.tenancy.oc1..your-tenancy-id"
    echo "fingerprint=your:key:fingerprint"
    echo "region=us-ashburn-1"
    echo "key_file=/path/to/private/key.pem"
    exit 1
fi

if [ ! -f "$SSH_PUBLIC_KEY_PATH" ]; then
    echo "❌ SSH public key not found at $SSH_PUBLIC_KEY_PATH"
    echo "Generate with: ssh-keygen -t rsa -b 4096"
    exit 1
fi

# Read SSH public key
SSH_PUBLIC_KEY=$(cat "$SSH_PUBLIC_KEY_PATH")

# Create terraform.tfvars
cat > templates/oracle-zos-instance/terraform.tfvars << EOF
compartment_id = "$COMPARTMENT_ID"
zos_domain = "$ZOS_DOMAIN"
ssh_public_key = "$SSH_PUBLIC_KEY"
EOF

echo "📋 Created terraform.tfvars"

# Create deployment package
DEPLOYMENT_DIR="zos-oracle-deployment-$(date +%Y%m%d-%H%M%S)"
mkdir -p "$DEPLOYMENT_DIR"

# Copy Terraform files
cp -r templates/oracle-zos-instance/* "$DEPLOYMENT_DIR/"

echo "📦 Created deployment package: $DEPLOYMENT_DIR"

# Create zip file for Resource Manager
cd "$DEPLOYMENT_DIR"
zip -r "../${DEPLOYMENT_DIR}.zip" .
cd ..

echo "📦 Created stack archive: ${DEPLOYMENT_DIR}.zip"

# Use Rust OCI client to create stack
STACK_NAME="zos-server-$(date +%Y%m%d-%H%M%S)"

echo "🔧 Creating OCI Resource Manager stack with Rust client..."

# Build and run the OCI client
cd zos-oracle
cargo build --release --features full

# Create stack using Rust client
STACK_OCID=$(cargo run --features full --bin create_stack -- \
    --config "$OCI_CONFIG_PATH" \
    --compartment "$COMPARTMENT_ID" \
    --name "$STACK_NAME" \
    --zip "../${DEPLOYMENT_DIR}.zip")

if [ $? -eq 0 ]; then
    echo "✅ Stack created successfully!"
    echo "   Stack OCID: $STACK_OCID"
    echo ""
    echo "🌐 OCI Console:"
    echo "   https://cloud.oracle.com/resourcemanager/stacks/$STACK_OCID"
    echo ""
    echo "📋 Next Steps:"
    echo "1. Apply the stack in OCI Console"
    echo "2. Configure DNS: $ZOS_DOMAIN -> <public_ip>"
    echo "3. Upload ZOS binary to the instance"
    echo "4. Setup SSL certificate"
else
    echo "❌ Failed to create stack"
    exit 1
fi

cd ..

echo ""
echo "📋 Deployment Summary:"
echo "   Stack Name: $STACK_NAME"
echo "   Stack OCID: $STACK_OCID"
echo "   Domain: $ZOS_DOMAIN"
echo "   Deployment Package: ${DEPLOYMENT_DIR}.zip"
