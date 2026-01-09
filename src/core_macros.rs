// Core ZOS Macros - Generate entire system from declarations
// mksys! mklang! mkrust! mkbuildrs! mklib! mkitem! mkenum! mkstruct! mkfn!

/// Generate complete system from high-level declaration
#[macro_export]
macro_rules! mksys {
    ($name:ident {
        $(lang: $lang:ident,)*
        $(build: $build:ident,)*
        $(lib: $lib:ident => $lib_path:literal,)*
        $(item: $item:ident => $item_def:tt,)*
    }) => {
        pub struct $name {
            $(pub $lang: $lang,)*
            $(pub $build: $build,)*
            $(pub $lib: $lib,)*
        }

        impl $name {
            pub fn new() -> Result<Self, String> {
                Ok($name {
                    $($lang: $lang::new()?,)*
                    $($build: $build::new()?,)*
                    $($lib: $lib::new($lib_path)?,)*
                })
            }
        }

        $(mkitem!($item => $item_def);)*
    };
}

/// Generate language runtime
#[macro_export]
macro_rules! mklang {
    ($name:ident {
        exec: $exec_fn:ident,
        compile: $compile_fn:ident,
        $(feature: $feature:ident => $feature_impl:expr,)*
    }) => {
        pub struct $name {
            runtime_path: String,
        }

        impl $name {
            pub fn new() -> Result<Self, String> {
                Ok($name {
                    runtime_path: Self::detect_runtime()?,
                })
            }

            fn detect_runtime() -> Result<String, String> {
                // Auto-detect language runtime
                std::env::var("PATH")
                    .map_err(|_| "PATH not found".to_string())
                    .map(|_| "/usr/bin".to_string()) // Simplified
            }

            pub fn $exec_fn(&self, code: &str) -> Result<String, String> {
                // Execute code in this language
                Ok(format!("Executed: {}", code))
            }

            pub fn $compile_fn(&self, source: &str, output: &str) -> Result<(), String> {
                // Compile source to output
                println!("Compiling {} to {}", source, output);
                Ok(())
            }

            $(
                pub fn $feature(&self) -> String {
                    $feature_impl.to_string()
                }
            )*
        }
    };
}

/// Generate Rust-specific tooling
#[macro_export]
macro_rules! mkrust {
    ($name:ident {
        $(crate: $crate_name:literal => $crate_version:literal,)*
        $(feature: $feature:ident,)*
        $(macro: $macro_name:ident => $macro_body:tt,)*
    }) => {
        pub struct $name {
            toolchain: String,
            $(pub $feature: bool,)*
        }

        impl $name {
            pub fn new() -> Result<Self, String> {
                Ok($name {
                    toolchain: Self::detect_toolchain()?,
                    $($feature: true,)*
                })
            }

            fn detect_toolchain() -> Result<String, String> {
                std::process::Command::new("rustc")
                    .arg("--version")
                    .output()
                    .map_err(|_| "rustc not found".to_string())
                    .map(|output| String::from_utf8_lossy(&output.stdout).to_string())
            }

            pub fn add_dependency(&self, name: &str, version: &str) -> String {
                format!("{} = \"{}\"", name, version)
            }

            $(
                pub fn use_crate_$crate_name(&self) -> String {
                    format!("{} = \"{}\"", $crate_name, $crate_version)
                }
            )*
        }

        $(
            #[macro_export]
            macro_rules! $macro_name $macro_body
        )*
    };
}

/// Generate build system
#[macro_export]
macro_rules! mkbuildrs {
    ($name:ident {
        $(step: $step:ident => $step_cmd:literal,)*
        $(dep: $dep:literal,)*
        $(env: $env_var:ident => $env_val:literal,)*
    }) => {
        pub struct $name {
            build_dir: String,
            $(pub $env_var: String,)*
        }

        impl $name {
            pub fn new() -> Result<Self, String> {
                Ok($name {
                    build_dir: std::env::current_dir()
                        .map_err(|_| "Cannot get current dir".to_string())?
                        .to_string_lossy()
                        .to_string(),
                    $($env_var: $env_val.to_string(),)*
                })
            }

            $(
                pub fn $step(&self) -> Result<(), String> {
                    println!("Running build step: {}", $step_cmd);
                    std::process::Command::new("sh")
                        .arg("-c")
                        .arg($step_cmd)
                        .current_dir(&self.build_dir)
                        .status()
                        .map_err(|e| format!("Build step failed: {}", e))?;
                    Ok(())
                }
            )*

            pub fn build_all(&self) -> Result<(), String> {
                $(self.$step()?;)*
                Ok(())
            }
        }
    };
}

/// Generate library interface
#[macro_export]
macro_rules! mklib {
    ($name:ident {
        path: $lib_path:literal,
        $(fn: $fn_name:ident($($arg:ident: $arg_type:ty),*) -> $ret_type:ty => $c_fn:literal,)*
    }) => {
        use libloading::{Library, Symbol};
        use std::ffi::{CString, CStr};
        use std::os::raw::{c_char, c_int};

        $(
            type $fn_name = unsafe extern "C" fn($($arg_type),*) -> $ret_type;
        )*

        pub struct $name {
            library: Library,
        }

        impl $name {
            pub fn new(path: &str) -> Result<Self, String> {
                let library = unsafe { 
                    Library::new(path).map_err(|e| e.to_string())? 
                };
                Ok($name { library })
            }

            $(
                pub fn $fn_name(&self, $($arg: $arg_type),*) -> Result<$ret_type, String> {
                    unsafe {
                        let func: Symbol<$fn_name> = self.library
                            .get($c_fn.as_bytes())
                            .map_err(|e| e.to_string())?;
                        let result = func($($arg),*);
                        Ok(result)
                    }
                }
            )*
        }
    };
}

/// Generate any item (enum, struct, fn, etc.)
#[macro_export]
macro_rules! mkitem {
    (enum $name:ident => {$($variant:ident$(($($field:ty),*))?),*}) => {
        mkenum!($name { $($variant$(($($field),*))?,)* });
    };
    (struct $name:ident => {$($field:ident: $field_type:ty),*}) => {
        mkstruct!($name { $($field: $field_type,)* });
    };
    (fn $name:ident => ($($arg:ident: $arg_type:ty),*) -> $ret:ty $body:block) => {
        mkfn!($name($($arg: $arg_type),*) -> $ret $body);
    };
}

/// Generate enum
#[macro_export]
macro_rules! mkenum {
    ($name:ident {
        $($variant:ident$(($($field:ty),*))?),*
    }) => {
        #[derive(Debug, Clone, PartialEq)]
        pub enum $name {
            $($variant$(($($field),*))?),*
        }

        impl $name {
            pub fn variant_name(&self) -> &'static str {
                match self {
                    $(Self::$variant$((..))? => stringify!($variant),)*
                }
            }
        }
    };
}

/// Generate struct
#[macro_export]
macro_rules! mkstruct {
    ($name:ident {
        $($field:ident: $field_type:ty),*
    }) => {
        #[derive(Debug, Clone)]
        pub struct $name {
            $(pub $field: $field_type,)*
        }

        impl $name {
            pub fn new($($field: $field_type),*) -> Self {
                Self { $($field),* }
            }
        }
    };
}

/// Generate function
#[macro_export]
macro_rules! mkfn {
    ($name:ident($($arg:ident: $arg_type:ty),*) -> $ret:ty $body:block) => {
        pub fn $name($($arg: $arg_type),*) -> $ret $body
    };
}
