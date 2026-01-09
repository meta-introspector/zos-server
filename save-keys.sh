#!/bin/bash
# Save server private key locally, commit public key only
# AGPL-3.0 License

INSTANCE_DIR="accounts/solfunmeme/instances/instance-1"
LOCAL_KEY_DIR="$HOME/.zos/keys"

# Create local key storage
mkdir -p "$LOCAL_KEY_DIR"

# Move private key to local storage
mv "$INSTANCE_DIR/peer_identity.json" "$LOCAL_KEY_DIR/instance-1-private.json"
chmod 600 "$LOCAL_KEY_DIR/instance-1-private.json"

# Create public-only version for git
PEER_ID=$(jq -r '.peer_id' "$LOCAL_KEY_DIR/instance-1-private.json")
CREATED=$(jq -r '.created' "$LOCAL_KEY_DIR/instance-1-private.json")

cat > "$INSTANCE_DIR/peer_identity.json" <<EOF
{
  "peer_id": "$PEER_ID",
  "instance_id": "1",
  "account": "solfunmeme",
  "created": $CREATED,
  "coordinator": true,
  "note": "Private key stored locally at ~/.zos/keys/instance-1-private.json"
}
EOF

echo "🔐 Private key saved to: $LOCAL_KEY_DIR/instance-1-private.json"
echo "📤 Public key ready for commit: $INSTANCE_DIR/peer_identity.json"
