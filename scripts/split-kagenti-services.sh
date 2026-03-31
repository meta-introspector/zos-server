#!/usr/bin/env bash
# Script: split-kagenti-services.sh
# CRQ: CRQ-ZOS-001 (extension)
# Purpose: Create a repo per kagenti service on Forgejo

set -euo pipefail

TOKEN=$(cat ~/.config/forgejo/token)
FORGEJO="http://localhost:3000"
REPO_ROOT="/mnt/data1/forgejo/repos/mdupont"
SPINOFFS="$HOME/03-march/30/spinoffs"
LOG="$SPINOFFS/kagenti-split-log.md"

echo "# kagenti Service Repos — $(date -Iseconds)" > "$LOG"
echo "" >> "$LOG"
echo "| Service | Repo | Status |" >> "$LOG"
echo "|---------|------|--------|" >> "$LOG"

create_service_repo() {
    local name="$1"
    local desc="$2"
    local OUT="$SPINOFFS/kagenti-$name"
    mkdir -p "$OUT"

    cat > "$OUT/README.md" << EOF
# kagenti: $name

$desc

## Service

\`\`\`bash
systemctl status $name.service
kagenti-ctl create agent --name $name --namespace default --service $name.service
\`\`\`

## See Also

- [kagenti FLEET](https://localhost:3000/mdupont/DOCS/src/branch/main/services/kagenti/FLEET.md)
- [kagenti INTEGRATION](https://localhost:3000/mdupont/DOCS/src/branch/main/services/kagenti/INTEGRATION.md)
EOF

    cd "$OUT"
    git init -q
    git add -A
    git commit -q -m "Initial: kagenti service $name (CRQ-ZOS-001)"
    ln -sf "$OUT/.git" "$REPO_ROOT/kagenti-$name.git" 2>/dev/null || true
    curl -s -X POST "$FORGEJO/api/v1/user/repos" \
        -H "Authorization: token $TOKEN" \
        -H "Content-Type: application/json" \
        -d "{\"name\":\"kagenti-$name\",\"description\":\"kagenti service: $desc\",\"auto_init\":false}" \
        > /dev/null 2>&1 || true
    echo "| $name | kagenti-$name | ✅ |" >> "$LOG"
}

# Active services
create_service_repo "activity-profile" "User timeline service (port 8093)"
create_service_repo "eco-research" "Eco research agent + spool (port 8091)"
create_service_repo "monster-cft-file-manager" "Monster CFT file manager API"
create_service_repo "solana-witness-protocol" "24-node Solana witness + FRACTRAN"
create_service_repo "zos-metameme" "ZOS server + meta-meme integration"
create_service_repo "zos-prod-node" "ZOS production node"
create_service_repo "zos-qa-node" "ZOS QA node"
create_service_repo "zos-zos" "ZOS-ZOS service"

# Kiro agents
create_service_repo "kiro-session" "Kiro session agent"
create_service_repo "kiro-http" "Kiro HTTP"
create_service_repo "kiro-irc" "Kiro IRC bridge"
create_service_repo "kiro-ldap" "Kiro LDAP"
create_service_repo "kiro-lmfdb" "Kiro LMFDB"
create_service_repo "kiro-proxy" "Kiro proxy"
create_service_repo "kiro-zone42" "Kiro zone42 agent"
create_service_repo "kiro-feed-uucp" "Kiro UUCP feed"
create_service_repo "kiro1-shard42" "Kiro1 shard 42 agent"
create_service_repo "kiro-qms" "Kiro QMS"

# Generated (pending deploy)
create_service_repo "fractranllama" "FRACTRAN LLM service (port 8120)"
create_service_repo "kagenti-portal" "kagenti portal (port 8201)"
create_service_repo "moltis" "Moltis service (port 8130)"
create_service_repo "nixwars-frens" "Nix-Wars friends (port 8114)"
create_service_repo "openclaw" "OpenClaw service (port 8140)"
create_service_repo "pastebin-service" "Pastebin service (port 8150)"
create_service_repo "wg-stego-tunnel" "WireGuard stego tunnel (port 8160)"
create_service_repo "rust-mcp-services" "Rust MCP services (port 8170)"
create_service_repo "solfunmeme-service" "Solfunmeme service (port 8180)"

# Other
create_service_repo "jocko-training" "Prolog training (timer: every 71 min)"
create_service_repo "monster-lattice-indexer" "Monster lattice indexer (timer)"
create_service_repo "nixwars" "Nix-Wars game server"
create_service_repo "snm-paste-server" "SNM paste server"

echo "" >> "$LOG"
TOTAL=$(grep -c '✅' "$LOG")
echo "**Total: $TOTAL service repos**" >> "$LOG"
echo "Done. Log: $LOG"
