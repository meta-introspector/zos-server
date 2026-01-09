// Runtime Plugins - NodeJS, Python, Nix
use libloading::{Library, Symbol};
use std::ffi::{CString, CStr};
use std::os::raw::{c_char, c_int};

pub struct NodeJsPlugin {
    library: Library,
}

pub struct PythonPlugin {
    library: Library,
}

pub struct NixPlugin {
    library: Library,
}

type ExecuteScriptFn = unsafe extern "C" fn(*const c_char) -> c_int;
type InstallPackageFn = unsafe extern "C" fn(*const c_char) -> c_int;

impl NodeJsPlugin {
    pub fn new(plugin_path: &str) -> Result<Self, String> {
        let library = unsafe { Library::new(plugin_path).map_err(|e| e.to_string())? };
        Ok(NodeJsPlugin { library })
    }

    pub fn execute_script(&self, script: &str) -> Result<(), String> {
        unsafe {
            let exec_fn: Symbol<ExecuteScriptFn> = self.library.get(b"nodejs_execute_script").map_err(|e| e.to_string())?;
            let c_script = CString::new(script).map_err(|e| e.to_string())?;
            let result = exec_fn(c_script.as_ptr());
            if result == 0 { Ok(()) } else { Err(format!("Execute failed: {}", result)) }
        }
    }
}

impl PythonPlugin {
    pub fn new(plugin_path: &str) -> Result<Self, String> {
        let library = unsafe { Library::new(plugin_path).map_err(|e| e.to_string())? };
        Ok(PythonPlugin { library })
    }

    pub fn execute_script(&self, script: &str) -> Result<(), String> {
        unsafe {
            let exec_fn: Symbol<ExecuteScriptFn> = self.library.get(b"python_execute_script").map_err(|e| e.to_string())?;
            let c_script = CString::new(script).map_err(|e| e.to_string())?;
            let result = exec_fn(c_script.as_ptr());
            if result == 0 { Ok(()) } else { Err(format!("Execute failed: {}", result)) }
        }
    }
}

impl NixPlugin {
    pub fn new(plugin_path: &str) -> Result<Self, String> {
        let library = unsafe { Library::new(plugin_path).map_err(|e| e.to_string())? };
        Ok(NixPlugin { library })
    }

    pub fn install_package(&self, package: &str) -> Result<(), String> {
        unsafe {
            let install_fn: Symbol<InstallPackageFn> = self.library.get(b"nix_install_package").map_err(|e| e.to_string())?;
            let c_package = CString::new(package).map_err(|e| e.to_string())?;
            let result = install_fn(c_package.as_ptr());
            if result == 0 { Ok(()) } else { Err(format!("Install failed: {}", result)) }
        }
    }
}
