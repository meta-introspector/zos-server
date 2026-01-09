// Solana Sealevel Plugin Interface
use libloading::{Library, Symbol};
use std::ffi::{CString, CStr};
use std::os::raw::{c_char, c_int};

pub struct SolanaPlugin {
    library: Library,
}

type DeployProgramFn = unsafe extern "C" fn(*const c_char, *const c_char) -> c_int;
type CallProgramFn = unsafe extern "C" fn(*const c_char, *const c_char) -> c_int;
type GetBalanceFn = unsafe extern "C" fn(*const c_char) -> c_int;

impl SolanaPlugin {
    pub fn new(plugin_path: &str) -> Result<Self, String> {
        let library = unsafe { Library::new(plugin_path).map_err(|e| e.to_string())? };
        Ok(SolanaPlugin { library })
    }

    pub fn deploy_program(&self, program_path: &str, keypair: &str) -> Result<(), String> {
        unsafe {
            let deploy_fn: Symbol<DeployProgramFn> = self.library.get(b"solana_deploy_program").map_err(|e| e.to_string())?;
            let c_path = CString::new(program_path).map_err(|e| e.to_string())?;
            let c_keypair = CString::new(keypair).map_err(|e| e.to_string())?;
            let result = deploy_fn(c_path.as_ptr(), c_keypair.as_ptr());
            if result == 0 { Ok(()) } else { Err(format!("Deploy failed: {}", result)) }
        }
    }
}
