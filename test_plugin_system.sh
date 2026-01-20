#!/bin/bash
# Test plugin loading in zos-server

set -e

echo "🔨 Step 1: Build block-collector plugin"
cd /mnt/data1/meta-introspector/tools/so-plugins/block-collector
cargo build --release
PLUGIN_PATH=$(pwd)/target/release/libblock_collector_plugin.so

echo ""
echo "✅ Plugin built: $PLUGIN_PATH"

echo ""
echo "📦 Step 2: Copy to zos-server plugins directory"
mkdir -p ~/zos-server/plugins
cp $PLUGIN_PATH ~/zos-server/plugins/
echo "✅ Copied to: ~/zos-server/plugins/libblock_collector_plugin.so"

echo ""
echo "🔧 Step 3: Add libloading to zos-server"
cd ~/zos-server
if ! grep -q "libloading" Cargo.toml; then
    echo 'libloading = "0.8"' >> Cargo.toml
    echo "✅ Added libloading dependency"
else
    echo "✅ libloading already in Cargo.toml"
fi

echo ""
echo "🧪 Step 4: Test plugin loading"
cat > test_plugin_load.rs << 'EOF'
use std::path::Path;

mod plugin_manager;
use plugin_manager::PluginManager;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔌 Testing plugin loading...");

    let mut manager = PluginManager::new();
    manager.load_all_plugins(Path::new("plugins"))?;

    println!("\n👤 Testing register_client...");
    let result = manager.call_register_client("block_collector", "test_peer_123")?;
    println!("Response: {}", result);

    println!("\n📦 Testing submit_block...");
    let block = r#"{"slot":12345,"hash":"abc","transactions":[],"timestamp":123,"client_id":"test"}"#;
    let result = manager.call_submit_block("block_collector", block)?;
    println!("Response: {}", result);

    println!("\n✅ All tests passed!");
    Ok(())
}
EOF

cargo run --bin test_plugin_load 2>&1 | tail -20

echo ""
echo "✅ Plugin system ready!"
echo ""
echo "Next: Add plugin_manager to zos-server main.rs"
