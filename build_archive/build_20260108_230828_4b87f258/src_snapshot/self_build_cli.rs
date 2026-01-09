// Self-Build CLI Commands
use crate::self_builder::SelfBuilder;
use std::env;

pub async fn handle_self_build_command(args: &[String]) -> Result<(), String> {
    let current_dir = env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {}", e))?
        .to_string_lossy()
        .to_string();

    let mut builder = SelfBuilder::new(current_dir);

    match args.get(0).map(|s| s.as_str()) {
        Some("build") => {
            println!("🚀 Starting self-build with LLM error fixing...");
            let success = builder.self_build().await?;
            if success {
                println!("🎉 Self-build completed successfully!");
            } else {
                println!("😞 Self-build failed after maximum iterations");
            }
        },
        Some("config") => {
            if let Some(endpoint) = args.get(1) {
                builder.set_llm_endpoint(endpoint.clone());
                println!("🔧 Set LLM endpoint to: {}", endpoint);
            }
            if let Some(max_iter) = args.get(2) {
                if let Ok(max) = max_iter.parse::<u32>() {
                    builder.set_max_iterations(max);
                    println!("🔧 Set max iterations to: {}", max);
                }
            }
        },
        Some("test") => {
            println!("🧪 Testing build without fixes...");
            let output = std::process::Command::new("cargo")
                .args(&["build", "--release"])
                .output()
                .map_err(|e| format!("Failed to run test build: {}", e))?;

            if output.status.success() {
                println!("✅ Build already successful!");
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let error_count = stderr.lines()
                    .filter(|line| line.contains("error[E"))
                    .count();
                println!("❌ Found {} compile errors", error_count);
                println!("Run 'self-build build' to fix them automatically");
            }
        },
        _ => {
            println!("Self-Build Commands:");
            println!("  build                     - Start self-build with LLM fixes");
            println!("  config <endpoint> [max]   - Configure LLM endpoint and max iterations");
            println!("  test                      - Test build without making changes");
            println!();
            println!("Examples:");
            println!("  ./zos-server self-build build");
            println!("  ./zos-server self-build config http://localhost:11434/api/generate 5");
            println!("  ./zos-server self-build test");
        }
    }

    Ok(())
}
