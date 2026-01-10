use axum::{
    extract::State,
    response::{Html, Json},
    routing::get,
    Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;

pub fn standard_html_header(title: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 0; padding: 20px; background: #1a1a1a; color: #fff; }}
        .header {{ background: #333; padding: 20px; border-radius: 10px; margin-bottom: 20px; }}
        .container {{ max-width: 1200px; margin: 0 auto; }}
        a {{ color: #4CAF50; text-decoration: none; }}
        a:hover {{ text-decoration: underline; }}
    </style>
</head>
<body>
    <div class="container">"#,
        title
    )
}

pub fn standard_html_footer() -> &'static str {
    r#"    </div>
</body>
</html>"#
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientRecord {
    pub ip: String,
    pub user_agent: Option<String>,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub request_count: u64,
    pub endpoints_accessed: Vec<String>,
    pub risk_score: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResourceTrace {
    pub verb: String,
    pub start_time: DateTime<Utc>,
    pub duration_ms: u64,
}

pub type ServerState = RwLock<HashMap<String, ClientRecord>>;

pub fn create_base_router(state: Arc<ServerState>) -> Router {
    Router::new()
        .route("/", get(root_handler))
        .route("/status", get(status_handler))
        .route("/health", get(health_handler))
        .with_state(state)
}

async fn root_handler() -> Html<String> {
    let content = format!(
        r#"{}
        <div class="header">
            <h1>🚀 ZOS Server - Zero Ontology System</h1>
            <p>Foundation build with plugin architecture</p>
        </div>
        <ul>
            <li><a href="/status">Server Status</a></li>
            <li><a href="/health">Health Check</a></li>
        </ul>
        {}"#,
        standard_html_header("ZOS Server"),
        standard_html_footer()
    );
    Html(content)
}

async fn status_handler(State(state): State<Arc<ServerState>>) -> Json<serde_json::Value> {
    let clients = state.read().await;
    Json(serde_json::json!({
        "status": "running",
        "clients_connected": clients.len(),
        "timestamp": Utc::now()
    }))
}

async fn health_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "timestamp": Utc::now()
    }))
}

pub async fn start_server(
    addr: SocketAddr,
    router: Router,
) -> Result<(), Box<dyn std::error::Error>> {
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("🚀 ZOS Server listening on {}", addr);
    axum::serve(listener, router).await?;
    Ok(())
}
