use std::time::Duration;
use anyhow::Result;
use moka::future::Cache;
use serde::{Serialize, Deserialize};
use tracing::{info, warn, error};
use std::sync::{Arc, atomic::{AtomicUsize, AtomicI64, AtomicU64, Ordering}};
use std::ops::{Deref, DerefMut};
use std::time::Instant;
use std::sync::atomic::AtomicBool;
use crate::metrics;
use crate::memory::metrics::{MetricsRegistry, CacheMetrics};

/// Performance metrics for adaptive cache
#[derive(Debug)]
pub struct AdaptiveCacheMetrics {
    /// Total number of get operations
    get_count: AtomicUsize,
    /// Total number of cache hits
    hit_count: AtomicUsize,
    /// Total number of cache misses
    miss_count: AtomicUsize,
    /// Total number of evictions
    eviction_count: AtomicUsize,
    /// Current memory usage in bytes
    memory_usage: AtomicUsize,
    /// Last recorded get latency in nanoseconds
    last_get_latency: AtomicU64,
    /// Exponential moving average of get latency
    avg_get_latency: AtomicF64,
    /// Total number of size adjustments
    size_adjustment_count: AtomicUsize,
    /// Peak memory usage in bytes
    peak_memory_usage: AtomicUsize,
    /// Moving average of memory pressure
    avg_memory_pressure: AtomicF64,
    /// Number of items currently cached
    item_count: AtomicUsize,
    /// Total bytes saved by cache hits
    bytes_saved: AtomicU64,
}

impl AdaptiveCacheMetrics {
    pub fn new() -> Self {
        Self {
            get_count: AtomicUsize::new(0),
            hit_count: AtomicUsize::new(0),
            miss_count: AtomicUsize::new(0),
            eviction_count: AtomicUsize::new(0),
            memory_usage: AtomicUsize::new(0),
            last_get_latency: AtomicU64::new(0),
            avg_get_latency: AtomicF64::new(0.0),
            size_adjustment_count: AtomicUsize::new(0),
            peak_memory_usage: AtomicUsize::new(0),
            avg_memory_pressure: AtomicF64::new(0.0),
            item_count: AtomicUsize::new(0),
            bytes_saved: AtomicU64::new(0),
        }
    }

    pub fn record_size_adjustment(&self) {
        self.size_adjustment_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn update_item_count(&self, count: usize) {
        self.item_count.store(count, Ordering::Relaxed);
    }

    pub fn record_bytes_saved(&self, bytes: u64) {
        self.bytes_saved.fetch_add(bytes, Ordering::Relaxed);
    }

    pub fn update_memory_pressure(&self, pressure: f64) {
        // Update exponential moving average with 0.1 smoothing factor
        let current_avg = self.avg_memory_pressure.load(Ordering::Relaxed);
        let new_avg = (0.9 * current_avg) + (0.1 * pressure);
        self.avg_memory_pressure.store(new_avg, Ordering::Relaxed);
    }
}

/// Cache statistics for monitoring
#[derive(Debug, Clone)]
pub struct CacheStatistics {
    pub total_gets: usize,
    pub hit_count: usize,
    pub miss_count: usize,
    pub eviction_count: usize,
    pub current_memory: usize,
    pub peak_memory: usize,
    pub avg_latency: Duration,
    pub size_adjustments: usize,
    pub item_count: usize,
    pub bytes_saved: u64,
    pub memory_pressure: f64,
}

impl AdaptiveCacheMetrics {
    pub fn get_statistics(&self) -> CacheStatistics {
        CacheStatistics {
            total_gets: self.get_count.load(Ordering::Relaxed),
            hit_count: self.hit_count.load(Ordering::Relaxed),
            miss_count: self.miss_count.load(Ordering::Relaxed),
            eviction_count: self.eviction_count.load(Ordering::Relaxed),
            current_memory: self.memory_usage.load(Ordering::Relaxed),
            peak_memory: self.peak_memory_usage.load(Ordering::Relaxed),
            avg_latency: Duration::from_nanos(self.avg_get_latency.load(Ordering::Relaxed) as u64),
            size_adjustments: self.size_adjustment_count.load(Ordering::Relaxed),
            item_count: self.item_count.load(Ordering::Relaxed),
            bytes_saved: self.bytes_saved.load(Ordering::Relaxed),
            memory_pressure: self.avg_memory_pressure.load(Ordering::Relaxed),
        }
    }

    pub fn record_get(&self, hit: bool, latency: Duration) {
        self.get_count.fetch_add(1, Ordering::Relaxed);
        if hit {
            self.hit_count.fetch_add(1, Ordering::Relaxed);
        } else {
            self.miss_count.fetch_add(1, Ordering::Relaxed);
        }

        let latency_ns = latency.as_nanos() as u64;
        self.last_get_latency.store(latency_ns, Ordering::Relaxed);
        
        // Update exponential moving average with 0.1 smoothing factor
        let current_avg = self.avg_get_latency.load(Ordering::Relaxed);
        let new_avg = (0.9 * current_avg) + (0.1 * latency_ns as f64);
        self.avg_get_latency.store(new_avg, Ordering::Relaxed);
    }

