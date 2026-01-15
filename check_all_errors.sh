#!/bin/bash
# Check all code and document errors

set -e

OUTPUT_DIR="check_errors"
mkdir -p "$OUTPUT_DIR"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

echo "🔍 Running cargo check on all code..."
echo "Results will be saved to $OUTPUT_DIR/"

# Full workspace check
echo "Checking workspace..."
cargo check --all-targets --all-features 2>&1 | tee "$OUTPUT_DIR/full_check_${TIMESTAMP}.log"

# Get all binary names
BINARIES=$(grep 'name = ' Cargo.toml | grep -A1 '\[\[bin\]\]' | grep 'name = ' | cut -d'"' -f2)

SUCCESS=0
FAILED=0

echo ""
echo "🔨 Checking individual binaries..."

for bin in $BINARIES; do
    echo -n "Checking $bin... "
    if cargo check --bin "$bin" 2>"$OUTPUT_DIR/${bin}_error.log" >/dev/null; then
        echo "✅"
        rm "$OUTPUT_DIR/${bin}_error.log"
        SUCCESS=$((SUCCESS + 1))
    else
        echo "❌"
        FAILED=$((FAILED + 1))
    fi
done

echo ""
echo "📊 Results:"
echo "  Success: $SUCCESS"
echo "  Failed:  $FAILED"
echo ""
echo "Error logs in: $OUTPUT_DIR/"

# Generate summary
{
    echo "# Cargo Check Error Summary"
    echo ""
    echo "Timestamp: $TIMESTAMP"
    echo "Total: $((SUCCESS + FAILED))"
    echo "Success: $SUCCESS"
    echo "Failed: $FAILED"
    echo ""
    echo "## Failed Binaries"
    for err in "$OUTPUT_DIR"/*_error.log; do
        if [ -f "$err" ]; then
            bin=$(basename "$err" _error.log)
            echo "- $bin"
        fi
    done
    echo ""
    echo "## Error Categories"
    echo ""
    echo "### Missing Dependencies"
    grep -l "use of undeclared\|unresolved import" "$OUTPUT_DIR"/*_error.log 2>/dev/null | wc -l || echo 0
    echo ""
    echo "### Type Errors"
    grep -l "type annotations needed\|mismatched types" "$OUTPUT_DIR"/*_error.log 2>/dev/null | wc -l || echo 0
    echo ""
    echo "### Other Errors"
    echo "$FAILED"
} > "$OUTPUT_DIR/SUMMARY_${TIMESTAMP}.md"

cat "$OUTPUT_DIR/SUMMARY_${TIMESTAMP}.md"
