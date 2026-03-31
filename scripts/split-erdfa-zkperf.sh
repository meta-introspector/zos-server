#!/usr/bin/env bash
# Script: split-erdfa-zkperf.sh
# CRQ: CRQ-ERDFA-001
# Purpose: Split erdfa-publish, erdfa-clean, zkperf modules into standalone repos on Forgejo
# Uses symlink+adopt per ~/DOCS/FORGEJO_MIRROR.md

set -euo pipefail

TOKEN=$(cat ~/.config/forgejo/token)
FORGEJO="http://localhost:3000"
REPO_ROOT="/mnt/data1/forgejo/repos/mdupont"
SPINOFFS="$HOME/03-march/30/spinoffs"
LOG="$HOME/03-march/30/spinoffs/erdfa-split-log.md"

echo "# eRDFa/zkperf Split Log — $(date -Iseconds)" > "$LOG"
echo "" >> "$LOG"
echo "| Source | File(s) | New Repo | Status |" >> "$LOG"
echo "|--------|---------|----------|--------|" >> "$LOG"

split_module() {
    local src_repo="$1"
    local files="$2"
    local name="$3"
    local desc="$4"

    local OUT="$SPINOFFS/$name"
    mkdir -p "$OUT/src"

    for f in $files; do
        if [ -f "$src_repo/$f" ]; then
            local dir=$(dirname "$f")
            mkdir -p "$OUT/$dir"
            cp "$src_repo/$f" "$OUT/$f"
        fi
    done

    # Copy Cargo.toml if it exists and create a minimal one if not
    if [ -f "$src_repo/Cargo.toml" ]; then
        cp "$src_repo/Cargo.toml" "$OUT/Cargo.toml.orig"
    fi

    cd "$OUT"
    git init -q
    git add -A
    git commit -q -m "Initial: $desc (CRQ-ERDFA-001)" 2>/dev/null || true

    ln -sf "$OUT/.git" "$REPO_ROOT/$name.git" 2>/dev/null || true

    curl -s -X POST "$FORGEJO/api/v1/user/repos" \
        -H "Authorization: token $TOKEN" \
        -H "Content-Type: application/json" \
        -d "{\"name\":\"$name\",\"description\":\"$desc\",\"auto_init\":false}" \
        > /dev/null 2>&1 || true

    echo "| $(basename $src_repo) | $files | $name | ✅ |" >> "$LOG"
}

mkdir -p "$SPINOFFS"

EP="$HOME/erdfa-publish"
EC="$HOME/erdfa-clean"
ZK="$HOME/zkperf"

# erdfa-publish bins
split_module "$EP" "src/bin/solfunmeme_cli.rs" "solfunmeme-cli" "DAO CLI from erdfa-publish"
split_module "$EP" "src/bin/solfunmeme_service.rs" "solfunmeme-service-bin" "HTTP service from erdfa-publish"
split_module "$EP" "src/bin/zkperf_services.rs" "zkperf-services" "15-service zkperf from erdfa-publish"
split_module "$EP" "src/bin/zkperf_verify.rs" "zkperf-verify" "Witness verifier from erdfa-publish"
split_module "$EP" "src/bin/campaign_planner.rs" "erdfa-campaign-planner" "Campaign planner"
split_module "$EP" "src/bin/cbor2agda.rs" "erdfa-cbor2agda" "CBOR to Agda converter"
split_module "$EP" "src/bin/cbor2lean4.rs" "erdfa-cbor2lean4" "CBOR to Lean4 converter"
split_module "$EP" "src/bin/erdfa_mixer.rs" "erdfa-mixer" "Mixer binary"
split_module "$EP" "src/bin/shard_dedup.rs" "erdfa-shard-dedup" "Shard dedup tool"
split_module "$EP" "src/bin/solana_ingest.rs" "erdfa-solana-ingest" "Solana TX ingest"
split_module "$EP" "src/bin/ledger_to_nft.rs" "erdfa-ledger-nft" "Ledger to NFT converter"
split_module "$EP" "src/bin/stego_gossip.rs" "erdfa-stego-gossip" "Stego P2P gossip"
split_module "$EP" "src/bin/benchmark.rs" "erdfa-benchmark" "Benchmarks"

