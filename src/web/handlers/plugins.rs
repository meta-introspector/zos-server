use crate::plugin_registry::{PluginRegistry, WebPlugin};
use crate::web::handlers::{
    process_monitor_plugin::process_monitor_plugin, value_lattice_plugin::value_lattice_plugin,
};
use crate::zos_api::ZosApi;
use axum::{
    http::{header, StatusCode},
    response::Response,
    routing::get,
    Router,
};
use std::sync::Arc;

pub fn create_plugin_routes() -> Router {
    Router::new()
        .route("/plugins/:name", get(handle_plugin))
        .route("/api/projects", get(api_projects))
        .route("/api/lattice/status", get(api_lattice_status))
        .route("/api/lattice/start", get(api_lattice_start))
        .route("/api/owners", get(api_owners))
        .route("/api/top-owners", get(api_top_owners))
        .route("/api/git-analysis", get(api_git_analysis))
        .route("/api/repo-status", get(api_repo_status))
        .route("/api/unpushed-summary", get(api_unpushed_summary))
}

pub async fn handle_plugin(
    axum::extract::Path(name): axum::extract::Path<String>,
) -> Response<String> {
    let registry = setup_plugins();

    if let Some(plugin) = registry.get_plugin(&format!("/plugins/{}", name)) {
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
        path: "/plugins/value-lattice".to_string(),
        description: "Manage value lattice indexing process".to_string(),
        icon: "⚙️".to_string(),
        handler: Arc::new(Box::new(|| value_lattice_plugin())),
    });

    // Register Process Monitor plugin
    registry.register_plugin(WebPlugin {
        name: "Process Monitor".to_string(),
        path: "/plugins/process-monitor".to_string(),
        description: "Monitor system processes".to_string(),
        icon: "📊".to_string(),
        handler: Arc::new(Box::new(|| process_monitor_plugin())),
    });

    registry
}

// API handlers
pub async fn api_projects() -> Result<Response<String>, StatusCode> {
    match ZosApi::get_projects_json() {
        Ok(json) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/json")
            .body(json)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR),
        Err(e) => {
            eprintln!("Projects API error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn api_lattice_status() -> Result<Response<String>, StatusCode> {
    match ZosApi::get_lattice_status_json() {
        Ok(json) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/json")
            .body(json)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR),
        Err(e) => {
            eprintln!("Lattice status error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn api_lattice_start() -> Result<Response<String>, StatusCode> {
    match ZosApi::start_lattice() {
        Ok(json) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/json")
            .body(json)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR),
        Err(e) => {
            eprintln!("Lattice start error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn api_owners() -> Result<Response<String>, StatusCode> {
    match ZosApi::get_projects_json() {
        Ok(json) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/json")
            .body(json)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR),
        Err(e) => {
            eprintln!("Owners API error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn api_top_owners() -> Result<Response<String>, StatusCode> {
    match ZosApi::get_top_owners_json() {
        Ok(json) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/json")
            .body(json)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR),
        Err(e) => {
            eprintln!("Top owners error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn api_git_analysis() -> Result<Response<String>, StatusCode> {
    match ZosApi::get_git_analysis_json() {
        Ok(json) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/json")
            .body(json)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR),
        Err(e) => {
            eprintln!("Git analysis error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn api_repo_status() -> Result<Response<String>, StatusCode> {
    match ZosApi::get_repo_status_json() {
        Ok(json) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/json")
            .body(json)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR),
        Err(e) => {
            eprintln!("Repo status error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn api_unpushed_summary() -> Result<Response<String>, StatusCode> {
    match ZosApi::get_unpushed_summary_json() {
        Ok(json) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/json")
            .body(json)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR),
        Err(e) => {
            eprintln!("Unpushed summary error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
