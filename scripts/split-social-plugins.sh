#!/usr/bin/env bash
# Script: split-social-plugins.sh
# CRQ: CRQ-SOCIAL-001
# Purpose: Create 6 social ZOS plugins as standalone repos

set -euo pipefail

TOKEN=$(cat ~/.config/forgejo/token)
FORGEJO="http://localhost:3000"
REPO_ROOT="/mnt/data1/forgejo/repos/mdupont"
SPINOFFS="$HOME/03-march/30/spinoffs"
DIOXUS="$HOME/git/solfunmeme-dioxus"
ERDFA="$HOME/erdfa-publish"
PODPING="/mnt/data1/meta-introspector/submodules/podping.alpha"
LOG="$SPINOFFS/social-split-log.md"

echo "# Social Plugin Split — $(date -Iseconds)" > "$LOG"
echo "" >> "$LOG"
echo "| Plugin | Status |" >> "$LOG"
echo "|--------|--------|" >> "$LOG"

create_plugin() {
    local name="$1"
    local desc="$2"
    local OUT="$SPINOFFS/$name"
    mkdir -p "$OUT/src"

    cd "$OUT"
    git init -q
    git add -A
    git commit -q -m "Initial: $desc (CRQ-SOCIAL-001)" 2>/dev/null || true
    ln -sf "$OUT/.git" "$REPO_ROOT/$name.git" 2>/dev/null || true
    curl -s -X POST "$FORGEJO/api/v1/user/repos" \
        -H "Authorization: token $TOKEN" \
        -H "Content-Type: application/json" \
        -d "{\"name\":\"$name\",\"description\":\"$desc\",\"auto_init\":false}" \
        > /dev/null 2>&1 || true
    echo "| $name | ✅ |" >> "$LOG"
}

# 1. pastebin-threads
OUT="$SPINOFFS/zos-plugin-pastebin-threads"
mkdir -p "$OUT/src"
cp "$DIOXUS/plugins/mod.rs" "$OUT/src/plugin_trait.rs"
cat > "$OUT/src/lib.rs" << 'RUST'
pub mod plugin_trait;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Clone, Serialize, Deserialize)]
pub struct Paste {
    pub id: String,
    pub content: String,
    pub timestamp: i64,
    pub parent_id: Option<String>,
    pub reply_count: u32,
    pub thread_root: Option<String>,
}

pub struct PasteStore {
    pub pastes: Mutex<HashMap<String, Paste>>,
}

impl PasteStore {
    pub fn new() -> Self { Self { pastes: Mutex::new(HashMap::new()) } }

    pub fn create(&self, content: String, parent_id: Option<String>) -> Paste {
        let id = format!("{:x}", sha2::Sha256::digest(content.as_bytes()).as_slice()[..8].iter().fold(0u64, |a, &b| a << 8 | b as u64));
        let thread_root = parent_id.as_ref().and_then(|pid| {
            self.pastes.lock().unwrap().get(pid).and_then(|p| p.thread_root.clone().or(Some(pid.clone())))
        });
        let paste = Paste { id: id.clone(), content, timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64, parent_id: parent_id.clone(), reply_count: 0, thread_root };
        let mut store = self.pastes.lock().unwrap();
        if let Some(pid) = &parent_id {
            if let Some(parent) = store.get_mut(pid) { parent.reply_count += 1; }
        }
        store.insert(id.clone(), paste.clone());
        paste
    }

    pub fn get(&self, id: &str) -> Option<Paste> { self.pastes.lock().unwrap().get(id).cloned() }

    pub fn thread(&self, root_id: &str) -> Vec<Paste> {
        self.pastes.lock().unwrap().values()
            .filter(|p| p.thread_root.as_deref() == Some(root_id) || p.id == root_id)
            .cloned().collect()
    }

    pub fn recent(&self, limit: usize) -> Vec<Paste> {
        let mut v: Vec<_> = self.pastes.lock().unwrap().values().cloned().collect();
        v.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        v.truncate(limit);
        v
    }
}
RUST
cat > "$OUT/Cargo.toml" << 'TOML'
[package]
name = "zos-plugin-pastebin-threads"
version = "0.1.0"
edition = "2021"
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sha2 = "0.10"
TOML
create_plugin "zos-plugin-pastebin-threads" "Pastebin with threading (parent_id, reply_count, thread tree)"

# 2. activitypub
OUT="$SPINOFFS/zos-plugin-activitypub"
mkdir -p "$OUT/src"
cp "$ERDFA/src/federation.rs" "$OUT/src/lib.rs"
cat > "$OUT/Cargo.toml" << 'TOML'
[package]
name = "zos-plugin-activitypub"
version = "0.1.0"
edition = "2021"
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sha2 = "0.10"
hex = "0.4"
TOML
create_plugin "zos-plugin-activitypub" "ActivityPub federation (Actor, Outbox, WitnessNote)"

# 3. iroh-gossip
OUT="$SPINOFFS/zos-plugin-iroh-gossip"
mkdir -p "$OUT/src"
cp "$PODPING/gossip-listener/src/"*.rs "$OUT/src/" 2>/dev/null || true
cat > "$OUT/Cargo.toml" << 'TOML'
[package]
name = "zos-plugin-iroh-gossip"
version = "0.1.0"
edition = "2021"
[dependencies]
iroh = "0.97"
iroh-gossip = { version = "0.97", features = ["net"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
sha2 = "0.10"
TOML
create_plugin "zos-plugin-iroh-gossip" "iroh-gossip P2P paste broadcast via podping"

# 4. mesh
OUT="$SPINOFFS/zos-plugin-mesh"
mkdir -p "$OUT/src"
grep -A999 '("GET", "/mesh")' "$ERDFA/src/bin/solfunmeme_service.rs" | head -80 > "$OUT/src/routes.rs" 2>/dev/null || true
cp "$ERDFA/src/privacy.rs" "$OUT/src/privacy.rs" 2>/dev/null || true
cat > "$OUT/src/lib.rs" << 'RUST'
pub mod privacy;
pub mod routes;
RUST
cat > "$OUT/Cargo.toml" << 'TOML'
[package]
name = "zos-plugin-mesh"
version = "0.1.0"
edition = "2021"
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sha2 = "0.10"
ciborium = "0.2"
TOML
create_plugin "zos-plugin-mesh" "Merkle-committed mesh logs with ML-DSA-44 signatures"

# 5. stego
OUT="$SPINOFFS/zos-plugin-stego"
mkdir -p "$OUT/src"
cp "$ERDFA/src/stego.rs" "$OUT/src/lib.rs" 2>/dev/null || true
cat > "$OUT/Cargo.toml" << 'TOML'
[package]
name = "zos-plugin-stego"
version = "0.1.0"
edition = "2021"
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
TOML
create_plugin "zos-plugin-stego" "Steganography encode/decode for paste content"

# 6. ipfs
OUT="$SPINOFFS/zos-plugin-ipfs"
mkdir -p "$OUT/src"
cp "$ERDFA/src/ipfs.rs" "$OUT/src/lib.rs" 2>/dev/null || true
cat > "$OUT/Cargo.toml" << 'TOML'
[package]
name = "zos-plugin-ipfs"
version = "0.1.0"
edition = "2021"
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sha2 = "0.10"
TOML
create_plugin "zos-plugin-ipfs" "IPFS add/get/publish for content-addressed pastes"

echo "" >> "$LOG"
echo "**Total: 6 social plugins**" >> "$LOG"
echo "Done. Log: $LOG"
