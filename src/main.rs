// ZOS Main - Automorphic Bootstrap System
use std::env;
use zos_server::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    // Handle CLI commands
    if args.len() > 1 {
        match args[1].as_str() {
            "bootstrap" => {
                return handle_bootstrap_command(&args[2..]).await;
            }
            "soul" => {
                return handle_soul_command(&args[2..]).await;
            }
            "orbit" => {
                return handle_orbit_command(&args[2..]).await;
            }
            _ => {}
        }
    }

    // Initialize Automorphic Bootstrap System
    println!("🌌 Starting ZOS Server - Automorphic Bootstrap System");

    let mut improvement = automorphic_bootstrap::AutomorphicImprovement::new()?;
    println!("✅ {}", improvement.status());

    // Show what we can build
    let available = improvement.available_features();
    println!("🔧 Available features to bootstrap: {:?}", available);

    // Test basic orbit functionality
    let test_data = b"Bootstrap test";
    let core = SystemInstance::core_system()?;
    let result = core.execute_all(test_data)?;
    println!(
        "🧪 Core orbit test: {} -> {} bytes",
        test_data.len(),
        result.len()
    );

    println!("🎉 ZOS Bootstrap System ready!");
    println!("💡 Available commands:");
    println!("  ./zos_server bootstrap status     - Show bootstrap status");
    println!("  ./zos_server bootstrap improve    - Improve system");
    println!("  ./zos_server soul extract         - Extract Rust soul eigenmatrix");
    println!("  ./zos_server soul verify          - Verify 3-phase bootstrap proof");
    println!("  ./zos_server orbit core           - Test core orbits");

    Ok(())
}

async fn handle_bootstrap_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let mut improvement = automorphic_bootstrap::AutomorphicImprovement::new()?;

    match args.get(0).map(|s| s.as_str()) {
        Some("status") => {
            println!("📊 Bootstrap Status:");
            println!("  {}", improvement.status());
            println!("  Capability Level: {}", improvement.capability_level());
            println!("  Tools: {}", improvement.tools_count());

            let available: Vec<String> = improvement
                .available_features()
                .into_iter()
                .filter(|f| improvement.can_enable_feature(f))
                .collect();
            println!("  Ready features: {:?}", available);

            let history = improvement.history();
            if !history.is_empty() {
                println!("  History: {:?}", history);
            }
        }
        Some("improve") => {
            let default_feature = "self-build".to_string();
            let feature = args.get(1).unwrap_or(&default_feature);
            println!("🚀 Improving system with feature: {}", feature);

            match improvement.improve(feature) {
                Ok(()) => println!("✅ Successfully improved system"),
                Err(e) => println!("❌ Improvement failed: {}", e),
            }
        }
        Some("path") => {
            let target_features = vec!["self-build", "networking", "security"];
            let path = improvement.improvement_path(&target_features);

            println!("🛤️ Improvement path to reach full capability:");
            for (i, step) in path.iter().enumerate() {
                println!("  {}. {}", i + 1, step);
            }
        }
        Some("verify") => {
            println!("🔍 Verifying system integrity...");
            match improvement.verify_integrity() {
                Ok(true) => println!("✅ System integrity verified"),
                Ok(false) => println!("❌ System integrity check failed"),
                Err(e) => println!("💥 Verification error: {}", e),
            }
        }
        _ => {
            println!("Bootstrap Commands:");
            println!("  status    - Show current bootstrap status");
            println!("  improve   - Improve system with new feature");
            println!("  path      - Show improvement path");
            println!("  verify    - Verify system integrity");
        }
    }

    Ok(())
}

async fn handle_orbit_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    match args.get(0).map(|s| s.as_str()) {
        Some("core") => {
            println!("🌌 Testing Core Orbit System (Level 11)");
            let core = SystemInstance::core_system()?;
            let test_data = b"Core orbit test";
            let result = core.execute_all(test_data)?;
            println!(
                "✅ Core execution: {} -> {} bytes",
                test_data.len(),
                result.len()
            );
            println!("📊 Signature: {}", core.signature());
        }
        _ => {
            println!("Orbit Commands:");
            println!("  core      - Test core orbit system");
        }
    }

    Ok(())
}
async fn handle_soul_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    match args.get(0).map(|s| s.as_str()) {
        Some("extract") => {
            println!("🔍 Extracting Rust soul eigenmatrix from Cargo.lock...");

            let lock_content = match std::fs::read_to_string("Cargo.lock") {
                Ok(content) => content,
                Err(_) => {
                    println!("❌ Cargo.lock not found. Run 'cargo build' first.");
                    return Ok(());
                }
            };

            let eigenmatrix =
                rust_soul_eigenmatrix::RustSoulEigenmatrix::extract_from_cargo_lock(&lock_content)?;

            println!("✅ Rust soul eigenmatrix extracted:");
            println!("  Soul Eigenvalue: {:.6}", eigenmatrix.soul_eigenvalue());
            println!("  Trace Signature: {}", eigenmatrix.trace_signature());
        }
        Some("verify") => {
            println!("🔍 Verifying 3-phase bootstrap proof...");

            let lock_content = std::fs::read_to_string("Cargo.lock").unwrap_or_default();
            let eigenmatrix =
                rust_soul_eigenmatrix::RustSoulEigenmatrix::extract_from_cargo_lock(&lock_content)?;

            match eigenmatrix.verify_bootstrap_proof() {
                Ok(true) => println!("✅ 3-phase bootstrap proof verified!"),
                Ok(false) => println!("❌ 3-phase bootstrap proof failed."),
                Err(e) => println!("💥 Verification error: {}", e),
            }
        }
        _ => {
            println!("Soul Commands:");
            println!("  extract   - Extract Rust soul eigenmatrix");
            println!("  verify    - Verify 3-phase bootstrap proof");
        }
    }

    Ok(())
}