    pub fn record_eviction(&self) {
        self.eviction_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn update_memory_usage(&self, bytes: usize) {
        self.memory_usage.store(bytes, Ordering::Relaxed);
        
        // Update peak memory usage if current usage is higher
        let peak = self.peak_memory_usage.load(Ordering::Relaxed);
        if bytes > peak {
            self.peak_memory_usage.store(bytes, Ordering::Relaxed);
        }
    }

    pub fn get_hit_rate(&self) -> f64 {
        let hits = self.hit_count.load(Ordering::Relaxed) as f64;
        let total = self.get_count.load(Ordering::Relaxed) as f64;
        if total > 0.0 {
            hits / total
        } else {
            0.0
        }
    }

    pub fn get_avg_latency(&self) -> Duration {
        Duration::from_nanos(self.avg_get_latency.load(Ordering::Relaxed) as u64)
    }

    pub fn get_memory_usage(&self) -> usize {
        self.memory_usage.load(Ordering::Relaxed)
    }
}

/// Generic adaptive cache implementation
pub struct AdaptiveCache<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    /// Underlying Moka cache instance
    cache: Cache<K, V>,
    /// Cache configuration
    config: AdaptiveCacheConfig,
    /// Performance metrics
    metrics: Arc<AdaptiveCacheMetrics>,
    /// Size controller
    size_controller: Arc<AdaptiveSizeController>,
    /// Last adjustment timestamp
    last_adjustment: AtomicI64,
}

impl<K, V> AdaptiveCache<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    pub fn new(config: AdaptiveCacheConfig) -> Self {
        let metrics = Arc::new(AdaptiveCacheMetrics::new());
        let size_controller = Arc::new(AdaptiveSizeController::new(config.min_size));
        
        let cache = Cache::builder()
            .max_capacity(config.max_size as u64)
            .build();

        Self {
            cache,
            config,
            metrics,
            size_controller,
            last_adjustment: AtomicI64::new(0),
        }
    }

    pub async fn get(&self, key: K) -> Option<V> {
        let start = Instant::now();
        let result = self.cache.get(&key).await;
        let latency = start.elapsed();
        
        // Update metrics
        self.metrics.record_get(result.is_some(), latency);
        self.size_controller.record_get(result.is_some());
        
        result
    }

    pub async fn insert(&self, key: K, value: V) -> Result<()> {
        self.cache.insert(key, value).await;
        Ok(())
    }

    pub fn get_metrics(&self) -> Arc<AdaptiveCacheMetrics> {
        self.metrics.clone()
    }

    pub fn get_size_controller(&self) -> Arc<AdaptiveSizeController> {
        self.size_controller.clone()
    }

    /// Adjust cache size based on hit rate and performance metrics
    pub async fn adjust_size(&mut self) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        // Check if enough time has passed since last adjustment
        let last = self.last_adjustment.load(Ordering::Relaxed);
        if now - last < self.config.adjustment_interval.as_secs() as i64 {
            return Ok(());
        }

        // Try to acquire adjustment lock using CAS
        if self.last_adjustment
            .compare_exchange(
                last,
                now,
                Ordering::SeqCst,
                Ordering::Relaxed
            )
            .is_err() {
            // Another thread is already adjusting
            return Ok(());
        }

        let hit_rate = self.size_controller.get_hit_rate();
        let current_size = self.size_controller.get_current_size();
        let mut new_size = current_size;

        // Calculate size adjustment based on hit rate
        if hit_rate < self.config.target_hit_rate {
            // Growth needed - hit rate too low
            new_size = ((current_size as f64) * self.config.growth_factor) as usize;
            info!(
                "Increasing cache size from {} to {} (hit rate {:.2}% below target {:.2}%)",
                current_size,
                new_size,
                hit_rate * 100.0,
                self.config.target_hit_rate * 100.0
            );
        } else if hit_rate > self.config.target_hit_rate + 0.05 {
            // Shrink possible - hit rate well above target
            new_size = ((current_size as f64) * self.config.shrink_factor) as usize;
            info!(
                "Decreasing cache size from {} to {} (hit rate {:.2}% above target {:.2}%)",
                current_size,
                new_size,
                hit_rate * 100.0,
                self.config.target_hit_rate * 100.0
            );
        }

        // Enforce size bounds
        new_size = new_size.clamp(self.config.min_size, self.config.max_size);

        if new_size != current_size {
            // Create new cache with adjusted capacity
            let mut new_cache = Cache::builder()
                .max_capacity(new_size as u64)
                .build();

            // Copy existing entries
            let entries: Vec<_> = self.cache.iter().collect();
            for (key, value) in entries {
                new_cache.insert((*key).clone(), value.clone()).await;
            }

            // Replace old cache using atomic swap
            self.cache = new_cache;
            self.size_controller.set_current_size(new_size);

            // Record metrics
            self.metrics.record_eviction();
            self.metrics.update_memory_usage(new_size);
        }

        // Update controller state
        self.size_controller.update_last_adjustment(now);

