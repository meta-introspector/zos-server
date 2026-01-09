use libloading::{Library, Symbol};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug)]
pub struct PluginManager {
    pub loaded_plugins: HashMap<String, LoadedPlugin>,
    pub plugin_registry: HashMap<String, PluginMetadata>,
    pub dao_approved: HashMap<String, String>, // plugin_name -> so_hash
}

#[derive(Debug)]
pub struct LoadedPlugin {
    pub library: Library,
    pub metadata: PluginMetadata,
    pub services: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub so_path: String,
    pub so_hash: String,
    pub dao_approved: bool,
    pub test_env_passed: bool,
    pub services: Vec<ServiceDefinition>,
}

#[derive(Debug, Clone)]
pub struct ServiceDefinition {
    pub name: String,
    pub function_name: String,
    pub input_type: String,
    pub output_type: String,
    pub access_level: String,
}

// Plugin ABI - every plugin must implement these
type PluginInitFn = unsafe extern "C" fn() -> i32;
type PluginGetServicesFn = unsafe extern "C" fn() -> *const u8;
type PluginCallServiceFn = unsafe extern "C" fn(*const u8, usize, *mut u8, *mut usize) -> i32;

impl PluginManager {
    pub fn new() -> Self {
        Self {
            loaded_plugins: HashMap::new(),
            plugin_registry: HashMap::new(),
            dao_approved: HashMap::new(),
        }
    }

    pub fn register_plugin(&mut self, metadata: PluginMetadata) -> Result<(), String> {
        // Check if DAO approved this plugin version
        if let Some(approved_hash) = self.dao_approved.get(&metadata.name) {
            if approved_hash != &metadata.so_hash {
                return Err("Plugin version not DAO approved".to_string());
            }
        } else if metadata.dao_approved {
            return Err("Plugin claims DAO approval but not in registry".to_string());
        }

        println!("📦 Plugin registered: {} v{} (DAO: {})",
                 metadata.name, metadata.version, metadata.dao_approved);

        self.plugin_registry.insert(metadata.name.clone(), metadata);
        Ok(())
    }

    pub fn load_plugin(&mut self, plugin_name: &str) -> Result<(), String> {
        let metadata = self.plugin_registry.get(plugin_name)
            .ok_or("Plugin not registered")?
            .clone();

        if !metadata.dao_approved {
            return Err("Plugin not DAO approved for production".to_string());
        }

        // Load the .so file
        let library = unsafe {
            Library::new(&metadata.so_path)
                .map_err(|e| format!("Failed to load {}: {}", metadata.so_path, e))?
        };

        // Initialize plugin
        let init_fn: Symbol<PluginInitFn> = unsafe {
            library.get(b"plugin_init")
                .map_err(|e| format!("Missing plugin_init: {}", e))?
        };

        let result = unsafe { init_fn() };
        if result != 0 {
            return Err("Plugin initialization failed".to_string());
        }

        // Get available services
        let get_services_fn: Symbol<PluginGetServicesFn> = unsafe {
            library.get(b"plugin_get_services")
                .map_err(|e| format!("Missing plugin_get_services: {}", e))?
        };

        let services_ptr = unsafe { get_services_fn() };
        let services_json = unsafe {
            std::ffi::CStr::from_ptr(services_ptr as *const i8)
                .to_string_lossy()
                .to_string()
        };

        let services: Vec<String> = serde_json::from_str(&services_json)
            .map_err(|e| format!("Invalid services JSON: {}", e))?;

        let loaded_plugin = LoadedPlugin {
            library,
            metadata: metadata.clone(),
            services,
        };

        println!("🔌 Plugin loaded: {} with {} services", plugin_name, loaded_plugin.services.len());
        self.loaded_plugins.insert(plugin_name.to_string(), loaded_plugin);

        Ok(())
    }

