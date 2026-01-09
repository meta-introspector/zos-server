// ZOS Server Main - Macro-Generated System
use zos_server::*;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    // Handle CLI commands
    if args.len() > 1 {
        match args[1].as_str() {
            "self-build" => {
                return self_build_cli::handle_self_build_command(&args[2..]).await
                    .map_err(|e| e.into());
            }
            "notebooklm" => {
                return notebooklm_cli::handle_notebooklm_command(&args[2..])
                    .map_err(|e| e.into());
            }
            _ => {}
        }
    }

    // Initialize ZOS system using macros
    println!("🚀 Starting ZOS Server - Macro-Generated System");
    
    let config = SystemConfig::new(
        "ZOS-Server".to_string(),
        "0.1.0".to_string(),
        true,
        64
    );

    let mut core = init_system(config)?;
    
    println!("✅ ZOS Core initialized: {}", core.config.name);
    println!("🔧 Running self-build...");
    
    self_build(&mut core)?;
    
    println!("🎉 ZOS Server ready!");
    println!("💡 Available commands:");
    println!("  ./zos_server self-build build    - Self-build with LLM");
    println!("  ./zos_server notebooklm import   - Import NotebookLM chunks");
    
    Ok(())
}
