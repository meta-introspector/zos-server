// Protocol Format Plugins - Second Layer
use libloading::{Library, Symbol};
use std::ffi::{CString, CStr};
use std::os::raw::{c_char, c_int};

// MCP Plugin
pub struct McpPlugin {
    library: Library,
}

type McpCallFn = unsafe extern "C" fn(*const c_char, *const c_char, *mut *mut c_char) -> c_int;
type McpRegisterFn = unsafe extern "C" fn(*const c_char, *const c_char) -> c_int;

impl McpPlugin {
    pub fn new(plugin_path: &str) -> Result<Self, String> {
        let library = unsafe { Library::new(plugin_path).map_err(|e| e.to_string())? };
        Ok(McpPlugin { library })
    }

    pub fn call_tool(&self, tool_name: &str, args: &str) -> Result<String, String> {
        unsafe {
            let call_fn: Symbol<McpCallFn> = self.library.get(b"mcp_call_tool").map_err(|e| e.to_string())?;
            let c_tool = CString::new(tool_name).map_err(|e| e.to_string())?;
            let c_args = CString::new(args).map_err(|e| e.to_string())?;
            let mut result_ptr: *mut c_char = std::ptr::null_mut();
            let status = call_fn(c_tool.as_ptr(), c_args.as_ptr(), &mut result_ptr);
            
            if status == 0 && !result_ptr.is_null() {
                let result_str = CStr::from_ptr(result_ptr).to_string_lossy().to_string();
                Ok(result_str)
            } else {
                Err(format!("MCP call failed: {}", status))
            }
        }
    }
}

// OpenAPI Plugin
pub struct OpenApiPlugin {
    library: Library,
}

type RestCallFn = unsafe extern "C" fn(*const c_char, *const c_char, *const c_char, *mut *mut c_char) -> c_int;

impl OpenApiPlugin {
    pub fn new(plugin_path: &str) -> Result<Self, String> {
        let library = unsafe { Library::new(plugin_path).map_err(|e| e.to_string())? };
        Ok(OpenApiPlugin { library })
    }

    pub fn rest_call(&self, method: &str, endpoint: &str, body: &str) -> Result<String, String> {
        unsafe {
            let call_fn: Symbol<RestCallFn> = self.library.get(b"openapi_rest_call").map_err(|e| e.to_string())?;
            let c_method = CString::new(method).map_err(|e| e.to_string())?;
            let c_endpoint = CString::new(endpoint).map_err(|e| e.to_string())?;
            let c_body = CString::new(body).map_err(|e| e.to_string())?;
            let mut result_ptr: *mut c_char = std::ptr::null_mut();
            let status = call_fn(c_method.as_ptr(), c_endpoint.as_ptr(), c_body.as_ptr(), &mut result_ptr);
            
            if status == 0 && !result_ptr.is_null() {
                let result_str = CStr::from_ptr(result_ptr).to_string_lossy().to_string();
                Ok(result_str)
            } else {
                Err(format!("REST call failed: {}", status))
            }
        }
    }
}

// SOAP Plugin
pub struct SoapPlugin {
    library: Library,
}

type SoapCallFn = unsafe extern "C" fn(*const c_char, *const c_char, *mut *mut c_char) -> c_int;

impl SoapPlugin {
    pub fn new(plugin_path: &str) -> Result<Self, String> {
        let library = unsafe { Library::new(plugin_path).map_err(|e| e.to_string())? };
        Ok(SoapPlugin { library })
    }

    pub fn soap_call(&self, operation: &str, envelope: &str) -> Result<String, String> {
        unsafe {
            let call_fn: Symbol<SoapCallFn> = self.library.get(b"soap_call_operation").map_err(|e| e.to_string())?;
            let c_operation = CString::new(operation).map_err(|e| e.to_string())?;
            let c_envelope = CString::new(envelope).map_err(|e| e.to_string())?;
            let mut result_ptr: *mut c_char = std::ptr::null_mut();
            let status = call_fn(c_operation.as_ptr(), c_envelope.as_ptr(), &mut result_ptr);
            
            if status == 0 && !result_ptr.is_null() {
                let result_str = CStr::from_ptr(result_ptr).to_string_lossy().to_string();
                Ok(result_str)
            } else {
                Err(format!("SOAP call failed: {}", status))
            }
        }
    }
}
