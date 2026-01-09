#!/bin/bash
# Test ZOS Server Compiler Integration

set -e

echo "🚀 Testing ZOS Server Compiler Integration"
echo "=========================================="

# Test 1: Build with compiler integration
echo "1️⃣ Building with compiler integration..."
cargo build --features self-build

# Test 2: Test server boot with compiler features
echo "2️⃣ Testing server boot..."
./target/debug/zos_server bootstrap status

# Test 3: Test self-building capability
echo "3️⃣ Testing self-building..."
./target/debug/zos_server bootstrap improve self-build

# Test 4: Create and run a simple compiler test
echo "4️⃣ Testing compiler integration..."
cat > test_compiler_integration.rs << 'EOF'
use zos_server::compiler_integration::{SelfBuildingSystem, CompilerFactory};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Testing ZOS Compiler Integration");

    // Test 1: Create compiler system
    let system = SelfBuildingSystem::new();
    println!("✅ Compiler system created");

    // Test 2: Get compiler info
    match system.get_compiler_info() {
        Ok(info) => {
            println!("✅ Compiler info: {}", info.version);
            println!("📋 Available targets: {}", info.targets.len());
        }
        Err(e) => {
            println!("❌ Failed to get compiler info: {}", e);
            return Err(e.into());
        }
    }

    // Test 3: Test different compiler implementations
    println!("🔧 Testing command compiler...");
    let cmd_compiler = CompilerFactory::create_command();
    let system_cmd = SelfBuildingSystem::with_compiler(cmd_compiler);

    match system_cmd.get_compiler_info() {
        Ok(info) => println!("✅ Command compiler: {}", info.version),
        Err(e) => println!("⚠️ Command compiler failed: {}", e),
    }

    // Test 4: Test embedded compiler
    println!("🔧 Testing embedded compiler...");
    let embedded_compiler = CompilerFactory::create_embedded();
    let system_embedded = SelfBuildingSystem::with_compiler(embedded_compiler);

    match system_embedded.get_compiler_info() {
        Ok(info) => println!("✅ Embedded compiler: {}", info.version),
        Err(e) => println!("⚠️ Embedded compiler: {}", e),
    }

    // Test 5: Create a simple source file and test compilation
    println!("🔧 Testing source compilation...");
    std::fs::write("hello.rs", r#"
fn main() {
    println!("Hello from ZOS compiled code!");
}
"#)?;

    match system.compile_source("hello.rs", "hello_compiled") {
        Ok(()) => {
            println!("✅ Source compilation successful");

            // Try to run the compiled binary
            match std::process::Command::new("./hello_compiled").output() {
                Ok(output) => {
                    if output.status.success() {
                        println!("✅ Compiled binary runs: {}",
                                String::from_utf8_lossy(&output.stdout).trim());
                    } else {
                        println!("❌ Compiled binary failed to run");
                    }
                }
                Err(e) => println!("❌ Failed to run compiled binary: {}", e),
            }
        }
        Err(e) => println!("❌ Source compilation failed: {}", e),
    }

    // Test 6: Test syntax checking
    println!("🔧 Testing syntax checking...");

    // Valid syntax
    std::fs::write("valid.rs", "fn main() {}")?;
    match system.check_syntax("valid.rs") {
        Ok(true) => println!("✅ Valid syntax detected correctly"),
        Ok(false) => println!("❌ Valid syntax reported as invalid"),
        Err(e) => println!("❌ Syntax check error: {}", e),
    }

    // Invalid syntax
    std::fs::write("invalid.rs", "fn main( { invalid")?;
    match system.check_syntax("invalid.rs") {
        Ok(false) => println!("✅ Invalid syntax detected correctly"),
        Ok(true) => println!("❌ Invalid syntax reported as valid"),
        Err(e) => println!("⚠️ Syntax check error (expected): {}", e),
    }

    // Cleanup
    std::fs::remove_file("hello.rs").ok();
    std::fs::remove_file("hello_compiled").ok();
    std::fs::remove_file("valid.rs").ok();
    std::fs::remove_file("invalid.rs").ok();

    println!("✅ All compiler integration tests completed!");
    Ok(())
}
EOF

echo "Compiling compiler integration test..."
rustc --extern zos_server=target/debug/libzos_server.rlib test_compiler_integration.rs -L target/debug/deps -o test_compiler_integration

echo "Running compiler integration test..."
./test_compiler_integration

# Test 5: Test with dynamic loading feature (if available)
echo "5️⃣ Testing with dynamic loading feature..."
if cargo build --features "self-build,dynamic-loading" 2>/dev/null; then
    echo "✅ Dynamic loading feature builds successfully"
else
    echo "⚠️ Dynamic loading feature not available or failed to build"
fi

# Cleanup
rm -f test_compiler_integration.rs test_compiler_integration

echo "✅ All compiler integration tests completed!"
