use crate::process_monitor_component::ProcessMonitorComponent;
use axum::{
    http::{header, StatusCode},
    response::Response,
};

pub fn process_monitor_plugin() -> Response<String> {
    let monitor = ProcessMonitorComponent::new();
    let processes = monitor.get_zos_user_processes();

    let mut process_list = String::new();
    for (name, info) in processes {
        process_list.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{:.1}%</td><td>{:.1} MB</td></tr>",
            name, info.pid, info.cpu_usage, info.memory_mb
        ));
    }

    let html = format!(
        r#"
<!DOCTYPE html>
<html>
<head>
    <title>Process Monitor</title>
    <style>
        body {{ font-family: monospace; background: #0a0a0a; color: #00ff00; padding: 20px; }}
        table {{ border-collapse: collapse; width: 100%; }}
        th, td {{ border: 1px solid #00ff00; padding: 8px; text-align: left; }}
        th {{ background-color: #004400; }}
        .zos-process {{ background-color: #002200; }}
    </style>
</head>
<body>
    <h1>📊 Process Monitor</h1>
    <table>
        <tr>
            <th>Process</th>
            <th>PID</th>
            <th>CPU %</th>
            <th>Memory</th>
        </tr>
        {}
    </table>
    <p><button onclick="location.reload()">Refresh</button></p>
</body>
</html>
    "#,
        process_list
    );

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(html)
        .unwrap()
}
