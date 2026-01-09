// Plugin ABI Template - every plugin crate must implement these functions

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

// Plugin metadata that gets compiled into the SO
#[no_mangle]
pub static PLUGIN_NAME: &str = "example_plugin";
#[no_mangle]
pub static PLUGIN_VERSION: &str = "1.0.0";

// Required ABI functions
#[no_mangle]
pub extern "C" fn plugin_init() -> i32 {
    println!("Initializing plugin: {}", PLUGIN_NAME);
    0 // Success
}

#[no_mangle]
pub extern "C" fn plugin_get_services() -> *const c_char {
    let services = vec![
        "process_data".to_string(),
        "validate_input".to_string(),
        "generate_report".to_string(),
    ];

    let services_json = serde_json::to_string(&services).unwrap();
    let c_string = CString::new(services_json).unwrap();
    c_string.into_raw()
}

#[no_mangle]
pub extern "C" fn plugin_call_service(
    input_ptr: *const u8,
    input_len: usize,
    output_ptr: *mut u8,
    output_len: *mut usize,
) -> i32 {
    let input_slice = unsafe { std::slice::from_raw_parts(input_ptr, input_len) };
    let input_str = match std::str::from_utf8(input_slice) {
        Ok(s) => s,
        Err(_) => return 1, // Error
    };

    let call_data: serde_json::Value = match serde_json::from_str(input_str) {
        Ok(data) => data,
        Err(_) => return 1, // Error
    };

    let service_name = call_data["service"].as_str().unwrap_or("");
    let input_data = call_data["input"].as_str().unwrap_or("");

    // Decode base64 input
    let decoded_input = match base64::decode(input_data) {
        Ok(data) => data,
        Err(_) => return 1, // Error
    };

    // Route to appropriate service function
    let result = match service_name {
        "process_data" => process_data(&decoded_input),
        "validate_input" => validate_input(&decoded_input),
        "generate_report" => generate_report(&decoded_input),
        _ => return 1, // Unknown service
    };

    let result_bytes = match result {
        Ok(data) => data,
        Err(_) => return 1, // Error
    };

    // Copy result to output buffer
    let copy_len = std::cmp::min(result_bytes.len(), unsafe { *output_len });
    unsafe {
        std::ptr::copy_nonoverlapping(result_bytes.as_ptr(), output_ptr, copy_len);
        *output_len = copy_len;
    }

    0 // Success
}

// Plugin service implementations
fn process_data(input: &[u8]) -> Result<Vec<u8>, String> {
    // Example: reverse the input bytes
    let mut result = input.to_vec();
    result.reverse();
    Ok(result)
}

fn validate_input(input: &[u8]) -> Result<Vec<u8>, String> {
    // Example: check if input is valid UTF-8
    let is_valid = std::str::from_utf8(input).is_ok();
    Ok(vec![if is_valid { 1 } else { 0 }])
}

fn generate_report(input: &[u8]) -> Result<Vec<u8>, String> {
    // Example: generate JSON report
    let report = serde_json::json!({
        "input_size": input.len(),
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "checksum": format!("{:x}", md5::compute(input))
    });

    Ok(report.to_string().into_bytes())
}

// Cleanup function (optional)
#[no_mangle]
pub extern "C" fn plugin_cleanup() {
    println!("Cleaning up plugin: {}", PLUGIN_NAME);
}
