// Telemetry Plugin Interface
use libloading::{Library, Symbol};
use std::ffi::{CString, CStr};
use std::os::raw::{c_char, c_int};

pub struct TelemetryPlugin {
    library: Library,
}

type StartMetricsFn = unsafe extern "C" fn(*const c_char) -> c_int;
type RecordMetricFn = unsafe extern "C" fn(*const c_char, c_int) -> c_int;
type GetMetricsFn = unsafe extern "C" fn(*mut *mut c_char) -> c_int;

impl TelemetryPlugin {
    pub fn new(plugin_path: &str) -> Result<Self, String> {
        let library = unsafe { Library::new(plugin_path).map_err(|e| e.to_string())? };
        Ok(TelemetryPlugin { library })
    }

    pub fn start_metrics(&self, endpoint: &str) -> Result<(), String> {
        unsafe {
            let start_fn: Symbol<StartMetricsFn> = self.library.get(b"telemetry_start_metrics").map_err(|e| e.to_string())?;
            let c_endpoint = CString::new(endpoint).map_err(|e| e.to_string())?;
            let result = start_fn(c_endpoint.as_ptr());
            if result == 0 { Ok(()) } else { Err(format!("Start failed: {}", result)) }
        }
    }

    pub fn record_metric(&self, name: &str, value: i32) -> Result<(), String> {
        unsafe {
            let record_fn: Symbol<RecordMetricFn> = self.library.get(b"telemetry_record_metric").map_err(|e| e.to_string())?;
            let c_name = CString::new(name).map_err(|e| e.to_string())?;
            let result = record_fn(c_name.as_ptr(), value);
            if result == 0 { Ok(()) } else { Err(format!("Record failed: {}", result)) }
        }
    }
}
