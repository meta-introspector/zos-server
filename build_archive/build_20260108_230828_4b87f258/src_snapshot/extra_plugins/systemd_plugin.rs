// SystemD Plugin Interface
// Plugin for systemd service management via libloading

use crate::traits::LibraryLoader;
use libloading::{Library, Symbol};
use std::collections::HashMap;
use std::ffi::{CString, CStr};
use std::os::raw::{c_char, c_int};

pub struct SystemDPlugin {
    library: Library,
    services: HashMap<String, String>,
}

type StartServiceFn = unsafe extern "C" fn(*const c_char) -> c_int;
type StopServiceFn = unsafe extern "C" fn(*const c_char) -> c_int;
type StatusServiceFn = unsafe extern "C" fn(*const c_char) -> c_int;
type EnableServiceFn = unsafe extern "C" fn(*const c_char) -> c_int;

impl SystemDPlugin {
    pub fn new(plugin_path: &str) -> Result<Self, String> {
        let library = unsafe {
            Library::new(plugin_path)
                .map_err(|e| format!("Failed to load systemd plugin: {}", e))?
        };

        Ok(SystemDPlugin {
            library,
            services: HashMap::new(),
        })
    }

    pub fn start_service(&self, service_name: &str) -> Result<(), String> {
        unsafe {
            let start_fn: Symbol<StartServiceFn> = self.library
                .get(b"systemd_start_service")
                .map_err(|e| format!("Function not found: {}", e))?;

            let c_service = CString::new(service_name)
                .map_err(|e| format!("Invalid service name: {}", e))?;

            let result = start_fn(c_service.as_ptr());
            if result == 0 { Ok(()) } else { Err(format!("Start failed: {}", result)) }
        }
    }

    pub fn stop_service(&self, service_name: &str) -> Result<(), String> {
        unsafe {
            let stop_fn: Symbol<StopServiceFn> = self.library
                .get(b"systemd_stop_service")
                .map_err(|e| format!("Function not found: {}", e))?;

            let c_service = CString::new(service_name)
                .map_err(|e| format!("Invalid service name: {}", e))?;

            let result = stop_fn(c_service.as_ptr());
            if result == 0 { Ok(()) } else { Err(format!("Stop failed: {}", result)) }
        }
    }
}
