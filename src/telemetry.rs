use anyhow::Result;
use opentelemetry::{global, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::{trace, Resource};
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{prelude::*, EnvFilter, Registry};

pub fn init_telemetry() -> Result<()> {
    // Redirect all `log`'s events to our subscriber
    LogTracer::init()?;

    // Set the global propagator to be a TraceContextPropagator
    global::set_text_map_propagator(TraceContextPropagator::new());

    // Create a new OpenTelemetry pipeline
    let otlp_exporter = opentelemetry_otlp::new_exporter().tonic().with_env();

    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(otlp_exporter)
        .with_trace_config(
            trace::config().with_resource(Resource::new(vec![KeyValue::new(
                "service.name",
                "jamey-3-backend",
            )])),
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)?;

    // Create a tracing layer with the configured tracer
    let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    // Create a bunyan formatting layer for structured logging
    let formatting_layer = BunyanFormattingLayer::new("jamey-3".into(), std::io::stdout);

    // Get the RUST_LOG environment variable
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    // Create a subscriber that combines all the layers
    let subscriber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
        .with(telemetry_layer);

    set_global_default(subscriber)?;

    Ok(())
}