// Client telemetry API
use axum::{
    extract::Json,
    http::StatusCode,
    routing::post,
    Router,
};
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Deserialize, Serialize)]
pub struct ClientLog {
    pub level: String,
    pub message: String,
    pub timestamp: String,
    pub context: Option<String>,
}

pub fn create_telemetry_routes() -> Router {
    Router::new()
        .route("/api/telemetry/log", post(log_handler))
}

async fn log_handler(Json(log): Json<ClientLog>) -> StatusCode {
    info!(
        "📱 Client [{}] {}: {} {}",
        log.timestamp,
        log.level,
        log.message,
        log.context.unwrap_or_default()
    );
    StatusCode::OK
}
