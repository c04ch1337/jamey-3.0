use std::sync::atomic::{AtomicU64, AtomicI64, Ordering};
use std::time::{Duration, Instant};
use crate::memory::cache::CacheType;

/// Tracks detailed performance metrics for a cache instance
pub struct CacheMetrics {
    // Access counts
    hits: AtomicU64,
    misses: AtomicU64,
    
    // Memory metrics
    memory_usage_bytes: AtomicU64,
    allocated_capacity_bytes: AtomicU64,
    
    // Latency tracking
    total_get_latency_ns: AtomicU64,
    get_operations: AtomicU64,
    total_insert_latency_ns: AtomicU64,
    insert_operations: AtomicU64,
    
    // Eviction metrics
    eviction_count: AtomicU64,
    last_eviction: AtomicI64,
    
    // Resizing metrics
    resize_count: AtomicU64,
    last_resize: AtomicI64,
}

impl CacheMetrics {
    pub fn new() -> Self {
        Self {
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
            memory_usage_bytes: AtomicU64::new(0),
            allocated_capacity_bytes: AtomicU64::new(0),
            total_get_latency_ns: AtomicU64::new(0),
            get_operations: AtomicU64::new(0),
            total_insert_latency_ns: AtomicU64::new(0),
            insert_operations: AtomicU64::new(0),
            eviction_count: AtomicU64::new(0),
            last_eviction: AtomicI64::new(0),
            resize_count: AtomicU64::new(0),
            last_resize: AtomicI64::new(0),
        }
    }

    pub fn record_hit(&self) {
        self.hits.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_miss(&self) {
        self.misses.fetch_add(1, Ordering::Relaxed);
    }

    pub fn update_memory_usage(&self, bytes: u64) {
        self.memory_usage_bytes.store(bytes, Ordering::Relaxed);
    }

    pub fn update_allocated_capacity(&self, bytes: u64) {
        self.allocated_capacity_bytes.store(bytes, Ordering::Relaxed);
    }

    pub fn record_get_latency(&self, duration: Duration) {
        self.total_get_latency_ns.fetch_add(duration.as_nanos() as u64, Ordering::Relaxed);
        self.get_operations.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_insert_latency(&self, duration: Duration) {
        self.total_insert_latency_ns.fetch_add(duration.as_nanos() as u64, Ordering::Relaxed);
        self.insert_operations.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_eviction(&self) {
        self.eviction_count.fetch_add(1, Ordering::Relaxed);
        self.last_eviction.store(
            Instant::now().duration_since(Instant::now().checked_sub(Duration::from_nanos(0)).unwrap()).as_nanos() as i64,
            Ordering::Relaxed
        );
    }

    pub fn record_resize(&self) {
        self.resize_count.fetch_add(1, Ordering::Relaxed);
        self.last_resize.store(
            Instant::now().duration_since(Instant::now().checked_sub(Duration::from_nanos(0)).unwrap()).as_nanos() as i64,
            Ordering::Relaxed
        );
    }

    pub fn get_stats(&self) -> CacheStats {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let total = hits + misses;
        
        let avg_get_latency = if self.get_operations.load(Ordering::Relaxed) > 0 {
            Duration::from_nanos(
                self.total_get_latency_ns.load(Ordering::Relaxed) / 
                self.get_operations.load(Ordering::Relaxed)
            )
        } else {
            Duration::from_nanos(0)
        };

        let avg_insert_latency = if self.insert_operations.load(Ordering::Relaxed) > 0 {
            Duration::from_nanos(
                self.total_insert_latency_ns.load(Ordering::Relaxed) / 
                self.insert_operations.load(Ordering::Relaxed)
            )
        } else {
            Duration::from_nanos(0)
        };

        CacheStats {
            hit_rate: if total > 0 { hits as f64 / total as f64 } else { 0.0 },
            memory_usage_bytes: self.memory_usage_bytes.load(Ordering::Relaxed),
            allocated_capacity_bytes: self.allocated_capacity_bytes.load(Ordering::Relaxed),
            avg_get_latency,
            avg_insert_latency,
            eviction_count: self.eviction_count.load(Ordering::Relaxed),
            resize_count: self.resize_count.load(Ordering::Relaxed),
        }
    }
}

/// Statistics derived from raw metrics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub hit_rate: f64,
    pub memory_usage_bytes: u64,
    pub allocated_capacity_bytes: u64,
    pub avg_get_latency: Duration,
    pub avg_insert_latency: Duration,
    pub eviction_count: u64,
    pub resize_count: u64,
}

/// Global metrics registry for all cache instances
pub struct MetricsRegistry {
    metrics: dashmap::DashMap<CacheType, CacheMetrics>,
}

impl MetricsRegistry {
    pub fn new() -> Self {
        let metrics = dashmap::DashMap::new();
        metrics.insert(CacheType::Response, CacheMetrics::new());
        metrics.insert(CacheType::Embedding, CacheMetrics::new());
        metrics.insert(CacheType::Persistent, CacheMetrics::new());
        Self { metrics }
    }

    pub fn get_metrics(&self, cache_type: CacheType) -> Option<CacheMetrics> {
        self.metrics.get(&cache_type).map(|m| m.clone())
    }

    pub fn get_all_stats(&self) -> Vec<(CacheType, CacheStats)> {
        self.metrics
            .iter()
            .map(|entry| (*entry.key(), entry.value().get_stats()))
            .collect()
    }
}

impl Default for MetricsRegistry {
    fn default() -> Self {
        Self::new()
    }
}