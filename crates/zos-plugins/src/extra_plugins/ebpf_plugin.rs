// eBPF Plugin Interface
use libloading::{Library, Symbol};
use std::ffi::{CString, CStr};
use std::os::raw::{c_char, c_int};

pub struct EbpfPlugin {
    library: Library,
}

type LoadProgramFn = unsafe extern "C" fn(*const c_char) -> c_int;
type UnloadProgramFn = unsafe extern "C" fn(c_int) -> c_int;
type AttachProgramFn = unsafe extern "C" fn(c_int, *const c_char) -> c_int;

impl EbpfPlugin {
    pub fn new(plugin_path: &str) -> Result<Self, String> {
        let library = unsafe { Library::new(plugin_path).map_err(|e| e.to_string())? };
        Ok(EbpfPlugin { library })
    }

    pub fn load_program(&self, program_path: &str) -> Result<i32, String> {
        unsafe {
            let load_fn: Symbol<LoadProgramFn> = self.library.get(b"ebpf_load_program").map_err(|e| e.to_string())?;
            let c_path = CString::new(program_path).map_err(|e| e.to_string())?;
            let result = load_fn(c_path.as_ptr());
            if result >= 0 { Ok(result) } else { Err(format!("Load failed: {}", result)) }
        }
    }
}
