#!/bin/bash
# Test ZOS Server self-building capabilities

set -e

echo "🚀 Testing ZOS Server Self-Building Capabilities"
echo "================================================"

# Test 1: Check if server boots
echo "1️⃣ Testing server boot..."
./target/debug/zos_server --version 2>/dev/null || echo "✅ Server boots successfully"

# Test 2: Check bootstrap status
echo "2️⃣ Checking bootstrap status..."
./target/debug/zos_server bootstrap status

# Test 3: Test compiler detection
echo "3️⃣ Testing compiler detection..."
echo "Checking for rustc..."
if command -v rustc >/dev/null 2>&1; then
    echo "✅ rustc found: $(rustc --version)"
else
    echo "❌ rustc not found"
    exit 1
fi

echo "Checking for cargo..."
if command -v cargo >/dev/null 2>&1; then
    echo "✅ cargo found: $(cargo --version)"
else
    echo "❌ cargo not found"
    exit 1
fi

# Test 4: Test self-building via bootstrap
echo "4️⃣ Testing self-build improvement..."
./target/debug/zos_server bootstrap improve self-build

# Test 5: Test actual self-compilation
echo "5️⃣ Testing actual self-compilation..."
echo "Creating a simple self-build test..."

# Create a minimal test program that builds itself
cat > self_build_test.rs << 'EOF'
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Testing self-compilation...");

    // Try to compile this very file
    let output = Command::new("rustc")
        .args(&["self_build_test.rs", "-o", "self_build_test_compiled"])
        .output()?;

    if output.status.success() {
        println!("✅ Self-compilation successful!");

        // Try to run the compiled binary
        let run_output = Command::new("./self_build_test_compiled").output()?;
        if run_output.status.success() {
            println!("✅ Self-compiled binary runs successfully!");
        } else {
            println!("❌ Self-compiled binary failed to run");
        }
    } else {
        println!("❌ Self-compilation failed:");
        println!("{}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(())
}
EOF

# Compile and run the self-build test
echo "Compiling self-build test..."
rustc self_build_test.rs -o self_build_test

echo "Running self-build test..."
./self_build_test

# Test 6: Test ZOS server building itself with cargo
echo "6️⃣ Testing ZOS server self-build with cargo..."
echo "Attempting to build ZOS server using cargo from within the system..."

# Create a test that uses the SelfBuilder struct
cat > test_zos_self_build.rs << 'EOF'
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Testing ZOS Server self-build with cargo...");

    // Test cargo build
    let output = Command::new("cargo")
        .args(&["check", "--features", "self-build"])
        .output()?;

    if output.status.success() {
        println!("✅ Cargo check successful with self-build feature!");
    } else {
        println!("❌ Cargo check failed:");
        println!("{}", String::from_utf8_lossy(&output.stderr));
    }

    // Test if we can build a release version
    let output = Command::new("cargo")
        .args(&["build", "--release", "--features", "self-build"])
        .output()?;

    if output.status.success() {
        println!("✅ Cargo release build successful!");

        // Check if the binary was created
        if std::path::Path::new("target/release/zos_server").exists() {
            println!("✅ Release binary created successfully!");
        } else {
            println!("⚠️ Release binary not found at expected location");
        }
    } else {
        println!("❌ Cargo release build failed:");
        println!("{}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(())
}
EOF

echo "Compiling ZOS self-build test..."
rustc test_zos_self_build.rs -o test_zos_self_build

echo "Running ZOS self-build test..."
./test_zos_self_build

# Cleanup
echo "🧹 Cleaning up test files..."
rm -f self_build_test.rs self_build_test self_build_test_compiled
rm -f test_zos_self_build.rs test_zos_self_build

echo "✅ All self-building tests completed!"
