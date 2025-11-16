use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::time::{Duration, sleep};
use tracing::{info, warn, error};

/// Represents the shutdown state and coordination mechanisms
pub struct ShutdownCoordinator {
    shutdown_signal: Arc<AtomicBool>,
    shutdown_tx: broadcast::Sender<()>,
}

impl ShutdownCoordinator {
    /// Creates a new ShutdownCoordinator instance
    pub fn new() -> Self {
        let (shutdown_tx, _) = broadcast::channel(1);
        Self {
            shutdown_signal: Arc::new(AtomicBool::new(false)),
            shutdown_tx,
        }
    }

    /// Subscribes to shutdown notifications
    pub fn subscribe(&self) -> broadcast::Receiver<()> {
        self.shutdown_tx.subscribe()
    }

    /// Initiates the shutdown sequence
    pub async fn shutdown(&self) {
        info!("Initiating graceful shutdown sequence");
        self.shutdown_signal.store(true, Ordering::SeqCst);
        
        // Broadcast shutdown signal to all subscribers
        if let Err(e) = self.shutdown_tx.send(()) {
            error!("Failed to broadcast shutdown signal: {}", e);
        }
    }

    /// Returns whether shutdown has been initiated
    pub fn is_shutdown_requested(&self) -> bool {
        self.shutdown_signal.load(Ordering::SeqCst)
    }

    /// Handles OS signals for graceful shutdown
    pub async fn handle_signals(coordinator: Arc<ShutdownCoordinator>) {
        let ctrl_c = tokio::signal::ctrl_c();
        
        #[cfg(unix)]
        let terminate = async {
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .expect("Failed to install SIGTERM handler")
                .recv()
                .await
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => {
                info!("Received Ctrl+C signal");
            }
            _ = terminate => {
                info!("Received SIGTERM signal");
            }
        }

        coordinator.shutdown().await;
    }
}

/// Represents a component that can be gracefully shut down
#[async_trait::async_trait]
pub trait GracefulShutdown: Send + Sync {
    /// Name of the component for logging
    fn name(&self) -> &str;
    
    /// Performs graceful shutdown of the component
    async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// Coordinates shutdown of multiple components
pub struct ComponentShutdownManager {
    components: Vec<Arc<dyn GracefulShutdown>>,
    timeout: Duration,
}

impl ComponentShutdownManager {
    /// Creates a new ComponentShutdownManager with the specified timeout
    pub fn new(timeout: Duration) -> Self {
        Self {
            components: Vec::new(),
            timeout,
        }
    }

    /// Registers a component for shutdown coordination
    pub fn register_component(&mut self, component: Arc<dyn GracefulShutdown>) {
        self.components.push(component);
    }

    /// Executes graceful shutdown of all registered components
    pub async fn shutdown_all(&self) {
        info!("Starting component shutdown sequence");

        for component in &self.components {
            let component_name = component.name();
            info!("Shutting down component: {}", component_name);

            match tokio::time::timeout(self.timeout, component.shutdown()).await {
                Ok(Ok(_)) => {
                    info!("Successfully shut down component: {}", component_name);
                }
                Ok(Err(e)) => {
                    error!("Error shutting down component {}: {}", component_name, e);
                }
                Err(_) => {
                    warn!("Timeout while shutting down component: {}", component_name);
                }
            }

            // Brief pause between components
            sleep(Duration::from_millis(100)).await;
        }

        info!("Component shutdown sequence complete");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    struct TestComponent {
        name: String,
        shutdown_duration: Duration,
    }

    #[async_trait::async_trait]
    impl GracefulShutdown for TestComponent {
        fn name(&self) -> &str {
            &self.name
        }

        async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            sleep(self.shutdown_duration).await;
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_shutdown_coordination() {
        let coordinator = ShutdownCoordinator::new();
        let mut receiver = coordinator.subscribe();

        // Trigger shutdown
        coordinator.shutdown().await;

        // Verify shutdown signal was received
        assert!(receiver.recv().await.is_ok());
        assert!(coordinator.is_shutdown_requested());
    }

    #[tokio::test]
    async fn test_component_shutdown() {
        let mut manager = ComponentShutdownManager::new(Duration::from_secs(5));
        
        let fast_component = Arc::new(TestComponent {
            name: "FastComponent".to_string(),
            shutdown_duration: Duration::from_millis(100),
        });

        let slow_component = Arc::new(TestComponent {
            name: "SlowComponent".to_string(),
            shutdown_duration: Duration::from_secs(1),
        });

        manager.register_component(fast_component);
        manager.register_component(slow_component);

        // Should complete within timeout
        manager.shutdown_all().await;
    }
}