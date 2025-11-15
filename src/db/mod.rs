use sqlx::{sqlite::{SqliteConnectOptions, SqlitePoolOptions}, SqlitePool, Row};
use std::str::FromStr;
use std::time::{Duration, Instant};
use tracing::{info, warn, error, debug};
use std::sync::Arc;
use tokio::sync::RwLock;
use metrics::{counter, histogram, gauge};
use crate::config::DatabaseConfig;

pub mod operations;
pub use operations::{DatabaseOperations, TableStats, MemoryRecord};

/// Database performance metrics
#[derive(Debug, Clone)]
pub struct DbMetrics {
    pub query_count: Arc<RwLock<u64>>,
    pub total_query_time: Arc<RwLock<Duration>>,
    pub active_connections: Arc<RwLock<u32>>,
    pub pool_size: Arc<RwLock<u32>>,
}

impl DbMetrics {
    pub fn new() -> Self {
        Self {
            query_count: Arc::new(RwLock::new(0)),
            total_query_time: Arc::new(RwLock::new(Duration::ZERO)),
            active_connections: Arc::new(RwLock::new(0)),
            pool_size: Arc::new(RwLock::new(0)),
        }
    }

    pub async fn record_query(&self, duration: Duration) {
        *self.query_count.write().await += 1;
        *self.total_query_time.write().await += duration;
        
        // Update metrics
        counter!("db.queries_total", 1);
        histogram!("db.query_duration_seconds", duration.as_secs_f64());
    }

    pub async fn update_pool_stats(&self, pool: &SqlitePool) {
        *self.active_connections.write().await = pool.size() - pool.num_idle();
        *self.pool_size.write().await = pool.size();
        
        gauge!("db.active_connections", (pool.size() - pool.num_idle()) as f64);
        gauge!("db.pool_size", pool.size() as f64);
        gauge!("db.idle_connections", pool.num_idle() as f64);
    }

    pub async fn get_avg_query_time(&self) -> Duration {
        let count = *self.query_count.read().await;
        let total = *self.total_query_time.read().await;
        
        if count > 0 {
            total / count as u32
        } else {
            Duration::ZERO
        }
    }
}


/// Get the database path, creating the data directory if needed
pub fn get_db_path() -> anyhow::Result<std::path::PathBuf> {
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
    init_db_with_config(DatabaseConfig::default()).await
}

/// Initialize the SQLite database with custom configuration
pub async fn init_db_with_config(config: DatabaseConfig) -> anyhow::Result<SqlitePool> {
    let db_path = get_db_path()?;
    info!("Connecting to database at: {} with {} max connections",
          db_path.display(), config.max_connections);

    // Use SqliteConnectOptions for more control
    let mut connect_options = SqliteConnectOptions::from_str(&format!("sqlite:{}", db_path.display()))?
        .create_if_missing(true)
        .busy_timeout(config.connect_timeout());

    // Configure journal mode based on config
    if config.enable_wal {
        connect_options = connect_options.journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);
    }

    connect_options = connect_options.synchronous(sqlx::sqlite::SqliteSynchronous::Normal);

    let pool = SqlitePoolOptions::new()
        .max_connections(config.max_connections)
        .acquire_timeout(config.connect_timeout())
        .idle_timeout(config.idle_timeout())
        .max_lifetime(config.max_lifetime())
        .connect_with(connect_options)
        .await?;

    // Run migrations
    let migrate_start = Instant::now();
    sqlx::migrate!("./migrations").run(&pool).await?;
    let migrate_duration = migrate_start.elapsed();
    
    info!("Database initialized and migrations applied in {:?}", migrate_duration);
    counter!("db.migrations_total", 1);
    histogram!("db.migration_duration_seconds", migrate_duration.as_secs_f64());

    // Performance optimizations
    if config.enable_wal {
        sqlx::query("PRAGMA journal_mode = WAL")
            .execute(&pool)
            .await?;
    }
    
    sqlx::query("PRAGMA synchronous = NORMAL")
        .execute(&pool)
        .await?;
    
    sqlx::query("PRAGMA cache_size = 10000")
        .execute(&pool)
        .await?;
    
    sqlx::query("PRAGMA temp_store = MEMORY")
        .execute(&pool)
        .await?;

    info!("Database performance optimizations applied");
    Ok(pool)
}

