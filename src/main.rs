use anyhow::Result;
use jamey_3::api::create_app;
use jamey_3::config::Config;
use jamey_3::db::init_db;
use tracing::info;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("Starting Jamey 3.0 - General & Guardian");

    // Load configuration (including OpenRouter API key)
    // This is optional - the app can run without it, but LLM features won't work
    let _config = match Config::from_env() {
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

    // Initialize database
    init_db().await?;
    info!("Database initialized");

    // Create and run the Axum app
    let app = create_app().await?;
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    info!("Server listening on http://0.0.0.0:3000");

    axum::serve(listener, app).await?;

    Ok(())
}

