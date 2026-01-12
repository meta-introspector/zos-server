use std::sync::Arc;
use tracing::{debug, error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub struct TelemetryServer;

impl TelemetryServer {
    pub fn init() -> Result<(), String> {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "zos_server=debug,tower_http=debug,axum=debug".into()),
            )
            .with(tracing_subscriber::fmt::layer())
            .init();

        info!("🔍 Telemetry server initialized");
        Ok(())
    }

    pub fn log_request(method: &str, path: &str, status: u16) {
        info!("📡 {} {} -> {}", method, path, status);
    }

    pub fn log_error(component: &str, error: &str) {
        error!("❌ {}: {}", component, error);
    }

    pub fn log_plugin_error(plugin: &str, error: &str) {
        error!("🔌 Plugin '{}' error: {}", plugin, error);
    }
}
