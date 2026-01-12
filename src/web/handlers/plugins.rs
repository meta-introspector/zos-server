use crate::plugin_registry::{PluginRegistry, WebPlugin};
use crate::process_monitor::ProcessMonitor;
use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::Response,
    routing::get,
    Router,
};
use std::sync::Arc;

pub type PluginState = Arc<PluginRegistry>;

pub fn create_plugin_router() -> Router {
    let registry = setup_plugins();

    Router::new()
        .route("/plugin/*path", get(plugin_handler))
        .with_state(registry)
}

pub async fn plugin_handler(
    Path(path): Path<String>,
    State(registry): State<PluginState>,
) -> Response<String> {
    let full_path = format!("/plugin/{}", path);

    if let Some(plugin) = registry.get_plugin(&full_path) {
        (plugin.handler)()
    } else {
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body("Plugin not found".to_string())
            .unwrap()
    }
}

fn setup_plugins() -> Arc<PluginRegistry> {
    let registry = Arc::new(PluginRegistry::new());

    // Register Value Lattice plugin
    registry.register_plugin(WebPlugin {
        name: "Value Lattice".to_string(),
        path: "/plugin/value-lattice".to_string(),
        description: "Zero ontology lattice dashboard with dynamic metrics".to_string(),
        icon: "🔗".to_string(),
        handler: Arc::new(Box::new(|| generate_value_lattice_html())),
    });

    // Register Process Monitor plugin
    registry.register_plugin(WebPlugin {
        name: "Process Monitor".to_string(),
        path: "/plugin/process-monitor".to_string(),
        description: "Real-time indexer process monitoring".to_string(),
        icon: "📊".to_string(),
        handler: Arc::new(Box::new(|| {
            let monitor = ProcessMonitor::new();
            monitor.generate_html_report()
        })),
    });

    registry
}

fn generate_value_lattice_html() -> Response<String> {
    // Implementation moved from value_lattice_handler
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
