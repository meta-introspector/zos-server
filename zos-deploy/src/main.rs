// ZOS Deploy - Hardware Info Plugin
// AGPL-3.0 License

use std::env;

/// Hardware info collection in pure Rust
macro_rules! collect_hwinfo {
    () => {{
        let mut info = HwInfo::new();
        info.collect_basic_info();
        info
    }};
}

#[derive(Debug)]
struct HwInfo {
    os: String,
    arch: String,
    cores: usize,
    memory_kb: u64,
    hostname: String,
    rust_version: String,
    capabilities: Vec<String>,
}

impl HwInfo {
    fn new() -> Self {
        Self {
            os: String::new(),
            arch: String::new(),
            cores: 0,
            memory_kb: 0,
            hostname: String::new(),
            rust_version: String::new(),
            capabilities: Vec::new(),
        }
    }

    fn collect_basic_info(&mut self) {
        // OS detection
        self.os = if cfg!(target_os = "linux") {
            "linux".to_string()
        } else if cfg!(target_os = "windows") {
            "windows".to_string()
        } else if cfg!(target_os = "android") {
            "android".to_string()
        } else if cfg!(target_os = "macos") {
            "macos".to_string()
        } else {
            "unknown".to_string()
        };

        // Architecture detection
        self.arch = if cfg!(target_arch = "x86_64") {
            "x86_64".to_string()
        } else if cfg!(target_arch = "aarch64") {
            "aarch64".to_string()
        } else if cfg!(target_arch = "arm") {
            "arm".to_string()
        } else {
            env::consts::ARCH.to_string()
        };

        // Basic system info
        self.cores = std::thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or(1);

        self.hostname = env::var("HOSTNAME")
            .or_else(|_| env::var("COMPUTERNAME"))
            .unwrap_or_else(|_| "unknown".to_string());

        self.rust_version = env::var("RUSTC_VERSION").unwrap_or_else(|_| "unknown".to_string());

        // Detect capabilities
        self.detect_capabilities();
    }

    fn detect_capabilities(&mut self) {
        // Git availability
        if self.command_exists("git") {
            self.capabilities.push("git".to_string());
        }

        // Docker availability
        if self.command_exists("docker") {
            self.capabilities.push("docker".to_string());
        }

        // Systemd (Linux)
        if self.os == "linux" && std::path::Path::new("/bin/systemctl").exists() {
            self.capabilities.push("systemd".to_string());
        }

        // WSL (Windows)
        if self.os == "windows" && env::var("WSL_DISTRO_NAME").is_ok() {
            self.capabilities.push("wsl".to_string());
        }

        // Termux (Android)
        if self.os == "android" && env::var("TERMUX_VERSION").is_ok() {
            self.capabilities.push("termux".to_string());
        }

        // Tmux availability
        if self.command_exists("tmux") {
            self.capabilities.push("tmux".to_string());
        }

        // Network capabilities
        if self.command_exists("curl") || self.command_exists("wget") {
            self.capabilities.push("network".to_string());
        }

        // Compilation capabilities
        if self.command_exists("rustc") {
            self.capabilities.push("rust_native".to_string());
        }

        // Cross-compilation detection
        if env::var("CARGO_TARGET_DIR").is_ok() {
            self.capabilities.push("rust_cross".to_string());
        }
    }

    fn command_exists(&self, cmd: &str) -> bool {
        std::process::Command::new(cmd)
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    fn to_json(&self) -> String {
        format!(
            r#"{{
  "os": "{}",
  "arch": "{}",
  "cores": {},
  "memory_kb": {},
  "hostname": "{}",
  "rust_version": "{}",
  "capabilities": [{}],
  "timestamp": "{}"
}}"#,
            self.os,
            self.arch,
            self.cores,
            self.memory_kb,
            self.hostname,
            self.rust_version,
            self.capabilities
                .iter()
                .map(|c| format!("\"{}\"", c))
                .collect::<Vec<_>>()
                .join(", "),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        )
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        match args[1].as_str() {
            "collect" => {
                let hwinfo = collect_hwinfo!();
                println!("{}", hwinfo.to_json());
            }
            "capabilities" => {
                let hwinfo = collect_hwinfo!();
                for cap in &hwinfo.capabilities {
                    println!("{}", cap);
                }
            }
            "summary" => {
                let hwinfo = collect_hwinfo!();
                println!("Node: {} ({}/{})", hwinfo.hostname, hwinfo.os, hwinfo.arch);
                println!("Cores: {}", hwinfo.cores);
                println!("Capabilities: {}", hwinfo.capabilities.join(", "));
            }
            _ => {
                println!("Usage: {} [collect|capabilities|summary]", args[0]);
            }
        }
    } else {
        // Default: show summary
        let hwinfo = collect_hwinfo!();
        println!("ZOS Hardware Info Plugin");
        println!("========================");
        println!("Hostname: {}", hwinfo.hostname);
        println!("OS: {}", hwinfo.os);
        println!("Architecture: {}", hwinfo.arch);
        println!("CPU Cores: {}", hwinfo.cores);
        println!("Rust Version: {}", hwinfo.rust_version);
        println!("Capabilities: {}", hwinfo.capabilities.join(", "));

        if hwinfo.capabilities.is_empty() {
            println!("\nWarning: No capabilities detected. Node may need setup.");
        } else {
            println!("\nNode ready for ZOS deployment.");
        }
    }
}
