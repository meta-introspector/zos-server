use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{Html, Json, Response},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};

// Minimal state for Stage 1
#[derive(Clone)]
pub struct AppState {
    pub user_sessions: Arc<RwLock<HashMap<String, UserSession>>>,
    pub config: ServerConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub http_port: u16,
    pub domain: String,
    pub max_users: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    pub wallet_address: String,
    pub allocated_port: Option<u16>,
    pub credits: u64,
    pub last_activity: u64,
}

impl ServerConfig {
    pub fn load() -> Self {
        Self {
            http_port: std::env::var("ZOS_HTTP_PORT")
                .unwrap_or("8080".to_string())
                .parse()
                .unwrap_or(8080),
            domain: std::env::var("ZOS_DOMAIN").unwrap_or("localhost".to_string()),
            max_users: 50,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ServerConfig::load();

    println!("🚀 ZOS Stage 1 Server");
    println!("   Domain: {}", config.domain);
    println!("   Port: {}", config.http_port);

    let state = AppState {
        user_sessions: Arc::new(RwLock::new(HashMap::new())),
        config: config.clone(),
    };

    let app = Router::new()
        .route("/", get(homepage))
        .route("/health", get(health))
        .route("/dashboard/:wallet", get(dashboard))
        .route("/api/allocate-port", post(allocate_port))
        .route("/api/status/:wallet", get(user_status))
        .route("/deploy", post(deploy_zos2))
        .route("/rebuild", post(rebuild_self))
        .route("/build-cross", post(build_cross_platform))
        .route("/source", get(serve_source))
        .route("/install.sh", get(serve_installer))
        .route("/tarball", get(serve_tarball))
        .route("/:wallet/:service", get(service_call))
        .with_state(state.clone());

    let addr = format!("0.0.0.0:{}", config.http_port);
    println!("🌐 Server running on {}", addr);

    // Run server and background tasks
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    println!("🌐 Server running on {}", addr);

    tokio::select! {
        _ = axum::serve(listener, app) => {},
        _ = background_tasks(state) => {}
    }

    Ok(())
}

async fn homepage() -> Html<&'static str> {
    Html(
        r#"
    <html>
    <head><title>ZOS Stage 1</title></head>
    <body style="font-family: Arial; max-width: 800px; margin: 0 auto; padding: 20px;">
        <h1>🚀 ZOS Stage 1 Server</h1>
        <p>Minimal decentralized compute platform</p>

        <h3>📊 Endpoints</h3>
        <ul>
            <li><code>GET /health</code> - Health check</li>
            <li><code>GET /dashboard/{wallet}</code> - User dashboard</li>
            <li><code>POST /api/allocate-port</code> - Allocate port</li>
            <li><code>GET /{wallet}/{service}</code> - Call service</li>
        </ul>

        <h3>🎮 Try It</h3>
        <p><a href="/dashboard/demo">Demo Dashboard</a></p>
    </body>
    </html>
    "#,
    )
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "version": "1.0.0-stage1",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

async fn dashboard(Path(wallet): Path<String>) -> Html<String> {
    Html(format!(
        r#"
    <html>
    <head><title>ZOS Dashboard - {}</title></head>
    <body style="font-family: Arial; margin: 0; padding: 20px; background: #f5f5f5;">
        <h1>🎯 ZOS Dashboard</h1>
        <p>Wallet: <code>{}</code></p>

        <div style="background: white; padding: 20px; border-radius: 8px; margin: 20px 0;">
            <h3>📊 Status</h3>
            <p>Credits: <strong>100</strong></p>
            <p>Port: <strong>None allocated</strong></p>
            <button onclick="allocatePort()" style="background: #4CAF50; color: white; border: none; padding: 10px 20px; border-radius: 4px; cursor: pointer;">
                Allocate Port
            </button>
        </div>

        <div style="background: white; padding: 20px; border-radius: 8px; margin: 20px 0;">
            <h3>🎮 Free Services</h3>
            <button onclick="callService('pi')" style="margin: 5px; padding: 8px 16px; border: 1px solid #ddd; border-radius: 4px; cursor: pointer;">
                🥧 Calculate Pi
            </button>
            <button onclick="callService('fibonacci')" style="margin: 5px; padding: 8px 16px; border: 1px solid #ddd; border-radius: 4px; cursor: pointer;">
                🐰 Fibonacci
            </button>
            <button onclick="callService('primes')" style="margin: 5px; padding: 8px 16px; border: 1px solid #ddd; border-radius: 4px; cursor: pointer;">
                🎭 Primes
            </button>
        </div>

        <script>
            async function allocatePort() {{
                try {{
                    const response = await fetch('/api/allocate-port', {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json' }},
                        body: JSON.stringify({{ wallet: '{}' }})
                    }});
                    const result = await response.json();
                    alert('Port allocated: ' + result.port);
                    location.reload();
                }} catch (e) {{
                    alert('Error: ' + e.message);
                }}
            }}

            async function callService(service) {{
                try {{
                    const response = await fetch('/{}/'+service);
                    const result = await response.json();
                    alert(service + ' result: ' + JSON.stringify(result.result));
                }} catch (e) {{
                    alert('Error: ' + e.message);
                }}
            }}
        </script>
    </body>
    </html>
    "#,
        wallet, wallet, wallet, wallet
    ))
}

async fn allocate_port(
    State(state): State<AppState>,
    axum::Json(request): axum::Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let wallet = request
        .get("wallet")
        .and_then(|w| w.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let port = 20000 + (wallet.len() % 1000) as u16;

    let mut sessions = state.user_sessions.write().await;
    let session = sessions.entry(wallet.to_string()).or_insert(UserSession {
        wallet_address: wallet.to_string(),
        allocated_port: None,
        credits: 100,
        last_activity: chrono::Utc::now().timestamp() as u64,
    });

    session.allocated_port = Some(port);
    session.last_activity = chrono::Utc::now().timestamp() as u64;

    println!("🔌 Port {} allocated to {}", port, &wallet[..8]);

    Ok(Json(serde_json::json!({
        "success": true,
        "port": port,
        "expires_in_seconds": 300
    })))
}

async fn user_status(
    Path(wallet): Path<String>,
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let sessions = state.user_sessions.read().await;

    if let Some(session) = sessions.get(&wallet) {
        Json(serde_json::json!({
            "wallet": wallet,
            "credits": session.credits,
            "allocated_port": session.allocated_port,
            "last_activity": session.last_activity
        }))
    } else {
        Json(serde_json::json!({
            "wallet": wallet,
            "status": "not_found"
        }))
    }
}

async fn service_call(
    Path((wallet, service)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    // Simple service implementations
    let result = match service.as_str() {
        "pi" => "π ≈ 3.1415926536 (calculated using Leibniz formula)".to_string(),
        "fibonacci" => "🐰 Fibonacci sequence: 1, 1, 2, 3, 5, 8, 13, 21, 34, 55...".to_string(),
        "primes" => "🎭 Prime numbers: 2, 3, 5, 7, 11, 13, 17, 19, 23, 29...".to_string(),
        _ => format!("Unknown service: {}", service),
    };

    println!("🎯 Service call: {} -> {}", service, &wallet[..8]);

    Json(serde_json::json!({
        "service": service,
        "wallet": wallet,
        "result": result,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

#[derive(Debug, Deserialize)]
struct RebuildRequest {
    prepare_windows: bool,
}

async fn rebuild_self(Json(req): Json<RebuildRequest>) -> Json<serde_json::Value> {
    println!("🔄 ZOS2 rebuilding itself");

    let rebuild_script = format!(
        r#"#!/bin/bash
set -e
echo "🔄 ZOS2 self-rebuild initiated"

# Rebuild from source
cargo build --release --bin zos-minimal-server

# Update binary (will restart via systemd)
sudo cp target/release/zos-minimal-server /opt/zos2/bin/
sudo systemctl restart zos2.service

{}

echo "✅ ZOS2 self-rebuild completed"
"#,
        if req.prepare_windows {
            r#"
# Prepare Windows binaries
echo "🪟 Preparing Windows binaries"
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu --bin zos-minimal-server

# Create Windows deployment package
mkdir -p /opt/zos2/data/windows-binaries
cp target/x86_64-pc-windows-gnu/release/zos-minimal-server.exe /opt/zos2/data/windows-binaries/
echo "✅ Windows binaries prepared"
"#
        } else {
            ""
        }
    );

    tokio::spawn(async move {
        let _ = tokio::process::Command::new("bash")
            .arg("-c")
            .arg(&rebuild_script)
            .output()
            .await;
    });

    Json(serde_json::json!({
        "status": "rebuilding",
        "message": "Self-rebuild initiated",
        "prepare_windows": req.prepare_windows
    }))
}

#[derive(Debug, Deserialize)]
struct DeployRequest {
    target_port: u16,
    instance_name: String,
    rebuild_self: bool,
    prepare_windows: bool,
}

#[derive(Debug, Serialize)]
struct DeployResponse {
    status: String,
    instance_name: String,
    port: u16,
    message: String,
}

async fn deploy_zos2(Json(req): Json<DeployRequest>) -> Json<DeployResponse> {
    println!("🚀 ZOS1 deploying ZOS2 instance: {}", req.instance_name);

    let instance_name = req.instance_name.clone();
    let target_port = req.target_port;

    // Deploy ZOS2 instance
    let deploy_result = tokio::spawn(async move {
        let script = format!(
            r#"#!/bin/bash
set -e
echo "🔧 ZOS1 deploying ZOS2 on port {}"

# Build ZOS2 binary
cargo build --release --bin zos-minimal-server

# Create ZOS2 user and directories
sudo useradd -r -s /bin/false -d /opt/{} -m {} 2>/dev/null || true
sudo mkdir -p /opt/{}/{{bin,data,config,logs}}
sudo chown -R {}:{} /opt/{}

# Install ZOS2 binary
sudo cp target/release/zos-minimal-server /opt/{}/bin/
sudo chmod +x /opt/{}/bin/zos-minimal-server

# Create ZOS2 systemd service
sudo tee /etc/systemd/system/{}.service > /dev/null <<EOF
[Unit]
Description=ZOS2 Server - Deployed by ZOS1
After=network.target zos-server.service
Wants=network.target

[Service]
Type=simple
User={}
Group={}
WorkingDirectory=/opt/{}
ExecStart=/opt/{}/bin/zos-minimal-server
Restart=always
RestartSec=5
Environment=ZOS_HTTP_PORT={}
Environment=ZOS_DATA_DIR=/opt/{}/data
Environment=ZOS_LOG_LEVEL=info

NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/opt/{}/data /opt/{}/logs

[Install]
WantedBy=multi-user.target
EOF

# Enable and start ZOS2
sudo systemctl daemon-reload
sudo systemctl enable {}.service
sudo systemctl start {}.service

echo "✅ ZOS2 deployed successfully"
"#,
            req.target_port,
            req.instance_name,
            req.instance_name,
            req.instance_name,
            req.instance_name,
            req.instance_name,
            req.instance_name,
            req.instance_name,
            req.instance_name,
            req.instance_name,
            req.instance_name,
            req.instance_name,
            req.instance_name,
            req.instance_name,
            req.target_port,
            req.instance_name,
            req.instance_name,
            req.instance_name,
            req.instance_name,
            req.instance_name
        );

        // Execute deployment script
        let output = tokio::process::Command::new("bash")
            .arg("-c")
            .arg(&script)
            .output()
            .await;

        match output {
            Ok(result) => {
                if result.status.success() {
                    println!("✅ ZOS2 deployment completed");

                    // If rebuild_self is requested, trigger ZOS2 self-rebuild
                    if req.rebuild_self {
                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                        let rebuild_url = format!("http://localhost:{}/rebuild", req.target_port);
                        let _ = reqwest::Client::new()
                            .post(&rebuild_url)
                            .json(&serde_json::json!({"prepare_windows": req.prepare_windows}))
                            .send()
                            .await;
                    }

                    Ok(())
                } else {
                    Err(format!(
                        "Deployment failed: {}",
                        String::from_utf8_lossy(&result.stderr)
                    ))
                }
            }
            Err(e) => Err(format!("Failed to execute deployment: {}", e)),
        }
    })
    .await;

    match deploy_result {
        Ok(Ok(())) => Json(DeployResponse {
            status: "success".to_string(),
            instance_name,
            port: target_port,
            message: "ZOS2 deployed successfully".to_string(),
        }),
        Ok(Err(e)) => Json(DeployResponse {
            status: "error".to_string(),
            instance_name,
            port: target_port,
            message: e,
        }),
        Err(e) => Json(DeployResponse {
            status: "error".to_string(),
            instance_name,
            port: target_port,
            message: format!("Task failed: {}", e),
        }),
    }
}

#[derive(Debug, Deserialize)]
struct CrossBuildRequest {
    targets: Vec<String>,
}

async fn build_cross_platform(Json(req): Json<CrossBuildRequest>) -> Json<serde_json::Value> {
    println!(
        "🔨 Cross-platform build requested for targets: {:?}",
        req.targets
    );

    let targets = req.targets.clone();
    let targets_for_response = targets.clone();
    let build_result = tokio::spawn(async move {
        let script = format!(
            r#"#!/bin/bash
set -e
echo "🔨 Starting cross-platform builds"

cd zos-minimal-server

{}

echo "✅ Cross-platform builds completed"
"#,
            targets
                .iter()
                .map(|target| {
                    format!(
                        "echo \"Building for {}...\" && cargo build --release --target {}",
                        target, target
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        );

        let output = tokio::process::Command::new("bash")
            .arg("-c")
            .arg(&script)
            .output()
            .await;

        match output {
            Ok(result) => {
                if result.status.success() {
                    Ok(String::from_utf8_lossy(&result.stdout).to_string())
                } else {
                    Err(String::from_utf8_lossy(&result.stderr).to_string())
                }
            }
            Err(e) => Err(format!("Failed to execute build: {}", e)),
        }
    })
    .await;

    match build_result {
        Ok(Ok(output)) => Json(serde_json::json!({
            "status": "success",
            "targets": targets_for_response,
            "output": output
        })),
        Ok(Err(error)) => Json(serde_json::json!({
            "status": "error",
            "targets": targets_for_response,
            "error": error
        })),
        Err(e) => Json(serde_json::json!({
            "status": "error",
            "targets": targets_for_response,
            "error": format!("Task failed: {}", e)
        })),
    }
}

async fn serve_source() -> Json<serde_json::Value> {
    println!("📦 Serving ZOS source information");

    Json(serde_json::json!({
        "name": "ZOS Server",
        "version": "1.0.0-stage1",
        "repository": "https://github.com/meta-introspector/zos-server.git",
        "branch": "main",
        "install_command": "curl -sSL http://solana.solfunmeme.com:8080/install.sh | bash",
        "tarball_url": "http://solana.solfunmeme.com:8080/tarball",
        "endpoints": {
            "/source": "Source information (this endpoint)",
            "/install.sh": "Installation script",
            "/tarball": "Source tarball download",
            "/health": "Health check",
            "/deploy": "Deploy new ZOS instance"
        }
    }))
}

async fn serve_installer() -> Response<String> {
    println!("🚀 Serving ZOS installer script");

    let installer_script = r#"#!/bin/bash
set -e

echo "🚀 ZOS Universal Installer"
echo "📡 Installing from: solana.solfunmeme.com:8080"
echo ""

# Detect platform
PLATFORM=$(uname -s)
ARCH=$(uname -m)
echo "🖥️  Platform: $PLATFORM $ARCH"

# Set installation directories
if [[ "$EUID" -eq 0 ]]; then
    INSTALL_DIR="/opt/zos"
    BIN_DIR="/opt/zos/bin"
    echo "🔧 Root install to: $INSTALL_DIR"
else
    INSTALL_DIR="$HOME/.zos"
    BIN_DIR="$HOME/.zos/bin"
    echo "🏠 User install to: $INSTALL_DIR"
fi

# Install dependencies based on platform
case "$PLATFORM" in
    "Linux")
        echo "🐧 Linux detected"
        if command -v nix >/dev/null 2>&1; then
            echo "❄️  Nix detected - using Nix environment"
            INSTALL_METHOD="nix"
        elif command -v apt >/dev/null 2>&1; then
            echo "📦 APT detected - installing dependencies"
            if [[ "$EUID" -eq 0 ]]; then
                apt update && apt install -y curl git build-essential pkg-config libssl-dev
            else
                sudo apt update && sudo apt install -y curl git build-essential pkg-config libssl-dev
            fi
            INSTALL_METHOD="cargo"
        elif command -v yum >/dev/null 2>&1; then
            echo "📦 YUM detected - installing dependencies"
            if [[ "$EUID" -eq 0 ]]; then
                yum install -y curl git gcc pkg-config openssl-devel
            else
                sudo yum install -y curl git gcc pkg-config openssl-devel
            fi
            INSTALL_METHOD="cargo"
        else
            echo "⚠️  Unknown package manager - assuming dependencies exist"
            INSTALL_METHOD="cargo"
        fi
        ;;
    "Darwin")
        echo "🍎 macOS detected"
        if ! command -v brew >/dev/null 2>&1; then
            echo "🍺 Installing Homebrew..."
            /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
        fi
        brew install pkg-config openssl
        INSTALL_METHOD="cargo"
        ;;
    "MINGW"*|"MSYS"*|"CYGWIN"*)
        echo "🪟 Windows/MinGW detected"
        INSTALL_DIR="$HOME/.zos"
        BIN_DIR="$HOME/.zos/bin"
        INSTALL_METHOD="cargo"
        ;;
    *)
        echo "❓ Unknown platform - attempting generic install"
        INSTALL_METHOD="cargo"
        ;;
esac

# Install Rust if not present
if ! command -v cargo >/dev/null 2>&1; then
    echo "🦀 Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
    export PATH="$HOME/.cargo/bin:$PATH"
fi

# Create installation directories
mkdir -p "$INSTALL_DIR" "$BIN_DIR"
cd "$INSTALL_DIR"

# Download and extract ZOS source
echo "📥 Downloading ZOS source to $INSTALL_DIR..."
if command -v git >/dev/null 2>&1; then
    echo "📂 Cloning from Git..."
    if [ -d "zos-server" ]; then
        rm -rf zos-server
    fi
    git clone https://github.com/meta-introspector/zos-server.git
    cd zos-server
else
    echo "📦 Downloading tarball..."
    curl -L http://solana.solfunmeme.com:8080/tarball -o zos-server.tar.gz
    tar -xzf zos-server.tar.gz
    # Handle potential directory name variations
    if [ -d "zos-server" ]; then
        cd zos-server
    else
        # Find the extracted directory
        EXTRACTED_DIR=$(find . -maxdepth 1 -type d -name "*zos*" | head -1)
        if [ -n "$EXTRACTED_DIR" ]; then
            cd "$EXTRACTED_DIR"
        else
            echo "❌ Could not find extracted ZOS directory"
            exit 1
        fi
    fi
fi

# Verify we're in the right place
if [ ! -d "zos-minimal-server" ]; then
    echo "❌ zos-minimal-server directory not found in $(pwd)"
    echo "📁 Available directories:"
    ls -la
    exit 1
fi

# Build ZOS
echo "🔨 Building ZOS..."
cd zos-minimal-server

# Remove lockfile to avoid version conflicts
rm -f Cargo.lock

case "$INSTALL_METHOD" in
    "nix")
        nix-shell -p rustc cargo pkg-config openssl git --run "cargo build --release"
        ;;
    "cargo")
        cargo build --release
        ;;
esac

# Install ZOS binary
echo "📦 Installing ZOS binary to $BIN_DIR..."
cp target/release/zos-minimal-server "$BIN_DIR/"

# Make it executable
chmod +x "$BIN_DIR/zos-minimal-server"

# Add to PATH if not already there
if [[ ":$PATH:" != *":$BIN_DIR:"* ]]; then
    echo "🔧 Adding $BIN_DIR to PATH..."
    echo "export PATH=\"$BIN_DIR:\$PATH\"" >> ~/.bashrc
    echo "export PATH=\"$BIN_DIR:\$PATH\"" >> ~/.profile 2>/dev/null || true
    export PATH="$BIN_DIR:$PATH"
fi

echo ""
echo "🎉 ZOS Installation Complete!"
echo ""
echo "📁 Installed to: $INSTALL_DIR"
echo "🚀 Binary at: $BIN_DIR/zos-minimal-server"
echo ""
echo "▶️  Start ZOS with: zos-minimal-server"
echo "🔗 Test with: curl http://localhost:8080/health"
echo ""
echo "📚 Documentation: http://solana.solfunmeme.com:8080/source"
echo "🌐 Join network: http://solana.solfunmeme.com:8080"
"#;

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .header(
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"install.sh\"",
        )
        .body(installer_script.to_string())
        .unwrap()
}

async fn serve_tarball() -> Result<Vec<u8>, StatusCode> {
    println!("📦 Creating and serving ZOS tarball");

    // Create tarball of current source
    let output = tokio::process::Command::new("tar")
        .args(&[
            "-czf",
            "/tmp/zos-server.tar.gz",
            "--exclude=target",
            "--exclude=.git",
            "--exclude=*.log",
            "--exclude=Cargo.lock",
            ".",
        ])
        .output()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !output.status.success() {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    // Read the tarball
    tokio::fs::read("/tmp/zos-server.tar.gz")
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn background_tasks(state: AppState) {
    let mut interval = interval(Duration::from_secs(60));

    loop {
        interval.tick().await;

        // Clean up old sessions
        let mut sessions = state.user_sessions.write().await;
        let current_time = chrono::Utc::now().timestamp() as u64;

        let before_count = sessions.len();
        sessions.retain(|_, session| {
            current_time - session.last_activity < 3600 // Keep for 1 hour
        });
        let after_count = sessions.len();

        if before_count != after_count {
            println!("🧹 Cleaned up {} old sessions", before_count - after_count);
        }
    }
}
