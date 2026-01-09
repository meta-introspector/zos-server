// Docker Plugin Interface
// Plugin for Docker operations via libloading

use crate::traits::LibraryLoader;
use libloading::{Library, Symbol};
use std::ffi::{CString, CStr};
use std::os::raw::{c_char, c_int};

pub struct DockerPlugin {
    library: Library,
}

type RunContainerFn = unsafe extern "C" fn(*const c_char, *const c_char) -> c_int;
type StopContainerFn = unsafe extern "C" fn(*const c_char) -> c_int;
type ListContainersFn = unsafe extern "C" fn(*mut *mut c_char) -> c_int;
type BuildImageFn = unsafe extern "C" fn(*const c_char, *const c_char) -> c_int;

impl DockerPlugin {
    pub fn new(plugin_path: &str) -> Result<Self, String> {
        let library = unsafe {
            Library::new(plugin_path)
                .map_err(|e| format!("Failed to load Docker plugin: {}", e))?
        };

        Ok(DockerPlugin { library })
    }

    pub fn run_container(&self, image: &str, args: &str) -> Result<String, String> {
        unsafe {
            let run_fn: Symbol<RunContainerFn> = self.library
                .get(b"docker_run_container")
                .map_err(|e| format!("Function not found: {}", e))?;

            let c_image = CString::new(image)
                .map_err(|e| format!("Invalid image: {}", e))?;
            let c_args = CString::new(args)
                .map_err(|e| format!("Invalid args: {}", e))?;

            let result = run_fn(c_image.as_ptr(), c_args.as_ptr());
            if result >= 0 {
                Ok(format!("container_{}", result))
            } else {
                Err(format!("Run failed: {}", result))
            }
        }
    }

    pub fn stop_container(&self, container_id: &str) -> Result<(), String> {
        unsafe {
            let stop_fn: Symbol<StopContainerFn> = self.library
                .get(b"docker_stop_container")
                .map_err(|e| format!("Function not found: {}", e))?;

            let c_container = CString::new(container_id)
                .map_err(|e| format!("Invalid container ID: {}", e))?;

            let result = stop_fn(c_container.as_ptr());
            if result == 0 { Ok(()) } else { Err(format!("Stop failed: {}", result)) }
        }
    }

    pub fn build_image(&self, dockerfile_path: &str, tag: &str) -> Result<(), String> {
        unsafe {
            let build_fn: Symbol<BuildImageFn> = self.library
                .get(b"docker_build_image")
                .map_err(|e| format!("Function not found: {}", e))?;

            let c_path = CString::new(dockerfile_path)
                .map_err(|e| format!("Invalid path: {}", e))?;
            let c_tag = CString::new(tag)
                .map_err(|e| format!("Invalid tag: {}", e))?;

            let result = build_fn(c_path.as_ptr(), c_tag.as_ptr());
            if result == 0 { Ok(()) } else { Err(format!("Build failed: {}", result)) }
        }
    }
}