/// Get a database connection pool
pub async fn get_pool() -> anyhow::Result<SqlitePool> {
    get_pool_with_config(DatabaseConfig::default()).await
}

/// Get a database connection pool with custom configuration
pub async fn get_pool_with_config(config: DatabaseConfig) -> anyhow::Result<SqlitePool> {
    let db_path = get_db_path()?;
    let connect_options = SqliteConnectOptions::from_str(&format!("sqlite:{}", db_path.display()))?
        .busy_timeout(config.connect_timeout());
    
    let pool = SqlitePoolOptions::new()
        .max_connections(config.max_connections)
        .acquire_timeout(config.connect_timeout())
        .connect_with(connect_options)
        .await?;
    Ok(pool)
}

/// Execute a query with performance monitoring
pub async fn execute_with_metrics<T>(
    pool: &SqlitePool,
    query: &str,
    metrics: &DbMetrics,
    operation: impl FnOnce(&SqlitePool) -> sqlx::Result<T>
) -> anyhow::Result<T> {
    let start = Instant::now();
    
    let result = operation(pool);
    let duration = start.elapsed();
    
    metrics.record_query(duration).await;
    metrics.update_pool_stats(pool).await;
    
    debug!("Query executed in {:?}: {}", duration, query);
    
    match result {
        Ok(value) => Ok(value),
        Err(e) => {
            error!("Query failed after {:?}: {} - Error: {}", duration, query, e);
            counter!("db.query_errors_total", 1);
            Err(anyhow::anyhow!("Database query failed: {}", e))
        }
    }
}

/// Database health check
pub async fn health_check(pool: &SqlitePool) -> anyhow::Result<DatabaseHealth> {
    let start = Instant::now();
    
    // Test basic connectivity
    let row = sqlx::query("SELECT 1 as test")
        .fetch_one(pool)
        .await?;
    
    let test_value: i32 = row.try_get("test")?;
    
    if test_value != 1 {
        return Err(anyhow::anyhow!("Database health check failed: unexpected test value"));
    }
    
    // Get pool statistics
    let pool_size = pool.size();
    let idle_connections = pool.num_idle();
    let active_connections = pool_size - idle_connections;
    
    // Check database size and integrity
    let size_row = sqlx::query("SELECT page_count * page_size as size FROM pragma_page_count(), pragma_page_size()")
        .fetch_one(pool)
        .await?;
    
    let db_size: i64 = size_row.try_get("size")?;
    
    let integrity_check = sqlx::query("PRAGMA integrity_check")
        .fetch_one(pool)
        .await?;
    
    let integrity_result: String = integrity_check.try_get("integrity_check")?;
    
    let duration = start.elapsed();
    
    Ok(DatabaseHealth {
        is_healthy: integrity_result == "ok",
        active_connections,
        idle_connections,
        total_connections: pool_size,
        database_size_bytes: db_size,
        response_time_ms: duration.as_millis() as u64,
        integrity_check: integrity_result,
    })
}

/// Database health information
#[derive(Debug, Clone)]
pub struct DatabaseHealth {
    pub is_healthy: bool,
    pub active_connections: u32,
    pub idle_connections: u32,
    pub total_connections: u32,
    pub database_size_bytes: i64,
    pub response_time_ms: u64,
    pub integrity_check: String,
}

/// Transaction wrapper for safe database operations
pub async fn with_transaction<F, R>(
    pool: &SqlitePool,
    metrics: &DbMetrics,
    operation: F,
) -> anyhow::Result<R>
where
    F: FnOnce(&mut sqlx::Transaction<sqlx::Sqlite>) -> anyhow::Result<R>,
{
    let start = Instant::now();
    let mut tx = pool.begin().await?;
    
    let result = operation(&mut tx);
    
    match result {
        Ok(value) => {
            tx.commit().await?;
            let duration = start.elapsed();
            metrics.record_query(duration).await;
            debug!("Transaction committed in {:?}", duration);
            counter!("db.transactions_committed_total", 1);
            Ok(value)
        }
        Err(e) => {
            tx.rollback().await?;
            let duration = start.elapsed();
            metrics.record_query(duration).await;
            warn!("Transaction rolled back after {:?}: {}", duration, e);
            counter!("db.transactions_rolled_back_total", 1);
            Err(e)
        }
    }
}

