// ZOS System Definition - Complete system generated from macros
use crate::core_macros::*;

// Define core languages
mklang!(Bash {
    exec: execute_script,
    compile: compile_script,
    feature: shell_expansion => "bash -c",
    feature: pipe_support => "| & && ||",
});

mklang!(Rust {
    exec: execute_rust,
    compile: compile_rust,
    feature: memory_safety => "ownership + borrowing",
    feature: zero_cost_abstractions => "compile-time optimization",
});

// Define Rust tooling
mkrust!(RustToolchain {
    crate: "serde" => "1.0",
    crate: "tokio" => "1.0",
    crate: "libloading" => "0.8",
    feature: async_runtime,
    feature: macro_system,
    macro: make_plugin => {
        ($name:ident, $path:literal) => {
            pub struct $name {
                lib: libloading::Library,
            }
            impl $name {
                pub fn new() -> Result<Self, String> {
                    let lib = unsafe { 
                        libloading::Library::new($path)
                            .map_err(|e| e.to_string())? 
                    };
                    Ok(Self { lib })
                }
            }
        };
    },
});

// Define build system
mkbuildrs!(ZosBuild {
    step: clean => "cargo clean",
    step: build => "cargo build --release",
    step: test => "cargo test",
    step: install => "cargo install --path .",
    dep: "libssl-dev",
    dep: "libcurl4-openssl-dev",
    env: RUST_LOG => "info",
    env: CARGO_TARGET_DIR => "target",
});

// Define core system libraries
mklib!(PosixLib {
    path: "/lib/x86_64-linux-gnu/libc.so.6",
    fn: system_call(cmd: *const c_char) -> c_int => b"system",
    fn: file_open(path: *const c_char, mode: *const c_char) -> c_int => b"open",
    fn: process_exec(cmd: *const c_char, args: *const *const c_char) -> c_int => b"execv",
});

mklib!(CurlLib {
    path: "/usr/lib/x86_64-linux-gnu/libcurl.so.4",
    fn: http_get(url: *const c_char, buffer: *mut c_char, size: usize) -> c_int => b"curl_easy_perform",
    fn: http_post(url: *const c_char, data: *const c_char) -> c_int => b"curl_easy_setopt",
});

mklib!(SslLib {
    path: "/usr/lib/x86_64-linux-gnu/libssl.so.3",
    fn: ssl_connect(host: *const c_char, port: c_int) -> c_int => b"SSL_connect",
    fn: ssl_verify(cert: *const c_char) -> c_int => b"SSL_verify_certificate",
});

mklib!(GitLib {
    path: "/usr/lib/x86_64-linux-gnu/libgit2.so.1.1",
    fn: git_clone(url: *const c_char, path: *const c_char) -> c_int => b"git_clone",
    fn: git_commit(repo: *const c_char, message: *const c_char) -> c_int => b"git_commit_create",
});

// Define core data structures
mkstruct!(SystemConfig {
    name: String,
    version: String,
    debug: bool,
    max_plugins: usize
});

mkenum!(SystemState {
    Initializing,
    Running,
    Paused,
    Shutdown,
    Error(String)
});

mkenum!(PluginType {
    Core,
    Extra,
    System,
    User(String)
});

// Define core functions
mkfn!(init_system(config: SystemConfig) -> Result<ZosCore, String> {
    println!("🚀 Initializing ZOS System: {}", config.name);
    
    let bash = Bash::new()?;
    let rust = Rust::new()?;
    let toolchain = RustToolchain::new()?;
    let build = ZosBuild::new()?;
    let posix = PosixLib::new("/lib/x86_64-linux-gnu/libc.so.6")?;
    let curl = CurlLib::new("/usr/lib/x86_64-linux-gnu/libcurl.so.4")?;
    let ssl = SslLib::new("/usr/lib/x86_64-linux-gnu/libssl.so.3")?;
    let git = GitLib::new("/usr/lib/x86_64-linux-gnu/libgit2.so.1.1")?;

    Ok(ZosCore {
        config,
        state: SystemState::Initializing,
        bash,
        rust,
        toolchain,
        build,
        posix,
        curl,
        ssl,
        git,
    })
});

mkfn!(self_build(core: &mut ZosCore) -> Result<(), String> {
    println!("🔧 Starting self-build process...");
    core.state = SystemState::Running;
    
    // Use our build system
    core.build.clean()?;
    core.build.build()?;
    core.build.test()?;
    
    println!("✅ Self-build completed successfully!");
    Ok(())
});

// Generate the complete ZOS system
mksys!(ZosCore {
    lang: Bash,
    lang: Rust,
    build: ZosBuild,
    lib: PosixLib => "/lib/x86_64-linux-gnu/libc.so.6",
    lib: CurlLib => "/usr/lib/x86_64-linux-gnu/libcurl.so.4",
    lib: SslLib => "/usr/lib/x86_64-linux-gnu/libssl.so.3",
    lib: GitLib => "/usr/lib/x86_64-linux-gnu/libgit2.so.1.1",
    item: SystemConfig => struct {
        name: String,
        version: String,
        debug: bool,
        max_plugins: usize
    },
    item: SystemState => enum {
        Initializing,
        Running,
        Paused,
        Shutdown,
        Error(String)
    },
});

// Export everything for use
pub use self::{
    ZosCore, SystemConfig, SystemState, PluginType,
    Bash, Rust, RustToolchain, ZosBuild,
    PosixLib, CurlLib, SslLib, GitLib,
    init_system, self_build,
};
