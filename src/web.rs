pub mod handlers;

use crate::core::{User, ZOSCore};
use crate::plugin_registry::{PluginRegistry, WebPlugin};
use crate::process_monitor::ProcessMonitor;
use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::Response,
    routing::get,
    Router,
};
use handlers::{create_plugin_router, dashboard_handler};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub type AppState = Arc<Mutex<ZOSCore>>;

pub fn create_router(core: AppState) -> Router {
    let registry = setup_plugins();

    Router::new()
        .route("/", get(root_handler))
        .route(
            "/dashboard",
            get(move || dashboard_handler(registry.clone())),
        )
        .route("/health", get(health_handler))
        .merge(create_plugin_router())
}

fn setup_plugins() -> Arc<PluginRegistry> {
    let registry = Arc::new(PluginRegistry::new());

    registry.register_plugin(WebPlugin {
        name: "Value Lattice".to_string(),
        path: "/plugin/value-lattice".to_string(),
        description: "Zero ontology lattice dashboard".to_string(),
        icon: "🔗".to_string(),
        handler: Arc::new(Box::new(|| value_lattice_handler_impl())),
    });

    registry.register_plugin(WebPlugin {
        name: "Process Monitor".to_string(),
        path: "/plugin/process-monitor".to_string(),
        description: "Real-time process monitoring".to_string(),
        icon: "📊".to_string(),
        handler: Arc::new(Box::new(|| {
            let monitor = ProcessMonitor::new();
            let html = monitor.generate_html_report();
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                .body(html)
                .unwrap()
        })),
    });

    registry
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

fn value_lattice_handler_impl() -> Response<String> {
    let html = r#"
<!DOCTYPE html>
<html>
<head>
    <title>Value Lattice Dashboard</title>
    <style>
        body { font-family: monospace; background: #0a0a0a; color: #00ff00; padding: 20px; }
        h1 { color: #00ffff; text-shadow: 0 0 10px #00ffff; }
    </style>
</head>
<body>
    <h1>🔗 Value Lattice Dashboard</h1>
    <p>Zero ontology convergence system active</p>
</body>
</html>
    "#;

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(html.to_string())
        .unwrap()
}
