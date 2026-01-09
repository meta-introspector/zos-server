use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("ZOS Server - Zero Ontology System");
        println!("Usage: {} <command> [args...]", args[0]);
        println!();
        println!("Commands:");
        #[cfg(feature = "tokio")]
        {
            println!("  llm-improve <prompt>     - Use LLM-compiler feedback loop");
            println!("  task-start <mode> <args> - Start task in specified mode");
        }
        #[cfg(all(feature = "axum", feature = "tokio"))]
        println!("  api-server               - Start secure API server");
        #[cfg(all(feature = "libp2p", feature = "tokio"))]
        println!("  p2p-node                 - Start P2P node");

        if !cfg!(feature = "tokio") {
            println!();
            println!("Note: This is a slim build. For full functionality, rebuild with:");
            println!("  cargo build --features all-plugins");
        }
        return Ok(());
    }

    match args[1].as_str() {
        #[cfg(feature = "tokio")]
        "llm-improve" => {
            if args.len() < 3 {
                eprintln!("Usage: {} llm-improve <prompt>", args[0]);
                return Ok(());
            }

            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(async {
                use zos_server::llm_compiler_service::LLMCompilerService;
                use zos_server::task_registry::TaskRegistry;

                let mut service = LLMCompilerService::new();
                let mut registry = TaskRegistry::new();

                match service.improve_code(&args[2], &mut registry).await {
                    Ok(result) => println!("Improvement result: {}", result),
                    Err(e) => eprintln!("Error: {}", e),
                }
            });
        }

        #[cfg(feature = "tokio")]
        "task-start" => {
            if args.len() < 3 {
                eprintln!("Usage: {} task-start <mode> [args...]", args[0]);
                eprintln!("Modes: interactive, callback, github, batch, stream, webhook");
                return Ok(());
            }

            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(async {
                use zos_server::task_modes::{TaskMode, TaskModeExecutor};

                let mode = match args[2].as_str() {
                    "interactive" => TaskMode::Interactive,
                    "callback" => TaskMode::Callback,
                    "github" => TaskMode::GitHub,
                    "batch" => TaskMode::Batch,
                    "stream" => TaskMode::Stream,
                    "webhook" => TaskMode::WebHook,
                    _ => {
                        eprintln!("Invalid mode. Available: interactive, callback, github, batch, stream, webhook");
                        return;
                    }
                };

                let executor = TaskModeExecutor::new();
                let task_args = if args.len() > 3 { &args[3..] } else { &[] };

                match executor.execute_task(mode, task_args).await {
                    Ok(_) => println!("Task completed successfully"),
                    Err(e) => eprintln!("Task failed: {}", e),
                }
            });
        }

        #[cfg(all(feature = "axum", feature = "tokio"))]
        "api-server" => {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(async {
                use zos_server::secure_api_server;

                match secure_api_server::start_server().await {
                    Ok(_) => println!("API server started"),
                    Err(e) => eprintln!("Failed to start API server: {}", e),
                }
            });
        }

        #[cfg(all(feature = "libp2p", feature = "tokio"))]
        "p2p-node" => {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(async {
                use zos_server::secure_libp2p_api;

                match secure_libp2p_api::start_node().await {
                    Ok(_) => println!("P2P node started"),
                    Err(e) => eprintln!("Failed to start P2P node: {}", e),
                }
            });
        }

        _ => {
            eprintln!("Unknown command: {}", args[1]);
            eprintln!("Run without arguments to see available commands");

            #[cfg(not(feature = "tokio"))]
            {
                eprintln!();
                eprintln!("This is a slim build with limited functionality.");
                eprintln!("For full features, rebuild with: cargo build --features all-plugins");
            }
        }
    }

    Ok(())
}
