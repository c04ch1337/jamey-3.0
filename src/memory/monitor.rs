use std::sync::atomic::{AtomicU64, AtomicI64, AtomicF64};
use std::sync::{Arc, RwLock};
use std::collections::VecDeque;

/// Point in time metric measurement
#[derive(Debug, Clone)]
pub struct MetricPoint {
    pub timestamp: i64,
    pub memory_usage: u64,
    pub hit_rate: f64,
    pub latency: u64,
}

/// Monitors cache memory usage and system memory pressure
pub struct MemoryTracker {
    system_memory_usage: AtomicU64,
    cache_memory_usage: AtomicU64,
    pressure_threshold: AtomicU64,
}

/// Tracks cache performance metrics like hits, misses and latency
pub struct PerformanceMetrics {
    hits: AtomicU64,
    misses: AtomicU64,
    total_latency: AtomicU64,
    operation_count: AtomicU64,
}

/// Controls cache optimization based on collected metrics
pub struct OptimizationController {
    last_adjustment: AtomicI64,
    trend_data: Arc<RwLock<VecDeque<MetricPoint>>>,
    adjustment_threshold: AtomicF64,
}

/// Main cache monitoring system that coordinates all monitoring components
pub struct CacheMonitor {
    memory_tracker: Arc<MemoryTracker>,
    performance_metrics: Arc<PerformanceMetrics>,
    optimization_controller: Arc<OptimizationController>,
}

impl MemoryTracker {
    pub fn new(pressure_threshold: u64) -> Self {
        Self {
            system_memory_usage: AtomicU64::new(0),
            cache_memory_usage: AtomicU64::new(0),
            pressure_threshold: AtomicU64::new(pressure_threshold),
        }
    }

    /// Updates the current system memory usage
    pub fn update_system_memory(&self, usage: u64) {
        self.system_memory_usage.store(usage, std::sync::atomic::Ordering::SeqCst);
    }

    /// Updates the current cache memory usage
    pub fn update_cache_memory(&self, usage: u64) {
        self.cache_memory_usage.store(usage, std::sync::atomic::Ordering::SeqCst);
    }

    /// Returns true if system memory pressure exceeds threshold
    pub fn is_memory_pressured(&self) -> bool {
        let usage = self.system_memory_usage.load(std::sync::atomic::Ordering::SeqCst);
        let threshold = self.pressure_threshold.load(std::sync::atomic::Ordering::SeqCst);
        usage > threshold
    }
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
            total_latency: AtomicU64::new(0),
            operation_count: AtomicU64::new(0),
        }
    }

    /// Records a cache hit
    pub fn record_hit(&self, latency: u64) {
        self.hits.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.record_operation(latency);
    }

    /// Records a cache miss
    pub fn record_miss(&self, latency: u64) {
        self.misses.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.record_operation(latency);
    }

    /// Records operation latency and increments operation count
    fn record_operation(&self, latency: u64) {
        self.total_latency.fetch_add(latency, std::sync::atomic::Ordering::SeqCst);
        self.operation_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }

    /// Calculates current hit rate
    pub fn hit_rate(&self) -> f64 {
        let hits = self.hits.load(std::sync::atomic::Ordering::SeqCst);
        let total = hits + self.misses.load(std::sync::atomic::Ordering::SeqCst);
        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }

    /// Calculates average operation latency
    pub fn average_latency(&self) -> u64 {
        let total = self.total_latency.load(std::sync::atomic::Ordering::SeqCst);
        let count = self.operation_count.load(std::sync::atomic::Ordering::SeqCst);
        if count == 0 {
            0
        } else {
            total / count
        }
    }
}

impl OptimizationController {
    pub fn new(adjustment_threshold: f64) -> Self {
        Self {
            last_adjustment: AtomicI64::new(0),
            trend_data: Arc::new(RwLock::new(VecDeque::with_capacity(100))),
            adjustment_threshold: AtomicF64::new(adjustment_threshold),
        }
    }

    /// Records a new metric point for trend analysis
    pub fn record_metrics(&self, memory_usage: u64, hit_rate: f64, latency: u64) {
        let point = MetricPoint {
            timestamp: chrono::Utc::now().timestamp(),
            memory_usage,
            hit_rate,
            latency,
        };

        if let Ok(mut trend_data) = self.trend_data.write() {
            if trend_data.len() >= 100 {
                trend_data.pop_front();
            }
            trend_data.push_back(point);
        }
    }

    /// Analyzes trends and determines if optimization is needed
    pub fn should_optimize(&self) -> bool {
        if let Ok(trend_data) = self.trend_data.read() {
            if trend_data.len() < 2 {
                return false;
            }

            let threshold = self.adjustment_threshold.load(std::sync::atomic::Ordering::SeqCst);
            let recent_hit_rate = trend_data.iter().rev().take(10)
                .map(|p| p.hit_rate)
                .sum::<f64>() / 10.0;

            recent_hit_rate < threshold
        } else {
            false
        }
    }
}

impl CacheMonitor {
    pub fn new(pressure_threshold: u64, adjustment_threshold: f64) -> Self {
        Self {
            memory_tracker: Arc::new(MemoryTracker::new(pressure_threshold)),
            performance_metrics: Arc::new(PerformanceMetrics::new()),
            optimization_controller: Arc::new(OptimizationController::new(adjustment_threshold)),
        }
    }

    /// Updates all monitoring metrics
    pub fn update_metrics(&self, system_memory: u64, cache_memory: u64) {
        self.memory_tracker.update_system_memory(system_memory);
        self.memory_tracker.update_cache_memory(cache_memory);

        let hit_rate = self.performance_metrics.hit_rate();
        let latency = self.performance_metrics.average_latency();

        self.optimization_controller.record_metrics(
            cache_memory,
            hit_rate,
            latency,
        );
    }

    /// Returns true if cache optimization is recommended
    pub fn needs_optimization(&self) -> bool {
        self.memory_tracker.is_memory_pressured() || 
        self.optimization_controller.should_optimize()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_memory_tracker() {
        let tracker = MemoryTracker::new(1000);
        tracker.update_system_memory(500);
        assert!(!tracker.is_memory_pressured());
        
        tracker.update_system_memory(1500);
        assert!(tracker.is_memory_pressured());
    }

    #[test]
    fn test_performance_metrics() {
        let metrics = PerformanceMetrics::new();
        
        metrics.record_hit(10);
        metrics.record_hit(20);
        metrics.record_miss(30);
        
        assert_eq!(metrics.hit_rate(), 2.0 / 3.0);
        assert_eq!(metrics.average_latency(), 20);
    }

    #[test]
    fn test_optimization_controller() {
        let controller = OptimizationController::new(0.8);
        
        controller.record_metrics(100, 0.7, 50);
        thread::sleep(Duration::from_millis(100));
        controller.record_metrics(120, 0.6, 60);
        
        assert!(controller.should_optimize());
    }

    #[test]
    fn test_cache_monitor_integration() {
        let monitor = CacheMonitor::new(1000, 0.8);
        
        monitor.update_metrics(500, 100);
        monitor.performance_metrics.record_hit(10);
        monitor.performance_metrics.record_miss(20);
        
        assert!(!monitor.needs_optimization());
        
        monitor.update_metrics(1500, 200);
        assert!(monitor.needs_optimization());
    }
}