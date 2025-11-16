use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, Instant};
use tracing::{info, warn, error};

/// Represents the health status of a component
#[derive(Debug, Clone)]
pub enum HealthStatus {
    Healthy,
    Degraded { reason: String },
    Unhealthy { reason: String },
}

/// Represents a health check result with additional metadata
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    pub status: HealthStatus,
    pub last_check: Instant,
    pub response_time: Duration,
}

/// Trait for components that can report their health status
#[async_trait::async_trait]
pub trait HealthCheck: Send + Sync {
    /// Name of the component
    fn name(&self) -> &str;
    
    /// Performs the health check
    async fn check_health(&self) -> HealthCheckResult;
}

/// Manages health checks for multiple components
pub struct HealthManager {
    components: RwLock<HashMap<String, Arc<dyn HealthCheck>>>,
    results: RwLock<HashMap<String, HealthCheckResult>>,
    check_interval: Duration,
}

impl HealthManager {
    /// Creates a new HealthManager with the specified check interval
    pub fn new(check_interval: Duration) -> Self {
        Self {
            components: RwLock::new(HashMap::new()),
            results: RwLock::new(HashMap::new()),
            check_interval,
        }
    }

    /// Registers a component for health monitoring
    pub async fn register_component(&self, component: Arc<dyn HealthCheck>) {
        let name = component.name().to_string();
        let mut components = self.components.write().await;
        components.insert(name, component);
    }

    /// Starts the health check monitoring loop
    pub async fn start_monitoring(&self) {
        info!("Starting health check monitoring");
        
        loop {
            self.check_all_components().await;
            tokio::time::sleep(self.check_interval).await;
        }
    }

    /// Performs health checks on all registered components
    async fn check_all_components(&self) {
        let components = self.components.read().await;
        let mut results = self.results.write().await;

        for (name, component) in components.iter() {
            let result = component.check_health().await;
            
            match &result.status {
                HealthStatus::Healthy => {
                    info!(
                        "Health check passed for {}: response_time={}ms",
                        name,
                        result.response_time.as_millis()
                    );
                }
                HealthStatus::Degraded { reason } => {
                    warn!(
                        "Component {} is degraded: {} (response_time={}ms)",
                        name,
                        reason,
                        result.response_time.as_millis()
                    );
                }
                HealthStatus::Unhealthy { reason } => {
                    error!(
                        "Component {} is unhealthy: {} (response_time={}ms)",
                        name,
                        reason,
                        result.response_time.as_millis()
                    );
                }
            }

            results.insert(name.clone(), result);
        }
    }

    /// Gets the current health status of all components
    pub async fn get_health_status(&self) -> HashMap<String, HealthCheckResult> {
        self.results.read().await.clone()
    }

    /// Gets the overall system health status
    pub async fn get_system_health(&self) -> HealthStatus {
        let results = self.results.read().await;
        
        let mut unhealthy_components = Vec::new();
        let mut degraded_components = Vec::new();

        for (name, result) in results.iter() {
            match &result.status {
                HealthStatus::Unhealthy { reason } => {
                    unhealthy_components.push(format!("{}: {}", name, reason));
                }
                HealthStatus::Degraded { reason } => {
                    degraded_components.push(format!("{}: {}", name, reason));
                }
                HealthStatus::Healthy => {}
            }
        }

        if !unhealthy_components.is_empty() {
            HealthStatus::Unhealthy {
                reason: format!("Unhealthy components: {}", unhealthy_components.join(", "))
            }
        } else if !degraded_components.is_empty() {
            HealthStatus::Degraded {
                reason: format!("Degraded components: {}", degraded_components.join(", "))
            }
        } else {
            HealthStatus::Healthy
        }
    }
}

// Component-specific health checks
pub mod components {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    /// Cache system health check
    pub struct CacheHealthCheck {
        name: String,
        hit_count: AtomicU64,
        miss_count: AtomicU64,
    }

    impl CacheHealthCheck {
        pub fn new(name: String) -> Self {
            Self {
                name,
                hit_count: AtomicU64::new(0),
                miss_count: AtomicU64::new(0),
            }
        }

        pub fn record_hit(&self) {
            self.hit_count.fetch_add(1, Ordering::Relaxed);
        }

        pub fn record_miss(&self) {
            self.miss_count.fetch_add(1, Ordering::Relaxed);
        }
    }

