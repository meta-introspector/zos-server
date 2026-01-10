use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::fs;
use std::io::Write;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Debug)]
#[allow(dead_code)]
struct DevConfig {
    host: String,
    port: u16,
    domain: String,
}

impl DevConfig {
    fn load() -> Self {
        // Try to load from config file
        if let Ok(content) = fs::read_to_string("dev-config.toml") {
            if let Ok(config) = toml::from_str::<toml::Value>(&content) {
                return DevConfig {
                    host: config
                        .get("host")
                        .and_then(|v| v.as_str())
                        .unwrap_or("0.0.0.0")
                        .to_string(),
                    port: config
                        .get("port")
                        .and_then(|v| v.as_integer())
                        .unwrap_or(8080) as u16,
                    domain: config
                        .get("domain")
                        .and_then(|v| v.as_str())
                        .unwrap_or("localhost")
                        .to_string(),
                };
            }
        }

        // Default config
        DevConfig {
            host: "0.0.0.0".to_string(),
            port: 8080,
            domain: "localhost".to_string(),
        }
    }
}

fn main() {
    let config = DevConfig::load();

    // Check if we should daemonize
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "--daemon" {
        daemonize(&config);
        return;
    }

    // Kill any existing dev server processes
    kill_existing_processes();

    // Start as daemon
    println!("🔥 Starting ZOS Dev Server daemon with auto-reload...");
    let daemon = Command::new(&args[0])
        .arg("--daemon")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();

    match daemon {
        Ok(mut child) => {
            // Write PID file
            if let Ok(mut file) = fs::File::create("/tmp/zos-dev.pid") {
                let _ = writeln!(file, "{}", child.id());
            }
            println!("✅ Dev server started in background (PID: {})", child.id());
            println!("👀 Watching src/ for changes...");
            println!("🌐 Server: http://{}:{}", config.domain, config.port);
            println!("🛑 Stop with: pkill -f zos-dev-minimal");

            // Don't wait for the daemon process
            let _ = child.wait();
        }
        Err(e) => {
            eprintln!("❌ Failed to start daemon: {}", e);
            std::process::exit(1);
        }
    }
}

fn daemonize(_config: &DevConfig) {
    let server_process: Arc<Mutex<Option<Child>>> = Arc::new(Mutex::new(None));

    // Initial build and start
    rebuild_and_restart(&server_process);

    // File watcher for src/ directory
    let (tx, rx) = channel();
    let mut watcher = RecommendedWatcher::new(
        move |res: Result<Event, notify::Error>| {
            if let Ok(_) = res {
                let _ = tx.send(());
            }
        },
        notify::Config::default(),
    )
    .unwrap();

    watcher
        .watch(std::path::Path::new("src"), RecursiveMode::Recursive)
        .unwrap();

    // Watch loop with debouncing
    let mut last_reload = std::time::Instant::now();
    loop {
        if rx.recv_timeout(Duration::from_millis(100)).is_ok() {
            if last_reload.elapsed() > Duration::from_millis(500) {
                rebuild_and_restart(&server_process);
                last_reload = std::time::Instant::now();
            }
        }
    }
}

fn kill_existing_processes() {
    // Kill by PID file
    if let Ok(pid_str) = fs::read_to_string("/tmp/zos-dev.pid") {
        if let Ok(pid) = pid_str.trim().parse::<u32>() {
            let _ = Command::new("kill").arg(pid.to_string()).output();
        }
    }

    // Kill by process name
    let _ = Command::new("pkill")
        .args(&["-f", "zos-dev-minimal --daemon"])
        .output();
    let _ = Command::new("pkill")
        .args(&["-f", "zos_server serve"])
        .output();

    std::thread::sleep(Duration::from_millis(500));
}

fn rebuild_and_restart(server_process: &Arc<Mutex<Option<Child>>>) {
    // Kill existing server
    if let Ok(mut process) = server_process.lock() {
        if let Some(mut child) = process.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }

    // Build
    let build = Command::new("cargo")
        .args(&["build", "--bin", "zos_server", "--quiet"])
        .output();

    match build {
        Ok(output) if output.status.success() => {
            // Start server
            match Command::new("./target/debug/zos_server")
                .arg("serve")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
            {
                Ok(child) => {
                    if let Ok(mut process) = server_process.lock() {
                        *process = Some(child);
                    }
                }
                Err(_) => {}
            }
        }
        Ok(_) => {}
        Err(_) => {}
    }
}
