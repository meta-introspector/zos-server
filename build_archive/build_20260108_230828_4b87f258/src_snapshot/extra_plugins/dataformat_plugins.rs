// Data Format Plugins - Second Layer
// Each export format becomes its own plugin with full CRUD operations

use libloading::{Library, Symbol};
use std::ffi::{CString, CStr};
use std::os::raw::{c_char, c_int};

// Parquet Plugin
pub struct ParquetPlugin {
    library: Library,
}

type WriteParquetFn = unsafe extern "C" fn(*const c_char, *const c_char) -> c_int;
type ReadParquetFn = unsafe extern "C" fn(*const c_char, *mut *mut c_char) -> c_int;
type QueryParquetFn = unsafe extern "C" fn(*const c_char, *const c_char, *mut *mut c_char) -> c_int;

impl ParquetPlugin {
    pub fn new(plugin_path: &str) -> Result<Self, String> {
        let library = unsafe { Library::new(plugin_path).map_err(|e| e.to_string())? };
        Ok(ParquetPlugin { library })
    }

    pub fn write_data(&self, path: &str, data: &str) -> Result<(), String> {
        unsafe {
            let write_fn: Symbol<WriteParquetFn> = self.library.get(b"parquet_write_data").map_err(|e| e.to_string())?;
            let c_path = CString::new(path).map_err(|e| e.to_string())?;
            let c_data = CString::new(data).map_err(|e| e.to_string())?;
            let result = write_fn(c_path.as_ptr(), c_data.as_ptr());
            if result == 0 { Ok(()) } else { Err(format!("Write failed: {}", result)) }
        }
    }

    pub fn query_data(&self, path: &str, query: &str) -> Result<String, String> {
        unsafe {
            let query_fn: Symbol<QueryParquetFn> = self.library.get(b"parquet_query_data").map_err(|e| e.to_string())?;
            let c_path = CString::new(path).map_err(|e| e.to_string())?;
            let c_query = CString::new(query).map_err(|e| e.to_string())?;
            let mut result_ptr: *mut c_char = std::ptr::null_mut();
            let status = query_fn(c_path.as_ptr(), c_query.as_ptr(), &mut result_ptr);

            if status == 0 && !result_ptr.is_null() {
                let result_str = CStr::from_ptr(result_ptr).to_string_lossy().to_string();
                Ok(result_str)
            } else {
                Err(format!("Query failed: {}", status))
            }
        }
    }
}

// HuggingFace Plugin
pub struct HuggingFacePlugin {
    library: Library,
}

type UploadDatasetFn = unsafe extern "C" fn(*const c_char, *const c_char) -> c_int;
type DownloadDatasetFn = unsafe extern "C" fn(*const c_char, *const c_char) -> c_int;

impl HuggingFacePlugin {
    pub fn new(plugin_path: &str) -> Result<Self, String> {
        let library = unsafe { Library::new(plugin_path).map_err(|e| e.to_string())? };
        Ok(HuggingFacePlugin { library })
    }

    pub fn upload_dataset(&self, dataset_name: &str, data: &str) -> Result<(), String> {
        unsafe {
            let upload_fn: Symbol<UploadDatasetFn> = self.library.get(b"hf_upload_dataset").map_err(|e| e.to_string())?;
            let c_name = CString::new(dataset_name).map_err(|e| e.to_string())?;
            let c_data = CString::new(data).map_err(|e| e.to_string())?;
            let result = upload_fn(c_name.as_ptr(), c_data.as_ptr());
            if result == 0 { Ok(()) } else { Err(format!("Upload failed: {}", result)) }
        }
    }
}

// RDF/SPARQL Plugin
pub struct RdfPlugin {
    library: Library,
}

type LoadRdfFn = unsafe extern "C" fn(*const c_char, *const c_char) -> c_int;
type SparqlQueryFn = unsafe extern "C" fn(*const c_char, *mut *mut c_char) -> c_int;

impl RdfPlugin {
    pub fn new(plugin_path: &str) -> Result<Self, String> {
        let library = unsafe { Library::new(plugin_path).map_err(|e| e.to_string())? };
        Ok(RdfPlugin { library })
    }

    pub fn sparql_query(&self, query: &str) -> Result<String, String> {
        unsafe {
            let sparql_fn: Symbol<SparqlQueryFn> = self.library.get(b"rdf_sparql_query").map_err(|e| e.to_string())?;
            let c_query = CString::new(query).map_err(|e| e.to_string())?;
            let mut result_ptr: *mut c_char = std::ptr::null_mut();
            let status = sparql_fn(c_query.as_ptr(), &mut result_ptr);

            if status == 0 && !result_ptr.is_null() {
                let result_str = CStr::from_ptr(result_ptr).to_string_lossy().to_string();
                Ok(result_str)
            } else {
                Err(format!("SPARQL failed: {}", status))
            }
        }
    }
}

// SQL Plugin
pub struct SqlPlugin {
    library: Library,
}

type ExecuteSqlFn = unsafe extern "C" fn(*const c_char, *mut *mut c_char) -> c_int;

impl SqlPlugin {
    pub fn new(plugin_path: &str) -> Result<Self, String> {
        let library = unsafe { Library::new(plugin_path).map_err(|e| e.to_string())? };
        Ok(SqlPlugin { library })
    }

    pub fn execute_query(&self, query: &str) -> Result<String, String> {
        unsafe {
            let exec_fn: Symbol<ExecuteSqlFn> = self.library.get(b"sql_execute_query").map_err(|e| e.to_string())?;
            let c_query = CString::new(query).map_err(|e| e.to_string())?;
            let mut result_ptr: *mut c_char = std::ptr::null_mut();
            let status = exec_fn(c_query.as_ptr(), &mut result_ptr);

            if status == 0 && !result_ptr.is_null() {
                let result_str = CStr::from_ptr(result_ptr).to_string_lossy().to_string();
                Ok(result_str)
            } else {
                Err(format!("SQL failed: {}", status))
            }
        }
    }
}