        Ok(())
    }

    /// Get the current memory pressure level (0.0 - 1.0)
    pub fn get_memory_pressure(&self) -> f64 {
        self.metrics.get_memory_usage() as f64 / (self.size_controller.get_current_size() * std::mem::size_of::<V>()) as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_adaptive_cache_size_adjustment() {
        let config = AdaptiveCacheConfig {
            min_size: 10,
            max_size: 1000,
            growth_factor: 2.0,
            shrink_factor: 0.5,
            target_hit_rate: 0.8,
            adjustment_interval: Duration::from_millis(100),
        };

        let cache: AdaptiveCache<String, String> = AdaptiveCache::new(config);

        // Insert some items
        for i in 0..20 {
            cache.insert(format!("key{}", i), format!("value{}", i)).await.unwrap();
        }

        // Simulate some hits and misses
        for i in 0..15 {
            cache.get(format!("key{}", i)).await;
        }

        // Force size adjustment
        cache.adjust_size().await.unwrap();

        // Verify size was adjusted based on hit rate
        let new_size = cache.size_controller.get_current_size();
        assert!(new_size > 10, "Cache should have grown due to good hit rate");

        // Test memory pressure
        let pressure = cache.get_memory_pressure();
        assert!(pressure >= 0.0 && pressure <= 1.0, "Memory pressure should be between 0 and 1");
    }
}

/// Thread-safe atomic f64 implementation using bit representation
#[derive(Debug)]
pub struct AtomicF64 {
    bits: AtomicU64,
}

impl AtomicF64 {
    pub fn new(val: f64) -> Self {
        Self {
            bits: AtomicU64::new(val.to_bits()),
        }
    }

    pub fn load(&self, order: Ordering) -> f64 {
        f64::from_bits(self.bits.load(order))
    }

    pub fn store(&self, val: f64, order: Ordering) {
        self.bits.store(val.to_bits(), order)
    }

    pub fn fetch_add(&self, val: f64, order: Ordering) -> f64 {
        let mut old = self.load(order);
        loop {
            let new = old + val;
            match self.compare_exchange_weak(old, new, order, order) {
                Ok(v) => return v,
                Err(v) => old = v,
            }
        }
    }

    pub fn compare_exchange_weak(&self, current: f64, new: f64, success: Ordering, failure: Ordering) -> Result<f64, f64> {
        let current_bits = current.to_bits();
        let new_bits = new.to_bits();
        self.bits
            .compare_exchange_weak(current_bits, new_bits, success, failure)
            .map(|v| f64::from_bits(v))
            .map_err(|v| f64::from_bits(v))
    }
}

/// Controller for adaptive cache sizing
#[derive(Debug)]
pub struct AdaptiveSizeController {
    /// Current cache size in items
    current_size: AtomicUsize,
    /// Current hit rate (0.0 - 1.0)
    hit_rate: AtomicF64,
    /// Timestamp of last size adjustment
    last_adjustment: AtomicI64,
    /// Total number of gets since last adjustment
    gets_since_adjustment: AtomicUsize,
    /// Number of hits since last adjustment
    hits_since_adjustment: AtomicUsize,
}

impl AdaptiveSizeController {
    pub fn new(initial_size: usize) -> Self {
        Self {
            current_size: AtomicUsize::new(initial_size),
            hit_rate: AtomicF64::new(1.0),
            last_adjustment: AtomicI64::new(0),
            gets_since_adjustment: AtomicUsize::new(0),
            hits_since_adjustment: AtomicUsize::new(0),
        }
    }

    pub fn record_get(&self, hit: bool) {
        self.gets_since_adjustment.fetch_add(1, Ordering::Relaxed);
        if hit {
            self.hits_since_adjustment.fetch_add(1, Ordering::Relaxed);
        }
        
        // Update hit rate
        let gets = self.gets_since_adjustment.load(Ordering::Relaxed);
        let hits = self.hits_since_adjustment.load(Ordering::Relaxed);
        if gets > 0 {
            let new_rate = hits as f64 / gets as f64;
            self.hit_rate.store(new_rate, Ordering::Relaxed);
        }
    }

    pub fn get_hit_rate(&self) -> f64 {
        self.hit_rate.load(Ordering::Relaxed)
    }

    pub fn get_current_size(&self) -> usize {
        self.current_size.load(Ordering::Relaxed)
    }

    pub fn set_current_size(&self, size: usize) {
        self.current_size.store(size, Ordering::Relaxed);
    }

    pub fn update_last_adjustment(&self, timestamp: i64) {
        self.last_adjustment.store(timestamp, Ordering::Relaxed);
        // Reset counters
        self.gets_since_adjustment.store(0, Ordering::Relaxed);
        self.hits_since_adjustment.store(0, Ordering::Relaxed);
    }

    pub fn get_last_adjustment(&self) -> i64 {
        self.last_adjustment.load(Ordering::Relaxed)
    }
}

