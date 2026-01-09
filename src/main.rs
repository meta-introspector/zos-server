// ZOS Main - Automorphic Bootstrap System
use std::env;
use zos_server::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    // Handle CLI commands
    if args.len() > 1 {
        match args[1].as_str() {
            "task-start" => {
                use zos_server::task_modes::{MultiModalTaskRunner, TaskConfig, TaskMode};

                if args.len() < 4 {
                    eprintln!("Usage: zos_server task-start <mode> <prompt> [options]");
                    eprintln!("Modes: interactive, callback, github, batch, stream, webhook");
                    std::process::exit(1);
                }

                let mode = match args[2].as_str() {
                    "interactive" => TaskMode::Interactive,
                    "callback" => TaskMode::Callback,
                    "github" => TaskMode::GitHub,
                    "batch" => TaskMode::Batch,
                    "stream" => TaskMode::Stream,
                    "webhook" => TaskMode::WebHook,
                    _ => {
                        eprintln!("Invalid mode. Use: interactive, callback, github, batch, stream, webhook");
                        std::process::exit(1);
                    }
                };

                let prompt = &args[3];
                let config = TaskConfig {
                    auto_continue: mode == TaskMode::Batch,
                    max_iterations: 5,
                    timeout_seconds: 30,
                    callback_url: None,
                    github_repo: args.get(4).cloned(),
                    github_issue: args.get(5).and_then(|s| s.parse().ok()),
                    webhook_url: None,
                    interactive_prompts: mode == TaskMode::Interactive,
                };

                let mut runner = MultiModalTaskRunner::new();
                let task_id = runner.start_task(prompt, mode, config).await;
                println!("🚀 Task started with ID: {}", task_id);
            }
            "task-step" => {
                use zos_server::task_modes::MultiModalTaskRunner;

                if args.len() < 3 {
                    eprintln!("Usage: zos_server task-step <task_id> [input]");
                    std::process::exit(1);
                }

                let task_id = uuid::Uuid::parse_str(&args[2])?;
                let input = args.get(3).cloned();

                let mut runner = MultiModalTaskRunner::new();
                match runner.execute_step(task_id, input).await {
                    Ok(result) => {
                        println!("✅ Step completed: {:?}", result.next_action);
                        if result.compile_success {
                            println!("Code:\n{}", result.code);
                        } else {
                            println!("Errors: {:?}", result.errors);
                        }
                    }
                    Err(e) => {
                        eprintln!("❌ Step failed: {}", e);
                        std::process::exit(1);
                    }
                }
            }
            "llm-improve" => {
                use zos_server::llm_compiler_service::LLMCompilerService;

                let prompt = args
                    .get(2)
                    .ok_or("Usage: zos_server llm-improve <prompt>")?;

                println!("🤖 Starting LLM-Compiler improvement loop...");
                let mut service = LLMCompilerService::new();

                match service
                    .improve_code(prompt, "user", vec!["ai-generated".to_string()])
                    .await
                {
                    Ok(task_id) => {
                        println!("🎉 Task completed! ID: {}", task_id);
                        if let Some(task) = service.get_task(task_id) {
                            println!("Final code:\n{}", task.final_code);
                        }
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to improve code: {}", e);
                        std::process::exit(1);
                    }
                }
            }
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
