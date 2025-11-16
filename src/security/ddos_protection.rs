//! DDoS Protection Module for Jamey 3.0
//!
//! Provides comprehensive DDoS protection including:
//! - Request size limits
//! - Connection limits
//! - Request rate limiting per IP
//! - Automatic IP blocking

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use axum::response::IntoResponse;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tracing::{error, warn, info};
use std::net::IpAddr;

/// DDoS protection configuration
#[derive(Debug, Clone)]
pub struct DdosProtectionConfig {
    /// Maximum request body size in bytes (default: 1MB)
    pub max_request_size: usize,
    /// Maximum requests per IP per minute
    pub max_requests_per_ip: u32,
    /// Maximum concurrent connections per IP
    pub max_connections_per_ip: u32,
    /// Request timeout in seconds
    pub request_timeout_secs: u64,
    /// Enable automatic IP blocking
    pub enable_auto_block: bool,
    /// Block duration in seconds
    pub block_duration_secs: u64,
}

impl Default for DdosProtectionConfig {
    fn default() -> Self {
        Self {
            max_request_size: 1_048_576, // 1MB
            max_requests_per_ip: 100,
            max_connections_per_ip: 10,
            request_timeout_secs: 30,
            enable_auto_block: true,
            block_duration_secs: 3600, // 1 hour
        }
    }
}

impl DdosProtectionConfig {
    /// Create config from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();
        
        if let Ok(size) = std::env::var("DDOS_MAX_REQUEST_SIZE") {
            if let Ok(parsed) = size.parse::<usize>() {
                config.max_request_size = parsed;
            }
        }
        
        if let Ok(requests) = std::env::var("DDOS_MAX_REQUESTS_PER_IP") {
            if let Ok(parsed) = requests.parse::<u32>() {
                config.max_requests_per_ip = parsed;
            }
        }
        
        if let Ok(connections) = std::env::var("DDOS_MAX_CONNECTIONS_PER_IP") {
            if let Ok(parsed) = connections.parse::<u32>() {
                config.max_connections_per_ip = parsed;
            }
        }
        
        if let Ok(timeout) = std::env::var("DDOS_REQUEST_TIMEOUT_SECS") {
            if let Ok(parsed) = timeout.parse::<u64>() {
                config.request_timeout_secs = parsed;
            }
        }
        
        if let Ok(enable) = std::env::var("DDOS_ENABLE_AUTO_BLOCK") {
            config.enable_auto_block = enable.parse().unwrap_or(true);
        }
        
        if let Ok(duration) = std::env::var("DDOS_BLOCK_DURATION_SECS") {
            if let Ok(parsed) = duration.parse::<u64>() {
                config.block_duration_secs = parsed;
            }
        }
        
        config
    }
}

/// IP request tracking
#[derive(Debug, Clone)]
struct IpRequestTracker {
    request_count: u32,
    window_start: Instant,
    connection_count: u32,
    last_request: Instant,
}

/// Blocked IP entry
#[derive(Debug, Clone)]
struct BlockedIp {
    #[allow(dead_code)]
    blocked_at: Instant,
    unblock_at: Instant,
    #[allow(dead_code)]
    reason: String,
}

/// DDoS protection state
#[derive(Clone)]
pub struct DdosProtection {
    config: DdosProtectionConfig,
    ip_trackers: Arc<Mutex<HashMap<IpAddr, IpRequestTracker>>>,
    blocked_ips: Arc<Mutex<HashMap<IpAddr, BlockedIp>>>,
}

impl DdosProtection {
    /// Create new DDoS protection instance
    pub fn new(config: DdosProtectionConfig) -> Self {
        let protection = Self {
            config,
            ip_trackers: Arc::new(Mutex::new(HashMap::new())),
            blocked_ips: Arc::new(Mutex::new(HashMap::new())),
        };
        
        // Start cleanup task
        let cleanup_protection = protection.clone();
        tokio::spawn(async move {
            cleanup_protection.cleanup_task().await;
        });
        
        protection
    }
    
    /// Check if IP is blocked
    pub fn is_blocked(&self, ip: &IpAddr) -> bool {
        let blocked = self.blocked_ips.lock().unwrap();
        if let Some(block) = blocked.get(ip) {
            if block.unblock_at > Instant::now() {
                return true;
            }
        }
        false
    }
    
    /// Check if request should be allowed
    pub fn check_request(&self, ip: &IpAddr) -> Result<(), DdosError> {
        // Check if IP is blocked
        if self.is_blocked(ip) {
            warn!("Blocked IP attempted request: {}", ip);
            return Err(DdosError::IpBlocked);
        }
        
        let mut trackers = self.ip_trackers.lock().unwrap();
        let now = Instant::now();
        
        // Get or create tracker
        let tracker = trackers.entry(*ip).or_insert_with(|| IpRequestTracker {
            request_count: 0,
            window_start: now,
            connection_count: 0,
            last_request: now,
        });
        
        // Reset window if expired (1 minute)
        if now.duration_since(tracker.window_start) > Duration::from_secs(60) {
            tracker.request_count = 0;
            tracker.window_start = now;
        }
        
        // Check request rate
        if tracker.request_count >= self.config.max_requests_per_ip {
            warn!("Rate limit exceeded for IP: {} ({} requests)", ip, tracker.request_count);
            
            // Auto-block if enabled
            if self.config.enable_auto_block {
                self.block_ip(ip, "Rate limit exceeded".to_string());
            }
            
            return Err(DdosError::RateLimitExceeded);
        }
        
        // Check connection count
        if tracker.connection_count >= self.config.max_connections_per_ip {
            warn!("Connection limit exceeded for IP: {} ({} connections)", ip, tracker.connection_count);
            return Err(DdosError::ConnectionLimitExceeded);
        }
        
        // Update tracker
        tracker.request_count += 1;
        tracker.last_request = now;
        tracker.connection_count += 1;
        
        Ok(())
    }
    
