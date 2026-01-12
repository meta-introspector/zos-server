// Re-export the modules for use by web.rs
pub mod dashboard;
pub mod plugins;

// Re-export the handler functions
pub use dashboard::dashboard_handler;
pub use plugins::create_plugin_router;

pub use dashboard::dashboard_handler;
pub use plugins::{create_plugin_router, plugin_handler};

use axum::{
    http::{header, StatusCode},
    response::Response,
};

pub async fn root_handler() -> Response<String> {
    let html = r#"
<!DOCTYPE html>
<html>
<head>
    <title>ZOS Server</title>
    <style>
        body { font-family: monospace; background: #0a0a0a; color: #00ff00; text-align: center; padding: 50px; }
        h1 { color: #00ffff; text-shadow: 0 0 10px #00ffff; }
        .links { margin: 30px 0; }
        .links a {
            color: #00ff00;
            text-decoration: none;
            margin: 0 20px;
            padding: 10px 20px;
            border: 1px solid #00ff00;
            border-radius: 4px;
            display: inline-block;
        }
        .links a:hover { background: #004400; }
    </style>
</head>
<body>
    <h1>🌌 ZOS SERVER</h1>
    <p>Zero Ontology System - Meta-Introspector Analysis Hub</p>
    <div class="links">
        <a href="/dashboard">📊 Dashboard</a>
        <a href="/health">💚 Health</a>
    </div>
</body>
</html>
    "#;

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(html.to_string())
        .unwrap()
}

pub async fn health_handler() -> Response<String> {
    let html = r#"
<!DOCTYPE html>
<html>
<head>
    <title>ZOS Health Check</title>
    <style>
        body { font-family: monospace; background: #0a0a0a; color: #00ff00; padding: 20px; }
        .status { color: #00ff00; font-size: 24px; text-align: center; }
    </style>
</head>
<body>
    <div class="status">✅ ZOS Server is healthy</div>
    <p>All systems operational</p>
</body>
</html>
    "#;

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(html.to_string())
        .unwrap()
}
