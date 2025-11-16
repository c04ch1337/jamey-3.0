use metrics::{counter, gauge};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Metrics collector for communication channels
#[derive(Debug, Clone)]
pub struct ChannelMetricsCollector {
    prefix: String,
    messages_sent: Arc<AtomicU64>,
    messages_received: Arc<AtomicU64>,
    retries: Arc<AtomicU64>,
    errors: Arc<AtomicU64>,
    current_capacity: Arc<AtomicU64>,
}

impl ChannelMetricsCollector {
    pub fn new(prefix: &str) -> Self {
        Self {
            prefix: prefix.to_string(),
            messages_sent: Arc::new(AtomicU64::new(0)),
            messages_received: Arc::new(AtomicU64::new(0)),
            retries: Arc::new(AtomicU64::new(0)),
            errors: Arc::new(AtomicU64::new(0)),
            current_capacity: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn record_send(&self) {
        let count = self.messages_sent.fetch_add(1, Ordering::Relaxed);
        counter!(&format!("{}.messages_sent", self.prefix), 1);
        gauge!(&format!("{}.messages_sent_total", self.prefix), count as f64);
    }

    pub fn record_receive(&self) {
        let count = self.messages_received.fetch_add(1, Ordering::Relaxed);
        counter!(&format!("{}.messages_received", self.prefix), 1);
        gauge!(&format!("{}.messages_received_total", self.prefix), count as f64);
    }

    pub fn record_retry(&self) {
        let count = self.retries.fetch_add(1, Ordering::Relaxed);
        counter!(&format!("{}.retries", self.prefix), 1);
        gauge!(&format!("{}.retries_total", self.prefix), count as f64);
    }

    pub fn record_error(&self) {
        let count = self.errors.fetch_add(1, Ordering::Relaxed);
        counter!(&format!("{}.errors", self.prefix), 1);
        gauge!(&format!("{}.errors_total", self.prefix), count as f64);
    }

    pub fn update_capacity(&self, capacity: u64) {
        self.current_capacity.store(capacity, Ordering::Relaxed);
        gauge!(&format!("{}.current_capacity", self.prefix), capacity as f64);
    }

    pub fn get_metrics(&self) -> ChannelMetrics {
        ChannelMetrics {
            messages_sent: self.messages_sent.load(Ordering::Relaxed),
            messages_received: self.messages_received.load(Ordering::Relaxed),
            retries: self.retries.load(Ordering::Relaxed),
            errors: self.errors.load(Ordering::Relaxed),
            current_capacity: self.current_capacity.load(Ordering::Relaxed) as usize,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChannelMetrics {
    pub messages_sent: u64,
    pub messages_received: u64,
    pub retries: u64,
    pub errors: u64,
    pub current_capacity: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_collector() {
        let collector = ChannelMetricsCollector::new("test_channel");
        
        collector.record_send();
        collector.record_send();
        collector.record_receive();
        collector.record_retry();
        collector.record_error();
        collector.update_capacity(100);

        let metrics = collector.get_metrics();
        assert_eq!(metrics.messages_sent, 2);
        assert_eq!(metrics.messages_received, 1);
        assert_eq!(metrics.retries, 1);
        assert_eq!(metrics.errors, 1);
        assert_eq!(metrics.current_capacity, 100);
    }
}