/// Configuration for adaptive cache behavior
#[derive(Debug, Clone)]
pub struct AdaptiveCacheConfig {
    /// Minimum cache size in items
    pub min_size: usize,
    /// Maximum cache size in items
    pub max_size: usize,
    /// Factor to grow cache by when hit rate is low (e.g. 1.5 = grow by 50%)
    pub growth_factor: f64,
    /// Factor to shrink cache by when hit rate is high (e.g. 0.8 = shrink by 20%)
    pub shrink_factor: f64,
    /// Target cache hit rate (e.g. 0.9 = 90%)
    pub target_hit_rate: f64,
    /// How often to check and adjust cache size
    pub adjustment_interval: Duration,
}

impl Default for AdaptiveCacheConfig {
    fn default() -> Self {
        Self {
            min_size: 1_000,
            max_size: 100_000,
            growth_factor: 1.5,
            shrink_factor: 0.8,
            target_hit_rate: 0.90,
            adjustment_interval: Duration::from_secs(300), // 5 minutes
        }
    }
}

/// Types of cache for different use cases
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CacheType {
    Response,
    Embedding,
    Persistent,
}

/// Eviction policy types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvictionPolicy {
    TinyLFU,
    FrequencyBased,
    LRU,
}

/// Access pattern types for predictive caching
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AccessPattern {
    Sequential(Vec<String>),
    Correlated(String, String),
    Periodic(String, Duration),
}

/// Pattern detection and prediction system
#[derive(Debug)]
struct PredictionEngine {
    /// Recent access sequences
    recent_accesses: Vec<(String, std::time::Instant)>,
    /// Detected sequential patterns
    sequential_patterns: std::collections::HashMap<String, Vec<String>>,
    /// Correlated key pairs
    correlated_pairs: std::collections::HashMap<String, std::collections::HashSet<String>>,
    /// Periodic access patterns
    periodic_patterns: std::collections::HashMap<String, Vec<(Duration, std::time::Instant)>>,
    /// Maximum length of access history
    max_history: usize,
    /// Minimum confidence for pattern detection
    min_confidence: f64,
}

impl PredictionEngine {
    fn new(max_history: usize, min_confidence: f64) -> Self {
        Self {
            recent_accesses: Vec::with_capacity(max_history),
            sequential_patterns: std::collections::HashMap::new(),
            correlated_pairs: std::collections::HashMap::new(),
            periodic_patterns: std::collections::HashMap::new(),
            max_history,
            min_confidence,
        }
    }

    /// Record a new access and update patterns
    fn record_access(&mut self, key: &str) {
        let now = std::time::Instant::now();
        
        // Update recent accesses
        self.recent_accesses.push((key.to_string(), now));
        if self.recent_accesses.len() > self.max_history {
            self.recent_accesses.remove(0);
        }

        // Update sequential patterns
        if self.recent_accesses.len() >= 2 {
            let prev_key = &self.recent_accesses[self.recent_accesses.len() - 2].0;
            self.sequential_patterns
                .entry(prev_key.clone())
                .or_insert_with(Vec::new)
                .push(key.to_string());
        }

        // Update periodic patterns
        if let Some(last_access) = self.periodic_patterns.get_mut(key) {
            if let Some(prev_time) = last_access.last().map(|(_, t)| *t) {
                let duration = now.duration_since(prev_time);
                last_access.push((duration, now));
                if last_access.len() > 5 { // Keep last 5 accesses
                    last_access.remove(0);
                }
            }
        } else {
            self.periodic_patterns.insert(
                key.to_string(),
                vec![(Duration::from_secs(0), now)]
            );
        }
    }

    /// Predict next likely accesses based on patterns
    fn predict_next_accesses(&self, key: &str) -> Vec<AccessPattern> {
        let mut predictions = Vec::new();

        // Check sequential patterns
        if let Some(sequences) = self.sequential_patterns.get(key) {
            let mut freq_map = std::collections::HashMap::new();
            for next_key in sequences {
                *freq_map.entry(next_key).or_insert(0) += 1;
            }

            // Find sequences that meet confidence threshold
            let total = sequences.len() as f64;
            for (next_key, count) in freq_map {
                let confidence = count as f64 / total;
                if confidence >= self.min_confidence {
                    predictions.push(AccessPattern::Sequential(vec![next_key.clone()]));
                }
            }
        }

        // Check correlated pairs
        if let Some(correlated) = self.correlated_pairs.get(key) {
            for corr_key in correlated {
                predictions.push(AccessPattern::Correlated(key.to_string(), corr_key.clone()));
            }
        }

        // Check periodic patterns
        if let Some(timings) = self.periodic_patterns.get(key) {
            if timings.len() >= 3 {
                // Calculate average period
                let mut total_duration = Duration::from_secs(0);
                let mut count = 0;
                for window in timings.windows(2) {
                    total_duration += window[1].0;
                    count += 1;
                }
                if count > 0 {
                    let avg_period = total_duration / count as u32;
                    predictions.push(AccessPattern::Periodic(key.to_string(), avg_period));
                }
            }
        }

        predictions
    }
}

