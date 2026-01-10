use std::process::Command;
use std::fs;
use std::time::Duration;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 && args[1] == "launch" {
        launch_server();
    } else {
        println!("Usage: zos-dev-minimal launch");
    }
}

fn launch_server() {
    // Kill existing processes
    println!("🧹 Killing existing processes...");
    let _ = Command::new("pkill").args(&["-f", "zos_server serve"]).output();
    std::thread::sleep(Duration::from_millis(500));

    // Start server in background
    println!("🚀 Starting server in background...");
    let result = Command::new("sh")
        .arg("-c")
        .arg("cargo run --bin zos_server serve &")
        .output();

    match result {
        Ok(_) => {
            println!("✅ Server launched in background");
            println!("🌐 Server: http://solana.solfunmeme.com:8080");
        }
        Err(e) => {
            eprintln!("❌ Failed to launch: {}", e);
        }
    }
}