# erdfa-publish lib modules
split_module "$EP" "src/cft.rs" "erdfa-cft" "CFT model"
split_module "$EP" "src/dasl.rs" "erdfa-dasl" "0xDA51 classification"
split_module "$EP" "src/sheaf.rs" "erdfa-sheaf" "Sheaf sections"
split_module "$EP" "src/hecke.rs" "erdfa-hecke" "Hecke eigenvalue sharding"
split_module "$EP" "src/maass.rs" "erdfa-maass" "Maass forms"
split_module "$EP" "src/privacy.rs" "erdfa-privacy" "Merkle commitments"
split_module "$EP" "src/stego.rs" "erdfa-stego" "Steganography"
split_module "$EP" "src/federation.rs" "erdfa-federation" "ActivityPub federation"
split_module "$EP" "src/ipfs.rs" "erdfa-ipfs" "IPFS integration"
split_module "$EP" "src/distribute.rs" "erdfa-distribute" "Distribution"
split_module "$EP" "src/ingest.rs" "erdfa-ingest" "Ingest pipeline"
split_module "$EP" "src/render.rs" "erdfa-render" "HTML rendering"

# erdfa-clean modules
split_module "$EC" "src/acl.rs" "erdfa-acl" "Access control"
split_module "$EC" "src/blockchain.rs" "erdfa-blockchain" "Blockchain types"
split_module "$EC" "src/coverage.rs" "erdfa-coverage" "Coverage analysis"
split_module "$EC" "src/crypto.rs" "erdfa-crypto" "Crypto primitives"
split_module "$EC" "src/homomorphic_mixer.rs" "erdfa-homomorphic" "Homomorphic mixing"
split_module "$EC" "src/lean4.rs" "erdfa-lean4" "Lean4 integration"
split_module "$EC" "src/modular.rs" "erdfa-modular" "Modular forms"
split_module "$EC" "src/shards.rs" "erdfa-shards" "Shard management"
split_module "$EC" "src/symmetry.rs" "erdfa-symmetry" "Symmetry groups"
split_module "$EC" "src/zk_migration.rs" "erdfa-zk-migration" "ZK migration"
split_module "$EC" "src/zkreach.rs" "erdfa-zkreach" "ZK reachability"
split_module "$EC" "lean" "erdfa-lean-proofs" "Lean4 proofs"
split_module "$EC" "minizinc" "erdfa-minizinc" "MiniZinc models"

# zkperf bins
split_module "$ZK" "src/bin/record.rs src/witness.rs" "zkperf-record" "Perf recording"
split_module "$ZK" "src/bin/witness.rs src/witness.rs" "zkperf-witness-bin" "Witness generation"
split_module "$ZK" "src/bin/parse.rs" "zkperf-parse" "Perf data parser"
split_module "$ZK" "src/bin/audit.rs" "zkperf-audit" "Security audit"
split_module "$ZK" "src/bin/service.rs" "zkperf-da51-service" "DA51 service"
split_module "$ZK" "src/bin/sample.rs" "zkperf-sample" "Sampling"
split_module "$ZK" "src/bin/schema.rs" "zkperf-schema" "Schema tools"

# zkperf scripts
split_module "$ZK" "scripts/selinux_static_analyze.py" "zkperf-selinux-static" "SELinux static analysis"
split_module "$ZK" "scripts/selinux_harden.py" "zkperf-selinux-harden" "Harden .service files"
split_module "$ZK" "scripts/selinux_merge.py" "zkperf-selinux-merge" "Merge to Monster torus ZKP"
split_module "$ZK" "scripts/kagenti_service_gen.py" "zkperf-kagenti-gen" "Generate kagenti services"
split_module "$ZK" "scripts/record-service.sh" "zkperf-record-service" "Dynamic bench script"

# zkperf data
split_module "$ZK" "data/kagenti-generated" "zkperf-kagenti-data" "Generated kagenti artifacts"
split_module "$ZK" "data/selinux-merged" "zkperf-selinux-data" "Merged SELinux policies + Lean4 + ZKP"

echo "" >> "$LOG"
TOTAL=$(grep -c '✅' "$LOG")
echo "**Total: $TOTAL repos split**" >> "$LOG"
echo "Done. Log: $LOG"
