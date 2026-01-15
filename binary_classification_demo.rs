// Demo: Binary Classification and Verification
use zos_server::binary_classifier::{BinaryClassifier, VerificationSystem};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 ZOS Binary Classification Demo");

    let classifier = BinaryClassifier::new();
    let verifier = VerificationSystem::new();

    // Simulate before/after .so files
    let original_so = "target/debug/deps/libzos_server_unsafe.so";
    let stripped_so = "target/debug/deps/libzos_server_safe.so";

    println!("\n--- Analyzing Original Binary ---");
    match classifier.classify_binary(original_so) {
        Ok(result) => {
            println!("📊 Classification Results:");
            println!("  File: {}", result.file_path);
            println!("  Is Safe: {}", result.is_safe);
            println!("  Risk Score: {:.2}", result.risk_score);
            println!("  Syscall Count: {}", result.syscall_count);
            println!("  Dangerous Symbols: {}", result.dangerous_symbols.len());

            if !result.dangerous_symbols.is_empty() {
                println!("  🚨 Dangerous symbols found:");
                for symbol in &result.dangerous_symbols {
                    println!("    - {}", symbol);
                }
            }
        }
        Err(e) => println!("❌ Analysis failed: {}", e),
    }

    println!("\n--- Analyzing Stripped Binary ---");
    match classifier.classify_binary(stripped_so) {
        Ok(result) => {
            println!("📊 Classification Results:");
            println!("  File: {}", result.file_path);
            println!("  Is Safe: {}", result.is_safe);
            println!("  Risk Score: {:.2}", result.risk_score);
            println!("  Syscall Count: {}", result.syscall_count);
            println!("  Dangerous Symbols: {}", result.dangerous_symbols.len());
            println!("  Stripped Verification: {}", result.stripped_verification);
        }
        Err(e) => println!("❌ Analysis failed: {}", e),
    }

    println!("\n--- Binary Comparison ---");
    match classifier.compare_binaries(original_so, stripped_so) {
        Ok(comparison) => {
            println!("🔄 Comparison Results:");
            println!("  Syscalls Removed: {}", comparison.syscalls_removed);
            println!(
                "  Risk Reduction: {:.2}%",
                comparison.risk_reduction * 100.0
            );
            println!("  Symbols Stripped: {}", comparison.symbols_stripped);
            println!("  Stripping Success: {}", comparison.stripping_successful);

            // Generate detailed report
            let report = verifier.generate_report(&comparison);
            println!("\n📋 Detailed Report:");
            println!("{}", report);
        }
        Err(e) => println!("❌ Comparison failed: {}", e),
    }

    println!("\n--- Verification ---");
    match verifier.verify_stripping(original_so, stripped_so) {
        Ok(verified) => {
            if verified {
                println!("✅ VERIFICATION PASSED: Syscall stripping successful");
                println!("🔒 Binary is provably safe for LLM execution");
            } else {
                println!("❌ VERIFICATION FAILED: Syscalls may still be present");
                println!("⚠️  Binary is NOT safe for LLM execution");
            }
        }
        Err(e) => println!("❌ Verification error: {}", e),
    }

    println!("\n--- Pattern Analysis Demo ---");
    demonstrate_pattern_detection();

    Ok(())
}

fn demonstrate_pattern_detection() {
    let classifier = BinaryClassifier::new();

    // Simulate binary with syscalls
    let unsafe_binary = vec![
        0x48, 0x89, 0xe5, // mov rbp, rsp (safe)
        0x0f, 0x05, // syscall (DANGEROUS!)
        0x48, 0xc7, 0xc0, 0x3b, // mov rax, 59 (execve - CRITICAL!)
        0xff, 0xd0, // call rax (suspicious)
        0xc3, // ret (safe)
    ];

    println!("🧬 Pattern Analysis:");
    println!("  Binary size: {} bytes", unsafe_binary.len());

    // This would normally be done internally, but we'll demo it
    let patterns = classifier.analyze_patterns(&unsafe_binary);
    println!("  Syscall patterns: {}", patterns.syscall_patterns);
    println!("  Dangerous patterns: {}", patterns.dangerous_patterns);
    println!("  Safe patterns: {}", patterns.safe_patterns);

    // Simulate safe binary (after stripping)
    let safe_binary = vec![
        0x48, 0x89, 0xe5, // mov rbp, rsp (safe)
        0x90, 0x90, // nop nop (syscalls replaced with nops)
        0x90, 0x90, 0x90, 0x90, // nops (execve removed)
        0x90, 0x90, // nops (call removed)
        0xc3, // ret (safe)
    ];

    let safe_patterns = classifier.analyze_patterns(&safe_binary);
    println!("\n🛡️  After Stripping:");
    println!("  Syscall patterns: {}", safe_patterns.syscall_patterns);
    println!("  Dangerous patterns: {}", safe_patterns.dangerous_patterns);
    println!("  Safe patterns: {}", safe_patterns.safe_patterns);

    println!("\n📈 Improvement:");
    println!(
        "  Syscalls removed: {}",
        patterns
            .syscall_patterns
            .saturating_sub(safe_patterns.syscall_patterns)
    );
    println!(
        "  Dangerous patterns removed: {}",
        patterns
            .dangerous_patterns
            .saturating_sub(safe_patterns.dangerous_patterns)
    );
}
