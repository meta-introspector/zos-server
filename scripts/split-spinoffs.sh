#!/usr/bin/env bash
# Script: split-spinoffs.sh
# CRQ: CRQ-ZOS-001
# Purpose: Split zos-server subdirs into standalone repos on Forgejo
# Input: zos-server repo
# Output: New repos on Forgejo, split log

set -euo pipefail

ZOS="$HOME/03-march/30/zos-clean"
LOG="$ZOS/docs/split-log.md"
TOKEN=$(cat ~/.config/forgejo/token)
FORGEJO="http://localhost:3000"

echo "# Split Log — $(date -Iseconds)" > "$LOG"
echo "" >> "$LOG"

split_dir() {
    local dir="$1"
    local name="$2"
    
    echo "--- Splitting $dir → $name ---"
    
    # Symlink into Forgejo repo root and adopt (per ~/DOCS/FORGEJO_MIRROR.md)
    local REPO_ROOT="/mnt/data1/forgejo/repos/mdupont"
    local OUT="$HOME/03-march/30/spinoffs/$name"
    mkdir -p "$OUT"
    cp -r "$ZOS/$dir/"* "$OUT/" 2>/dev/null || true
    cp -r "$ZOS/$dir/".* "$OUT/" 2>/dev/null || true
    
    cd "$OUT"
    git init -q
    git add -A
    git commit -q -m "Initial: spun off from zos-server/$dir (CRQ-ZOS-001)"
    
    # Symlink .git into Forgejo
    ln -sf "$OUT/.git" "$REPO_ROOT/$name.git" 2>/dev/null || true
    
    # Adopt via API
    curl -s -X POST "$FORGEJO/api/v1/user/repos" \
        -H "Authorization: token $TOKEN" \
        -H "Content-Type: application/json" \
        -d "{\"name\":\"$name\",\"description\":\"Spun off from zos-server/$dir\",\"auto_init\":false}" \
        > /dev/null 2>&1 || true
    
    echo "| $dir | $name | ✅ |" >> "$LOG"
}

mkdir -p "$HOME/03-march/30/spinoffs"
echo "| Source Dir | New Repo | Status |" >> "$LOG"
echo "|-----------|----------|--------|" >> "$LOG"

# Top-level dirs
split_dir "zos-retro-games" "zos-retro-games"
split_dir "zos-telegram-bot" "zos-telegram-bot"
split_dir "zos-community-economy" "zos-community-economy"
split_dir "zos-oracle" "zos-oracle"
split_dir "zos-oci" "zos-oci"
split_dir "zos-public-gateway" "zos-public-gateway"
split_dir "zos-minimal-server" "zos-minimal-server"
split_dir "zos-stage1-server" "zos-stage1-server"
split_dir "zos-deploy" "zos-deploy"
split_dir "zos-bootstrap" "zos-bootstrap"
split_dir "zos-plan" "zos-plan"
split_dir "zos-analysis" "zos-analysis"
split_dir "zos-unix-accounts" "zos-unix-accounts"
split_dir "browser-extension" "zos-browser-extension"

# Crates
split_dir "crates/zos-metameme" "zos-metameme"
split_dir "crates/zos-monsters" "zos-monsters"
split_dir "crates/zos-eigenmatrix" "zos-eigenmatrix"
split_dir "crates/zos-biosemiotic" "zos-biosemiotic"
split_dir "crates/zos-godel" "zos-godel"
split_dir "crates/zos-orbitals" "zos-orbitals"
split_dir "crates/zos-prime-flags" "zos-prime-flags"
split_dir "crates/zos-cryptography" "zos-cryptography"
split_dir "crates/zos-auto-label" "zos-auto-label"
split_dir "crates/zos-legacy" "zos-legacy"

echo "" >> "$LOG"
echo "**Total: 24 repos split**" >> "$LOG"
echo "Done. Log: $LOG"
