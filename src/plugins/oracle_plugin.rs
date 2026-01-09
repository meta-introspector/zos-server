// Oracle Cloud Plugin Interface
// Plugin for OCI operations via libloading

use crate::traits::LibraryLoader;
use libloading::{Library, Symbol};
use std::ffi::{CString, CStr};
use std::os::raw::{c_char, c_int};

pub struct OraclePlugin {
    library: Library,
    config_path: String,
}

type ListStacksFn = unsafe extern "C" fn(*const c_char, *mut *mut c_char) -> c_int;
type CreateStackFn = unsafe extern "C" fn(*const c_char, *const c_char) -> c_int;
type DestroyStackFn = unsafe extern "C" fn(*const c_char) -> c_int;
type GetStackStatusFn = unsafe extern "C" fn(*const c_char, *mut *mut c_char) -> c_int;

impl OraclePlugin {
    pub fn new(plugin_path: &str, config_path: &str) -> Result<Self, String> {
        let library = unsafe {
            Library::new(plugin_path)
                .map_err(|e| format!("Failed to load Oracle plugin: {}", e))?
        };

        Ok(OraclePlugin {
            library,
            config_path: config_path.to_string(),
        })
    }

    pub fn list_stacks(&self) -> Result<Vec<String>, String> {
        unsafe {
            let list_fn: Symbol<ListStacksFn> = self.library
                .get(b"oci_list_stacks")
                .map_err(|e| format!("Function not found: {}", e))?;
            
            let c_config = CString::new(&self.config_path)
                .map_err(|e| format!("Invalid config path: {}", e))?;
            
            let mut result_ptr: *mut c_char = std::ptr::null_mut();
            let status = list_fn(c_config.as_ptr(), &mut result_ptr);
            
            if status == 0 && !result_ptr.is_null() {
                let result_str = CStr::from_ptr(result_ptr).to_string_lossy();
                Ok(result_str.split(',').map(|s| s.to_string()).collect())
            } else {
                Err(format!("List stacks failed: {}", status))
            }
        }
    }

    pub fn create_stack(&self, stack_name: &str, template: &str) -> Result<(), String> {
        unsafe {
            let create_fn: Symbol<CreateStackFn> = self.library
                .get(b"oci_create_stack")
                .map_err(|e| format!("Function not found: {}", e))?;
            
            let c_name = CString::new(stack_name)
                .map_err(|e| format!("Invalid stack name: {}", e))?;
            let c_template = CString::new(template)
                .map_err(|e| format!("Invalid template: {}", e))?;
            
            let result = create_fn(c_name.as_ptr(), c_template.as_ptr());
            if result == 0 { Ok(()) } else { Err(format!("Create failed: {}", result)) }
        }
    }
}
