mod batch;

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time;
use sysinfo::{System, SystemExt, CpuExt};
use thiserror::Error;
use metrics::{counter, describe_counter, describe_gauge, describe_histogram, gauge, histogram};

use self::batch::{BatchConfig, BatchOptimizer};

#[derive(Error, Debug)]
pub enum MonitoringError {
    #[error("Failed to collect system metrics: {0}")]
    MetricsCollectionError(String),
    
    #[error("System resource unavailable: {0}")]
    ResourceUnavailable(String),
    
    #[error("Invalid metric value: {0}")]
    InvalidMetricValue(String),
}

/// Monitors system performance metrics and provides adaptive optimization capabilities
pub struct SystemLoadMonitor {
    // System info collector
    sys: System,
    
    // Core system metrics
    cpu_usage: Arc<AtomicU64>,
    memory_usage: Arc<AtomicU64>,
    
    // Channel metrics
    channel_capacity: Arc<AtomicU64>,
    channel_utilization: Arc<AtomicU64>,
    
    // Cache metrics  
    cache_hits: Arc<AtomicU64>,
    cache_misses: Arc<AtomicU64>,
    
    // Response time tracking
    response_times: Arc<AtomicU64>,
    error_count: Arc<AtomicU64>,
    
    // Batch processing
    batch_optimizer: BatchOptimizer,
}

impl SystemLoadMonitor {
    /// Creates a new SystemLoadMonitor instance and initializes all metrics
    pub fn new() -> Self {
        // Initialize metric descriptions
        describe_gauge!("system.cpu_usage", "Current CPU usage percentage");
        describe_gauge!("system.memory_usage", "Current memory usage in bytes");
        describe_gauge!("system.memory_usage_percent", "Current memory usage percentage");
        describe_gauge!("system.channel_capacity", "Current channel capacity");
        describe_gauge!("system.channel_utilization", "Current channel utilization percentage");
        describe_counter!("system.cache_hits", "Number of cache hits");
        describe_counter!("system.cache_misses", "Number of cache misses");
        describe_histogram!("system.response_time", "Response time distribution");
        describe_counter!("system.errors", "Number of errors encountered");
        describe_gauge!("system.batch_size", "Current batch size");
        describe_histogram!("system.batch_processing_time", "Batch processing time distribution");

        Self {
            sys: System::new_all(),
            cpu_usage: Arc::new(AtomicU64::new(0)),
            memory_usage: Arc::new(AtomicU64::new(0)),
            channel_capacity: Arc::new(AtomicU64::new(0)),
            channel_utilization: Arc::new(AtomicU64::new(0)),
            cache_hits: Arc::new(AtomicU64::new(0)),
            cache_misses: Arc::new(AtomicU64::new(0)),
            response_times: Arc::new(AtomicU64::new(0)),
            error_count: Arc::new(AtomicU64::new(0)),
            batch_optimizer: BatchOptimizer::new(BatchConfig::default()),
        }
    }

    /// Starts the background metrics collection task
    pub async fn start_monitoring(&mut self, interval: Duration) -> Result<(), MonitoringError> {
        let mut interval = time::interval(interval);
        
        loop {
            interval.tick().await;
            if let Err(e) = self.collect_system_metrics().await {
                self.record_error();
                eprintln!("Error collecting metrics: {}", e);
            }
        }
    }

    /// Collects current system metrics
    async fn collect_system_metrics(&mut self) -> Result<(), MonitoringError> {
        // Refresh system information
        self.sys.refresh_all();

        // Update CPU usage
        let cpu_usage = self.sys.global_cpu_info().cpu_usage();
        if cpu_usage.is_nan() {
            return Err(MonitoringError::InvalidMetricValue("CPU usage is NaN".to_string()));
        }
        self.update_cpu_usage(cpu_usage);

        // Update memory usage
        let total_memory = self.sys.total_memory();
        let used_memory = self.sys.used_memory();
        if total_memory == 0 {
            return Err(MonitoringError::ResourceUnavailable("Cannot get total memory".to_string()));
        }
        
        self.update_memory_usage(used_memory);
        let memory_percentage = (used_memory as f64 / total_memory as f64) * 100.0;
        gauge!("system.memory_usage_percent").set(memory_percentage);

        Ok(())
    }

    /// Updates CPU usage metric
    pub fn update_cpu_usage(&self, usage: f64) {
        gauge!("system.cpu_usage").set(usage);
        self.cpu_usage.store((usage * 100.0) as u64, Ordering::Relaxed);
    }

    /// Updates memory usage metric
    pub fn update_memory_usage(&self, bytes: u64) {
        gauge!("system.memory_usage").set(bytes as f64);
        self.memory_usage.store(bytes, Ordering::Relaxed);
    }

    /// Records a cache hit
    pub fn record_cache_hit(&self) {
        counter!("system.cache_hits").increment(1);
        self.cache_hits.fetch_add(1, Ordering::Relaxed);
    }

    /// Records a cache miss
    pub fn record_cache_miss(&self) {
        counter!("system.cache_misses").increment(1);
        self.cache_misses.fetch_add(1, Ordering::Relaxed);
    }

