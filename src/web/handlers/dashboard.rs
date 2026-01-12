use crate::plugin_registry::PluginRegistry;
use axum::{
    http::{header, StatusCode},
    response::Response,
};
use std::sync::Arc;

pub async fn dashboard_handler(registry: Arc<PluginRegistry>) -> Response<String> {
    let navigation = registry.generate_navigation_html();
    let plugin_cards = registry.generate_dashboard_cards();

    let html = format!(
        r#"
<!DOCTYPE html>
<html>
<head>
    <title>ZOS Dashboard</title>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <style>
        body {{
            font-family: 'Courier New', monospace;
            background: #0a0a0a;
            color: #00ff00;
            margin: 0;
            padding: 0;
        }}
        .header {{
            background: linear-gradient(135deg, #001100, #003300);
            padding: 20px;
            text-align: center;
            border-bottom: 2px solid #00ff00;
        }}
        h1 {{
            color: #00ffff;
            text-shadow: 0 0 20px #00ffff;
            margin: 0;
            font-size: 2.5em;
        }}
        .nav-links {{
            margin: 20px 0;
            text-align: center;
        }}
        .nav-link {{
            color: #00ff00;
            text-decoration: none;
            margin: 0 15px;
            padding: 8px 16px;
            border: 1px solid #00ff00;
            border-radius: 4px;
            display: inline-block;
            transition: all 0.3s ease;
        }}
        .nav-link:hover {{
            background: #004400;
            box-shadow: 0 0 10px #00ff00;
        }}
        .dashboard-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
            gap: 20px;
            padding: 20px;
            max-width: 1200px;
            margin: 0 auto;
        }}
        .dashboard-card {{
            background: rgba(0, 20, 0, 0.8);
            border: 1px solid #00ff00;
            border-radius: 8px;
            padding: 20px;
            text-align: center;
            transition: all 0.3s ease;
        }}
        .dashboard-card:hover {{
            box-shadow: 0 0 20px rgba(0, 255, 0, 0.5);
            transform: translateY(-5px);
        }}
        .card-icon {{
            font-size: 3em;
            margin-bottom: 10px;
        }}
        .card-link {{
            color: #00ffff;
            text-decoration: none;
            font-weight: bold;
        }}
        .card-link:hover {{ color: #44ffff; }}
    </style>
</head>
<body>
    <div class="header">
        <h1>🌌 ZOS DASHBOARD</h1>
        <p>Zero Ontology System - Meta-Introspector Analysis Hub</p>
        <div class="nav-links">
            {}
        </div>
    </div>

    <div class="dashboard-grid">
        {}
    </div>
</body>
</html>
    "#,
        navigation, plugin_cards
    );

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(html)
        .unwrap()
}
