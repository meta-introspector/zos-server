// Block collector plugin integration for ZOS server
use crate::plugin_driver::PluginDriver;
use std::ffi::{CStr, CString};
use tracing::{info, error};

pub struct BlockCollectorPlugin {
    driver: PluginDriver,
    plugin_loaded: bool,
}

impl BlockCollectorPlugin {
    pub fn new() -> Self {
        Self {
            driver: PluginDriver::new(),
            plugin_loaded: false,
        }
    }

    pub fn load(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let plugin_path = "plugins/libblock_collector_plugin.so";

        info!("🔌 Loading block-collector plugin from: {}", plugin_path);

        self.driver.load_plugin("block_collector", plugin_path)?;
        self.plugin_loaded = true;

        info!("✅ Block-collector plugin loaded");
        Ok(())
    }

    pub fn register_client(&self, peer_id: &str) -> Result<String, Box<dyn std::error::Error>> {
        if !self.plugin_loaded {
            return Err("Plugin not loaded".into());
        }

        let lib = self.driver.plugins.get("block_collector")
            .ok_or("Plugin not found")?;

        unsafe {
            let register: libloading::Symbol<unsafe extern "C" fn(*const i8) -> *const i8> =
                lib.get(b"register_client")?;

            let peer_id_c = CString::new(peer_id)?;
            let result_ptr = register(peer_id_c.as_ptr());
            let result = CStr::from_ptr(result_ptr).to_string_lossy().to_string();

            Ok(result)
        }
    }

    pub fn submit_block(&self, block_json: &str) -> Result<String, Box<dyn std::error::Error>> {
        if !self.plugin_loaded {
            return Err("Plugin not loaded".into());
        }

        let lib = self.driver.plugins.get("block_collector")
            .ok_or("Plugin not found")?;

        unsafe {
            let submit: libloading::Symbol<unsafe extern "C" fn(*const i8) -> *const i8> =
                lib.get(b"submit_block")?;

            let block_c = CString::new(block_json)?;
            let result_ptr = submit(block_c.as_ptr());
            let result = CStr::from_ptr(result_ptr).to_string_lossy().to_string();

            Ok(result)
        }
    }
}

impl Default for BlockCollectorPlugin {
    fn default() -> Self {
        Self::new()
    }
}
