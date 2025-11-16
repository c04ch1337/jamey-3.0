use prometheus::{
    register_int_counter_vec, register_histogram_vec,
    IntCounterVec, HistogramVec,
};
use lazy_static::lazy_static;
use std::time::Instant;

lazy_static! {
    // Rate Limiting Metrics
    static ref RATE_LIMIT_HITS: IntCounterVec = register_int_counter_vec!(
        "rate_limit_hits_total",
        "Total number of rate limit hits",
        &["endpoint", "ip", "user_id"]
    ).unwrap();

    static ref RATE_LIMIT_REMAINING: HistogramVec = register_histogram_vec!(
        "rate_limit_remaining_tokens",
        "Number of remaining tokens in the bucket",
        &["endpoint", "ip", "user_id"],
        vec![1.0, 5.0, 10.0, 20.0, 50.0, 100.0]
    ).unwrap();

    // Audit Logging Metrics
    static ref AUDIT_EVENTS_TOTAL: IntCounterVec = register_int_counter_vec!(
        "audit_events_total",
        "Total number of audit events logged",
        &["event_type", "severity"]
    ).unwrap();

    static ref AUDIT_LOG_WRITE_DURATION: HistogramVec = register_histogram_vec!(
        "audit_log_write_duration_seconds",
        "Time taken to write audit log entries",
        &["event_type"],
        vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0]
    ).unwrap();

    static ref AUDIT_LOG_ROTATION_COUNT: IntCounterVec = register_int_counter_vec!(
        "audit_log_rotation_total",
        "Total number of log rotations performed",
        &["status"]
    ).unwrap();
}

pub struct SecurityMetrics;

impl SecurityMetrics {
    pub fn record_rate_limit_hit(endpoint: &str, ip: &str, user_id: &str) {
        RATE_LIMIT_HITS
            .with_label_values(&[endpoint, ip, user_id])
            .inc();
    }

    pub fn record_remaining_tokens(endpoint: &str, ip: &str, user_id: &str, tokens: f64) {
        RATE_LIMIT_REMAINING
            .with_label_values(&[endpoint, ip, user_id])
            .observe(tokens);
    }

    pub fn record_audit_event(event_type: &str, severity: &str) {
        AUDIT_EVENTS_TOTAL
            .with_label_values(&[event_type, severity])
            .inc();
    }

    pub fn start_audit_write_timer(event_type: &str) -> AuditWriteTimer {
        AuditWriteTimer {
            start: Instant::now(),
            event_type: event_type.to_string(),
        }
    }

    pub fn record_log_rotation(status: &str) {
        AUDIT_LOG_ROTATION_COUNT
            .with_label_values(&[status])
            .inc();
    }
}

pub struct AuditWriteTimer {
    start: Instant,
    event_type: String,
}

impl Drop for AuditWriteTimer {
    fn drop(&mut self) {
        let duration = self.start.elapsed().as_secs_f64();
        AUDIT_LOG_WRITE_DURATION
            .with_label_values(&[&self.event_type])
            .observe(duration);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_metrics() {
        SecurityMetrics::record_rate_limit_hit(
            "/api/test",
            "127.0.0.1",
            "user123",
        );
        SecurityMetrics::record_remaining_tokens(
            "/api/test",
            "127.0.0.1",
            "user123",
            50.0,
        );
    }

    #[test]
    fn test_audit_metrics() {
        SecurityMetrics::record_audit_event(
            "Authentication",
            "Info",
        );
        
        let _timer = SecurityMetrics::start_audit_write_timer("Authentication");
        // Timer will automatically record duration when dropped
        
        SecurityMetrics::record_log_rotation("success");
    }
}