/// Cache configuration for each cache type
#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub max_capacity: u64,
    pub min_capacity: u64,
    pub time_to_live: Duration,
    pub time_to_idle: Duration,
    pub target_hit_rate: f64,
    pub growth_factor: f64,
    pub shrink_factor: f64,
    pub adjustment_interval: Duration,
    pub eviction_policy: EvictionPolicy,
    pub frequency_counter_max: u32,
    pub frequency_reset_interval: Duration,
    
    // Advanced adaptive sizing parameters
    pub memory_pressure_threshold: f64,      // Memory usage % that triggers size reduction
    pub latency_threshold_ms: u64,           // Average get latency threshold in ms
    pub min_items_for_adaptation: u64,       // Minimum number of items before adapting
    pub aggressive_growth_threshold: f64,     // Hit rate below which to use aggressive growth
    pub conservative_shrink_threshold: f64,   // Hit rate above which to use conservative shrink
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_capacity: 10_000,
            min_capacity: 1_000,
            time_to_live: Duration::from_secs(3600), // 1 hour
            time_to_idle: Duration::from_secs(1800), // 30 minutes
            target_hit_rate: 0.90,                   // 90% target hit rate
            growth_factor: 1.5,                      // Grow by 50% when hit rate is low
            shrink_factor: 0.8,                      // Shrink by 20% when hit rate is high
            adjustment_interval: Duration::from_secs(300), // Adjust every 5 minutes
            eviction_policy: EvictionPolicy::TinyLFU,
            frequency_counter_max: 15,               // 4-bit counter
            frequency_reset_interval: Duration::from_secs(3600), // Reset counters hourly
            
            // Advanced adaptive sizing defaults
            memory_pressure_threshold: 0.85,         // Reduce size at 85% memory usage
            latency_threshold_ms: 100,              // Target 100ms average latency
            min_items_for_adaptation: 1000,         // Wait for 1000 items before adapting
            aggressive_growth_threshold: 0.70,      // Grow more aggressively below 70% hit rate
            conservative_shrink_threshold: 0.95,    // Shrink conservatively above 95% hit rate
        }
    }
}

/// Generic cache value that can store different types of data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheValue {
    Response(String),
    Embedding(Vec<f32>),
    Persistent(Vec<u8>),
}

/// Frequency counter for TinyLFU
#[derive(Debug)]
struct FrequencyCounter {
    counters: std::collections::HashMap<String, u32>,
    last_reset: std::time::Instant,
    max_value: u32,
    reset_interval: Duration,
}

impl FrequencyCounter {
    fn new(max_value: u32, reset_interval: Duration) -> Self {
        Self {
            counters: std::collections::HashMap::new(),
            last_reset: std::time::Instant::now(),
            max_value,
            reset_interval,
        }
    }

    fn increment(&mut self, key: &str) {
        self.check_reset();
        let counter = self.counters.entry(key.to_string()).or_insert(0);
        if *counter < self.max_value {
            *counter += 1;
        }
    }

    fn get_frequency(&self, key: &str) -> u32 {
        *self.counters.get(key).unwrap_or(&0)
    }

    fn check_reset(&mut self) {
        let now = std::time::Instant::now();
        if now.duration_since(self.last_reset) >= self.reset_interval {
            self.counters.clear();
            self.last_reset = now;
        }
    }
}

/// Smart caching system that manages different types of caches
pub struct SmartCache {
    response_cache: Cache<String, CacheValue>,
    embedding_cache: Cache<String, CacheValue>,
    persistent_cache: Cache<String, CacheValue>,
    config: CacheConfig,
    last_adjustment: std::time::Instant,
    metrics_registry: Arc<MetricsRegistry>,
    frequency_counters: std::collections::HashMap<CacheType, FrequencyCounter>,
    prediction_engines: std::collections::HashMap<CacheType, PredictionEngine>,
}

impl SmartCache {
    /// Create a new SmartCache instance with default configurations
    pub fn new() -> Self {
        Self::with_config(CacheConfig::default(), Arc::new(MetricsRegistry::new()))
    }

    /// Create a new SmartCache instance with custom configuration and metrics registry
    pub fn with_metrics(metrics_registry: Arc<MetricsRegistry>) -> Self {
        Self::with_config(CacheConfig::default(), metrics_registry)
    }

    /// Initialize frequency counters and prediction engines for each cache type
    fn init_cache_components(config: &CacheConfig) -> (
        std::collections::HashMap<CacheType, FrequencyCounter>,
        std::collections::HashMap<CacheType, PredictionEngine>
    ) {
        let mut counters = std::collections::HashMap::new();
        let mut engines = std::collections::HashMap::new();
        
        for cache_type in [CacheType::Response, CacheType::Embedding, CacheType::Persistent] {
            counters.insert(
                cache_type,
                FrequencyCounter::new(config.frequency_counter_max, config.frequency_reset_interval)
            );
            engines.insert(
                cache_type,
                PredictionEngine::new(1000, 0.7) // 1000 recent accesses, 70% confidence threshold
            );
        }
        
        (counters, engines)
    }

