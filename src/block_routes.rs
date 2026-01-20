// API routes for block collection
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use crate::block_collector_plugin::BlockCollectorPlugin;

#[derive(Clone)]
pub struct AppState {
    pub plugin: Arc<Mutex<BlockCollectorPlugin>>,
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    peer_id: String,
}

#[derive(Deserialize)]
pub struct SubmitBlockRequest {
    block_json: String,
}

#[derive(Serialize)]
pub struct ApiResponse {
    status: String,
    data: serde_json::Value,
}

pub fn create_block_routes() -> Router<AppState> {
    Router::new()
        .route("/api/register", post(register_client))
        .route("/api/submit", post(submit_block))
        .route("/api/health", get(health_check))
}

async fn register_client(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<ApiResponse>, StatusCode> {
    let plugin = state.plugin.lock().unwrap();

    match plugin.register_client(&req.peer_id) {
        Ok(result) => {
            let data: serde_json::Value = serde_json::from_str(&result)
                .unwrap_or_else(|_| serde_json::json!({"response": result}));

            Ok(Json(ApiResponse {
                status: "success".to_string(),
                data,
            }))
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn submit_block(
    State(state): State<AppState>,
    Json(req): Json<SubmitBlockRequest>,
) -> Result<Json<ApiResponse>, StatusCode> {
    let plugin = state.plugin.lock().unwrap();

    match plugin.submit_block(&req.block_json) {
        Ok(result) => {
            let data: serde_json::Value = serde_json::from_str(&result)
                .unwrap_or_else(|_| serde_json::json!({"response": result}));

            Ok(Json(ApiResponse {
                status: "success".to_string(),
                data,
            }))
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn health_check() -> Json<ApiResponse> {
    Json(ApiResponse {
        status: "ok".to_string(),
        data: serde_json::json!({"service": "block-collector"}),
    })
}