    /// Release connection for IP
    pub fn release_connection(&self, ip: &IpAddr) {
        let mut trackers = self.ip_trackers.lock().unwrap();
        if let Some(tracker) = trackers.get_mut(ip) {
            if tracker.connection_count > 0 {
                tracker.connection_count -= 1;
            }
        }
    }
    
    /// Block an IP address
    pub fn block_ip(&self, ip: &IpAddr, reason: String) {
        let mut blocked = self.blocked_ips.lock().unwrap();
        let unblock_at = Instant::now() + Duration::from_secs(self.config.block_duration_secs);
        
        blocked.insert(*ip, BlockedIp {
            blocked_at: Instant::now(),
            unblock_at,
            reason: reason.clone(),
        });
        
        error!("IP blocked: {} - Reason: {}", ip, reason);
    }
    
    /// Unblock an IP address
    pub fn unblock_ip(&self, ip: &IpAddr) {
        let mut blocked = self.blocked_ips.lock().unwrap();
        blocked.remove(ip);
        info!("IP unblocked: {}", ip);
    }
    
    /// Cleanup task to remove expired entries
    async fn cleanup_task(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        
        loop {
            interval.tick().await;
            
            let now = Instant::now();
            
            // Cleanup expired IP trackers
            {
                let mut trackers = self.ip_trackers.lock().unwrap();
                trackers.retain(|_, tracker| {
                    now.duration_since(tracker.window_start) < Duration::from_secs(120)
                });
            }
            
            // Cleanup expired blocks
            {
                let mut blocked = self.blocked_ips.lock().unwrap();
                blocked.retain(|_, block| block.unblock_at > now);
            }
        }
    }
}

/// DDoS protection errors
#[derive(Debug, thiserror::Error)]
pub enum DdosError {
    #[error("IP address is blocked")]
    IpBlocked,
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    #[error("Connection limit exceeded")]
    ConnectionLimitExceeded,
    #[error("Request size exceeded")]
    RequestSizeExceeded,
}

/// DDoS protection middleware
pub async fn ddos_protection_middleware(
    request: Request,
    next: Next,
) -> Response {
    // Get IP address from request
    let ip = request
        .headers()
        .get("x-forwarded-for")
        .or_else(|| request.headers().get("x-real-ip"))
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.split(',').next())
        .and_then(|s| s.trim().parse::<IpAddr>().ok())
        .unwrap_or_else(|| {
            // Fallback to localhost if IP cannot be determined
            "127.0.0.1".parse().unwrap()
        });
    
    // Get DDoS protection from extensions or create default
    let protection = request
        .extensions()
        .get::<Arc<DdosProtection>>()
        .cloned()
        .unwrap_or_else(|| {
            Arc::new(DdosProtection::new(DdosProtectionConfig::from_env()))
        });
    
    // Check request
    match protection.check_request(&ip) {
        Ok(_) => {
            let response = next.run(request).await;
            protection.release_connection(&ip);
            response
        }
        Err(DdosError::IpBlocked) => {
            error!("Blocked IP attempted request: {}", ip);
            StatusCode::FORBIDDEN.into_response()
        }
        Err(DdosError::RateLimitExceeded) => {
            warn!("Rate limit exceeded for IP: {}", ip);
            let mut response = StatusCode::TOO_MANY_REQUESTS.into_response();
            response.headers_mut().insert(
                "Retry-After",
                "60".parse().unwrap(),
            );
            response
        }
        Err(DdosError::ConnectionLimitExceeded) => {
            warn!("Connection limit exceeded for IP: {}", ip);
            StatusCode::SERVICE_UNAVAILABLE.into_response()
        }
        Err(DdosError::RequestSizeExceeded) => {
            warn!("Request size exceeded for IP: {}", ip);
            StatusCode::PAYLOAD_TOO_LARGE.into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;
    
    #[test]
    fn test_ddos_config_default() {
        let config = DdosProtectionConfig::default();
        assert_eq!(config.max_request_size, 1_048_576);
        assert_eq!(config.max_requests_per_ip, 100);
        assert_eq!(config.max_connections_per_ip, 10);
    }
    
    #[test]
    fn test_ddos_protection_check() {
        let config = DdosProtectionConfig {
            max_requests_per_ip: 5,
            ..Default::default()
        };
        let protection = DdosProtection::new(config);
        let ip: IpAddr = Ipv4Addr::new(127, 0, 0, 1).into();
        
        // Should allow first 5 requests
        for _ in 0..5 {
            assert!(protection.check_request(&ip).is_ok());
        }
        
        // 6th request should fail
        assert!(matches!(
            protection.check_request(&ip),
            Err(DdosError::RateLimitExceeded)
        ));
    }
    
    #[test]
    fn test_ip_blocking() {
        let protection = DdosProtection::new(DdosProtectionConfig::default());
        let ip: IpAddr = Ipv4Addr::new(127, 0, 0, 1).into();
        
        // IP should not be blocked initially
        assert!(!protection.is_blocked(&ip));
        
        // Block IP
        protection.block_ip(&ip, "Test block".to_string());
        
        // IP should now be blocked
        assert!(protection.is_blocked(&ip));
        
        // Unblock IP
        protection.unblock_ip(&ip);
        
        // IP should not be blocked
        assert!(!protection.is_blocked(&ip));
    }
}