    /// Create a cache builder with the specified eviction policy
    fn create_cache_with_policy(config: &CacheConfig) -> Cache<String, CacheValue> {
        let mut builder = Cache::builder()
            .max_capacity(config.max_capacity)
            .time_to_live(config.time_to_live)
            .time_to_idle(config.time_to_idle);

        match config.eviction_policy {
            EvictionPolicy::TinyLFU => {
                builder = builder
                    .weigher(|_k, _v| 1);
            },
            EvictionPolicy::FrequencyBased => {
                builder = builder
                    .weigher(|_k, _v| 1);
            },
            EvictionPolicy::LRU => {
                // Default Moka cache behavior is already LRU
            }
        }

        builder.build()
    }

    /// Create a new SmartCache instance with custom configuration and metrics registry
    pub fn with_config(config: CacheConfig, metrics_registry: Arc<MetricsRegistry>) -> Self {
        let response_cache = Self::create_cache_with_policy(&config);
        let embedding_cache = Self::create_cache_with_policy(&config);
        let persistent_cache = Self::create_cache_with_policy(&config);

        let (counters, engines) = Self::init_cache_components(&config);
        
        Self {
            response_cache,
            embedding_cache,
            persistent_cache,
            config: config.clone(),
            last_adjustment: std::time::Instant::now(),
            metrics_registry,
            frequency_counters: counters,
            prediction_engines: engines,
        }
    }

    /// Adjust cache size based on hit rate and memory usage
    async fn adjust_cache_size(&mut self, cache_type: CacheType) {
        let now = std::time::Instant::now();
        if now.duration_since(self.last_adjustment) < self.config.adjustment_interval {
            return;
        }

        let metrics = match self.metrics_registry.get_metrics(cache_type) {
            Some(m) => m,
            None => return,
        };

        let stats = metrics.get_stats();
        let cache = self.get_cache(cache_type);
        let current_capacity = self.config.max_capacity;
        
        // Don't adapt if we don't have enough data
        if stats.hit_rate == 0.0 || current_capacity < self.config.min_items_for_adaptation {
            return;
        }

        // Check memory pressure and latency thresholds
        let memory_usage_ratio = stats.memory_usage_bytes as f64 / stats.allocated_capacity_bytes as f64;
        let avg_latency_ms = stats.avg_get_latency.as_millis() as u64;

        let mut new_capacity = current_capacity;
        let mut resize_reason = String::new();

        // Handle high memory pressure scenario
        if memory_usage_ratio > self.config.memory_pressure_threshold {
            new_capacity = (current_capacity as f64 * self.config.shrink_factor) as u64;
            resize_reason = format!("high memory pressure ({:.1}%)", memory_usage_ratio * 100.0);
        }
        // Handle high latency scenario
        else if avg_latency_ms > self.config.latency_threshold_ms {
            new_capacity = (current_capacity as f64 * self.config.shrink_factor) as u64;
            resize_reason = format!("high latency ({}ms)", avg_latency_ms);
        }
        // Normal hit rate based adaptation
        else {
            new_capacity = if stats.hit_rate < self.config.target_hit_rate {
                if stats.hit_rate < self.config.aggressive_growth_threshold {
                    // Aggressive growth for very low hit rates
                    let growth = self.config.growth_factor * 1.5;
                    resize_reason = format!("aggressive growth (hit rate: {:.1}%)", stats.hit_rate * 100.0);
                    (current_capacity as f64 * growth) as u64
                } else {
                    // Normal growth
                    resize_reason = format!("normal growth (hit rate: {:.1}%)", stats.hit_rate * 100.0);
                    (current_capacity as f64 * self.config.growth_factor) as u64
                }
            } else if stats.hit_rate > self.config.conservative_shrink_threshold {
                // Conservative shrinking when hit rate is very high
                resize_reason = format!("conservative shrink (hit rate: {:.1}%)", stats.hit_rate * 100.0);
                (current_capacity as f64 * (self.config.shrink_factor + 0.1)) as u64
            } else if stats.hit_rate > self.config.target_hit_rate + 0.05 {
                // Normal shrinking
                resize_reason = format!("normal shrink (hit rate: {:.1}%)", stats.hit_rate * 100.0);
                (current_capacity as f64 * self.config.shrink_factor) as u64
            } else {
                current_capacity
            };
        }

        // Enforce capacity bounds
        new_capacity = new_capacity.max(self.config.min_capacity).min(self.config.max_capacity);

        if new_capacity != current_capacity {
            info!(
                "Adjusting {} cache size from {} to {} (reason: {})",
                format!("{:?}", cache_type),
                current_capacity,
                new_capacity,
                resize_reason
            );

            // Create new cache with adjusted capacity and policy
            let mut config = self.config.clone();
            config.max_capacity = new_capacity;
            let new_cache = Self::create_cache_with_policy(&config);

            // Copy existing entries to new cache
            let entries: Vec<_> = cache.iter().collect();
            for (key, value) in entries {
                new_cache.insert((*key).clone(), value.clone()).await;
            }

            // Replace old cache with new one
            match cache_type {
                CacheType::Response => self.response_cache = new_cache,
                CacheType::Embedding => self.embedding_cache = new_cache,
                CacheType::Persistent => self.persistent_cache = new_cache,
            }
        }

        // Update metrics and last adjustment time
        if let Some(metrics) = self.metrics_registry.get_metrics(cache_type) {
            metrics.record_resize();
        }
        self.last_adjustment = now;
    }

