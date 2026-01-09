// Linux Kernel Plugin Interface
// Plugin for kernel operations via libloading

use crate::traits::LibraryLoader;
use libloading::{Library, Symbol};
use std::ffi::{CString, CStr};
use std::os::raw::{c_char, c_int, c_void};

pub struct KernelPlugin {
    library: Library,
}

type LoadModuleFn = unsafe extern "C" fn(*const c_char) -> c_int;
type UnloadModuleFn = unsafe extern "C" fn(*const c_char) -> c_int;
type ListModulesFn = unsafe extern "C" fn(*mut *mut c_char) -> c_int;
type GetSysInfoFn = unsafe extern "C" fn(*mut *mut c_char) -> c_int;
type SetSysctlFn = unsafe extern "C" fn(*const c_char, *const c_char) -> c_int;

impl KernelPlugin {
    pub fn new(plugin_path: &str) -> Result<Self, String> {
        let library = unsafe {
            Library::new(plugin_path)
                .map_err(|e| format!("Failed to load kernel plugin: {}", e))?
        };

        Ok(KernelPlugin { library })
    }

    pub fn load_module(&self, module_path: &str) -> Result<(), String> {
        unsafe {
            let load_fn: Symbol<LoadModuleFn> = self.library
                .get(b"kernel_load_module")
                .map_err(|e| format!("Function not found: {}", e))?;

            let c_path = CString::new(module_path)
                .map_err(|e| format!("Invalid module path: {}", e))?;

            let result = load_fn(c_path.as_ptr());
            if result == 0 { Ok(()) } else { Err(format!("Load failed: {}", result)) }
        }
    }

    pub fn unload_module(&self, module_name: &str) -> Result<(), String> {
        unsafe {
            let unload_fn: Symbol<UnloadModuleFn> = self.library
                .get(b"kernel_unload_module")
                .map_err(|e| format!("Function not found: {}", e))?;

            let c_name = CString::new(module_name)
                .map_err(|e| format!("Invalid module name: {}", e))?;

            let result = unload_fn(c_name.as_ptr());
            if result == 0 { Ok(()) } else { Err(format!("Unload failed: {}", result)) }
        }
    }

    pub fn get_system_info(&self) -> Result<String, String> {
        unsafe {
            let info_fn: Symbol<GetSysInfoFn> = self.library
                .get(b"kernel_get_sysinfo")
                .map_err(|e| format!("Function not found: {}", e))?;

            let mut result_ptr: *mut c_char = std::ptr::null_mut();
            let status = info_fn(&mut result_ptr);

            if status == 0 && !result_ptr.is_null() {
                let result_str = CStr::from_ptr(result_ptr).to_string_lossy().to_string();
                Ok(result_str)
            } else {
                Err(format!("Get sysinfo failed: {}", status))
            }
        }
    }

    pub fn set_sysctl(&self, param: &str, value: &str) -> Result<(), String> {
        unsafe {
            let sysctl_fn: Symbol<SetSysctlFn> = self.library
                .get(b"kernel_set_sysctl")
                .map_err(|e| format!("Function not found: {}", e))?;

            let c_param = CString::new(param)
                .map_err(|e| format!("Invalid param: {}", e))?;
            let c_value = CString::new(value)
                .map_err(|e| format!("Invalid value: {}", e))?;

            let result = sysctl_fn(c_param.as_ptr(), c_value.as_ptr());
            if result == 0 { Ok(()) } else { Err(format!("Sysctl failed: {}", result)) }
        }
    }
}
