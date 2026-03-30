#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/zos-sync-smoke.XXXXXX")"
LOG1="$TMP_DIR/peer1.log"
LOG2="$TMP_DIR/peer2.log"
OVERLAY1="$TMP_DIR/peer1-overlay.json"
OVERLAY2="$TMP_DIR/peer2-overlay.json"
PAYLOAD="$TMP_DIR/smoke-artifact.bin"

cleanup() {
  if [[ -n "${PID1:-}" ]]; then kill "$PID1" >/dev/null 2>&1 || true; fi
  if [[ -n "${PID2:-}" ]]; then kill "$PID2" >/dev/null 2>&1 || true; fi
}
trap cleanup EXIT

printf 'smoke payload %s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" > "$PAYLOAD"
DIGEST="sha256:$(sha256sum "$PAYLOAD" | cut -d' ' -f1)"

cat > "$OVERLAY1" <<EOF
{
  "contractVersion": "artifact-ack/v1",
  "artifactId": "artifact://smoke-demo",
  "artifactRevision": "rev-smoke-1",
  "acknowledgedRevision": "rev-smoke-1",
  "publishedAt": "2026-03-30T00:00:00Z",
  "publishStatus": "verified",
  "memberCount": 1,
  "containerObjectRef": {
    "sink": "file",
    "uri": "file://$PAYLOAD",
    "contentDigest": "$DIGEST",
    "sizeBytes": $(wc -c < "$PAYLOAD")
  }
}
EOF

printf '{}\n' > "$OVERLAY2"

cargo build -p zos-server --bin zos_server >/dev/null

ZOS_SYNC_LISTEN_ADDR=/ip4/127.0.0.1/tcp/40101 \
ZOS_SYNC_INVENTORY_FILE="$OVERLAY1" \
./target/debug/zos_server serve 18080 >"$LOG1" 2>&1 &
PID1=$!

for _ in $(seq 1 20); do
  if grep -q "listening_on=/ip4/127.0.0.1/tcp/40101" "$LOG1"; then
    break
  fi
  sleep 1
done

ZOS_SYNC_LISTEN_ADDR=/ip4/127.0.0.1/tcp/40102 \
ZOS_SYNC_BOOTSTRAP_ADDRS=/ip4/127.0.0.1/tcp/40101 \
ZOS_SYNC_INVENTORY_FILE="$OVERLAY2" \
./target/debug/zos_server serve 18081 >"$LOG2" 2>&1 &
PID2=$!

sleep 25

if ! grep -q "connected_to=" "$LOG2"; then
  echo "smoke failed: peer2 did not log a libp2p connection"
  echo "peer1 log: $LOG1"
  echo "peer2 log: $LOG2"
  exit 1
fi

if ! grep -q "replay recovered artifact artifact://smoke-demo" "$LOG2"; then
  echo "smoke failed: peer2 did not recover the artifact"
  echo "peer1 log: $LOG1"
  echo "peer2 log: $LOG2"
  exit 1
fi

echo "smoke ok"
echo "tmp_dir: $TMP_DIR"
echo "peer1_log: $LOG1"
echo "peer2_log: $LOG2"
