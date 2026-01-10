use libloading::{Library, Symbol};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};

/// Generic FFI plugin wrapper that handles common plugin loading patterns
pub struct FfiPlugin {
    library: Library,
}

impl FfiPlugin {
    pub fn new(plugin_path: &str) -> Result<Self, String> {
        let library = unsafe { Library::new(plugin_path).map_err(|e| e.to_string())? };
        Ok(FfiPlugin { library })
    }

    /// Call a C function with no parameters that returns an int
    pub fn call_int_fn(&self, symbol_name: &[u8]) -> Result<i32, String> {
        unsafe {
            type IntFn = unsafe extern "C" fn() -> c_int;
            let func: Symbol<IntFn> = self.library.get(symbol_name).map_err(|e| e.to_string())?;
            Ok(func())
        }
    }

    /// Call a C function with one string parameter that returns an int
    pub fn call_string_int_fn(&self, symbol_name: &[u8], param: &str) -> Result<i32, String> {
        unsafe {
            type StringIntFn = unsafe extern "C" fn(*const c_char) -> c_int;
            let func: Symbol<StringIntFn> =
                self.library.get(symbol_name).map_err(|e| e.to_string())?;
            let c_param = CString::new(param).map_err(|e| e.to_string())?;
            let result = func(c_param.as_ptr());
            if result >= 0 {
                Ok(result)
            } else {
                Err(format!("Function failed: {}", result))
            }
        }
    }

    /// Call a C function with two string parameters that returns an int
    pub fn call_two_string_int_fn(
        &self,
        symbol_name: &[u8],
        param1: &str,
        param2: &str,
    ) -> Result<i32, String> {
        unsafe {
            type TwoStringIntFn = unsafe extern "C" fn(*const c_char, *const c_char) -> c_int;
            let func: Symbol<TwoStringIntFn> =
                self.library.get(symbol_name).map_err(|e| e.to_string())?;
            let c_param1 = CString::new(param1).map_err(|e| e.to_string())?;
            let c_param2 = CString::new(param2).map_err(|e| e.to_string())?;
            let result = func(c_param1.as_ptr(), c_param2.as_ptr());
            if result >= 0 {
                Ok(result)
            } else {
                Err(format!("Function failed: {}", result))
            }
        }
    }

    /// Call a C function that returns a string
    pub fn call_string_fn(&self, symbol_name: &[u8], param: &str) -> Result<String, String> {
        unsafe {
            type StringStringFn = unsafe extern "C" fn(*const c_char, *mut *mut c_char) -> c_int;
            let func: Symbol<StringStringFn> =
                self.library.get(symbol_name).map_err(|e| e.to_string())?;
            let c_param = CString::new(param).map_err(|e| e.to_string())?;
            let mut result_ptr: *mut c_char = std::ptr::null_mut();
            let status = func(c_param.as_ptr(), &mut result_ptr);

            if status >= 0 && !result_ptr.is_null() {
                let c_str = CStr::from_ptr(result_ptr);
                let result = c_str.to_string_lossy().to_string();
                // Note: In real implementation, the C library should handle memory management
                // This is a simplified version
                Ok(result)
            } else {
                Err(format!("Function failed: {}", status))
            }
        }
    }
}
