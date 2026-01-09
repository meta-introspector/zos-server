// Core POSIX/System Plugins - Essential for basic operation
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};

/// Macro to define core plugins with shared object attributes
macro_rules! define_core_plugin {
    (
        $plugin_name:ident,
        so_path: $so_path:expr,
        symbol: $symbol:expr,
        fn_type: $fn_type:ty,
        fallback: $fallback:expr
    ) => {
        pub struct $plugin_name {
            #[cfg(feature = "dynamic-loading")]
            lib_handle: Option<libloading::Library>,
            #[cfg(feature = "dynamic-loading")]
            function: Option<libloading::Symbol<'static, $fn_type>>,
        }

        impl $plugin_name {
            pub fn new() -> Self {
                Self {
                    #[cfg(feature = "dynamic-loading")]
                    lib_handle: None,
                    #[cfg(feature = "dynamic-loading")]
                    function: None,
                }
            }

            #[cfg(feature = "dynamic-loading")]
            pub fn load_shared_object(&mut self) -> Result<(), String> {
                use libloading::{Library, Symbol};

                unsafe {
                    let lib = Library::new($so_path)
                        .map_err(|e| format!("Failed to load {}: {}", $so_path, e))?;

                    let func: Symbol<$fn_type> = lib
                        .get($symbol.as_bytes())
                        .map_err(|e| format!("Failed to find symbol {}: {}", $symbol, e))?;

                    // Extend lifetime to 'static (unsafe but necessary for storage)
                    let func: Symbol<'static, $fn_type> = std::mem::transmute(func);

                    self.function = Some(func);
                    self.lib_handle = Some(lib);
                    Ok(())
                }
            }

            #[cfg(not(feature = "dynamic-loading"))]
            pub fn load_shared_object(&mut self) -> Result<(), String> {
                Ok(()) // No-op when dynamic loading is disabled
            }

            pub fn is_loaded(&self) -> bool {
                #[cfg(feature = "dynamic-loading")]
                {
                    self.function.is_some()
                }
                #[cfg(not(feature = "dynamic-loading"))]
                {
                    true // Always "loaded" when using fallback
                }
            }

            pub fn execute(&self, input: &str) -> Result<String, String> {
                #[cfg(feature = "dynamic-loading")]
                {
                    if let Some(ref func) = self.function {
                        // Use loaded shared object function
                        let c_input = CString::new(input)
                            .map_err(|e| format!("Invalid input string: {}", e))?;

                        unsafe {
                            let result = func(c_input.as_ptr());
                            if result == 0 {
                                Ok(format!(
                                    "SO execution successful for {}",
                                    stringify!($plugin_name)
                                ))
                            } else {
                                Err(format!("SO execution failed with code: {}", result))
                            }
                        }
                    } else {
                        // Fall back to built-in implementation
                        $fallback(input)
                    }
                }
                #[cfg(not(feature = "dynamic-loading"))]
                {
                    // Always use fallback when dynamic loading is disabled
                    $fallback(input)
                }
            }
        }

        impl Default for $plugin_name {
            fn default() -> Self {
                Self::new()
            }
        }
    };
}

// Fallback implementations for core plugins
fn posix_fallback(input: &str) -> Result<String, String> {
    // Built-in POSIX operations
    match input {
        "pwd" => Ok(std::env::current_dir()
            .map_err(|e| format!("pwd failed: {}", e))?
            .to_string_lossy()
            .to_string()),
        "whoami" => Ok(std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .unwrap_or_else(|_| "unknown".to_string())),
        _ => Ok(format!("POSIX fallback executed: {}", input)),
    }
}