    pub fn call_service(&self, plugin_name: &str, service_name: &str, input: &[u8]) -> Result<Vec<u8>, String> {
        let plugin = self.loaded_plugins.get(plugin_name)
            .ok_or("Plugin not loaded")?;

        if !plugin.services.contains(&service_name.to_string()) {
            return Err("Service not available".to_string());
        }

        // Call the service
        let call_fn: Symbol<PluginCallServiceFn> = unsafe {
            plugin.library.get(b"plugin_call_service")
                .map_err(|e| format!("Missing plugin_call_service: {}", e))?
        };

        // Prepare service call data
        let call_data = serde_json::json!({
            "service": service_name,
            "input": base64::encode(input)
        });
        let call_json = call_data.to_string();

        let mut output_buffer = vec![0u8; 1024 * 1024]; // 1MB buffer
        let mut output_size = output_buffer.len();

        let result = unsafe {
            call_fn(
                call_json.as_ptr(),
                call_json.len(),
                output_buffer.as_mut_ptr(),
                &mut output_size
            )
        };

        if result != 0 {
            return Err("Service call failed".to_string());
        }

        output_buffer.truncate(output_size);
        Ok(output_buffer)
    }

    pub fn dao_approve_plugin(&mut self, plugin_name: &str, so_hash: &str) {
        println!("✅ DAO approved plugin: {} (hash: {})", plugin_name, &so_hash[..8]);
        self.dao_approved.insert(plugin_name.to_string(), so_hash.to_string());

        // Update existing registration if present
        if let Some(metadata) = self.plugin_registry.get_mut(plugin_name) {
            if metadata.so_hash == so_hash {
                metadata.dao_approved = true;
            }
        }
    }

    pub fn test_plugin_in_sandbox(&self, plugin_name: &str) -> Result<bool, String> {
        let metadata = self.plugin_registry.get(plugin_name)
            .ok_or("Plugin not registered")?;

        // Create isolated test environment
        let test_dir = format!("/tmp/zos_plugin_test_{}", plugin_name);
        std::fs::create_dir_all(&test_dir)
            .map_err(|e| format!("Failed to create test dir: {}", e))?;

        // Copy plugin to test environment
        let test_so_path = format!("{}/plugin.so", test_dir);
        std::fs::copy(&metadata.so_path, &test_so_path)
            .map_err(|e| format!("Failed to copy plugin: {}", e))?;

        // Load and test in sandbox
        let test_result = unsafe {
            match Library::new(&test_so_path) {
                Ok(lib) => {
                    // Test basic plugin interface
                    if let Ok(init_fn) = lib.get::<PluginInitFn>(b"plugin_init") {
                        init_fn() == 0
                    } else {
                        false
                    }
                },
                Err(_) => false,
            }
        };

        // Cleanup
        std::fs::remove_dir_all(&test_dir).ok();

        Ok(test_result)
    }

    pub fn get_plugin_status(&self) -> String {
        let mut status = serde_json::json!({
            "loaded_plugins": self.loaded_plugins.len(),
            "registered_plugins": self.plugin_registry.len(),
            "dao_approved": self.dao_approved.len(),
            "plugins": []
        });

        for (name, metadata) in &self.plugin_registry {
            let plugin_info = serde_json::json!({
                "name": name,
                "version": metadata.version,
                "dao_approved": metadata.dao_approved,
                "loaded": self.loaded_plugins.contains_key(name),
                "services": metadata.services.len()
            });
            status["plugins"].as_array_mut().unwrap().push(plugin_info);
        }

        status.to_string()
    }
}

// Helper for building plugins
pub fn build_plugin_from_crate(crate_path: &str, output_dir: &str) -> Result<String, String> {
    let crate_name = Path::new(crate_path)
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or("Invalid crate path")?;

    let so_path = format!("{}/lib{}.so", output_dir, crate_name);

    // Build as cdylib
    let output = std::process::Command::new("cargo")
        .args(&["build", "--release", "--crate-type", "cdylib"])
        .current_dir(crate_path)
        .output()
        .map_err(|e| format!("Cargo build failed: {}", e))?;

    if !output.status.success() {
        return Err(format!("Build failed: {}", String::from_utf8_lossy(&output.stderr)));
    }

    // Copy to output directory
    let built_so = format!("{}/target/release/lib{}.so", crate_path, crate_name);
    std::fs::copy(&built_so, &so_path)
        .map_err(|e| format!("Failed to copy SO: {}", e))?;

    println!("🔨 Built plugin: {} → {}", crate_name, so_path);
    Ok(so_path)
}
