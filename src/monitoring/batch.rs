use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use metrics::{gauge, histogram};

/// Configuration for adaptive batch sizing
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// Minimum allowed batch size
    pub min_batch_size: u64,
    /// Maximum allowed batch size
    pub max_batch_size: u64,
    /// Target processing time in milliseconds
    pub target_processing_time_ms: u64,
    /// CPU usage threshold percentage for scaling down
    pub cpu_threshold: f64,
    /// Memory usage threshold percentage for scaling down
    pub memory_threshold: f64,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            min_batch_size: 10,
            max_batch_size: 1000,
            target_processing_time_ms: 100,
            cpu_threshold: 80.0,
            memory_threshold: 85.0,
        }
    }
}

/// Manages adaptive batch sizing based on system performance metrics
pub struct BatchOptimizer {
    config: BatchConfig,
    current_batch_size: Arc<AtomicU64>,
    processing_time_window: Vec<Duration>,
    window_size: usize,
}

impl BatchOptimizer {
    /// Creates a new BatchOptimizer with the given configuration
    pub fn new(config: BatchConfig) -> Self {
        let current_batch_size = Arc::new(AtomicU64::new(config.min_batch_size));
        gauge!("system.batch.target_size").set(config.min_batch_size as f64);
        
        Self {
            config,
            current_batch_size,
            processing_time_window: Vec::with_capacity(10),
            window_size: 10,
        }
    }

    /// Updates the batch size based on current system metrics and processing time
    pub fn optimize_batch_size(
        &mut self,
        cpu_usage: f64,
        memory_usage: f64,
        last_processing_time: Duration,
    ) -> u64 {
        // Record the processing time
        self.processing_time_window.push(last_processing_time);
        if self.processing_time_window.len() > self.window_size {
            self.processing_time_window.remove(0);
        }

        // Calculate average processing time
        let avg_processing_time = self.calculate_average_processing_time();
        let current_size = self.current_batch_size.load(Ordering::Relaxed);

        // Record metrics
        histogram!("system.batch.processing_time").record(avg_processing_time.as_secs_f64() * 1000.0);
        gauge!("system.batch.current_size").set(current_size as f64);

        // Determine if we need to adjust the batch size
        let new_batch_size = if cpu_usage > self.config.cpu_threshold || memory_usage > self.config.memory_threshold {
            // Scale down if system is under heavy load
            self.scale_down_batch_size(current_size)
        } else {
            // Adjust based on processing time
            self.adjust_for_processing_time(current_size, avg_processing_time)
        };

        // Update and return the new batch size
        self.current_batch_size.store(new_batch_size, Ordering::Relaxed);
        gauge!("system.batch.target_size").set(new_batch_size as f64);
        
        new_batch_size
    }

    /// Calculates the average processing time from the window
    fn calculate_average_processing_time(&self) -> Duration {
        if self.processing_time_window.is_empty() {
            return Duration::from_millis(0);
        }

        let total = self.processing_time_window.iter().sum::<Duration>();
        total / self.processing_time_window.len() as u32
    }

    /// Scales down the batch size when system is under heavy load
    fn scale_down_batch_size(&self, current_size: u64) -> u64 {
        let new_size = current_size / 2;
        new_size.max(self.config.min_batch_size)
    }

    /// Adjusts batch size based on processing time
    fn adjust_for_processing_time(&self, current_size: u64, avg_processing_time: Duration) -> u64 {
        let target_time = Duration::from_millis(self.config.target_processing_time_ms);
        let processing_ms = avg_processing_time.as_millis() as f64;
        let target_ms = target_time.as_millis() as f64;

        // Calculate adjustment factor based on how far we are from target time
        let adjustment_factor = target_ms / processing_ms;
        let new_size = (current_size as f64 * adjustment_factor).round() as u64;

        // Ensure we stay within configured bounds
        new_size.clamp(self.config.min_batch_size, self.config.max_batch_size)
    }

    /// Gets the current batch size
    pub fn get_current_batch_size(&self) -> u64 {
        self.current_batch_size.load(Ordering::Relaxed)
    }

    /// Gets the current configuration
    pub fn get_config(&self) -> &BatchConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_batch_optimizer_initialization() {
        let config = BatchConfig::default();
        let optimizer = BatchOptimizer::new(config.clone());
        assert_eq!(optimizer.get_current_batch_size(), config.min_batch_size);
    }

    #[test]
    fn test_batch_size_scaling() {
        let config = BatchConfig {
            min_batch_size: 10,
            max_batch_size: 1000,
            target_processing_time_ms: 100,
            cpu_threshold: 80.0,
            memory_threshold: 85.0,
        };
        let mut optimizer = BatchOptimizer::new(config);

        // Test scaling down under high CPU load
        let new_size = optimizer.optimize_batch_size(
            90.0, // High CPU
            50.0, // Normal memory
            Duration::from_millis(50),
        );
        assert!(new_size <= optimizer.get_current_batch_size());

        // Test scaling up under normal conditions
        let new_size = optimizer.optimize_batch_size(
            30.0, // Normal CPU
            30.0, // Normal memory
            Duration::from_millis(50), // Fast processing
        );
        assert!(new_size >= optimizer.get_current_batch_size());
    }

    #[test]
    fn test_processing_time_window() {
        let config = BatchConfig::default();
        let mut optimizer = BatchOptimizer::new(config);

        // Add several processing times
        for i in 1..=5 {
            optimizer.optimize_batch_size(
                50.0,
                50.0,
                Duration::from_millis(i * 10),
            );
        }

        // Verify window size hasn't exceeded capacity
        assert!(optimizer.processing_time_window.len() <= optimizer.window_size);
    }

    #[test]
    fn test_thread_safety() {
        let config = BatchConfig::default();
        let optimizer = Arc::new(BatchOptimizer::new(config));
        let mut handles = vec![];

        for _ in 0..10 {
            let optimizer_clone = Arc::clone(&optimizer);
            handles.push(thread::spawn(move || {
                assert!(optimizer_clone.get_current_batch_size() > 0);
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }
    }
}