    #[async_trait::async_trait]
    impl HealthCheck for CacheHealthCheck {
        fn name(&self) -> &str {
            &self.name
        }

        async fn check_health(&self) -> HealthCheckResult {
            let start = Instant::now();
            
            let hits = self.hit_count.load(Ordering::Relaxed);
            let misses = self.miss_count.load(Ordering::Relaxed);
            let total = hits + misses;

            let status = if total == 0 {
                HealthStatus::Healthy
            } else {
                let hit_rate = (hits as f64) / (total as f64);
                if hit_rate < 0.5 {
                    HealthStatus::Degraded {
                        reason: format!("Low cache hit rate: {:.2}%", hit_rate * 100.0)
                    }
                } else {
                    HealthStatus::Healthy
                }
            };

            HealthCheckResult {
                status,
                last_check: Instant::now(),
                response_time: start.elapsed(),
            }
        }
    }

    /// Memory system health check
    pub struct MemoryHealthCheck {
        name: String,
        allocation_threshold: usize,
    }

    impl MemoryHealthCheck {
        pub fn new(name: String, allocation_threshold: usize) -> Self {
            Self {
                name,
                allocation_threshold,
            }
        }
    }

    #[async_trait::async_trait]
    impl HealthCheck for MemoryHealthCheck {
        fn name(&self) -> &str {
            &self.name
        }

        async fn check_health(&self) -> HealthCheckResult {
            let start = Instant::now();
            
            // Get system memory info
            let sys_info = sys_info::mem_info().unwrap_or_default();
            let used_percent = ((sys_info.total - sys_info.free) as f64 / sys_info.total as f64) * 100.0;

            let status = if used_percent >= self.allocation_threshold as f64 {
                HealthStatus::Degraded {
                    reason: format!("High memory usage: {:.1}%", used_percent)
                }
            } else {
                HealthStatus::Healthy
            };

            HealthCheckResult {
                status,
                last_check: Instant::now(),
                response_time: start.elapsed(),
            }
        }
    }

    /// Database connectivity health check
    pub struct DatabaseHealthCheck {
        name: String,
        pool: sqlx::Pool<sqlx::Sqlite>,
    }

    impl DatabaseHealthCheck {
        pub fn new(name: String, pool: sqlx::Pool<sqlx::Sqlite>) -> Self {
            Self { name, pool }
        }
    }

    #[async_trait::async_trait]
    impl HealthCheck for DatabaseHealthCheck {
        fn name(&self) -> &str {
            &self.name
        }

        async fn check_health(&self) -> HealthCheckResult {
            let start = Instant::now();

            let result = sqlx::query("SELECT 1").execute(&self.pool).await;
            
            let status = match result {
                Ok(_) => HealthStatus::Healthy,
                Err(e) => HealthStatus::Unhealthy {
                    reason: format!("Database connection failed: {}", e)
                },
            };

            HealthCheckResult {
                status,
                last_check: Instant::now(),
                response_time: start.elapsed(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    struct TestHealthCheck {
        name: String,
        status: HealthStatus,
    }

    #[async_trait::async_trait]
    impl HealthCheck for TestHealthCheck {
        fn name(&self) -> &str {
            &self.name
        }

        async fn check_health(&self) -> HealthCheckResult {
            HealthCheckResult {
                status: self.status.clone(),
                last_check: Instant::now(),
                response_time: Duration::from_millis(1),
            }
        }
    }

    #[tokio::test]
    async fn test_health_manager() {
        let manager = HealthManager::new(Duration::from_secs(1));

        let healthy_component = Arc::new(TestHealthCheck {
            name: "HealthyComponent".to_string(),
            status: HealthStatus::Healthy,
        });

        let unhealthy_component = Arc::new(TestHealthCheck {
            name: "UnhealthyComponent".to_string(),
            status: HealthStatus::Unhealthy {
                reason: "Test failure".to_string(),
            },
        });

        manager.register_component(healthy_component).await;
        manager.register_component(unhealthy_component).await;

        manager.check_all_components().await;
        let status = manager.get_system_health().await;

        match status {
            HealthStatus::Unhealthy { reason } => {
                assert!(reason.contains("UnhealthyComponent"));
            }
            _ => panic!("Expected unhealthy system status"),
        }
    }
}