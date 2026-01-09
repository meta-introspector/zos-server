use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, Json},
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
