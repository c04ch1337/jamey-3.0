use sqlx::{sqlite::{SqliteConnectOptions, SqlitePoolOptions}, SqlitePool};
use std::str::FromStr;
use tracing::info;

/// Get the database path, creating the data directory if needed
fn get_db_path() -> anyhow::Result<std::path::PathBuf> {
    // Get the current directory
    let current_dir = std::env::current_dir()?;
    let data_dir = current_dir.join("data");
    let db_path = data_dir.join("jamey.db");
    
    // Ensure parent directory exists
    std::fs::create_dir_all(&data_dir)?;
    
    Ok(db_path)
}

/// Initialize the SQLite database and run migrations
pub async fn init_db() -> anyhow::Result<SqlitePool> {
    let db_path = get_db_path()?;
    info!("Connecting to database at: {}", db_path.display());

    // Use SqliteConnectOptions for more control
    let connect_options = SqliteConnectOptions::from_str(&format!("sqlite:{}", db_path.display()))?
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(connect_options)
        .await?;

    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await?;

    info!("Database initialized and migrations applied");
    Ok(pool)
}

/// Get a database connection pool
pub async fn get_pool() -> anyhow::Result<SqlitePool> {
    let db_path = get_db_path()?;
    let connect_options = SqliteConnectOptions::from_str(&format!("sqlite:{}", db_path.display()))?;
    
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(connect_options)
        .await?;
    Ok(pool)
}