    /// Get the appropriate cache based on cache type
    fn get_cache(&self, cache_type: CacheType) -> &Cache<String, CacheValue> {
        match cache_type {
            CacheType::Response => &self.response_cache,
            CacheType::Embedding => &self.embedding_cache,
            CacheType::Persistent => &self.persistent_cache,
        }
    }

    /// Insert a value into the specified cache
    pub async fn insert(&self, cache_type: CacheType, key: String, value: CacheValue) -> Result<()> {
        let start = std::time::Instant::now();
        let cache = self.get_cache(cache_type);
        
        // Calculate approximate memory usage for the value
        let value_size = match &value {
            CacheValue::Response(s) => s.len(),
            CacheValue::Embedding(v) => v.len() * std::mem::size_of::<f32>(),
            CacheValue::Persistent(b) => b.len(),
        };

        cache.insert(key.clone(), value).await;
        
        // Update metrics
        if let Some(metrics) = self.metrics_registry.get_metrics(cache_type) {
            metrics.record_insert_latency(start.elapsed());
            metrics.update_memory_usage(
                self.get_size(cache_type) * value_size as u64
            );
            metrics.update_allocated_capacity(
                self.config.max_capacity * value_size as u64
            );
        }
        
        info!(cache_type = ?cache_type, key = %key, "Cache insert successful");
        Ok(())
    }

    /// Get a value from the specified cache with predictive loading
    pub async fn get(&self, cache_type: CacheType, key: &str) -> Option<CacheValue> {
        let start = std::time::Instant::now();
        let cache = self.get_cache(cache_type);
        let result = cache.get(key).await;

        // Update metrics
        if let Some(metrics) = self.metrics_registry.get_metrics(cache_type) {
            metrics.record_get_latency(start.elapsed());
            if result.is_some() {
                metrics.record_hit();
            } else {
                metrics.record_miss();
                warn!(cache_type = ?cache_type, key = %key, "Cache miss");
            }
        }
        
        // Update prediction engine and get predictions
        // Note: We can't mutate through Arc, so we'll skip prediction updates for now
        // TODO: Use interior mutability (Mutex/RwLock) for prediction engines

        // Adjust cache size if needed
        // Note: We can't mutate through Arc, so we'll skip size adjustment for now
        // TODO: Use interior mutability (Mutex/RwLock) for cache adjustment
        
        result
    }

    /// Prefetch values based on predicted access patterns
    pub async fn prefetch(&self, cache_type: CacheType, key: &str) {
        if let Some(engine) = self.prediction_engines.get(&cache_type) {
            let predictions = engine.predict_next_accesses(key);
            for pattern in predictions {
                match pattern {
                    AccessPattern::Sequential(keys) => {
                        for predicted_key in keys {
                            // TODO: Implement actual data fetching logic
                            info!("Prefetching sequential key: {}", predicted_key);
                        }
                    },
                    AccessPattern::Correlated(_, corr_key) => {
                        info!("Prefetching correlated key: {}", corr_key);
                    },
                    AccessPattern::Periodic(periodic_key, _) => {
                        info!("Prefetching periodic key: {}", periodic_key);
                    }
                }
            }
        }
    }

    /// Remove a value from the specified cache
    pub async fn remove(&self, cache_type: CacheType, key: &str) -> Result<()> {
        let cache = self.get_cache(cache_type);
        cache.remove(key).await;
        
        // Record metrics
        // Record cache operation metrics
        crate::metrics::record_memory_metrics(&format!("{:?}", cache_type), "remove", std::time::Duration::ZERO);
        info!(cache_type = ?cache_type, key = %key, "Cache remove successful");
        
        Ok(())
    }

    /// Clear all entries from the specified cache
    pub async fn clear(&self, cache_type: CacheType) -> Result<()> {
        let cache = self.get_cache(cache_type);
        cache.invalidate_all();
        
        // Record metrics
        // Record cache operation metrics
        crate::metrics::record_memory_metrics(&format!("{:?}", cache_type), "clear", std::time::Duration::ZERO);
        info!(cache_type = ?cache_type, "Cache cleared");
        
        Ok(())
    }

    /// Get the current size of the specified cache
    pub fn get_size(&self, cache_type: CacheType) -> u64 {
        let cache = self.get_cache(cache_type);
        cache.entry_count()
    }

    /// Get detailed performance metrics for a specific cache type
    pub fn get_cache_metrics(&self, cache_type: CacheType) -> Option<CachePerformanceReport> {
        let metrics = self.metrics_registry.get_metrics(cache_type)?;
        let stats = metrics.get_stats();
        let cache = self.get_cache(cache_type);

        Some(CachePerformanceReport {
            cache_type,
            current_size: self.get_size(cache_type),
            capacity: self.config.max_capacity,
            hit_rate: stats.hit_rate,
            memory_usage: MemoryMetrics {
                current_bytes: stats.memory_usage_bytes,
                allocated_bytes: stats.allocated_capacity_bytes,
                utilization: stats.memory_usage_bytes as f64 / stats.allocated_capacity_bytes as f64,
            },
            latency: LatencyMetrics {
                avg_get_ms: stats.avg_get_latency.as_millis() as f64,
                avg_insert_ms: stats.avg_insert_latency.as_millis() as f64,
            },
            operations: OperationMetrics {
                eviction_count: stats.eviction_count,
                resize_count: stats.resize_count,
            },
        })
    }

