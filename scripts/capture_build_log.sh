#!/bin/bash
# Canonical Build Log Capture System
# Captures all build attempts as structured data

set -e

REPO_NAME=$(basename $(pwd))
SESSION_ID=$(date +%Y%m%d_%H%M%S)
LOG_DIR="data/build_logs/$SESSION_ID"

mkdir -p "$LOG_DIR"

echo "🔨 Canonical Build Capture: $REPO_NAME"
echo "📁 Session: $SESSION_ID"
echo ""

# Capture full build log
echo "Building all binaries..."
nix develop -c cargo build --bins 2>&1 | tee "$LOG_DIR/full_build.log"

# Extract error summary
echo ""
echo "📊 Analyzing errors..."
grep "^error\[" "$LOG_DIR/full_build.log" | sort | uniq -c | sort -rn > "$LOG_DIR/error_summary.txt"

# Extract warnings
grep "^warning:" "$LOG_DIR/full_build.log" | sort | uniq -c | sort -rn > "$LOG_DIR/warning_summary.txt"

# Count successes
grep "Finished" "$LOG_DIR/full_build.log" | wc -l > "$LOG_DIR/success_count.txt"

# Generate metadata
cat > "$LOG_DIR/metadata.json" <<EOF
{
  "repo": "$REPO_NAME",
  "session_id": "$SESSION_ID",
  "timestamp": "$(date -Iseconds)",
  "total_binaries": $(grep '^\[\[bin\]\]' Cargo.toml | wc -l),
  "errors": $(wc -l < "$LOG_DIR/error_summary.txt"),
  "warnings": $(wc -l < "$LOG_DIR/warning_summary.txt"),
  "successes": $(cat "$LOG_DIR/success_count.txt")
}
EOF

echo ""
echo "✅ Build log captured: $LOG_DIR/"
echo ""
cat "$LOG_DIR/metadata.json"
echo ""
echo "📊 Top 5 Errors:"
head -5 "$LOG_DIR/error_summary.txt"
echo ""
echo "📝 First occurrence of each top error:"
for error_code in $(head -3 "$LOG_DIR/error_summary.txt" | grep -oP 'error\[E\d+\]'); do
    echo ""
    echo "=== $error_code ==="
    grep -F "$error_code" "$LOG_DIR/full_build.log" -m 1 -A 4
done
