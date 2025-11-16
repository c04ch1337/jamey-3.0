use std::future::Future;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, warn};

/// Configuration for retry behavior
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Initial delay between retries
    pub initial_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Multiplier for exponential backoff
    pub backoff_factor: f64,
    /// Random jitter factor (0.0 to 1.0)
    pub jitter: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            max_retries: 3,
            backoff_factor: 2.0,
            jitter: 0.1,
        }
    }
}

/// Retry operation with exponential backoff
pub async fn retry_with_backoff<F, Fut, T, E>(
    operation: F,
    config: &RetryConfig,
    context: &str,
) -> Result<T, E>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let mut current_delay = config.initial_delay;
    let mut attempt = 0;

    loop {
        match operation().await {
            Ok(value) => {
                if attempt > 0 {
                    debug!(
                        "Operation '{}' succeeded after {} retries",
                        context, attempt
                    );
                }
                return Ok(value);
            }
            Err(e) => {
                attempt += 1;
                if attempt >= config.max_retries {
                    warn!(
                        "Operation '{}' failed after {} attempts. Last error: {}",
                        context, attempt, e
                    );
                    return Err(e);
                }

                warn!(
                    "Operation '{}' failed (attempt {}/{}): {}. Retrying in {:?}",
                    context, attempt, config.max_retries, e, current_delay
                );

                // Add random jitter
                let jitter = rand::random::<f64>() * config.jitter;
                let delay_with_jitter = current_delay.mul_f64(1.0 + jitter);
                sleep(delay_with_jitter).await;

                // Calculate next delay with exponential backoff
                current_delay = std::cmp::min(
                    Duration::from_nanos((current_delay.as_nanos() as f64 * config.backoff_factor) as u64),
                    config.max_delay,
                );
            }
        }
    }
}

/// Helper trait for adding retry capability to operations
#[async_trait::async_trait]
pub trait Retryable {
    type Output;
    type Error;

    /// Execute the operation with retry logic
    async fn retry_with_config(&self, config: &RetryConfig, context: &str) -> Result<Self::Output, Self::Error>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_successful_retry() {
        let attempts = Arc::new(AtomicUsize::new(0));
        let attempts_clone = attempts.clone();

        let operation = || async move {
            let current = attempts_clone.fetch_add(1, Ordering::SeqCst);
            if current < 2 {
                Err("Temporary failure")
            } else {
                Ok("Success")
            }
        };

        let config = RetryConfig {
            initial_delay: Duration::from_millis(10),
            max_delay: Duration::from_millis(100),
            max_retries: 3,
            backoff_factor: 2.0,
            jitter: 0.1,
        };

        let result = retry_with_backoff(operation, &config, "test_operation").await;
        assert!(result.is_ok());
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_max_retries_exceeded() {
        let config = RetryConfig {
            initial_delay: Duration::from_millis(10),
            max_delay: Duration::from_millis(100),
            max_retries: 3,
            backoff_factor: 2.0,
            jitter: 0.1,
        };

        let operation = || async { Err::<(), &str>("Persistent failure") };
        let result = retry_with_backoff(operation, &config, "test_operation").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_exponential_backoff() {
        let timestamps = Arc::new(parking_lot::Mutex::new(Vec::new()));
        let timestamps_clone = timestamps.clone();

        let operation = || async move {
            timestamps_clone.lock().push(tokio::time::Instant::now());
            Err::<(), &str>("Failure")
        };

        let config = RetryConfig {
            initial_delay: Duration::from_millis(10),
            max_delay: Duration::from_millis(100),
            max_retries: 3,
            backoff_factor: 2.0,
            jitter: 0.0, // Disable jitter for predictable testing
        };

        let _ = retry_with_backoff(operation, &config, "test_operation").await;
        
        let timestamps = timestamps.lock();
        for i in 1..timestamps.len() {
            let duration = timestamps[i] - timestamps[i-1];
            assert!(duration >= config.initial_delay.mul_f64(config.backoff_factor.powi(i as i32 - 1)));
        }
    }
}