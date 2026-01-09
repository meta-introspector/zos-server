// Core system plugins only - POSIX, bash, cargo, rust, ssh, curl, ssl, regex, git
pub mod core_plugins;

// Optional layer 2 plugins - can be disabled for minimal builds
#[cfg(feature = "extra-plugins")]
pub mod extra_plugins;
