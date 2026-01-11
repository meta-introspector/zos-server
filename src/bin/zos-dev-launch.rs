use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::process::{Child, Command};
use std::sync::mpsc::channel;
use std::time::{Duration, Instant};

struct ZosDevLauncher {
    server_process: Option<Child>,
    last_rebuild: Instant,
}

impl ZosDevLauncher {
    fn new() -> Self {
        Self {
            server_process: None,
            last_rebuild: Instant::now(),
        }
    }

    fn start_server(&mut self) {
        println!("🚀 [ZOS-DEV] Starting ZOS server...");

        // Kill existing server
        if let Some(mut process) = self.server_process.take() {
            println!("🛑 [ZOS-DEV] Killing existing server process");
            let _ = process.kill();
            let _ = process.wait();
        }

        // Build first
        println!("🔨 [ZOS-DEV] Building server...");
        let build_output = Command::new("cargo")
            .args(&["build", "--bin", "zos_server"])
            .output();

        match build_output {
            Ok(output) if output.status.success() => {
                println!("✅ [ZOS-DEV] Build successful");
            }
            Ok(output) => {
                println!("❌ [ZOS-DEV] Build failed:");
                println!("{}", String::from_utf8_lossy(&output.stderr));
                return;
            }
            Err(e) => {
                println!("❌ [ZOS-DEV] Build command failed: {}", e);
                return;
            }
        }

        // Start server
        match Command::new("./target/debug/zos_server")
            .arg("serve")
            .spawn()
        {
            Ok(child) => {
                println!("✅ [ZOS-DEV] Server started with PID: {}", child.id());
                self.server_process = Some(child);
            }
            Err(e) => {
                println!("❌ [ZOS-DEV] Failed to start server: {}", e);
            }
        }
    }

    fn handle_file_change(&mut self, event: Event) {
        // Debounce rapid changes
        if self.last_rebuild.elapsed() < Duration::from_millis(500) {
            return;
        }

        println!("📝 [ZOS-DEV] File change detected: {:?}", event.paths);

        match event.kind {
            EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_) => {
                for path in &event.paths {
                    if let Some(ext) = path.extension() {
                        if ext == "rs" || ext == "toml" {
                            println!("🔄 [ZOS-DEV] Rust/Cargo file changed: {:?}", path);
                            self.last_rebuild = Instant::now();
                            self.start_server();
                            return;
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn run(&mut self) {
        println!("🎯 [ZOS-DEV] ZOS Development Server Launcher");

        // Start initial server
        self.start_server();

        // Set up file watcher
        let (tx, rx) = channel();
        let mut watcher = RecommendedWatcher::new(
            tx,
            notify::Config::default().with_poll_interval(Duration::from_millis(100)),
        )
        .expect("Failed to create watcher");

        // Watch src directory and Cargo.toml
        watcher
            .watch(std::path::Path::new("src"), RecursiveMode::Recursive)
            .expect("Failed to watch src directory");
        watcher
            .watch(
                std::path::Path::new("Cargo.toml"),
                RecursiveMode::NonRecursive,
            )
            .expect("Failed to watch Cargo.toml");

        println!("👀 [ZOS-DEV] Watching for changes...");

        // Event loop
        loop {
            match rx.recv() {
                Ok(Ok(event)) => {
                    self.handle_file_change(event);
                }
                Ok(Err(e)) => {
                    println!("❌ [ZOS-DEV] Watch error: {:?}", e);
                }
                Err(e) => {
                    println!("❌ [ZOS-DEV] Channel error: {:?}", e);
                    break;
                }
            }
        }
    }
}

fn main() {
    let mut launcher = ZosDevLauncher::new();
    launcher.run();
}
