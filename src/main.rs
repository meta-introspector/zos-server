// ZOS Main - LMFDB Orbit System
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
            "orbit" => {
                return handle_orbit_command(&args[2..]).await;
            }
            _ => {}
        }
    }

    // Initialize LMFDB Orbit System
    println!("🌌 Starting ZOS Server - LMFDB Orbit System");
    
    // Create core system (Level 11 orbits)
    let core = SystemInstance::core_system()?;
    println!("✅ Core system initialized: {}", core.signature());
    
    // Create extended system (Level 11 + 23 orbits)  
    let extended = SystemInstance::extended_system()?;
    println!("✅ Extended system initialized: {}", extended.signature());
    
    // Test orbit execution
    let test_data = b"Hello ZOS Orbit System!";
    println!("🧪 Testing orbit execution...");
    
    let core_result = core.execute_all(test_data)?;
    println!("🔄 Core orbit result: {} bytes", core_result.len());
    
    let extended_result = extended.execute_all(test_data)?;
    println!("🔄 Extended orbit result: {} bytes", extended_result.len());
    
    // Test orbit enums
    let posix_orbit = CoreOrbit::from_label("11.a1")?;
    let blockchain_orbit = ExtendedOrbit::from_label("23.a1")?;
    
    println!("🎯 POSIX orbit: {}", posix_orbit.orbit().label);
    println!("🎯 Blockchain orbit: {}", blockchain_orbit.orbit().label);
    
    println!("🎉 ZOS Orbit System ready!");
    println!("💡 Available commands:");
    println!("  ./zos_server orbit core           - Test core orbits");
    println!("  ./zos_server orbit extended       - Test extended orbits");
    println!("  ./zos_server orbit compose        - Compose orbits");
    println!("  ./zos_server self-build build     - Self-build with LLM");
    println!("  ./zos_server notebooklm import    - Import NotebookLM chunks");
    
    Ok(())
}

async fn handle_orbit_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    match args.get(0).map(|s| s.as_str()) {
        Some("core") => {
            println!("🌌 Testing Core Orbit System (Level 11)");
            let core = CoreSystem::new()?;
            let test_data = b"Core orbit test";
            let result = core.execute(test_data)?;
            println!("✅ Core execution: {} -> {} bytes", test_data.len(), result.len());
            println!("📊 Signature: {}", core.signature());
        },
        Some("extended") => {
            println!("🌌 Testing Extended Orbit System (Level 23)");
            let extended = ExtendedSystem::new()?;
            let test_data = b"Extended orbit test";
            let result = extended.execute(test_data)?;
            println!("✅ Extended execution: {} -> {} bytes", test_data.len(), result.len());
            println!("📊 Signature: {}", extended.signature());
        },
        Some("compose") => {
            println!("🌌 Testing Orbit Composition");
            let posix = SystemArg::from_lmfdb("11.a1")?;
            let bash = SystemArg::from_lmfdb("11.a2")?;
            let composition = compose_orbits(&posix, &bash)?;
            println!("✅ Composed orbits: {} bytes", composition.len());
            
            let core_orbit = SystemArg::from_lmfdb("11.a1")?;
            let blockchain_orbit = SystemArg::from_lmfdb("23.a1")?;
            let transform = orbit_transform(&core_orbit, &blockchain_orbit)?;
            println!("🔄 Orbit transform: {}", transform);
        },
        _ => {
            println!("Orbit Commands:");
            println!("  core      - Test core orbit system");
            println!("  extended  - Test extended orbit system");
            println!("  compose   - Test orbit composition");
        }
    }
    
    Ok(())
}