fn bash_fallback(input: &str) -> Result<String, String> {
    use std::process::Command;

    let output = Command::new("sh")
        .args(&["-c", input])
        .output()
        .map_err(|e| format!("bash fallback failed: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

fn cargo_fallback(input: &str) -> Result<String, String> {
    use std::process::Command;

    let args: Vec<&str> = input.split_whitespace().collect();
    if args.is_empty() {
        return Err("No cargo command provided".to_string());
    }

    let output = Command::new("cargo")
        .args(&args)
        .output()
        .map_err(|e| format!("cargo fallback failed: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

fn rustc_fallback(input: &str) -> Result<String, String> {
    use std::process::Command;

    let args: Vec<&str> = input.split_whitespace().collect();
    if args.is_empty() {
        return Err("No rustc command provided".to_string());
    }

    let output = Command::new("rustc")
        .args(&args)
        .output()
        .map_err(|e| format!("rustc fallback failed: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

// Core system function types
type PosixFn = unsafe extern "C" fn(*const c_char) -> c_int;
type BashFn = unsafe extern "C" fn(*const c_char) -> c_int;
type CargoFn = unsafe extern "C" fn(*const c_char) -> c_int;
type RustcFn = unsafe extern "C" fn(*const c_char) -> c_int;

// Define core plugins using the macro
define_core_plugin!(
    PosixPlugin,
    so_path: "/usr/lib/libposix_plugin.so",
    symbol: "posix_execute",
    fn_type: PosixFn,
    fallback: posix_fallback
);

define_core_plugin!(
    BashPlugin,
    so_path: "/usr/lib/libbash_plugin.so",
    symbol: "bash_execute",
    fn_type: BashFn,
    fallback: bash_fallback
);

define_core_plugin!(
    CargoPlugin,
    so_path: "/usr/lib/libcargo_plugin.so",
    symbol: "cargo_execute",
    fn_type: CargoFn,
    fallback: cargo_fallback
);

define_core_plugin!(
    RustcPlugin,
    so_path: "/usr/lib/librustc_plugin.so",
    symbol: "rustc_execute",
    fn_type: RustcFn,
    fallback: rustc_fallback
);

/// Core plugin manager that handles all core system plugins
pub struct CorePluginManager {
    pub posix: PosixPlugin,
    pub bash: BashPlugin,
    pub cargo: CargoPlugin,
    pub rustc: RustcPlugin,
}

impl CorePluginManager {
    pub fn new() -> Self {
        Self {
            posix: PosixPlugin::new(),
            bash: BashPlugin::new(),
            cargo: CargoPlugin::new(),
            rustc: RustcPlugin::new(),
        }
    }

    /// Attempt to load all shared objects, falling back gracefully
    pub fn load_shared_objects(&mut self) -> Vec<String> {
        let mut results = Vec::new();

        match self.posix.load_shared_object() {
            Ok(()) => results.push("✅ POSIX plugin loaded from SO".to_string()),
            Err(e) => results.push(format!("⚠️ POSIX plugin using fallback: {}", e)),
        }

        match self.bash.load_shared_object() {
            Ok(()) => results.push("✅ Bash plugin loaded from SO".to_string()),
            Err(e) => results.push(format!("⚠️ Bash plugin using fallback: {}", e)),
        }

        match self.cargo.load_shared_object() {
            Ok(()) => results.push("✅ Cargo plugin loaded from SO".to_string()),
            Err(e) => results.push(format!("⚠️ Cargo plugin using fallback: {}", e)),
        }

        match self.rustc.load_shared_object() {
            Ok(()) => results.push("✅ Rustc plugin loaded from SO".to_string()),
            Err(e) => results.push(format!("⚠️ Rustc plugin using fallback: {}", e)),
        }

        results
    }

    /// Get status of all plugins
    pub fn get_status(&self) -> Vec<(String, bool)> {
        vec![
            ("POSIX".to_string(), self.posix.is_loaded()),
            ("Bash".to_string(), self.bash.is_loaded()),
            ("Cargo".to_string(), self.cargo.is_loaded()),
            ("Rustc".to_string(), self.rustc.is_loaded()),
        ]
    }

    /// Execute a command using the appropriate plugin
    pub fn execute(&self, plugin: &str, command: &str) -> Result<String, String> {
        match plugin.to_lowercase().as_str() {
            "posix" => self.posix.execute(command),
            "bash" => self.bash.execute(command),
            "cargo" => self.cargo.execute(command),
            "rustc" => self.rustc.execute(command),
            _ => Err(format!("Unknown plugin: {}", plugin)),
        }
    }
}

impl Default for CorePluginManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_plugin_manager() {
        let mut manager = CorePluginManager::new();

        // Test loading (will use fallbacks if SOs not available)
        let results = manager.load_shared_objects();
        assert_eq!(results.len(), 4);

        // Test status
        let status = manager.get_status();
        assert_eq!(status.len(), 4);

        // Test execution
        let result = manager.execute("posix", "pwd");
        assert!(result.is_ok());
    }

    #[test]
    fn test_individual_plugins() {
        let posix = PosixPlugin::new();
        assert!(posix.execute("pwd").is_ok());

        let bash = BashPlugin::new();
        assert!(bash.execute("echo hello").is_ok());
    }
}
