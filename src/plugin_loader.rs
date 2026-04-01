use anyhow::Result;
use libloading::{Library, Symbol};
use std::collections::HashMap;
use std::path::Path;

#[repr(C)]
pub struct PluginMetadata {
    pub name: *const i8,
    pub version: *const i8,
}

pub struct Plugin {
    _lib: Library,
    pub coords: [u64; 6],
}

pub struct PluginRegistry {
    plugins: HashMap<String, Plugin>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    pub fn load(&mut self, name: &str, path: &Path) -> Result<()> {
        unsafe {
            let lib = Library::new(path)?;

            let get_coords: Symbol<extern "C" fn() -> [u64; 6]> = lib.get(b"plugin_a11y_coords")?;

            let coords = get_coords();

            self.plugins
                .insert(name.to_string(), Plugin { _lib: lib, coords });

            Ok(())
        }
    }

    pub fn get_coords(&self, name: &str) -> Option<[u64; 6]> {
        self.plugins.get(name).map(|p| p.coords)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_plugins() {
        let so_path = std::path::PathBuf::from(env!("HOME"))
            .join("projects/zos-plugins/target/release/libpayphone.so");
        if !so_path.exists() {
            eprintln!("skipping: {:?} not found", so_path);
            return;
        }
        let mut registry = PluginRegistry::new();

        registry
            .load("payphone", &so_path)
            .unwrap();

        let coords = registry.get_coords("payphone").unwrap();
        assert_eq!(coords, [17, 8, 20, 63, 41, 4]);
    }
}
