// Binary Analysis Plugins - objdump, binutils, ld
use libloading::{Library, Symbol};
use std::ffi::{CString, CStr};
use std::os::raw::{c_char, c_int};

pub struct ObjdumpPlugin {
    library: Library,
}

pub struct BinutilsPlugin {
    library: Library,
}

pub struct LinkerPlugin {
    library: Library,
}

type DisassembleFn = unsafe extern "C" fn(*const c_char, *mut *mut c_char) -> c_int;
type AnalyzeBinaryFn = unsafe extern "C" fn(*const c_char, *mut *mut c_char) -> c_int;
type LinkObjectsFn = unsafe extern "C" fn(*const c_char, *const c_char, *const c_char) -> c_int;

impl ObjdumpPlugin {
    pub fn new(plugin_path: &str) -> Result<Self, String> {
        let library = unsafe { Library::new(plugin_path).map_err(|e| e.to_string())? };
        Ok(ObjdumpPlugin { library })
    }

    pub fn disassemble(&self, binary_path: &str) -> Result<String, String> {
        unsafe {
            let disasm_fn: Symbol<DisassembleFn> = self.library.get(b"objdump_disassemble").map_err(|e| e.to_string())?;
            let c_path = CString::new(binary_path).map_err(|e| e.to_string())?;
            let mut result_ptr: *mut c_char = std::ptr::null_mut();
            let status = disasm_fn(c_path.as_ptr(), &mut result_ptr);
            
            if status == 0 && !result_ptr.is_null() {
                let result_str = CStr::from_ptr(result_ptr).to_string_lossy().to_string();
                Ok(result_str)
            } else {
                Err(format!("Disassemble failed: {}", status))
            }
        }
    }
}

impl BinutilsPlugin {
    pub fn new(plugin_path: &str) -> Result<Self, String> {
        let library = unsafe { Library::new(plugin_path).map_err(|e| e.to_string())? };
        Ok(BinutilsPlugin { library })
    }

    pub fn analyze_binary(&self, binary_path: &str) -> Result<String, String> {
        unsafe {
            let analyze_fn: Symbol<AnalyzeBinaryFn> = self.library.get(b"binutils_analyze_binary").map_err(|e| e.to_string())?;
            let c_path = CString::new(binary_path).map_err(|e| e.to_string())?;
            let mut result_ptr: *mut c_char = std::ptr::null_mut();
            let status = analyze_fn(c_path.as_ptr(), &mut result_ptr);
            
            if status == 0 && !result_ptr.is_null() {
                let result_str = CStr::from_ptr(result_ptr).to_string_lossy().to_string();
                Ok(result_str)
            } else {
                Err(format!("Analysis failed: {}", status))
            }
        }
    }
}

impl LinkerPlugin {
    pub fn new(plugin_path: &str) -> Result<Self, String> {
        let library = unsafe { Library::new(plugin_path).map_err(|e| e.to_string())? };
        Ok(LinkerPlugin { library })
    }

    pub fn link_objects(&self, objects: &str, output: &str, flags: &str) -> Result<(), String> {
        unsafe {
            let link_fn: Symbol<LinkObjectsFn> = self.library.get(b"ld_link_objects").map_err(|e| e.to_string())?;
            let c_objects = CString::new(objects).map_err(|e| e.to_string())?;
            let c_output = CString::new(output).map_err(|e| e.to_string())?;
            let c_flags = CString::new(flags).map_err(|e| e.to_string())?;
            let result = link_fn(c_objects.as_ptr(), c_output.as_ptr(), c_flags.as_ptr());
            if result == 0 { Ok(()) } else { Err(format!("Link failed: {}", result)) }
        }
    }
}
