// Storage Plugins - IPFS, S3, SFTP
use libloading::{Library, Symbol};
use std::ffi::{CString, CStr};
use std::os::raw::{c_char, c_int};

pub struct IpfsPlugin {
    library: Library,
}

pub struct S3Plugin {
    library: Library,
}

pub struct SftpPlugin {
    library: Library,
}

type AddFileFn = unsafe extern "C" fn(*const c_char) -> c_int;
type GetFileFn = unsafe extern "C" fn(*const c_char, *const c_char) -> c_int;
type UploadFileFn = unsafe extern "C" fn(*const c_char, *const c_char) -> c_int;

impl IpfsPlugin {
    pub fn new(plugin_path: &str) -> Result<Self, String> {
        let library = unsafe { Library::new(plugin_path).map_err(|e| e.to_string())? };
        Ok(IpfsPlugin { library })
    }

    pub fn add_file(&self, file_path: &str) -> Result<String, String> {
        unsafe {
            let add_fn: Symbol<AddFileFn> = self.library.get(b"ipfs_add_file").map_err(|e| e.to_string())?;
            let c_path = CString::new(file_path).map_err(|e| e.to_string())?;
            let result = add_fn(c_path.as_ptr());
            if result >= 0 { Ok(format!("Qm{}", result)) } else { Err(format!("Add failed: {}", result)) }
        }
    }
}

impl S3Plugin {
    pub fn new(plugin_path: &str) -> Result<Self, String> {
        let library = unsafe { Library::new(plugin_path).map_err(|e| e.to_string())? };
        Ok(S3Plugin { library })
    }

    pub fn upload_file(&self, local_path: &str, s3_key: &str) -> Result<(), String> {
        unsafe {
            let upload_fn: Symbol<UploadFileFn> = self.library.get(b"s3_upload_file").map_err(|e| e.to_string())?;
            let c_local = CString::new(local_path).map_err(|e| e.to_string())?;
            let c_key = CString::new(s3_key).map_err(|e| e.to_string())?;
            let result = upload_fn(c_local.as_ptr(), c_key.as_ptr());
            if result == 0 { Ok(()) } else { Err(format!("Upload failed: {}", result)) }
        }
    }
}

impl SftpPlugin {
    pub fn new(plugin_path: &str) -> Result<Self, String> {
        let library = unsafe { Library::new(plugin_path).map_err(|e| e.to_string())? };
        Ok(SftpPlugin { library })
    }

    pub fn upload_file(&self, local_path: &str, remote_path: &str) -> Result<(), String> {
        unsafe {
            let upload_fn: Symbol<UploadFileFn> = self.library.get(b"sftp_upload_file").map_err(|e| e.to_string())?;
            let c_local = CString::new(local_path).map_err(|e| e.to_string())?;
            let c_remote = CString::new(remote_path).map_err(|e| e.to_string())?;
            let result = upload_fn(c_local.as_ptr(), c_remote.as_ptr());
            if result == 0 { Ok(()) } else { Err(format!("Upload failed: {}", result)) }
        }
    }
}