    /// Updates channel metrics
    pub fn update_channel_metrics(&self, capacity: u64, utilization: f64) {
        gauge!("system.channel_capacity").set(capacity as f64);
        gauge!("system.channel_utilization").set(utilization);
        self.channel_capacity.store(capacity, Ordering::Relaxed);
        self.channel_utilization.store((utilization * 100.0) as u64, Ordering::Relaxed);
    }

    /// Records response time for an operation
    pub fn record_response_time(&self, duration: Duration) {
        let duration_ms = duration.as_secs_f64() * 1000.0;
        histogram!("system.response_time").record(duration_ms);
        self.response_times.store(duration_ms as u64, Ordering::Relaxed);
    }

    /// Records batch processing metrics and optimizes batch size
    pub fn record_batch_processing(&mut self, batch_size: u64, duration: Duration) -> u64 {
        let duration_ms = duration.as_secs_f64() * 1000.0;
        histogram!("system.batch_processing_time").record(duration_ms);
        
        // Get current system metrics for optimization
        let cpu_usage = self.get_cpu_usage();
        let memory_usage = self.get_memory_usage_percent();
        
        // Optimize batch size based on current metrics
        self.batch_optimizer.optimize_batch_size(cpu_usage, memory_usage, duration)
    }

    /// Records an error occurrence
    pub fn record_error(&self) {
        counter!("system.errors").increment(1);
        self.error_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Gets the current cache hit ratio
    pub fn get_cache_hit_ratio(&self) -> f64 {
        let hits = self.cache_hits.load(Ordering::Relaxed);
        let misses = self.cache_misses.load(Ordering::Relaxed);
        let total = hits + misses;
        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }

    /// Gets the current CPU usage percentage
    pub fn get_cpu_usage(&self) -> f64 {
        self.cpu_usage.load(Ordering::Relaxed) as f64 / 100.0
    }

    /// Gets the current memory usage in bytes
    pub fn get_memory_usage(&self) -> u64 {
        self.memory_usage.load(Ordering::Relaxed)
    }

    /// Gets the current memory usage percentage
    pub fn get_memory_usage_percent(&self) -> f64 {
        let total_memory = self.sys.total_memory();
        if total_memory == 0 {
            return 0.0;
        }
        (self.get_memory_usage() as f64 / total_memory as f64) * 100.0
    }

    /// Gets the current batch size
    pub fn get_current_batch_size(&self) -> u64 {
        self.batch_optimizer.get_current_batch_size()
    }

    /// Gets the current error count
    pub fn get_error_count(&self) -> u64 {
        self.error_count.load(Ordering::Relaxed)
    }

    /// Gets the batch processing configuration
    pub fn get_batch_config(&self) -> &BatchConfig {
        self.batch_optimizer.get_config()
    }
}

impl Default for SystemLoadMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use tokio_test::block_on;

    #[test]
    fn test_system_load_monitor() {
        let mut monitor = SystemLoadMonitor::new();

        // Test CPU usage updates
        monitor.update_cpu_usage(45.5);
        assert_eq!(monitor.cpu_usage.load(Ordering::Relaxed), 4550);

        // Test memory usage updates
        monitor.update_memory_usage(1024 * 1024);
        assert_eq!(monitor.memory_usage.load(Ordering::Relaxed), 1024 * 1024);

        // Test cache metrics
        monitor.record_cache_hit();
        monitor.record_cache_hit();
        monitor.record_cache_miss();
        assert_eq!(monitor.get_cache_hit_ratio(), 2.0 / 3.0);

        // Test batch processing
        let new_batch_size = monitor.record_batch_processing(100, Duration::from_millis(50));
        assert!(new_batch_size > 0);

        // Test response time recording
        monitor.record_response_time(Duration::from_millis(100));
        assert!(monitor.response_times.load(Ordering::Relaxed) > 0);

        // Test error counting
        monitor.record_error();
        monitor.record_error();
        assert_eq!(monitor.get_error_count(), 2);
    }

    #[test]
    fn test_thread_safety() {
        let monitor = Arc::new(SystemLoadMonitor::new());
        let mut handles = vec![];

        for _ in 0..10 {
            let monitor_clone = Arc::clone(&monitor);
            handles.push(thread::spawn(move || {
                monitor_clone.record_cache_hit();
                monitor_clone.record_cache_miss();
                monitor_clone.record_error();
                monitor_clone.record_response_time(Duration::from_millis(50));
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(monitor.cache_hits.load(Ordering::Relaxed), 10);
        assert_eq!(monitor.cache_misses.load(Ordering::Relaxed), 10);
        assert_eq!(monitor.get_error_count(), 10);
    }

    #[test]
    fn test_system_metrics_collection() {
        let mut monitor = SystemLoadMonitor::new();
        
        // Test a single metrics collection
        block_on(async {
            assert!(monitor.collect_system_metrics().await.is_ok());
        });

        // Verify metrics were collected
        assert!(monitor.get_cpu_usage() >= 0.0);
        assert!(monitor.get_memory_usage() > 0);
    }
}