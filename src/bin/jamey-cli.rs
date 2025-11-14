use anyhow::Result;
use jamey_3::cli::ChatCLI;
use jamey_3::config::Config;
use jamey_3::conscience::ConscienceEngine;
use jamey_3::db::init_db;
use jamey_3::memory::MemorySystem;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::info;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("Starting Jamey 3.0 CLI - General & Guardian");

    // Load configuration (required for CLI)
    let config = match Config::from_env_required() {
        Ok(cfg) => {
            cfg.validate()?;
            info!("OpenRouter API key loaded");
            Arc::new(cfg)
        }
        Err(e) => {
            eprintln!("\n❌ Configuration Error: {}\n", e);
            eprintln!("📝 To fix this:");
            eprintln!("   1. Create a .env file in the project root");
            eprintln!("   2. Add: OPENROUTER_API_KEY=your-api-key-here");
            eprintln!("   3. Get your key from: https://openrouter.ai/keys\n");
            eprintln!("   See API_KEY_SETUP.md for detailed instructions.\n");
            return Err(e);
        }
    };

    // Initialize database (optional, but good to have)
    if let Err(e) = init_db().await {
        tracing::warn!("Database initialization failed: {}. Continuing without database.", e);
    } else {
        info!("Database initialized");
    }

    // Initialize conscience engine
    let conscience = Arc::new(ConscienceEngine::new());
    info!("Conscience Engine initialized");

    // Initialize memory system
    let data_dir = PathBuf::from("data/memory");
    let memory = match MemorySystem::new(data_dir).await {
        Ok(mem) => {
            info!("Memory System initialized");
            Arc::new(mem)
        }
        Err(e) => {
            eprintln!("⚠️  Warning: Memory system initialization failed: {}", e);
            eprintln!("   Continuing without memory system...\n");
            // Create a dummy memory system - this is not ideal but allows CLI to run
            return Err(anyhow::anyhow!("Memory system required for CLI"));
        }
    };

    // Create and run CLI
    let mut cli = ChatCLI::new(config, conscience, memory);
    cli.run().await?;

    Ok(())
}

