#!/bin/bash
# Test ZOS Server with different rustc flag combinations

set -e

LATTICE_FILE="rustc_flag_lattice.json"
MAX_TESTS=50
RESULTS_DIR="lattice_test_results"

mkdir -p "$RESULTS_DIR"

echo "🧪 Testing ZOS Server with rustc flag lattice"
echo "=============================================="

if [ ! -f "$LATTICE_FILE" ]; then
    echo "❌ Lattice file not found. Run generate_flag_lattice.py first"
    exit 1
fi

# Extract test cases
python3 -c "
import json
with open('$LATTICE_FILE') as f:
    data = json.load(f)
    for i, test in enumerate(data[:$MAX_TESTS]):
        print(f\"{test['coordinate']}|{test['rustflags']}|{test['features']}|{test['profile']}\")
" | while IFS='|' read -r coord rustflags features profile; do

    echo "🔬 Testing $coord: $features with $profile profile"

    # Set environment
    export RUSTFLAGS="$rustflags"

    # Test build
    result_file="$RESULTS_DIR/${coord}_${features}_${profile}.log"

    if timeout 120 cargo build --features "$features" --profile "$profile" > "$result_file" 2>&1; then
        echo "✅ $coord: SUCCESS"
        echo "SUCCESS" > "$RESULTS_DIR/${coord}_${features}_${profile}.result"
    else
        echo "❌ $coord: FAILED"
        echo "FAILED" > "$RESULTS_DIR/${coord}_${features}_${profile}.result"
    fi

    # Clean for next test
    cargo clean > /dev/null 2>&1 || true
    unset RUSTFLAGS
done

echo ""
echo "📊 Test Results Summary:"
echo "======================="

success_count=$(find "$RESULTS_DIR" -name "*.result" -exec grep -l "SUCCESS" {} \; | wc -l)
total_count=$(find "$RESULTS_DIR" -name "*.result" | wc -l)

echo "✅ Successful builds: $success_count/$total_count"
echo "📁 Detailed logs in: $RESULTS_DIR/"

# Generate summary report
echo "# Rustc Flag Lattice Test Results" > "$RESULTS_DIR/summary.md"
echo "" >> "$RESULTS_DIR/summary.md"
echo "Generated: $(date -Iseconds)" >> "$RESULTS_DIR/summary.md"
echo "Total tests: $total_count" >> "$RESULTS_DIR/summary.md"
echo "Successful: $success_count" >> "$RESULTS_DIR/summary.md"
echo "" >> "$RESULTS_DIR/summary.md"

echo "## Results by Coordinate" >> "$RESULTS_DIR/summary.md"
for result_file in "$RESULTS_DIR"/*.result; do
    if [ -f "$result_file" ]; then
        basename=$(basename "$result_file" .result)
        result=$(cat "$result_file")
        echo "- $basename: $result" >> "$RESULTS_DIR/summary.md"
    fi
done

echo "📋 Summary report: $RESULTS_DIR/summary.md"
