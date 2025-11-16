use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use uuid::Uuid;
use chrono::Utc;

use crate::shutdown::{ShutdownCoordinator, ComponentShutdownManager, GracefulShutdown};
use crate::health::{HealthManager, HealthCheck, HealthStatus, HealthCheckResult};
use crate::backup::{
    BackupManager, BackupConfig, RestoreManager, RestoreOptions,
    scheduler::{BackupScheduler, ScheduleConfig}
};
use crate::metrics::production::init_production_monitoring;

#[tokio::test]
async fn test_graceful_shutdown_integration() {
    // Initialize components
    let coordinator = Arc::new(ShutdownCoordinator::new());
    let mut manager = ComponentShutdownManager::new(Duration::from_secs(5));
    
    // Create test components
    struct TestComponent {
        name: String,
        shutdown_called: std::sync::atomic::AtomicBool,
    }

    #[async_trait::async_trait]
    impl GracefulShutdown for TestComponent {
        fn name(&self) -> &str {
            &self.name
        }

        async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            self.shutdown_called.store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        }
    }

    let component1 = Arc::new(TestComponent {
        name: "test1".to_string(),
        shutdown_called: std::sync::atomic::AtomicBool::new(false),
    });

    let component2 = Arc::new(TestComponent {
        name: "test2".to_string(),
        shutdown_called: std::sync::atomic::AtomicBool::new(false),
    });

    // Register components
    manager.register_component(component1.clone());
    manager.register_component(component2.clone());

    // Trigger shutdown
    coordinator.shutdown().await;
    manager.shutdown_all().await;

    // Verify components were shut down
    assert!(component1.shutdown_called.load(std::sync::atomic::Ordering::SeqCst));
    assert!(component2.shutdown_called.load(std::sync::atomic::Ordering::SeqCst));
}

#[tokio::test]
async fn test_health_check_integration() {
    // Initialize health manager
    let manager = HealthManager::new(Duration::from_secs(60));

    // Create test health check component
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
                last_check: std::time::Instant::now(),
                response_time: Duration::from_millis(100),
            }
        }
    }

    let healthy_component = Arc::new(TestHealthCheck {
        name: "healthy_service".to_string(),
        status: HealthStatus::Healthy,
    });

    let degraded_component = Arc::new(TestHealthCheck {
        name: "degraded_service".to_string(),
        status: HealthStatus::Degraded {
            reason: "High latency".to_string(),
        },
    });

    // Register components
    manager.register_component(healthy_component).await;
    manager.register_component(degraded_component).await;

    // Perform health checks
    manager.check_all_components().await;
    let system_health = manager.get_system_health().await;

    // Verify system health reflects component states
    match system_health {
        HealthStatus::Degraded { reason } => {
            assert!(reason.contains("degraded_service"));
        }
        _ => panic!("Expected degraded system status"),
    }
}

#[tokio::test]
async fn test_backup_restore_integration() {
    // Initialize test directory
    let temp_dir = tempfile::tempdir().unwrap();
    let backup_config = BackupConfig {
        backup_dir: temp_dir.path().to_path_buf(),
        max_backups: 3,
        encryption_key: [0u8; 32],
        components: vec!["test_component".to_string()],
    };

    // Create backup manager
    let backup_manager = Arc::new(BackupManager::new(backup_config.clone()).unwrap());

    // Create test data
    let test_data_dir = temp_dir.path().join("test_data");
    tokio::fs::create_dir_all(&test_data_dir).await.unwrap();
    tokio::fs::write(
        test_data_dir.join("test.txt"),
        "test content",
    ).await.unwrap();

    // Perform backup
    let backup_id = backup_manager.create_backup(None).await.unwrap();

    // Verify backup was created
    let backup_path = temp_dir.path().join(backup_id.to_string());
    assert!(backup_path.exists());

    // Create restore manager
    let restore_manager = RestoreManager::new(
        Arc::new(backup_manager.vault.clone()),
        temp_dir.path().to_path_buf(),
    );

    // Perform restore
    let restore_result = restore_manager.restore_from_backup(
        backup_id,
        RestoreOptions {
            components: vec!["test_component".to_string()],
            force: true,
            verify_integrity: true,
        },
    ).await.unwrap();

    // Verify restore was successful
    assert!(restore_result.failed_components.is_empty());
}

#[tokio::test]
async fn test_backup_scheduler_integration() {
    // Initialize test components
    let temp_dir = tempfile::tempdir().unwrap();
    let backup_config = BackupConfig {
        backup_dir: temp_dir.path().to_path_buf(),
        max_backups: 3,
        encryption_key: [0u8; 32],
        components: vec!["test_component".to_string()],
    };

    let backup_manager = Arc::new(BackupManager::new(backup_config).unwrap());

    let schedule_config = ScheduleConfig {
        daily_backup_hour: 2,
        weekly_backup_day: 0,
        monthly_backup_day: 1,
        daily_enabled: true,
        weekly_enabled: true,
        monthly_enabled: true,
    };

    let scheduler = BackupScheduler::new(backup_manager, schedule_config);

    // Get next scheduled backups
    let scheduled = scheduler.get_next_scheduled_backups().await;

    // Verify scheduling logic
    assert!(scheduled.next_daily.is_some());
    assert!(scheduled.next_weekly.is_some());
    assert!(scheduled.next_monthly.is_some());
}

#[tokio::test]
async fn test_metrics_integration() {
    // Initialize production monitoring
    init_production_monitoring();

    // Record test metrics
    use crate::metrics::production::{
        record_shutdown_operation,
        record_health_check,
        record_backup_operation,
        record_restore_operation,
    };

    // Record shutdown metrics
    record_shutdown_operation(
        "test_component",
        true,
        Duration::from_secs(1),
    );

    // Record health check metrics
    record_health_check(
        "test_component",
        "healthy",
        Duration::from_secs(1),
    );

    // Record backup metrics
    record_backup_operation(
        "test",
        "success",
        1000,
        Duration::from_secs(1),
    );

    // Record restore metrics
    record_restore_operation(
        "test",
        "success",
        Duration::from_secs(1),
    );

    // Verify metrics were recorded
    // Note: In a real test we would use a test registry to verify exact values
    // Here we're just ensuring no panics occur during recording
}

// Run all tests with:
// cargo test --test production_readiness -- --nocapture