#![allow(unused)]

// Web layer - handles HTTP requests and responses
use crate::core::{User, ZOSCore};
use crate::telemetry::trace_user_operation;
use crate::*; // Import Clip2Secure macros
use axum::{
    extract::{Query, State},
    http::{header, StatusCode},
    response::Response,
    routing::get,
    Router,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::{info, instrument};

pub type AppState = Arc<Mutex<ZOSCore>>;

#[security_context(level = "Public", price_tier = 0.0, matrix_access = "DiagonalOnly")]
#[complexity(level = "Trivial", orbit_size = 1, time = "O(1)", space = "O(1)")]
pub fn create_router(core: AppState) -> Router {
    Router::new()
        .route("/", get(root_handler))
        .route("/dashboard", get(dashboard_handler))
        .route("/health", get(health_handler))
        .with_state(core)
}

async fn root_handler() -> Response {
    let html = r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>ZOS Server</title>
</head>
<body>
    <h1>🚀 ZOS Server</h1>
    <p><a href="/dashboard">Dashboard</a></p>
</body>
</html>
    "#;

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(html.into())
        .unwrap()
}

async fn health_handler() -> &'static str {
    "OK"
}

#[instrument(skip_all)]
async fn dashboard_handler(
    Query(params): Query<HashMap<String, String>>,
    State(core): State<AppState>,
) -> Response {
    let token = params.get("token");

    if let Some(token) = token {
        let validation_result = {
            let core = core.lock().unwrap();
            core.validate_session(token).map(|u| u.clone())
        };

        if let Some(user) = validation_result {
            let username = user.username.clone();
            return trace_user_operation(&username, "dashboard_access", async move {
                info!("Dashboard access granted for user: {}", user.username);
                dashboard_html(&user)
            })
            .await;
        }
    }

    login_html()
}

fn dashboard_html(user: &User) -> Response {
    let html = format!(
        r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>ZOS Dashboard</title>
</head>
<body>
    <h1>🎯 ZOS Dashboard</h1>
    <p>Welcome, {}!</p>
    <p>Permissions: {}</p>
</body>
</html>
    "#,
        user.username,
        user.permissions.join(", ")
    );

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(html.into())
        .unwrap()
}

fn login_html() -> Response {
    let html = r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>ZOS Login</title>
</head>
<body>
    <h1>🔐 ZOS Login</h1>
    <p>Run: <code>cargo run --bin zos_server login &lt;username&gt;</code></p>
</body>
</html>
    "#;

    Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(html.into())
        .unwrap()
}
