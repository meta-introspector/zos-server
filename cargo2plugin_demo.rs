// Cargo2Plugin Demo - Convert Cargo projects to secure plugin hierarchies
use zos_server::cargo2plugin_loader::Cargo2PluginLoader;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔐 ZOS Cargo2Plugin Loader Demo");

    let mut loader = Cargo2PluginLoader::new();

    // Load current Cargo project as plugin
    println!("\n--- Loading Cargo Project as Plugin ---");
    match loader.load_cargo_project("./Cargo.toml") {
        Ok(hierarchy) => {
            println!("📦 Crate: {}", hierarchy.crate_name);

            // Show security classification
            println!("\n🔒 Security Classification:");
            println!(
                "  Safe functions: {}",
                hierarchy.security_classification.safe_count
            );
            println!(
                "  Controlled functions: {}",
                hierarchy.security_classification.controlled_count
            );
            println!(
                "  Privileged functions: {}",
                hierarchy.security_classification.privileged_count
            );
            println!(
                "  Critical functions: {}",
                hierarchy.security_classification.critical_count
            );

            // Show public functions (user accessible)
            println!("\n👥 Public Functions (User Accessible):");
            for func in &hierarchy.public_functions {
                println!("  ✅ {} - Level: {:?}", func.name, func.security_level);
                println!("     Accessible by: {:?}", func.accessible_by);
            }

            // Show secured functions (root/admin only)
            println!("\n🔐 Secured Functions (Restricted Access):");
            for func in &hierarchy.secured_functions {
                println!("  🔒 {} - Level: {:?}", func.name, func.security_level);
                println!("     Requires role: {}", func.required_role);
                println!("     Virtualized: {}", func.virtualized);
            }

            // Show virtualization features
            println!("\n🎭 Virtualization Features:");
            for feature in &hierarchy.virtualization_features {
                println!("  📋 Feature: {}", feature.feature_name);
                println!("     Macro: {}", feature.macro_name);
                println!("     Virtual functions: {:?}", feature.virtual_functions);
            }

            // Generate plugin code
            println!("\n--- Generated Plugin Code ---");
            let plugin_code = loader.generate_plugin_code(&hierarchy);
            println!("📝 Generated plugin structure:");
            println!("{}", plugin_code);

            // Demonstrate feature-based access
            println!("\n--- Feature-Based Access Control ---");
            println!("🎯 Users can access functions based on their security level:");
            println!(
                "  👤 user      -> {} functions",
                hierarchy
                    .public_functions
                    .iter()
                    .filter(|f| f.accessible_by.contains(&"user".to_string()))
                    .count()
            );
            println!(
                "  👨‍💻 developer -> {} functions",
                hierarchy
                    .public_functions
                    .iter()
                    .filter(|f| f.accessible_by.contains(&"developer".to_string()))
                    .count()
            );
            println!(
                "  👨‍💼 admin     -> {} functions",
                hierarchy
                    .public_functions
                    .iter()
                    .filter(|f| f.accessible_by.contains(&"admin".to_string()))
                    .count()
                    + hierarchy
                        .secured_functions
                        .iter()
                        .filter(|f| f.required_role == "admin")
                        .count()
            );
            println!(
                "  🔑 root      -> {} functions",
                hierarchy.public_functions.len() + hierarchy.secured_functions.len()
            );

            // Show macro usage examples
            println!("\n--- Macro Usage Examples ---");
            for feature in &hierarchy.virtualization_features {
                println!("// Using {} macro:", feature.macro_name);
                println!("{}!(some_function);", feature.macro_name);
                println!("// Expands to: virtual_impl::some_function()");
                println!();
            }

            println!("🎯 Plugin Hierarchy Complete:");
            println!("  ✅ Functions split by security level");
            println!("  ✅ Public API for user access");
            println!("  ✅ Secured API for privileged access");
            println!("  ✅ Virtualization macros generated");
            println!("  ✅ Feature flags for conditional compilation");
        }
        Err(e) => println!("❌ Failed to load Cargo project: {}", e),
    }

    Ok(())
}
