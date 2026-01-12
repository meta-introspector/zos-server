use crate::cicd_dashboard::CICDDashboard;
use crate::meta_introspector::MetaIntrospectorManager;
use crate::plugin_registry::{PluginRegistry, WebPlugin};
use crate::process_monitor::ProcessMonitor;
use crate::value_lattice_manager::ValueLatticeManager;
use crate::zos_api::ZosApi;
use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::Response,
    routing::{get, post},
    Router,
};
use std::sync::Arc;

pub type PluginState = Arc<PluginRegistry>;

pub fn create_plugin_router() -> Router {
    let registry = setup_plugins();

    Router::new()
        .route("/plugin/*path", get(plugin_handler))
        .route("/api/projects", get(api_projects))
        .route("/api/lattice/status", get(api_lattice_status))
        .route("/api/lattice/start", post(api_lattice_start))
        .route("/api/owners", get(api_top_owners))
        .route("/api/git-analysis", get(api_git_analysis))
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

    // Register Value Lattice Manager plugin
    registry.register_plugin(WebPlugin {
        name: "Value Lattice Manager".to_string(),
        path: "/plugin/lattice-manager".to_string(),
        description: "Manage value lattice indexing process".to_string(),
        icon: "⚙️".to_string(),
        handler: Arc::new(Box::new(|| {
            let manager = ValueLatticeManager::new(
                "/home/mdupont/zombie_driver2/target/release/value_lattice_server".to_string()
            );
            let status = manager.status();

            let html = format!(r#"
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
        <button onclick="fetch('/api/lattice/compile', {{method: 'POST'}}).then(() => location.reload())">Compile</button>
    </div>
</body>
</html>
            "#, 
                status.status.clone(),
                status.status,
                status.pid,
                status.restart_count,
                status.started_at,
                status.last_error.map_or(String::new(), |e| format!("<p>Error: {}</p>", e))
            );

            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                .body(html)
                .unwrap()
        })),
    });

    // Register Process Monitor plugin
    registry.register_plugin(WebPlugin {
        name: "Process Monitor".to_string(),
        path: "/plugin/process-monitor".to_string(),
        description: "Real-time indexer process monitoring".to_string(),
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

    // Register Meta-Introspector plugin
    registry.register_plugin(WebPlugin {
        name: "Meta-Introspector".to_string(),
        path: "/plugin/meta-introspector".to_string(),
        description: "300+ repository ontology collection dashboard".to_string(),
        icon: "🔍".to_string(),
        handler: Arc::new(Box::new(|| {
            let mut manager =
                MetaIntrospectorManager::new("/mnt/data1/meta-introspector".to_string());

            // Scan repositories (this might be slow, consider caching)
            if let Err(e) = manager.scan_repositories() {
                let error_html = format!(
                    r#"
<!DOCTYPE html>
<html>
<head><title>Meta-Introspector Error</title></head>
<body style="font-family: monospace; background: #0a0a0a; color: #ff0000; padding: 20px;">
    <h1>🔍 Meta-Introspector Error</h1>
    <p>Failed to scan repositories: {}</p>
</body>
</html>
                "#,
                    e
                );

                return Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                    .body(error_html)
                    .unwrap();
            }

            let html = manager.generate_html_dashboard();
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                .body(html)
                .unwrap()
        })),
    });

    // Register CI/CD Dashboard plugin
    registry.register_plugin(WebPlugin {
        name: "CI/CD Dashboard".to_string(),
        path: "/plugin/cicd".to_string(),
        description: "Project status, git branches, lattice processing, rustc compilation"
            .to_string(),
        icon: "🚀".to_string(),
        handler: Arc::new(Box::new(|| {
            let mut meta_manager =
                MetaIntrospectorManager::new("/mnt/data1/meta-introspector".to_string());

            let mut dashboard = CICDDashboard::new();

            // Scan repositories and build dashboard
            match meta_manager.scan_repositories() {
                Ok(_) => {
                    let repos = meta_manager.get_repos();
                    if let Err(e) = dashboard.scan_projects(&repos) {
                        let error_html = format!(
                            r#"
<!DOCTYPE html>
<html>
<head><title>CI/CD Dashboard Error</title></head>
<body style="font-family: monospace; background: #0a0a0a; color: #ff0000; padding: 20px;">
    <h1>🚀 CI/CD Dashboard Error</h1>
    <p>Failed to scan projects: {}</p>
</body>
</html>
                        "#,
                            e
                        );

                        return Response::builder()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                            .body(error_html)
                            .unwrap();
                    }
                }
                Err(e) => {
                    let error_html = format!(
                        r#"
<!DOCTYPE html>
<html>
<head><title>CI/CD Dashboard Error</title></head>
<body style="font-family: monospace; background: #0a0a0a; color: #ff0000; padding: 20px;">
    <h1>🚀 CI/CD Dashboard Error</h1>
    <p>Failed to scan repositories: {}</p>
</body>
</html>
                    "#,
                        e
                    );

                    return Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                        .body(error_html)
                        .unwrap();
                }
            }

            let html = dashboard.generate_dashboard_html();
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                .body(html)
                .unwrap()
        })),
    });

    registry
}

async fn api_projects() -> Response<String> {
    match ZosApi::get_projects_json() {
        Ok(json) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/json")
            .body(json)
            .unwrap(),
        Err(e) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header(header::CONTENT_TYPE, "application/json")
            .body(format!(r#"{{"success":false,"error":"{}"}}"#, e))
            .unwrap(),
    }
}

async fn api_lattice_status() -> Response<String> {
    match ZosApi::get_lattice_status_json() {
        Ok(json) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/json")
            .body(json)
            .unwrap(),
        Err(e) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header(header::CONTENT_TYPE, "application/json")
            .body(format!(r#"{{"success":false,"error":"{}"}}"#, e))
            .unwrap(),
    }
}

async fn api_lattice_start() -> Response<String> {
    match ZosApi::start_lattice() {
        Ok(json) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/json")
            .body(json)
            .unwrap(),
        Err(e) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header(header::CONTENT_TYPE, "application/json")
            .body(format!(r#"{{"success":false,"error":"{}"}}"#, e))
            .unwrap(),
    }
}

async fn api_top_owners() -> Response<String> {
    match ZosApi::get_top_owners_json() {
        Ok(json) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/json")
            .body(json)
            .unwrap(),
        Err(e) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header(header::CONTENT_TYPE, "application/json")
            .body(format!(r#"{{"success":false,"error":"{}"}}"#, e))
            .unwrap(),
    }
}

async fn api_git_analysis() -> Response<String> {
    match ZosApi::get_git_analysis_json() {
        Ok(json) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/json")
            .body(json)
            .unwrap(),
        Err(e) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header(header::CONTENT_TYPE, "application/json")
            .body(format!(r#"{{"success":false,"error":"{}"}}"#, e))
            .unwrap(),
    }
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
