use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;

use crate::secure_libp2p_api::{
    ApiError, CoordinationRequest, Permission, RateLimit, SecureLibP2PApi, TestResults,
};

#[derive(Clone)]
pub struct AppState {
    pub api: Arc<SecureLibP2PApi>,
}

#[derive(Deserialize)]
pub struct AuthQuery {
    api_key: Option<String>,
}

pub fn create_secure_router() -> Router {
    let api = Arc::new(SecureLibP2PApi::new());

    // Create some default API keys for testing
    let read_key = api.create_api_key(
        vec![Permission::ReadLattice, Permission::ReadResults],
        RateLimit {
            requests_per_minute: 100,
            burst_limit: 20,
        },
    );

    let coord_key = api.create_api_key(
        vec![
            Permission::ReadLattice,
            Permission::Coordinate,
            Permission::WriteResults,
        ],
        RateLimit {
            requests_per_minute: 200,
            burst_limit: 50,
        },
    );

    println!("🔑 Created API keys:");
    println!("  Read key: {}", read_key);
    println!("  Coordination key: {}", coord_key);

    let state = AppState { api };

    Router::new()
        // Public endpoints (rate limited)
        .route("/api/v1/health", get(health_check))
        .route("/api/v1/lattice/status", get(lattice_status))
        .route("/api/v1/network/info", get(network_info))
        // Authenticated endpoints
        .route("/api/v1/coordinate", post(coordinate_work))
        .route("/api/v1/results", post(submit_results))
        .route("/api/v1/admin/keys", post(create_api_key))
        .layer(ServiceBuilder::new().layer(CorsLayer::permissive()))
        .with_state(state)
}

