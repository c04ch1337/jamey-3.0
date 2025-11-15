use serde::Serialize;
use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::fs;
use tracing::warn;

use crate::mqtt::MqttClient;
use crate::memory::MemorySystem;

/// Health check response
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    /// Overall status: "ok", "degraded", or "down"
    pub status: &'static str,
    
    /// Service name and version
    pub service: &'static str,
    pub version: &'static str,

    /// Component health statuses
    pub components: ComponentHealth,

    /// System metrics
    pub metrics: SystemMetrics,
}

/// Component-specific health statuses
#[derive(Debug, Serialize)]
pub struct ComponentHealth {
    /// Database connectivity
    pub database: ComponentStatus,

    /// Memory system status
    pub memory: ComponentStatus,

    /// MQTT connection (if configured)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mqtt: Option<ComponentStatus>,
}

/// Individual component status
#[derive(Debug, Serialize)]
pub struct ComponentStatus {
    pub status: &'static str,
    pub message: String,
}

/// System metrics
#[derive(Debug, Serialize)]
pub struct SystemMetrics {
    /// Available disk space in bytes
    pub disk_free_bytes: u64,

    /// Memory usage in bytes (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_usage_bytes: Option<u64>,

    /// Uptime in seconds
    pub uptime_seconds: u64,
}

/// Health checker service
pub struct HealthChecker {
    db_pool: SqlitePool,
    memory: Arc<MemorySystem>,
    mqtt: Option<Arc<MqttClient>>,
    start_time: std::time::Instant,
}

impl HealthChecker {
    /// Create a new health checker
    pub fn new(
        db_pool: SqlitePool,
        memory: Arc<MemorySystem>,
        mqtt: Option<Arc<MqttClient>>,
    ) -> Self {
        Self {
            db_pool,
            memory,
            mqtt,
            start_time: std::time::Instant::now(),
        }
    }

    /// Run a basic health check (liveness probe)
    pub async fn check_liveness(&self) -> HealthResponse {
        HealthResponse {
            status: "ok",
            service: "Jamey 3.0",
            version: "3.0.0",
            components: ComponentHealth {
                database: ComponentStatus {
                    status: "ok",
                    message: "Database available".into(),
                },
                memory: ComponentStatus {
                    status: "ok",
                    message: "Memory system available".into(),
                },
                mqtt: None,
            },
            metrics: SystemMetrics {
                disk_free_bytes: 0,
                memory_usage_bytes: None,
                uptime_seconds: self.start_time.elapsed().as_secs(),
            },
        }
    }

    /// Run a detailed health check
    pub async fn check_detailed(&self) -> HealthResponse {
        // Check database
        let db_status = match sqlx::query("SELECT 1").fetch_one(&self.db_pool).await {
            Ok(_) => ComponentStatus {
                status: "ok",
                message: "Database connection successful".into(),
            },
            Err(e) => {
                warn!("Database health check failed: {}", e);
                ComponentStatus {
                    status: "error",
                    message: format!("Database error: {}", e),
                }
            }
        };

        // Check memory system
        let memory_status = match self.check_memory_system().await {
            Ok(msg) => ComponentStatus {
                status: "ok",
                message: msg,
            },
            Err(e) => {
                warn!("Memory system health check failed: {}", e);
                ComponentStatus {
                    status: "error",
                    message: format!("Memory error: {}", e),
                }
            }
        };

        // Check MQTT if configured
        let mqtt_status = if let Some(mqtt) = &self.mqtt {
            Some(match mqtt.state().await {
                crate::mqtt::ConnectionState::Connected => ComponentStatus {
                    status: "ok",
                    message: "MQTT connected".into(),
                },
                state => {
                    warn!("MQTT not connected: {:?}", state);
                    ComponentStatus {
                        status: "error",
                        message: format!("MQTT state: {:?}", state),
                    }
                }
            })
        } else {
            None
        };

        // Get system metrics
        let metrics = self.collect_metrics().await;

        // Determine overall status
        let status = if db_status.status == "error" {
            "down"
        } else if memory_status.status == "error" 
            || mqtt_status.as_ref().map_or(false, |s| s.status == "error") {
            "degraded"
        } else {
            "ok"
        };

        HealthResponse {
            status,
            service: "Jamey 3.0",
            version: "3.0.0",
            components: ComponentHealth {
                database: db_status,
                memory: memory_status,
                mqtt: mqtt_status,
            },
            metrics,
        }
    }

    /// Check memory system health
    async fn check_memory_system(&self) -> anyhow::Result<String> {
        // Try to store and retrieve a test memory
        let content = "Health check test memory";
        let id = self.memory.store(
            crate::memory::MemoryLayer::ShortTerm,
            content.to_string(),
        ).await?;

        let memories = self.memory.search(
            crate::memory::MemoryLayer::ShortTerm,
            content,
            1,
        ).await?;

        if memories.is_empty() {
            anyhow::bail!("Test memory not found after storage");
        }

        Ok(format!("Memory system operational (test ID: {})", id))
    }

    /// Collect system metrics
    async fn collect_metrics(&self) -> SystemMetrics {
        // Get disk space
        let disk_free_bytes = match fs::metadata("data").await {
            Ok(meta) => meta.len(),
            Err(_) => 0,
        };

        // Get memory usage (platform-specific)
        let memory_usage_bytes = if cfg!(target_os = "linux") {
            std::fs::read_to_string("/proc/self/statm")
                .ok()
                .and_then(|s| s.split_whitespace().next())
                .and_then(|pages| pages.parse::<u64>().ok())
                .map(|pages| pages * 4096)
        } else {
            None
        };

        SystemMetrics {
            disk_free_bytes,
            memory_usage_bytes,
            uptime_seconds: self.start_time.elapsed().as_secs(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use crate::db::init_db;

    #[tokio::test]
    async fn test_health_checker() {
        // Set up test database
        let db_dir = tempdir().unwrap();
        std::env::set_var(
            "DATABASE_URL",
            format!("sqlite:{}/test.db", db_dir.path().display())
        );
        let pool = init_db().await.unwrap();

        // Set up memory system
        let memory_dir = tempdir().unwrap();
        let memory = Arc::new(MemorySystem::new(memory_dir.path().to_path_buf()).await.unwrap());

        // Create health checker
        let checker = HealthChecker::new(pool, memory, None);

        // Test liveness check
        let liveness = checker.check_liveness().await;
        assert_eq!(liveness.status, "ok");

        // Test detailed check
        let detailed = checker.check_detailed().await;
        assert_eq!(detailed.components.database.status, "ok");
        assert_eq!(detailed.components.memory.status, "ok");
        assert!(detailed.components.mqtt.is_none());
    }
}