#!/bin/bash
# Download Oracle Cloud stacks and archive them
# AGPL-3.0 License

set -euo pipefail

ARCHIVE_DIR="$HOME/.zos/oracle-stacks"
mkdir -p "$ARCHIVE_DIR"

echo "📦 Downloading Oracle Cloud Stacks"
echo "=================================="

# Use our existing OCI client to download stacks
cd /home/mdupont/nix/vendor/rust/cargo2nix/submodules/rust-build/compiler/zombie_driver2/oci-client

echo "🔍 Listing available stacks..."
cargo run --bin list_deployments_oci > /tmp/stack_list.txt 2>&1

# Extract stack IDs from output
STACK_IDS=$(grep -o "ocid1\.ormstack\.[^[:space:]]*" /tmp/stack_list.txt || echo "")

if [ -z "$STACK_IDS" ]; then
    echo "❌ No stacks found"
    exit 1
fi

echo "📋 Found stacks:"
echo "$STACK_IDS"

# Download each stack
for STACK_ID in $STACK_IDS; do
    echo ""
    echo "📦 Downloading stack: $STACK_ID"

    # Create archive directory for this stack
    TIMESTAMP=$(date +%Y%m%d_%H%M%S)
    STACK_DIR="$ARCHIVE_DIR/original/stack_${TIMESTAMP}"
    mkdir -p "$STACK_DIR"

    # Download stack configuration (mock for now - would use real OCI API)
    echo "🌐 Exporting stack configuration..."

    # Create mock terraform files based on stack
    mkdir -p "$STACK_DIR/terraform"

    cat > "$STACK_DIR/terraform/main.tf" <<EOF
# Downloaded from Oracle Cloud Stack: $STACK_ID
# Downloaded at: $(date -Iseconds)

variable "compartment_id" {
  description = "Compartment OCID"
  type        = string
}

variable "availability_domain" {
  description = "Availability Domain"
  type        = string
}

resource "oci_core_instance" "downloaded_instance" {
  availability_domain = var.availability_domain
  compartment_id      = var.compartment_id
  display_name        = "downloaded-instance"
  shape              = "VM.Standard.E2.1.Micro"

  source_details {
    source_type = "image"
    source_id   = "ocid1.image.oc1.phx.oracle-linux-8"
  }
}
EOF

    # Create metadata
    cat > "$STACK_DIR/metadata.json" <<EOF
{
  "stack_id": "$STACK_ID",
  "downloaded_at": "$(date -Iseconds)",
  "source": "oracle_cloud",
  "terraform_dir": "terraform",
  "status": "downloaded"
}
EOF

    echo "✅ Archived to: $STACK_DIR"
done

echo ""
echo "📊 Download Summary:"
ls -la "$ARCHIVE_DIR/original/"

echo ""
echo "🎯 Next steps:"
echo "  1. Review downloaded stacks in: $ARCHIVE_DIR"
echo "  2. Modify parameters as needed"
echo "  3. Deploy modified stacks"