    /// Get performance reports for all cache types
    pub fn get_performance_report(&self) -> Vec<CachePerformanceReport> {
        let cache_types = [CacheType::Response, CacheType::Embedding, CacheType::Persistent];
        cache_types
            .iter()
            .filter_map(|&cache_type| self.get_cache_metrics(cache_type))
            .collect()
    }

    /// Log current performance metrics for monitoring
    pub fn log_performance_metrics(&self) {
        for report in self.get_performance_report() {
            let utilization_pct = (report.current_size as f64 / report.capacity as f64 * 100.0) as u64;
            let memory_used_mb = report.memory_usage.current_bytes as f64 / 1_048_576.0;
            let memory_allocated_mb = report.memory_usage.allocated_bytes as f64 / 1_048_576.0;
            let memory_utilization_pct = report.memory_usage.utilization * 100.0;
            
            info!(
                cache_type = ?report.cache_type,
                current_size = report.current_size,
                capacity = report.capacity,
                utilization_pct = utilization_pct,
                hit_rate_pct = report.hit_rate * 100.0,
                memory_used_mb = memory_used_mb,
                memory_allocated_mb = memory_allocated_mb,
                memory_utilization_pct = memory_utilization_pct,
                avg_get_ms = report.latency.avg_get_ms,
                avg_insert_ms = report.latency.avg_insert_ms,
                eviction_count = report.operations.eviction_count,
                resize_count = report.operations.resize_count,
                "Cache Performance Report: Size: {}/{} items ({}% full), Hit Rate: {:.1}%, Memory: {:.2}MB used / {:.2}MB allocated ({:.1}% utilization), Latency: {:.2}ms get / {:.2}ms insert, Operations: {} evictions, {} resizes",
                report.current_size,
                report.capacity,
                utilization_pct,
                report.hit_rate * 100.0,
                memory_used_mb,
                memory_allocated_mb,
                memory_utilization_pct,
                report.latency.avg_get_ms,
                report.latency.avg_insert_ms,
                report.operations.eviction_count,
                report.operations.resize_count,
            );
        }
    }
}

/// Detailed performance metrics for memory usage
#[derive(Debug, Clone)]
pub struct MemoryMetrics {
    pub current_bytes: u64,
    pub allocated_bytes: u64,
    pub utilization: f64,
}

/// Detailed performance metrics for operation latency
#[derive(Debug, Clone)]
pub struct LatencyMetrics {
    pub avg_get_ms: f64,
    pub avg_insert_ms: f64,
}

/// Detailed performance metrics for cache operations
#[derive(Debug, Clone)]
pub struct OperationMetrics {
    pub eviction_count: u64,
    pub resize_count: u64,
}

/// Comprehensive performance report for a cache instance
#[derive(Debug, Clone)]
pub struct CachePerformanceReport {
    pub cache_type: CacheType,
    pub current_size: u64,
    pub capacity: u64,
    pub hit_rate: f64,
    pub memory_usage: MemoryMetrics,
    pub latency: LatencyMetrics,
    pub operations: OperationMetrics,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_cache_operations() {
        let cache = SmartCache::new();
        
        // Test response cache
        let key = "test_key".to_string();
        let value = CacheValue::Response("test_response".to_string());
        
        cache.insert(CacheType::Response, key.clone(), value.clone()).await.unwrap();
        let retrieved = cache.get(CacheType::Response, &key).await.unwrap();
        
        match retrieved {
            CacheValue::Response(response) => assert_eq!(response, "test_response"),
            _ => panic!("Wrong cache value type"),
        }
        
        // Test cache removal
        cache.remove(CacheType::Response, &key).await.unwrap();
        assert!(cache.get(CacheType::Response, &key).await.is_none());
    }


    #[tokio::test]
    async fn test_cache_expiration() {
        let config = CacheConfig {
            max_capacity: 10,
            min_capacity: 5,
            time_to_live: Duration::from_millis(100),
            time_to_idle: Duration::from_millis(50),
            target_hit_rate: 0.9,
            growth_factor: 1.5,
            shrink_factor: 0.8,
            adjustment_interval: Duration::from_secs(300),
            eviction_policy: EvictionPolicy::TinyLFU,
            frequency_counter_max: 15,
            frequency_reset_interval: Duration::from_secs(3600),
        };
        
        let cache = SmartCache::with_config(config);
        let key = "expiring_key".to_string();
        let value = CacheValue::Response("expiring_response".to_string());
        
        cache.insert(CacheType::Response, key.clone(), value).await.unwrap();
        
        // Wait for cache to expire
        tokio::time::sleep(Duration::from_millis(150)).await;
        
        assert!(cache.get(CacheType::Response, &key).await.is_none());
    }
}