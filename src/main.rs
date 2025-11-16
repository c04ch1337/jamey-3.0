use jamey::api::create_app;
use jamey::db::init_db;
use jamey::telemetry;

use tracing::info;
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize telemetry
    if let Err(e) = telemetry::init_telemetry() {
        eprintln!("Failed to initialize telemetry: {}", e);
    }

    info!("Starting Jamey 3.0...");

    // Initialize database
    let pool = init_db().await?;

    // Create Axum app
    let app = create_app(pool, None, None).await?;

    // Get bind address
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    info!("Listening on {}", addr);

    // Start server
    axum::serve(
        tokio::net::TcpListener::bind(&addr).await?,
        app.into_make_service(),
    )
    .await?;

    info!("Server shutdown complete");
    Ok(())
}
