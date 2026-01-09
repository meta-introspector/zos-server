// WASM Plugin Interface
use libloading::{Library, Symbol};
use std::ffi::{CString, CStr};
use std::os::raw::{c_char, c_int};

pub struct WasmPlugin {
    library: Library,
}

type LoadModuleFn = unsafe extern "C" fn(*const c_char) -> c_int;
type CallFunctionFn = unsafe extern "C" fn(c_int, *const c_char, *const c_char) -> c_int;

impl WasmPlugin {
    pub fn new(plugin_path: &str) -> Result<Self, String> {
        let library = unsafe { Library::new(plugin_path).map_err(|e| e.to_string())? };
        Ok(WasmPlugin { library })
    }

    pub fn load_module(&self, wasm_path: &str) -> Result<i32, String> {
        unsafe {
            let load_fn: Symbol<LoadModuleFn> = self.library.get(b"wasm_load_module").map_err(|e| e.to_string())?;
            let c_path = CString::new(wasm_path).map_err(|e| e.to_string())?;
            let result = load_fn(c_path.as_ptr());
            if result >= 0 { Ok(result) } else { Err(format!("Load failed: {}", result)) }
        }
    }
}
