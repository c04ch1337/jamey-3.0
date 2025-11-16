use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use uuid::Uuid;
use crate::security::metrics::SecurityMetrics;

// Rate limit configurations
pub const DEFAULT_ENDPOINT_LIMIT: u32 = 1000;
pub const DEFAULT_IP_LIMIT: u32 = 100;
pub const DEFAULT_USER_LIMIT: u32 = 500;
pub const DEFAULT_REFILL_RATE: f64 = 10.0; // tokens per second

#[derive(Debug)]
pub struct TokenBucket {
    capacity: u32,
    tokens: f64,
    refill_rate: f64,
    last_update: Instant,
}

impl TokenBucket {
    pub fn new(capacity: u32, refill_rate: f64) -> Self {
        Self {
            capacity,
            tokens: capacity as f64,
            refill_rate,
            last_update: Instant::now(),
        }
    }

    pub fn try_consume(&mut self, tokens: u32) -> bool {
        self.refill();
        
        if self.tokens >= tokens as f64 {
            self.tokens -= tokens as f64;
            true
        } else {
            false
        }
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update).as_secs_f64();
        self.tokens = (self.tokens + self.refill_rate * elapsed).min(self.capacity as f64);
        self.last_update = now;
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum RateLimitKey {
    IP(String),
    User(Uuid),
    Endpoint(String),
}

pub struct RateLimiter {
    buckets: Arc<RwLock<HashMap<RateLimitKey, TokenBucket>>>,
    default_capacity: u32,
    default_refill_rate: f64,
}

impl RateLimiter {
    pub fn new(default_capacity: u32, refill_rate_per_second: f64) -> Self {
        Self {
            buckets: Arc::new(RwLock::new(HashMap::new())),
            default_capacity,
            default_refill_rate: refill_rate_per_second,
        }
    }

    pub async fn is_rate_limited(&self, key: RateLimitKey, cost: u32) -> bool {
        let mut buckets = self.buckets.write().await;
        
        let bucket = buckets.entry(key.clone()).or_insert_with(|| {
            TokenBucket::new(self.default_capacity, self.default_refill_rate)
        });

        let is_limited = !bucket.try_consume(cost);
        
        // Record metrics
        match &key {
            RateLimitKey::Endpoint(endpoint) => {
                if is_limited {
                    SecurityMetrics::record_rate_limit_hit(endpoint, "-", "-");
                }
                SecurityMetrics::record_remaining_tokens(endpoint, "-", "-", bucket.tokens);
            }
            RateLimitKey::IP(ip) => {
                if is_limited {
                    SecurityMetrics::record_rate_limit_hit("-", ip, "-");
                }
                SecurityMetrics::record_remaining_tokens("-", ip, "-", bucket.tokens);
            }
            RateLimitKey::User(user_id) => {
                if is_limited {
                    SecurityMetrics::record_rate_limit_hit("-", "-", &user_id.to_string());
                }
                SecurityMetrics::record_remaining_tokens("-", "-", &user_id.to_string(), bucket.tokens);
            }
        }

        is_limited
    }

    pub async fn get_remaining_tokens(&self, key: RateLimitKey) -> f64 {
        let mut buckets = self.buckets.write().await;
        
        let bucket = buckets.entry(key).or_insert_with(|| {
            TokenBucket::new(self.default_capacity, self.default_refill_rate)
        });

        bucket.refill();
        bucket.tokens
    }

    pub async fn check_request_limits(
        &self,
        endpoint: &str,
        ip: &str,
        user_id: Option<Uuid>,
    ) -> bool {
        // Check endpoint limit
        if self.is_rate_limited(RateLimitKey::Endpoint(endpoint.to_string()), 1).await {
            return true;
        }

        // Check IP limit
        if self.is_rate_limited(RateLimitKey::IP(ip.to_string()), 1).await {
            return true;
        }

        // Check user limit if user_id is provided
        if let Some(user_id) = user_id {
            if self.is_rate_limited(RateLimitKey::User(user_id), 1).await {
                return true;
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_rate_limiter() {
        let limiter = RateLimiter::new(10, 1.0); // 10 tokens, 1 token per second
        let key = RateLimitKey::IP("127.0.0.1".to_string());

        // Should allow initial requests
        assert!(!limiter.is_rate_limited(key.clone(), 1).await);
        assert!(!limiter.is_rate_limited(key.clone(), 1).await);

        // Should be rate limited after consuming all tokens
        for _ in 0..8 {
            assert!(!limiter.is_rate_limited(key.clone(), 1).await);
        }
        assert!(limiter.is_rate_limited(key.clone(), 1).await);

        // Should refill after waiting
        sleep(Duration::from_secs(2)).await;
        assert!(!limiter.is_rate_limited(key.clone(), 1).await);
    }

    #[test]
    fn test_token_bucket() {
        let mut bucket = TokenBucket::new(10, 1.0);
        
        assert!(bucket.try_consume(5));
        assert!(!bucket.try_consume(6)); // Not enough tokens
        assert!(bucket.try_consume(4)); // Can still consume remaining tokens
    }
}

// Create a new rate limiter with default configurations
pub fn create_default_rate_limiter() -> RateLimiter {
    RateLimiter::new(DEFAULT_ENDPOINT_LIMIT, DEFAULT_REFILL_RATE)
}

// Rate limit middleware for Axum
pub async fn rate_limit_middleware(
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    use axum::http::StatusCode;
    
    // Create a default rate limiter (in production, this should be shared state)
    let limiter = create_default_rate_limiter();
    
    let path = request.uri().path();
    let remote_addr = request
        .headers()
        .get("x-forwarded-for")
        .or_else(|| request.headers().get("x-real-ip"))
        .and_then(|h| h.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or("unknown").trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // Check rate limit
    if limiter.check_request_limits(path, &remote_addr, None).await {
        return axum::response::Response::builder()
            .status(StatusCode::TOO_MANY_REQUESTS)
            .body(axum::body::Body::from("Rate limit exceeded"))
            .unwrap();
    }

    next.run(request).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_comprehensive_rate_limiting() {
        let limiter = create_default_rate_limiter();
        let endpoint = "/api/test";
        let ip = "127.0.0.1";
        let user_id = Some(Uuid::new_v4());

        // Test normal request flow
        assert!(!limiter.check_request_limits(endpoint, ip, user_id).await);

        // Test endpoint limit
        for _ in 0..DEFAULT_ENDPOINT_LIMIT {
            let _ = limiter.is_rate_limited(RateLimitKey::Endpoint(endpoint.to_string()), 1).await;
        }
        assert!(limiter.is_rate_limited(RateLimitKey::Endpoint(endpoint.to_string()), 1).await);

        // Test IP limit
        for _ in 0..DEFAULT_IP_LIMIT {
            let _ = limiter.is_rate_limited(RateLimitKey::IP(ip.to_string()), 1).await;
        }
        assert!(limiter.is_rate_limited(RateLimitKey::IP(ip.to_string()), 1).await);

        // Test user limit
        if let Some(user_id) = user_id {
            for _ in 0..DEFAULT_USER_LIMIT {
                let _ = limiter.is_rate_limited(RateLimitKey::User(user_id), 1).await;
            }
            assert!(limiter.is_rate_limited(RateLimitKey::User(user_id), 1).await);
        }
    }
}