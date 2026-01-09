// Debug/Trace Plugins - GDB, strace, ptrace, chroot
use libloading::{Library, Symbol};
use std::ffi::{CString, CStr};
use std::os::raw::{c_char, c_int};

pub struct GdbPlugin {
    library: Library,
}

pub struct StracePlugin {
    library: Library,
}

pub struct PtracePlugin {
    library: Library,
}

pub struct ChrootPlugin {
    library: Library,
}

type StartDebugFn = unsafe extern "C" fn(*const c_char, *const c_char) -> c_int;
type AttachProcessFn = unsafe extern "C" fn(c_int) -> c_int;
type TraceSyscallsFn = unsafe extern "C" fn(c_int, *mut *mut c_char) -> c_int;
type CreateChrootFn = unsafe extern "C" fn(*const c_char, *const c_char) -> c_int;

impl GdbPlugin {
    pub fn new(plugin_path: &str) -> Result<Self, String> {
        let library = unsafe { Library::new(plugin_path).map_err(|e| e.to_string())? };
        Ok(GdbPlugin { library })
    }

    pub fn start_debug(&self, binary: &str, args: &str) -> Result<i32, String> {
        unsafe {
            let debug_fn: Symbol<StartDebugFn> = self.library.get(b"gdb_start_debug").map_err(|e| e.to_string())?;
            let c_binary = CString::new(binary).map_err(|e| e.to_string())?;
            let c_args = CString::new(args).map_err(|e| e.to_string())?;
            let result = debug_fn(c_binary.as_ptr(), c_args.as_ptr());
            if result >= 0 { Ok(result) } else { Err(format!("Debug failed: {}", result)) }
        }
    }
}

impl StracePlugin {
    pub fn new(plugin_path: &str) -> Result<Self, String> {
        let library = unsafe { Library::new(plugin_path).map_err(|e| e.to_string())? };
        Ok(StracePlugin { library })
    }

    pub fn trace_syscalls(&self, pid: i32) -> Result<String, String> {
        unsafe {
            let trace_fn: Symbol<TraceSyscallsFn> = self.library.get(b"strace_trace_syscalls").map_err(|e| e.to_string())?;
            let mut result_ptr: *mut c_char = std::ptr::null_mut();
            let status = trace_fn(pid, &mut result_ptr);
            
            if status == 0 && !result_ptr.is_null() {
                let result_str = CStr::from_ptr(result_ptr).to_string_lossy().to_string();
                Ok(result_str)
            } else {
                Err(format!("Trace failed: {}", status))
            }
        }
    }
}

impl PtracePlugin {
    pub fn new(plugin_path: &str) -> Result<Self, String> {
        let library = unsafe { Library::new(plugin_path).map_err(|e| e.to_string())? };
        Ok(PtracePlugin { library })
    }

    pub fn attach_process(&self, pid: i32) -> Result<(), String> {
        unsafe {
            let attach_fn: Symbol<AttachProcessFn> = self.library.get(b"ptrace_attach_process").map_err(|e| e.to_string())?;
            let result = attach_fn(pid);
            if result == 0 { Ok(()) } else { Err(format!("Attach failed: {}", result)) }
        }
    }
}

impl ChrootPlugin {
    pub fn new(plugin_path: &str) -> Result<Self, String> {
        let library = unsafe { Library::new(plugin_path).map_err(|e| e.to_string())? };
        Ok(ChrootPlugin { library })
    }

    pub fn create_chroot(&self, root_path: &str, command: &str) -> Result<(), String> {
        unsafe {
            let chroot_fn: Symbol<CreateChrootFn> = self.library.get(b"chroot_create_chroot").map_err(|e| e.to_string())?;
            let c_root = CString::new(root_path).map_err(|e| e.to_string())?;
            let c_command = CString::new(command).map_err(|e| e.to_string())?;
            let result = chroot_fn(c_root.as_ptr(), c_command.as_ptr());
            if result == 0 { Ok(()) } else { Err(format!("Chroot failed: {}", result)) }
        }
    }
}
