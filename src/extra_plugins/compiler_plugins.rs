// Compiler Plugins - Rustc, GCC, LLVM
use libloading::{Library, Symbol};
use std::ffi::{CString, CStr};
use std::os::raw::{c_char, c_int};

pub struct RustcPlugin {
    library: Library,
}

pub struct GccPlugin {
    library: Library,
}

pub struct LlvmPlugin {
    library: Library,
}

type CompileSourceFn = unsafe extern "C" fn(*const c_char, *const c_char, *const c_char) -> c_int;
type OptimizeFn = unsafe extern "C" fn(*const c_char, *const c_char) -> c_int;

impl RustcPlugin {
    pub fn new(plugin_path: &str) -> Result<Self, String> {
        let library = unsafe { Library::new(plugin_path).map_err(|e| e.to_string())? };
        Ok(RustcPlugin { library })
    }

    pub fn compile_source(&self, source: &str, output: &str, flags: &str) -> Result<(), String> {
        unsafe {
            let compile_fn: Symbol<CompileSourceFn> = self.library.get(b"rustc_compile_source").map_err(|e| e.to_string())?;
            let c_source = CString::new(source).map_err(|e| e.to_string())?;
            let c_output = CString::new(output).map_err(|e| e.to_string())?;
            let c_flags = CString::new(flags).map_err(|e| e.to_string())?;
            let result = compile_fn(c_source.as_ptr(), c_output.as_ptr(), c_flags.as_ptr());
            if result == 0 { Ok(()) } else { Err(format!("Compile failed: {}", result)) }
        }
    }
}

impl GccPlugin {
    pub fn new(plugin_path: &str) -> Result<Self, String> {
        let library = unsafe { Library::new(plugin_path).map_err(|e| e.to_string())? };
        Ok(GccPlugin { library })
    }

    pub fn compile_source(&self, source: &str, output: &str, flags: &str) -> Result<(), String> {
        unsafe {
            let compile_fn: Symbol<CompileSourceFn> = self.library.get(b"gcc_compile_source").map_err(|e| e.to_string())?;
            let c_source = CString::new(source).map_err(|e| e.to_string())?;
            let c_output = CString::new(output).map_err(|e| e.to_string())?;
            let c_flags = CString::new(flags).map_err(|e| e.to_string())?;
            let result = compile_fn(c_source.as_ptr(), c_output.as_ptr(), c_flags.as_ptr());
            if result == 0 { Ok(()) } else { Err(format!("Compile failed: {}", result)) }
        }
    }
}

impl LlvmPlugin {
    pub fn new(plugin_path: &str) -> Result<Self, String> {
        let library = unsafe { Library::new(plugin_path).map_err(|e| e.to_string())? };
        Ok(LlvmPlugin { library })
    }

    pub fn optimize_ir(&self, input_ir: &str, output_ir: &str) -> Result<(), String> {
        unsafe {
            let opt_fn: Symbol<OptimizeFn> = self.library.get(b"llvm_optimize_ir").map_err(|e| e.to_string())?;
            let c_input = CString::new(input_ir).map_err(|e| e.to_string())?;
            let c_output = CString::new(output_ir).map_err(|e| e.to_string())?;
            let result = opt_fn(c_input.as_ptr(), c_output.as_ptr());
            if result == 0 { Ok(()) } else { Err(format!("Optimize failed: {}", result)) }
        }
    }
}
