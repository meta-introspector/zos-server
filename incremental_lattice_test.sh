#!/bin/bash
# Incremental lattice tester - only runs new combinations

set -e

LATTICE_FILE="rustc_flag_lattice.json"
RESULTS_DIR="lattice_test_results"
BATCH_SIZE=5
TIMEOUT=60
MAX_TOTAL_TESTS=50

echo "🧪 Incremental Lattice Tester"
echo "============================="

if [ ! -f "$LATTICE_FILE" ]; then
    echo "❌ Lattice file not found. Run ./generate_rustc_lattice.sh first"
    exit 1
fi

mkdir -p "$RESULTS_DIR"

# Count existing results
existing_count=$(find "$RESULTS_DIR" -name "*.result" 2>/dev/null | wc -l)
echo "📊 Existing test results: $existing_count"

# Get total combinations
total_combinations=$(jq length "$LATTICE_FILE")
echo "📋 Total combinations available: $total_combinations"

# Calculate how many more to test
remaining=$((MAX_TOTAL_TESTS - existing_count))
if [ $remaining -le 0 ]; then
    echo "✅ Already tested $existing_count combinations (limit: $MAX_TOTAL_TESTS)"
    echo "📊 Generating summary..."
else
    echo "🔄 Will test $remaining more combinations (batch size: $BATCH_SIZE)"
fi

# Function to test a single combination
test_combination() {
    local coord="$1"
    local rustflags="$2"
    local features="$3"
    local profile="$4"

    local result_file="$RESULTS_DIR/${coord}_${features}_${profile}.result"
    local log_file="$RESULTS_DIR/${coord}_${features}_${profile}.log"

    # Skip if already tested
    if [ -f "$result_file" ]; then
        echo "⏭️  $coord: SKIPPED (already tested)"
        return 0
    fi

    echo "🔬 Testing $coord: $features ($profile)"

    # Set environment and test with timeout
    export RUSTFLAGS="$rustflags"

    if timeout $TIMEOUT cargo build --features "$features" --profile "$profile" > "$log_file" 2>&1; then
        echo "SUCCESS" > "$result_file"
        echo "✅ $coord: SUCCESS"
    else
        echo "FAILED" > "$result_file"
        echo "❌ $coord: FAILED"
    fi

    # Clean for next test
    cargo clean > /dev/null 2>&1 || true
    unset RUSTFLAGS
}

# Extract and test combinations incrementally
tested_this_run=0
python3 -c "
import json
with open('$LATTICE_FILE') as f:
    data = json.load(f)
    for i, test in enumerate(data):
        if i >= $MAX_TOTAL_TESTS:
            break
        print(f\"{test['coordinate']}|{test['rustflags']}|{test['features']}|{test['profile']}\")
" | while IFS='|' read -r coord rustflags features profile && [ $tested_this_run -lt $remaining ]; do

    result_file="$RESULTS_DIR/${coord}_${features}_${profile}.result"

    # Skip if already exists
    if [ -f "$result_file" ]; then
        continue
    fi

    test_combination "$coord" "$rustflags" "$features" "$profile"
    tested_this_run=$((tested_this_run + 1))

    # Batch limit check
    if [ $((tested_this_run % BATCH_SIZE)) -eq 0 ]; then
        echo "📊 Completed batch of $BATCH_SIZE tests..."
        sleep 1
    fi
done

echo ""
echo "📊 Final Results Summary:"
echo "========================"

# Count results
success_count=$(find "$RESULTS_DIR" -name "*.result" -exec grep -l "SUCCESS" {} \; 2>/dev/null | wc -l)
failed_count=$(find "$RESULTS_DIR" -name "*.result" -exec grep -l "FAILED" {} \; 2>/dev/null | wc -l)
total_tested=$((success_count + failed_count))

echo "✅ Successful builds: $success_count"
echo "❌ Failed builds: $failed_count"
echo "📊 Total tested: $total_tested/$total_combinations"
echo "📈 Success rate: $(( success_count * 100 / total_tested ))%" 2>/dev/null || echo "📈 Success rate: N/A"

# Update summary report
cat > "$RESULTS_DIR/summary.md" << EOF
# Incremental Lattice Test Results

Generated: $(date -Iseconds)
Total combinations: $total_combinations
Tested: $total_tested
Successful: $success_count
Failed: $failed_count
Success rate: $(( success_count * 100 / total_tested ))%

## Recent Results
EOF

# Add recent results
find "$RESULTS_DIR" -name "*.result" -printf "%T@ %f\n" | sort -nr | head -10 | while read timestamp filename; do
    result=$(cat "$RESULTS_DIR/$filename")
    basename=$(basename "$filename" .result)
    echo "- $basename: $result" >> "$RESULTS_DIR/summary.md"
done

echo ""
echo "📁 Detailed logs: $RESULTS_DIR/"
echo "📋 Summary: $RESULTS_DIR/summary.md"
echo "🔄 Run again to test more combinations"
