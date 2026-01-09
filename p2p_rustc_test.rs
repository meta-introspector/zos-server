use crate::p2p_rustc_loader::P2PRustcLoader;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧟 P2P RUSTC LOADER TEST");
    println!("========================");

    let mut loader = P2PRustcLoader::new();

    // Load the massive 2.8GB rustc_driver.so
    let so_path = "/mnt/data1/nix/vendor/rust/cargo2nix/submodules/rust/compiler/zombie_driver2/target/debug/deps/librustc_driver.so";

    println!("🔌 Loading 2.8GB rustc_driver...");
    loader.load_rustc_driver(so_path)?;

    // Test compilation via P2P wrapper
    let test_code = r#"
        fn main() {
            println!("Hello from P2P rustc!");
        }
    "#;

    println!("🚀 Testing compilation via P2P...");
    let result = loader.compile_via_p2p(test_code)?;
    println!("📊 Result: {}", result);

    Ok(())
}
