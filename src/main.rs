use anyhow::Result;
use jamey_3::api::create_app;
use jamey_3::config::Config;
use jamey_3::db::init_db_with_config;
use opentelemetry::{global, sdk::propagation::TraceContextPropagator};
use tracing::info;
use tracing_subscriber::{prelude::*, EnvFilter, Registry};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use opentelemetry_otlp::WithExportConfig;
use jamey_3::telemetry::init_telemetry;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing and logging
    init_telemetry()?;

    info!("Starting Jamey 3.0 - General & Guardian");

    // Load configuration (including OpenRouter API key)
    // This is optional - the app can run without it, but LLM features won't work
    let config = match Config::from_env() {
        Ok(Some(cfg)) => {
            cfg.validate()?;
            info!("OpenRouter API key loaded - LLM features available");
            Some(cfg)
        }
        Ok(None) => {
            info!("OpenRouter API key not set - LLM features unavailable");
            info!("To enable LLM features, create a .env file with OPENROUTER_API_KEY=your-key");
            None
        }
        Err(e) => {
            eprintln!("\n⚠️  Configuration Warning: {}\n", e);
            None
        }
    };

    // Initialize database with configuration
    let db_config = config.as_ref().map(|c| c.database.clone()).unwrap_or_default();
    init_db_with_config(db_config).await?;
    info!("Database initialized with configuration: max_connections={}, enable_metrics={}",
          db_config.max_connections, db_config.enable_metrics);

    // Create and run the Axum app
    let app = create_app().await?;
    
    // Get server binding configuration from environment
    let host = std::env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("SERVER_PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .unwrap_or(3000);
    
    let bind_addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    info!("Server listening on http://{}", bind_addr);

    axum::serve(listener, app).await?;

    Ok(())
}

