// Static file server for WASM GUI
use axum::Router;
use tower_http::services::ServeDir;

pub fn create_static_routes() -> Router {
    Router::new().nest_service("/", ServeDir::new("www"))
}
