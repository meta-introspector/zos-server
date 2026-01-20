#!/bin/bash
STORE_PATH=$(readlink -f /nix/store/zos-server-latest)
HASH=$(basename "$STORE_PATH" | cut -d'-' -f1)
echo "🚀 Starting ZOS server from store: $HASH"
cd "$STORE_PATH"
exec ./bin/zos_server serve
