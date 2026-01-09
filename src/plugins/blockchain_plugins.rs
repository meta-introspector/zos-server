// Blockchain Plugins - Ethereum, Bitcoin
use libloading::{Library, Symbol};
use std::ffi::{CString, CStr};
use std::os::raw::{c_char, c_int};

pub struct EthereumPlugin {
    library: Library,
}

pub struct BitcoinPlugin {
    library: Library,
}

type DeployContractFn = unsafe extern "C" fn(*const c_char, *const c_char) -> c_int;
type CallContractFn = unsafe extern "C" fn(*const c_char, *const c_char) -> c_int;
type SendTransactionFn = unsafe extern "C" fn(*const c_char, *const c_char, c_int) -> c_int;

impl EthereumPlugin {
    pub fn new(plugin_path: &str) -> Result<Self, String> {
        let library = unsafe { Library::new(plugin_path).map_err(|e| e.to_string())? };
        Ok(EthereumPlugin { library })
    }

    pub fn deploy_contract(&self, bytecode: &str, abi: &str) -> Result<String, String> {
        unsafe {
            let deploy_fn: Symbol<DeployContractFn> = self.library.get(b"eth_deploy_contract").map_err(|e| e.to_string())?;
            let c_bytecode = CString::new(bytecode).map_err(|e| e.to_string())?;
            let c_abi = CString::new(abi).map_err(|e| e.to_string())?;
            let result = deploy_fn(c_bytecode.as_ptr(), c_abi.as_ptr());
            if result >= 0 { Ok(format!("0x{:x}", result)) } else { Err(format!("Deploy failed: {}", result)) }
        }
    }

    pub fn call_contract(&self, address: &str, data: &str) -> Result<String, String> {
        unsafe {
            let call_fn: Symbol<CallContractFn> = self.library.get(b"eth_call_contract").map_err(|e| e.to_string())?;
            let c_address = CString::new(address).map_err(|e| e.to_string())?;
            let c_data = CString::new(data).map_err(|e| e.to_string())?;
            let result = call_fn(c_address.as_ptr(), c_data.as_ptr());
            if result >= 0 { Ok(format!("0x{:x}", result)) } else { Err(format!("Call failed: {}", result)) }
        }
    }
}

impl BitcoinPlugin {
    pub fn new(plugin_path: &str) -> Result<Self, String> {
        let library = unsafe { Library::new(plugin_path).map_err(|e| e.to_string())? };
        Ok(BitcoinPlugin { library })
    }

    pub fn send_transaction(&self, from: &str, to: &str, amount: i32) -> Result<String, String> {
        unsafe {
            let send_fn: Symbol<SendTransactionFn> = self.library.get(b"btc_send_transaction").map_err(|e| e.to_string())?;
            let c_from = CString::new(from).map_err(|e| e.to_string())?;
            let c_to = CString::new(to).map_err(|e| e.to_string())?;
            let result = send_fn(c_from.as_ptr(), c_to.as_ptr(), amount);
            if result >= 0 { Ok(format!("txid_{:x}", result)) } else { Err(format!("Send failed: {}", result)) }
        }
    }
}
