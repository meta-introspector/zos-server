use std::sync::{Arc, Mutex};
use tokio;
use tracing::{error, info};
use zos_server::{
    core::ZOSCore, project_watcher::ProjectWatcher, telemetry,
    value_lattice_processor::ValueLatticeProcessor, web,
};

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        show_help();
        return;
    }

    match args[1].as_str() {
        "serve" => serve().await,
        "login" => {
            if args.len() < 3 {
                eprintln!("Usage: {} login <username>", args[0]);
                return;
            }
            login(&args[2]).await;
        }
        _ => show_help(),
    }
}

async fn serve() {
    // Initialize high-performance OpenTelemetry
    if let Err(e) = telemetry::TelemetryServer::init() {
        eprintln!("Failed to initialize telemetry: {}", e);
        return;
    }

    info!("🚀 Starting ZOS Server...");

    // Start project watcher
    let (mut watcher, mut change_rx) = ProjectWatcher::new();
    watcher.start_watching().await;

    // Start value lattice processor
    let mut lattice_processor = ValueLatticeProcessor::new();

    // Handle file changes
    tokio::spawn(async move {
        while let Some(change) = change_rx.recv().await {
            info!(
                "📁 File changed: {} in {}",
                change.path.display(),
                change.project_root.display()
            );

            // Process change through value lattice
            if let Err(e) = lattice_processor.process_file_change(&change) {
                error!("❌ Lattice processing error: {}", e);
            }
        }
    });

    let core = Arc::new(Mutex::new(ZOSCore::new()));

    // Minimal overhead instrumentation
    let app = web::create_router(core).layer(tower_http::trace::TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();

    info!("⚡ ZOS Server v1.0.0 - High Performance Mode");
    info!("🔍 Jaeger UI: http://localhost:16686");
    info!("🌐 Server running on 0.0.0.0:8080");

    let result = axum::serve(listener, app).await;
    // Telemetry cleanup handled automatically

    if let Err(e) = result {
        eprintln!("Server error: {}", e);
    }
}

async fn login(username: &str) {
    let mut core = ZOSCore::new();

    // Create user if doesn't exist
    let _user = core
        .create_user(username.to_string())
        .unwrap_or_else(|_| core.get_user(username).unwrap().clone());

    // Create session
    let session = core.create_session(username).unwrap();

    // Load config to get domain
    let config_content = std::fs::read_to_string("zos-config.toml").unwrap_or_else(|_| {
        r#"
[server]
domain = "localhost"
port = 8080
        "#
        .to_string()
    });

    let config: toml::Value = toml::from_str(&config_content).unwrap();
    let domain = config
        .get("server")
        .and_then(|s| s.get("domain"))
        .and_then(|d| d.as_str())
        .unwrap_or("localhost");
    let port = config
        .get("server")
        .and_then(|s| s.get("port"))
        .and_then(|p| p.as_integer())
        .unwrap_or(8080);

    println!("🔐 Dashboard Login");
    println!(
        "   URL: http://{}:{}/dashboard?token={}",
        domain, port, session.token
    );
    println!("   Token expires: {}", session.expires_at);
}

fn show_help() {
    println!("🚀 ZOS Server - Zero Ontology System");
    println!("Foundation build with plugin architecture");
    println!();
    println!("Available commands:");
    println!("  serve                 - Start web server");
    println!("  login <username>      - Generate login token");
}
