pub mod metrics;
pub mod retry;

use std::time::Duration;
use tokio::sync::{mpsc, oneshot};
use tracing::{debug, error, warn};
// Metrics macros are used with metrics:: prefix, not imported

use metrics::ChannelMetricsCollector;
use retry::{RetryConfig, retry_with_backoff};

// Message priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Low,
    Normal,
    High,
    Critical,
}

// Generic message wrapper for system communication
#[derive(Debug, Clone)]
pub struct Message<T> 
where
    T: Clone,
{
    pub id: uuid::Uuid,
    pub priority: Priority,
    pub payload: T,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub retry_count: u32,
}

impl<T: Clone> Message<T> {
    pub fn new(payload: T, priority: Priority) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            priority,
            payload,
            timestamp: chrono::Utc::now(),
            retry_count: 0,
        }
    }
}

// Channel configuration
#[derive(Debug, Clone)]
pub struct ChannelConfig {
    pub capacity: usize,
    pub retry_config: RetryConfig,
}

impl Default for ChannelConfig {
    fn default() -> Self {
        Self {
            capacity: 1000,
            retry_config: RetryConfig::default(),
        }
    }
}

// Error types for communication
#[derive(Debug, thiserror::Error)]
pub enum CommError {
    #[error("Channel send error: {0}")]
    SendError(String),
    
    #[error("Channel receive error: {0}")]
    RecvError(String),
    
    #[error("Channel closed")]
    ChannelClosed,
    
    #[error("Operation timed out")]
    Timeout,
    
    #[error("Max retries exceeded")]
    MaxRetriesExceeded,
}

// Bounded channel wrapper with backpressure handling and retries
#[derive(Debug)]
pub struct BoundedChannel<T: Clone> {
    sender: mpsc::Sender<Message<T>>,
    receiver: mpsc::Receiver<Message<T>>,
    config: ChannelConfig,
    metrics: ChannelMetricsCollector,
}

impl<T: Send + Clone + 'static> BoundedChannel<T> 
where
    T: Clone,
{
    pub fn new(name: &str, config: ChannelConfig) -> Self {
        let (sender, receiver) = mpsc::channel(config.capacity);
        Self {
            sender,
            receiver,
            config,
            metrics: ChannelMetricsCollector::new(name),
        }
    }

    pub async fn send(&self, msg: Message<T>) -> Result<(), CommError> {
        let metrics = self.metrics.clone();
        let sender = self.sender.clone();
        let msg_clone = msg.clone();

        // Use retry mechanism for send operation
        retry_with_backoff(
            || async {
                match sender.try_send(msg_clone.clone()) {
                    Ok(_) => {
                        metrics.record_send();
                        metrics.update_capacity(sender.capacity() as u64);
                        Ok(())
                    }
                    Err(mpsc::error::TrySendError::Full(_)) => {
                        metrics.record_retry();
                        Err(CommError::SendError("Channel full".to_string()))
                    }
                    Err(mpsc::error::TrySendError::Closed(_)) => {
                        metrics.record_error();
                        Err(CommError::ChannelClosed)
                    }
                }
            },
            &self.config.retry_config,
            &format!("send_message_{}", msg.id),
        )
        .await
    }

    pub async fn receive(&mut self) -> Option<Message<T>> {
        match self.receiver.recv().await {
            Some(msg) => {
                self.metrics.record_receive();
                self.metrics.update_capacity(self.sender.capacity() as u64);
                Some(msg)
            }
            None => {
                self.metrics.record_error();
                None
            }
        }
    }

    pub fn get_metrics(&self) -> metrics::ChannelMetrics {
        self.metrics.get_metrics()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::timeout;

    #[tokio::test]
    async fn test_channel_basic_operation() {
        let config = ChannelConfig {
            capacity: 2,
            retry_config: RetryConfig {
                initial_delay: Duration::from_millis(10),
                max_delay: Duration::from_millis(100),
                max_retries: 3,
                backoff_factor: 2.0,
                jitter: 0.1,
            },
        };

        let mut channel = BoundedChannel::<String>::new("test_channel", config);
        let msg = Message::new("test".to_string(), Priority::Normal);
        
        assert!(channel.send(msg.clone()).await.is_ok());
        
        let received = channel.receive().await.unwrap();
        assert_eq!(received.payload, "test");
        
        let metrics = channel.get_metrics();
        assert_eq!(metrics.messages_sent, 1);
        assert_eq!(metrics.messages_received, 1);
        assert_eq!(metrics.errors, 0);
    }

    #[tokio::test]
    async fn test_channel_backpressure_and_retry() {
        let config = ChannelConfig {
            capacity: 1,
            retry_config: RetryConfig {
                initial_delay: Duration::from_millis(10),
                max_delay: Duration::from_millis(50),
                max_retries: 2,
                backoff_factor: 2.0,
                jitter: 0.1,
            },
        };

        let channel = BoundedChannel::<i32>::new("test_backpressure", config);
        
        // Fill the channel
        assert!(channel.send(Message::new(1, Priority::Normal)).await.is_ok());
        
        // This should trigger backpressure and retries
        let result = timeout(
            Duration::from_millis(200),
            channel.send(Message::new(2, Priority::Normal))
        ).await.unwrap();
        
        assert!(result.is_err());
        
        let metrics = channel.get_metrics();
        assert!(metrics.retries > 0);
        assert_eq!(metrics.messages_sent, 1);
    }

    #[tokio::test]
    async fn test_priority_ordering() {
        let config = ChannelConfig::default();
        let mut channel = BoundedChannel::<&str>::new("test_priority", config);

        // Send messages with different priorities
        let messages = vec![
            (Priority::Low, "low"),
            (Priority::Normal, "normal"),
            (Priority::High, "high"),
            (Priority::Critical, "critical"),
        ];

        for (priority, msg) in messages {
            channel.send(Message::new(msg, priority)).await.unwrap();
        }

        // Verify messages are received in priority order
        let mut received = Vec::new();
        while let Some(msg) = channel.receive().await {
            received.push((msg.priority, msg.payload));
        }

        // Check that higher priority messages were processed first
        for i in 1..received.len() {
            assert!(received[i-1].0 >= received[i].0);
        }
    }
}