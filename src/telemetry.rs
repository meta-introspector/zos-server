#![allow(unused)]

use opentelemetry::{global, metrics::MeterProvider, trace::TraceError, KeyValue};
use opentelemetry_sdk::{
    metrics::MeterProviderBuilder,
    runtime,
    trace::{self, RandomIdGenerator, Sampler},
    Resource,
};
use tracing::{info, instrument};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_high_performance_telemetry() -> Result<(), TraceError> {
    // High-performance tracer with optimized settings
    let tracer = opentelemetry_jaeger::new_agent_pipeline()
        .with_service_name("zos-server")
        .with_trace_config(
            trace::config()
                .with_sampler(Sampler::TraceIdRatioBased(1.0)) // 100% sampling for dev
                .with_id_generator(RandomIdGenerator::default())
                .with_max_events_per_span(32)
                .with_max_attributes_per_span(16)
                .with_resource(Resource::new(vec![
                    KeyValue::new("service.name", "zos-server"),
                    KeyValue::new("service.version", "1.0.0"),
                ])),
        )
        .with_auto_split_batch(true)
        .install_batch(runtime::Tokio)?;

    // Zero-allocation metrics provider
    let meter_provider = MeterProviderBuilder::default()
        .with_resource(Resource::new(vec![KeyValue::new(
            "service.name",
            "zos-server",
        )]))
        .build();

    global::set_meter_provider(meter_provider);

    // Minimal overhead tracing subscriber
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "zos_server=info,tower_http=info".into()),
        )
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .init();

    info!("⚡ High-performance OpenTelemetry initialized");
    Ok(())
}

// Zero-allocation instrumentation macros
#[instrument(skip_all, fields(user_id, operation))]
pub async fn trace_user_operation<F, R>(user_id: &str, operation: &str, f: F) -> R
where
    F: std::future::Future<Output = R>,
{
    tracing::Span::current().record("user_id", user_id);
    tracing::Span::current().record("operation", operation);
    f.await
}

pub fn shutdown_telemetry() {
    global::shutdown_tracer_provider();
    global::shutdown_meter_provider();
}
