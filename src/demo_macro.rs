/// Demo marker macro - flags code that needs to be made production-ready
/// Usage: demo!("this authentication is hardcoded")
#[macro_export]
macro_rules! demo {
    ($msg:expr) => {
        #[cfg(debug_assertions)]
        eprintln!("DEMO CODE: {} at {}:{}", $msg, file!(), line!());

        #[cfg(not(debug_assertions))]
        compile_error!(concat!("Demo code found in release build: ", $msg));
    };
}

/// Production-ready assertion - opposite of demo!()
#[macro_export]
macro_rules! production {
    () => {
        // This code is production-ready
    };
}
