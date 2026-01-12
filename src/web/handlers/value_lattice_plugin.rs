use crate::value_lattice_manager::ValueLatticeManager;
use axum::{
    http::{header, StatusCode},
    response::Response,
};

pub fn value_lattice_plugin() -> Response<String> {
    let manager = ValueLatticeManager::new(
        "/home/mdupont/zombie_driver2/target/release/value_lattice_server".to_string(),
    );
    let status = manager.status();

    let html = format!(
        r#"
<!DOCTYPE html>
<html>
<head>
    <title>Value Lattice Manager</title>
    <style>
        body {{ font-family: monospace; background: #0a0a0a; color: #00ff00; padding: 20px; }}
        .status {{ padding: 10px; margin: 10px 0; border: 1px solid #00ff00; }}
        .running {{ border-color: #00ff00; }}
        .stopped {{ border-color: #ff0000; }}
        .failed {{ border-color: #ff8800; }}
        button {{ background: #004400; color: #00ff00; border: 1px solid #00ff00; padding: 10px; margin: 5px; }}
    </style>
</head>
<body>
    <h1>⚙️ Value Lattice Manager</h1>
    <div class="status {}">
        <h3>Process Status: {}</h3>
        <p>PID: {:?}</p>
        <p>Restart Count: {}</p>
        <p>Started: {:?}</p>
        {}
    </div>
    
    <div>
        <button onclick="fetch('/api/lattice/start', {{method: 'POST'}}).then(() => location.reload())">Start</button>
        <button onclick="fetch('/api/lattice/stop', {{method: 'POST'}}).then(() => location.reload())">Stop</button>
        <button onclick="fetch('/api/lattice/restart', {{method: 'POST'}}).then(() => location.reload())">Restart</button>
    </div>
</body>
</html>
    "#,
        status.status.clone(),
        status.status,
        status.pid,
        status.restart_count,
        status.started_at,
        status
            .last_error
            .map_or(String::new(), |e| format!("<p>Error: {}</p>", e))
    );

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(html)
        .unwrap()
}
