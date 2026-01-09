// ZOS Oracle Plugin - Standalone Oracle Cloud integration
// AGPL-3.0 License

pub mod wallet_auth;
pub mod libp2p_verbs;
pub mod plugin_loader;
pub mod bootstrap_engine;
pub mod dev_workflow;
pub mod user_fingerprint;
pub mod block_port_manager;
pub mod ranking_system;
pub mod user_dashboard;
pub mod ai_marketplace;

pub use wallet_auth::*;

// C-compatible plugin interface
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};

/// C-compatible plugin entry point
#[no_mangle]
pub extern "C" fn zos_oracle_init() -> c_int {
    println!("🔧 ZOS Oracle Plugin initialized");
    0
}

/// List Oracle Cloud stacks via CLI
#[no_mangle]
pub extern "C" fn zos_oracle_list_stacks(compartment_id: *const c_char) -> *mut c_char {
    if compartment_id.is_null() {
        return std::ptr::null_mut();
    }

    let compartment = unsafe { CStr::from_ptr(compartment_id) };
    let compartment_str = compartment.to_string_lossy();

    // Call OCI CLI instead of compiling crypto dependencies
    let output = std::process::Command::new("oci")
        .args(&[
            "resource-manager", "stack", "list",
            "--compartment-id", &compartment_str,
            "--lifecycle-state", "ACTIVE",
            "--output", "json"
        ])
        .output();

    match output {
        Ok(result) => {
            let json_str = String::from_utf8_lossy(&result.stdout);
            match CString::new(json_str.as_ref()) {
                Ok(c_str) => c_str.into_raw(),
                Err(_) => std::ptr::null_mut(),
            }
        }
        Err(_) => std::ptr::null_mut(),
    }
}

/// Deploy stack via CLI
#[no_mangle]
pub extern "C" fn zos_oracle_deploy_stack(stack_path: *const c_char) -> c_int {
    if stack_path.is_null() {
        return -1;
    }

    let path = unsafe { CStr::from_ptr(stack_path) };
    let path_str = path.to_string_lossy();

    // Use OCI CLI for deployment
    let result = std::process::Command::new("oci")
        .args(&[
            "resource-manager", "stack", "create-from-zip-file",
            "--config-source", &path_str,
            "--compartment-id", "ocid1.tenancy.oc1..aaaaaaaapxfkcjaczqslvnbekbqq2eefxgwx7kqbakvddhzaaiym62vmt5la"
        ])
        .status();

    match result {
        Ok(status) => if status.success() { 0 } else { -1 },
        Err(_) => -1,
    }
}

/// Bootstrap Oracle instance with ZOS
pub fn bootstrap_oracle_instance() -> Result<String, String> {
    println!("🚀 Bootstrapping Oracle Cloud instance for ZOS...");

    // Use terraform CLI for deployment
    let result = std::process::Command::new("terraform")
        .args(&["apply", "-auto-approve"])
        .current_dir("~/terraform/accounts/solfunmeme-oci")
        .status();

    match result {
        Ok(status) => {
            if status.success() {
                Ok("Oracle instance deployed successfully".to_string())
            } else {
                Err("Terraform apply failed".to_string())
            }
        }
        Err(e) => Err(format!("Failed to run terraform: {}", e)),
    }
}
