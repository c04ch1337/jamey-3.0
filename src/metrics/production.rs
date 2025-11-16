use prometheus::{
    Registry, Counter, Histogram, HistogramOpts, HistogramVec,
    Opts, CounterVec, register_counter_vec, register_histogram_vec,
};
use lazy_static::lazy_static;
use std::time::Duration;
use tracing::{info, warn, error};

// Define metrics for production monitoring
lazy_static! {
    // Shutdown metrics
    pub static ref SHUTDOWN_OPERATIONS: CounterVec = register_counter_vec!(
        "shutdown_operations_total",
        "Total number of shutdown operations",
        &["component", "status"]
    ).unwrap();

    pub static ref SHUTDOWN_DURATION: HistogramVec = register_histogram_vec!(
        "shutdown_duration_seconds",
        "Time taken for shutdown operations",
        &["component"],
        vec![0.1, 0.5, 1.0, 2.0, 5.0, 10.0]
    ).unwrap();

    // Health check metrics
    pub static ref HEALTH_CHECK_OPERATIONS: CounterVec = register_counter_vec!(
        "health_check_operations_total",
        "Total number of health check operations",
        &["component", "status"]
    ).unwrap();

    pub static ref HEALTH_CHECK_DURATION: HistogramVec = register_histogram_vec!(
        "health_check_duration_seconds",
        "Time taken for health check operations",
        &["component"],
        vec![0.01, 0.05, 0.1, 0.5, 1.0, 2.0]
    ).unwrap();

    // Backup metrics
    pub static ref BACKUP_OPERATIONS: CounterVec = register_counter_vec!(
        "backup_operations_total",
        "Total number of backup operations",
        &["type", "status"]
    ).unwrap();

    pub static ref BACKUP_SIZE: HistogramVec = register_histogram_vec!(
        "backup_size_bytes",
        "Size of backup operations in bytes",
        &["type"],
        vec![1_000_000.0, 10_000_000.0, 100_000_000.0, 1_000_000_000.0]
    ).unwrap();

    pub static ref BACKUP_DURATION: HistogramVec = register_histogram_vec!(
        "backup_duration_seconds",
        "Time taken for backup operations",
        &["type"],
        vec![1.0, 5.0, 15.0, 30.0, 60.0, 300.0]
    ).unwrap();

    // Restore metrics
    pub static ref RESTORE_OPERATIONS: CounterVec = register_counter_vec!(
        "restore_operations_total",
        "Total number of restore operations",
        &["type", "status"]
    ).unwrap();

    pub static ref RESTORE_DURATION: HistogramVec = register_histogram_vec!(
        "restore_duration_seconds",
        "Time taken for restore operations",
        &["type"],
        vec![1.0, 5.0, 15.0, 30.0, 60.0, 300.0]
    ).unwrap();
}

/// Records metrics for a shutdown operation
pub fn record_shutdown_operation(component: &str, success: bool, duration: Duration) {
    let status = if success { "success" } else { "failure" };
    SHUTDOWN_OPERATIONS.with_label_values(&[component, status]).inc();
    SHUTDOWN_DURATION.with_label_values(&[component]).observe(duration.as_secs_f64());
}

/// Records metrics for a health check operation
pub fn record_health_check(component: &str, status: &str, duration: Duration) {
    HEALTH_CHECK_OPERATIONS.with_label_values(&[component, status]).inc();
    HEALTH_CHECK_DURATION.with_label_values(&[component]).observe(duration.as_secs_f64());
}

/// Records metrics for a backup operation
pub fn record_backup_operation(
    backup_type: &str,
    status: &str,
    size_bytes: u64,
    duration: Duration
) {
    BACKUP_OPERATIONS.with_label_values(&[backup_type, status]).inc();
    BACKUP_SIZE.with_label_values(&[backup_type]).observe(size_bytes as f64);
    BACKUP_DURATION.with_label_values(&[backup_type]).observe(duration.as_secs_f64());
}

/// Records metrics for a restore operation
pub fn record_restore_operation(
    restore_type: &str,
    status: &str,
    duration: Duration
) {
    RESTORE_OPERATIONS.with_label_values(&[restore_type, status]).inc();
    RESTORE_DURATION.with_label_values(&[restore_type]).observe(duration.as_secs_f64());
}

// Configure logging for production readiness features
pub fn configure_production_logging() {
    use tracing_subscriber::{
        fmt::format::FmtSpan,
        EnvFilter,
    };

    // Set up logging with environment-based filtering
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            EnvFilter::new(
                "info,jamey=debug,tower_http=debug,axum::rejection=trace"
            )
        });

    // Configure subscriber with JSON formatting for production
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_span_events(FmtSpan::CLOSE)
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true)
        .json()
        .with_current_span(true)
        .with_span_list(true)
        .with_timer(tracing_subscriber::fmt::time::UtcTime::rfc_3339())
        .build();

    // Set as global default
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set tracing subscriber");

    info!("Production logging configured");
}

/// Initialize all production metrics and logging
pub fn init_production_monitoring() {
    // Configure logging first
    configure_production_logging();
    
    info!("Initializing production monitoring");

    // Register metrics with Prometheus
    let registry = Registry::new();
    
    registry.register(Box::new(SHUTDOWN_OPERATIONS.clone())).unwrap();
    registry.register(Box::new(SHUTDOWN_DURATION.clone())).unwrap();
    registry.register(Box::new(HEALTH_CHECK_OPERATIONS.clone())).unwrap();
    registry.register(Box::new(HEALTH_CHECK_DURATION.clone())).unwrap();
    registry.register(Box::new(BACKUP_OPERATIONS.clone())).unwrap();
    registry.register(Box::new(BACKUP_SIZE.clone())).unwrap();
    registry.register(Box::new(BACKUP_DURATION.clone())).unwrap();
    registry.register(Box::new(RESTORE_OPERATIONS.clone())).unwrap();
    registry.register(Box::new(RESTORE_DURATION.clone())).unwrap();

    info!("Production monitoring initialized");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_metrics_recording() {
        // Test shutdown metrics
        record_shutdown_operation("test_component", true, Duration::from_secs(1));
        let shutdown_counter = SHUTDOWN_OPERATIONS.with_label_values(&["test_component", "success"]);
        assert_eq!(shutdown_counter.get(), 1.0);

        // Test health check metrics
        record_health_check("test_component", "healthy", Duration::from_secs(1));
        let health_counter = HEALTH_CHECK_OPERATIONS.with_label_values(&["test_component", "healthy"]);
        assert_eq!(health_counter.get(), 1.0);

        // Test backup metrics
        record_backup_operation("daily", "success", 1000, Duration::from_secs(1));
        let backup_counter = BACKUP_OPERATIONS.with_label_values(&["daily", "success"]);
        assert_eq!(backup_counter.get(), 1.0);

        // Test restore metrics
        record_restore_operation("full", "success", Duration::from_secs(1));
        let restore_counter = RESTORE_OPERATIONS.with_label_values(&["full", "success"]);
        assert_eq!(restore_counter.get(), 1.0);
    }
}