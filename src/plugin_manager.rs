// Plugin loader for ZOS server
use libloading::Library;
use std::ffi::{CStr, CString};
use std::path::Path;
use tracing::{info, error};

pub struct PluginManager {
    plugins: Vec<LoadedPlugin>,
}

struct LoadedPlugin {
    name: String,
    library: Library,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    pub fn load_plugin(&mut self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        info!("🔌 Loading plugin: {}", path.display());

        let library = unsafe { Library::new(path)? };
        let name = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        self.plugins.push(LoadedPlugin { name, library });

        info!("✅ Plugin loaded: {}", path.display());
        Ok(())
    }

    pub fn load_all_plugins(&mut self, plugin_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
        info!("📂 Loading plugins from: {}", plugin_dir.display());

        if !plugin_dir.exists() {
            std::fs::create_dir_all(plugin_dir)?;
        }

        for entry in std::fs::read_dir(plugin_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("so") {
                if let Err(e) = self.load_plugin(&path) {
                    error!("❌ Failed to load {}: {}", path.display(), e);
                }
            }
        }

        info!("✅ Loaded {} plugins", self.plugins.len());
        Ok(())
    }

    pub fn call_register_client(&self, plugin_name: &str, peer_id: &str) -> Result<String, Box<dyn std::error::Error>> {
        let plugin = self.plugins.iter()
            .find(|p| p.name.contains(plugin_name))
            .ok_or("Plugin not found")?;

        unsafe {
            let register = plugin.library.get::<unsafe extern "C" fn(*const i8) -> *const i8>(b"register_client")?;
            let peer_id_c = CString::new(peer_id)?;
            let result_ptr = register(peer_id_c.as_ptr());
            let result = CStr::from_ptr(result_ptr).to_string_lossy().to_string();
            Ok(result)
        }
    }

    pub fn call_submit_block(&self, plugin_name: &str, block_json: &str) -> Result<String, Box<dyn std::error::Error>> {
        let plugin = self.plugins.iter()
            .find(|p| p.name.contains(plugin_name))
            .ok_or("Plugin not found")?;

        unsafe {
            let submit = plugin.library.get::<unsafe extern "C" fn(*const i8) -> *const i8>(b"submit_block")?;
            let block_c = CString::new(block_json)?;
            let result_ptr = submit(block_c.as_ptr());
            let result = CStr::from_ptr(result_ptr).to_string_lossy().to_string();
            Ok(result)
        }
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}
