use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn main() {
    println!("🔥 ZOS Dev Server - One Command Does Everything");

    // Kill existing processes
    println!("🧹 Cleaning up existing processes...");
    let _ = Command::new("pkill").args(&["-f", "zos"]).output();
    thread::sleep(Duration::from_millis(500));

    // Server process handle
    let server_process: Arc<Mutex<Option<Child>>> = Arc::new(Mutex::new(None));
    let server_clone = server_process.clone();

    // Initial build and start
    println!("🔨 Initial build and start...");
    rebuild_and_restart(&server_clone);

    // Health check thread
    let health_clone = server_process.clone();
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(5));

            let health_check = Command::new("curl")
                .args(&["-s", "http://localhost:8080/health"])
                .output();

            match health_check {
                Ok(output) if output.status.success() => {
                    // Server is healthy
                }
                _ => {
                    println!("⚠️ Server health check failed, restarting...");
                    restart_server(&health_clone);
                }
            }
        }
    });

    // File watcher
    let (tx, rx) = channel();
    let mut watcher = RecommendedWatcher::new(
        move |res: Result<Event, notify::Error>| {
            if let Ok(_event) = res {
                let _ = tx.send(());
            }
        },
        notify::Config::default(),
    )
    .unwrap();

    watcher
        .watch(std::path::Path::new("src"), RecursiveMode::Recursive)
        .unwrap();

    // Show status after initial startup
    thread::sleep(Duration::from_secs(2));
    show_status();

    // Watch for changes
    let _watcher = watcher; // Keep alive
    let mut debounce = std::time::Instant::now();

    loop {
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(_) => {
                if debounce.elapsed() > Duration::from_millis(500) {
                    println!("🔄 Files changed, rebuilding...");
                    rebuild_and_restart(&server_clone);
                    debounce = std::time::Instant::now();
                }
            }
            Err(_) => {} // Timeout, continue
        }
    }
}

fn show_status() {
    // Check if server is responding
    let health_check = Command::new("curl")
        .args(&["-s", "http://localhost:8080/health"])
        .output();

    match health_check {
        Ok(output) if output.status.success() => {
            println!("✅ Dev server is running and healthy");

            // Get dashboard URL
            let login_result = Command::new("./target/debug/zos_server")
                .args(&["login", "alice"])
                .output();

            if let Ok(output) = login_result {
                let output_str = String::from_utf8_lossy(&output.stdout);
                if let Some(line) = output_str.lines().find(|l| l.contains("Dashboard URL:")) {
                    if let Some(url) = line.split_whitespace().nth(2) {
                        println!("🌐 Dashboard: {}", url);
                    }
                }
            }

            println!("");
            println!("🔥 Dev server active with:");
            println!("   • Auto-rebuild on file changes");
            println!("   • Health monitoring & restart");
            println!("   • Error reporting to dashboard");
            println!("   • Client auto-refresh");
            println!("");
            println!("Edit files in src/ to see auto-reload!");
        }
        _ => {
            println!("❌ Server failed to start properly");
        }
    }
}

fn restart_server(server_process: &Arc<Mutex<Option<Child>>>) {
    // Kill existing server
    if let Ok(mut process) = server_process.lock() {
        if let Some(mut child) = process.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }

    // Start new server
    match Command::new("./target/debug/zos_server")
        .arg("serve")
        .env("ZOS_DEV_MODE", "true")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => {
            if let Ok(mut process) = server_process.lock() {
                *process = Some(child);
            }
            println!("🚀 Server restarted");
        }
        Err(e) => {
            println!("❌ Failed to start server: {}", e);
        }
    }
}

fn rebuild_and_restart(server_process: &Arc<Mutex<Option<Child>>>) {
    // Build
    let build_result = Command::new("cargo")
        .args(&["build", "--bin", "zos_server", "--quiet"])
        .output();

    match build_result {
        Ok(output) if output.status.success() => {
            println!("✅ Build successful");
            restart_server(server_process);
            thread::sleep(Duration::from_millis(1000)); // Give server time to start
        }
        Ok(output) => {
            let error = String::from_utf8_lossy(&output.stderr);
            println!("❌ Build failed: {}", error);
        }
        Err(e) => {
            println!("❌ Failed to run build: {}", e);
        }
    }
}