async fn health_check(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(auth): Query<AuthQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let ip = extract_ip(&headers);

    match state
        .api
        .authenticate_request(auth.api_key.as_deref(), ip, "/api/v1/health")
        .await
    {
        Ok(_) => Ok(Json(serde_json::json!({
            "status": "healthy",
            "service": "ZOS Server LibP2P API",
            "version": "1.0.0",
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))),
        Err(ApiError::RateLimited) => Err((
            StatusCode::TOO_MANY_REQUESTS,
            "Rate limit exceeded".to_string(),
        )),
        Err(e) => Err((StatusCode::UNAUTHORIZED, format!("Auth error: {:?}", e))),
    }
}

async fn lattice_status(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(auth): Query<AuthQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let ip = extract_ip(&headers);

    match state
        .api
        .authenticate_request(auth.api_key.as_deref(), ip, "/api/v1/lattice/status")
        .await
    {
        Ok(permissions) => match state.api.handle_lattice_status(&permissions).await {
            Ok(response) => Ok(Json(response)),
            Err(ApiError::InsufficientPermissions) => Err((
                StatusCode::FORBIDDEN,
                "Insufficient permissions".to_string(),
            )),
            Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {:?}", e))),
        },
        Err(ApiError::RateLimited) => Err((
            StatusCode::TOO_MANY_REQUESTS,
            "Rate limit exceeded".to_string(),
        )),
        Err(e) => Err((StatusCode::UNAUTHORIZED, format!("Auth error: {:?}", e))),
    }
}

async fn network_info(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(auth): Query<AuthQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let ip = extract_ip(&headers);

    match state
        .api
        .authenticate_request(auth.api_key.as_deref(), ip, "/api/v1/network/info")
        .await
    {
        Ok(permissions) => match state.api.handle_network_info(&permissions).await {
            Ok(response) => Ok(Json(response)),
            Err(ApiError::InsufficientPermissions) => Err((
                StatusCode::FORBIDDEN,
                "Insufficient permissions".to_string(),
            )),
            Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {:?}", e))),
        },
        Err(ApiError::RateLimited) => Err((
            StatusCode::TOO_MANY_REQUESTS,
            "Rate limit exceeded".to_string(),
        )),
        Err(e) => Err((StatusCode::UNAUTHORIZED, format!("Auth error: {:?}", e))),
    }
}

async fn coordinate_work(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(auth): Query<AuthQuery>,
    Json(request): Json<CoordinationRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let ip = extract_ip(&headers);

    match state
        .api
        .authenticate_request(auth.api_key.as_deref(), ip, "/api/v1/coordinate")
        .await
    {
        Ok(permissions) => {
            match state
                .api
                .handle_coordinate_work(&permissions, request)
                .await
            {
                Ok(response) => Ok(Json(response)),
                Err(ApiError::InsufficientPermissions) => Err((
                    StatusCode::FORBIDDEN,
                    "Coordination permission required".to_string(),
                )),
                Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {:?}", e))),
            }
        }
        Err(ApiError::RateLimited) => Err((
            StatusCode::TOO_MANY_REQUESTS,
            "Rate limit exceeded".to_string(),
        )),
        Err(e) => Err((StatusCode::UNAUTHORIZED, format!("Auth error: {:?}", e))),
    }
}

async fn submit_results(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(auth): Query<AuthQuery>,
    Json(results): Json<TestResults>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let ip = extract_ip(&headers);

    match state
        .api
        .authenticate_request(auth.api_key.as_deref(), ip, "/api/v1/results")
        .await
    {
        Ok(permissions) => match state.api.handle_submit_results(&permissions, results).await {
            Ok(response) => Ok(Json(response)),
            Err(ApiError::InsufficientPermissions) => Err((
                StatusCode::FORBIDDEN,
                "Write results permission required".to_string(),
            )),
            Err(ApiError::InvalidRequest) => {
                Err((StatusCode::BAD_REQUEST, "Invalid request data".to_string()))
            }
            Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {:?}", e))),
        },
        Err(ApiError::RateLimited) => Err((
            StatusCode::TOO_MANY_REQUESTS,
            "Rate limit exceeded".to_string(),
        )),
        Err(e) => Err((StatusCode::UNAUTHORIZED, format!("Auth error: {:?}", e))),
    }
}

async fn create_api_key(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(auth): Query<AuthQuery>,
    Json(request): Json<CreateKeyRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let ip = extract_ip(&headers);

    match state
        .api
        .authenticate_request(auth.api_key.as_deref(), ip, "/api/v1/admin/keys")
        .await
    {
        Ok(permissions) => {
            if !state.api.check_permission(&permissions, Permission::Admin) {
                return Err((
                    StatusCode::FORBIDDEN,
                    "Admin permission required".to_string(),
                ));
            }

            let new_key = state
                .api
                .create_api_key(request.permissions, request.rate_limit);

            Ok(Json(serde_json::json!({
                "api_key": new_key,
                "created_at": chrono::Utc::now().to_rfc3339()
            })))
        }
        Err(ApiError::RateLimited) => Err((
            StatusCode::TOO_MANY_REQUESTS,
            "Rate limit exceeded".to_string(),
        )),
        Err(e) => Err((StatusCode::UNAUTHORIZED, format!("Auth error: {:?}", e))),
    }
}

#[derive(Deserialize)]
struct CreateKeyRequest {
    permissions: Vec<Permission>,
    rate_limit: RateLimit,
}

fn extract_ip(headers: &HeaderMap) -> IpAddr {
    // Try to get real IP from headers (for reverse proxy setups)
    if let Some(forwarded) = headers.get("x-forwarded-for") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            if let Ok(ip) = forwarded_str.split(',').next().unwrap_or("").trim().parse() {
                return ip;
            }
        }
    }

    if let Some(real_ip) = headers.get("x-real-ip") {
        if let Ok(ip_str) = real_ip.to_str() {
            if let Ok(ip) = ip_str.parse() {
                return ip;
            }
        }
    }

    // Default to localhost
    "127.0.0.1".parse().unwrap()
}

pub async fn start_secure_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let app = create_secure_router();
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    println!("🚀 Starting secure LibP2P API server on {}", addr);
    println!("📋 Available endpoints:");
    println!("  GET  /api/v1/health");
    println!("  GET  /api/v1/lattice/status");
    println!("  GET  /api/v1/network/info");
    println!("  POST /api/v1/coordinate");
    println!("  POST /api/v1/results");
    println!("  POST /api/v1/admin/keys");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
