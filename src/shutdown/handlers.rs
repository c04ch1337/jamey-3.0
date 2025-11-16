use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{info, warn, error};
use crate::shutdown::GracefulShutdown;

/// Cache system shutdown handler
pub struct CacheShutdownHandler {
    name: String,
    cache: Arc<crate::cache::Cache>,
}

impl CacheShutdownHandler {
    pub fn new(name: String, cache: Arc<crate::cache::Cache>) -> Self {
        Self { name, cache }
    }
}

#[async_trait::async_trait]
impl GracefulShutdown for CacheShutdownHandler {
    fn name(&self) -> &str {
        &self.name
    }

    async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting cache system shutdown");
        
        // Flush any pending writes
        self.cache.flush().await?;
        
        // Clear volatile memory
        self.cache.clear().await?;
        
        info!("Cache system shutdown complete");
        Ok(())
    }
}

/// Async channel shutdown handler
pub struct ChannelShutdownHandler {
    name: String,
    shutdown_tx: broadcast::Sender<()>,
}

impl ChannelShutdownHandler {
    pub fn new(name: String, shutdown_tx: broadcast::Sender<()>) -> Self {
        Self { name, shutdown_tx }
    }
}

#[async_trait::async_trait]
impl GracefulShutdown for ChannelShutdownHandler {
    fn name(&self) -> &str {
        &self.name
    }

    async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting async channel shutdown");
        
        // Notify all channel subscribers
        if let Err(e) = self.shutdown_tx.send(()) {
            warn!("Failed to send shutdown signal to all subscribers: {}", e);
        }
        
        info!("Async channel shutdown complete");
        Ok(())
    }
}

/// Context system shutdown handler
pub struct ContextShutdownHandler {
    name: String,
    context_manager: Arc<crate::context::ContextManager>,
}

impl ContextShutdownHandler {
    pub fn new(name: String, context_manager: Arc<crate::context::ContextManager>) -> Self {
        Self { name, context_manager }
    }
}

#[async_trait::async_trait]
impl GracefulShutdown for ContextShutdownHandler {
    fn name(&self) -> &str {
        &self.name
    }

    async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting context system shutdown");
        
        // Save current context state
        self.context_manager.save_state().await?;
        
        // Clear active contexts
        self.context_manager.clear_all().await?;
        
        info!("Context system shutdown complete");
        Ok(())
    }
}

/// Monitoring system shutdown handler
pub struct MonitoringShutdownHandler {
    name: String,
    metrics_reporter: Arc<crate::monitoring::MetricsReporter>,
}

impl MonitoringShutdownHandler {
    pub fn new(name: String, metrics_reporter: Arc<crate::monitoring::MetricsReporter>) -> Self {
        Self { name, metrics_reporter }
    }
}

#[async_trait::async_trait]
impl GracefulShutdown for MonitoringShutdownHandler {
    fn name(&self) -> &str {
        &self.name
    }

    async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting monitoring system shutdown");
        
        // Flush any pending metrics
        self.metrics_reporter.flush_metrics().await?;
        
        // Stop metric collection
        self.metrics_reporter.stop_collection().await?;
        
        info!("Monitoring system shutdown complete");
        Ok(())
    }
}

/// Database connection shutdown handler
pub struct DatabaseShutdownHandler {
    name: String,
    pool: sqlx::Pool<sqlx::Sqlite>,
}

impl DatabaseShutdownHandler {
    pub fn new(name: String, pool: sqlx::Pool<sqlx::Sqlite>) -> Self {
        Self { name, pool }
    }
}

#[async_trait::async_trait]
impl GracefulShutdown for DatabaseShutdownHandler {
    fn name(&self) -> &str {
        &self.name
    }

    async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting database connection shutdown");
        
        // Close the connection pool
        self.pool.close().await;
        
        info!("Database connection shutdown complete");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::sync::broadcast;

    #[tokio::test]
    async fn test_channel_shutdown() {
        let (tx, mut rx1) = broadcast::channel(1);
        let mut rx2 = tx.subscribe();

        let handler = ChannelShutdownHandler::new("TestChannel".to_string(), tx);
        
        // Start two receivers
        let recv1 = tokio::spawn(async move {
            rx1.recv().await.is_ok()
        });
        
        let recv2 = tokio::spawn(async move {
            rx2.recv().await.is_ok()
        });

        // Trigger shutdown
        handler.shutdown().await.unwrap();

        // Both receivers should get the shutdown signal
        assert!(recv1.await.unwrap());
        assert!(recv2.await.unwrap());
    }

    // Add more tests for other handlers as needed
}