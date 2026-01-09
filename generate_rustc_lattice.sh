#!/bin/bash
# Generate comprehensive Rust compiler flag lattice for testing

set -e

echo "🦀 Rust Compiler Flag Lattice Generator"
echo "======================================="

# Get rustc version and basic info
echo "📋 Rust Version:"
rustc --version
echo ""

# Extract all rustc flags
echo "🔍 Extracting all rustc flags..."
rustc --help > rustc_help.txt
rustc -C help > rustc_codegen_help.txt
rustc -Z help 2> rustc_unstable_help.txt || echo "Unstable flags require nightly"

# Parse codegen flags
echo "⚙️  Codegen Flags (-C):"
grep -E "^\s*[a-zA-Z0-9_-]+\s*=" rustc_codegen_help.txt | head -20

# Parse optimization levels
echo ""
echo "🚀 Optimization Levels:"
echo "  -C opt-level=0,1,2,3,s,z"

# Parse target features
echo ""
echo "🎯 Target Features:"
rustc --print target-features | head -10

# Parse target list
echo ""
echo "🏗️  Available Targets:"
rustc --print target-list | head -10

echo ""
echo "📊 Generating flag lattice combinations..."

# Create lattice generator
cat > generate_flag_lattice.py << 'EOF'
#!/usr/bin/env python3
import itertools
import json
from typing import List, Dict, Any

class RustcFlagLattice:
    def __init__(self):
        # Core optimization flags
        self.opt_levels = ['0', '1', '2', '3', 's', 'z']

        # Debug info levels
        self.debug_levels = ['0', '1', '2']

        # Common codegen flags
        self.codegen_flags = {
            'panic': ['unwind', 'abort'],
            'lto': ['off', 'thin', 'fat'],
            'target-cpu': ['native', 'generic'],
            'overflow-checks': ['on', 'off'],
            'debug-assertions': ['on', 'off'],
        }

        # Link-time flags
        self.link_flags = {
            'prefer-dynamic': [True, False],
            'static-crt': [True, False],
        }

        # Feature flags for ZOS Server
        self.zos_features = [
            'default',
            'all-plugins',
            'self-build',
            'core-only',
            'extra-plugins',
            'notebooklm',
            'dynamic-loading'
        ]

    def generate_lattice_coordinates(self, max_combinations=50):
        """Generate lattice coordinates for flag combinations"""
        combinations = []

        # Generate systematic combinations
        for i, opt in enumerate(self.opt_levels[:3]):  # Limit to first 3 opt levels
            for j, debug in enumerate(self.debug_levels):
                for k, panic in enumerate(self.codegen_flags['panic']):
                    for l, lto in enumerate(self.codegen_flags['lto']):
                        # Create lattice coordinate
                        coord = f"L{i}.{j}.{k}.{l}"

                        # Build flag combination
                        flags = {
                            'opt_level': opt,
                            'debug_info': debug,
                            'panic': panic,
                            'lto': lto,
                            'coordinate': coord
                        }

                        combinations.append(flags)

                        if len(combinations) >= max_combinations:
                            return combinations

        return combinations

    def generate_rustflags(self, combo: Dict[str, Any]) -> str:
        """Convert combination to RUSTFLAGS string"""
        flags = []

        flags.append(f"-C opt-level={combo['opt_level']}")
        flags.append(f"-C debuginfo={combo['debug_info']}")
        flags.append(f"-C panic={combo['panic']}")
        flags.append(f"-C lto={combo['lto']}")

        return " ".join(flags)

    def generate_test_matrix(self):
        """Generate complete test matrix"""
        lattice = self.generate_lattice_coordinates()

        matrix = []
        for combo in lattice:
            for feature in self.zos_features[:5]:  # Limit features
                test_case = {
                    'coordinate': combo['coordinate'],
                    'rustflags': self.generate_rustflags(combo),
                    'features': feature,
                    'profile': 'dev' if combo['opt_level'] in ['0', '1'] else 'release'
                }
                matrix.append(test_case)

        return matrix

# Generate the lattice
generator = RustcFlagLattice()
test_matrix = generator.generate_test_matrix()

print(f"Generated {len(test_matrix)} test combinations")
print("\nSample combinations:")
for i, test in enumerate(test_matrix[:10]):
    print(f"{test['coordinate']}: {test['rustflags']} --features {test['features']}")

# Save to JSON
with open('rustc_flag_lattice.json', 'w') as f:
    json.dump(test_matrix, f, indent=2)

print(f"\nFull lattice saved to rustc_flag_lattice.json")
EOF

python3 generate_flag_lattice.py

echo ""
echo "🧪 Creating test runner for flag lattice..."

cat > test_flag_lattice.sh << 'EOF'
#!/bin/bash
# Test ZOS Server with different rustc flag combinations

set -e

LATTICE_FILE="rustc_flag_lattice.json"
MAX_TESTS=10
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
EOF

chmod +x test_flag_lattice.sh

echo ""
echo "🎯 Generated files:"
echo "  - rustc_help.txt: Basic rustc help"
echo "  - rustc_codegen_help.txt: Codegen flags"
echo "  - generate_flag_lattice.py: Lattice generator"
echo "  - rustc_flag_lattice.json: Flag combinations"
echo "  - test_flag_lattice.sh: Test runner"
echo ""
echo "🚀 Usage:"
echo "  ./test_flag_lattice.sh    # Run lattice tests"
echo "  cat rustc_flag_lattice.json | jq '.[:5]'  # View first 5 combinations"
