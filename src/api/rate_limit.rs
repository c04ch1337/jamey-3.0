//! Rate limiting middleware
//!
//! Simple rate limiting based on IP address using in-memory storage.

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::warn;

/// Rate limit entry
struct RateLimitEntry {
    count: u32,
    reset_at: Instant,
}

/// Rate limiter state
#[derive(Clone)]
pub struct RateLimiter {
    /// Requests per minute allowed
    limit: u32,
    /// In-memory store of IP -> rate limit entry
    store: Arc<Mutex<HashMap<String, RateLimitEntry>>>,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(requests_per_minute: u32) -> Self {
        Self {
            limit: requests_per_minute,
            store: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Check if a request should be allowed
    fn check(&self, key: &str) -> Result<(), StatusCode> {
        let mut store = self.store.lock().unwrap();
        let now = Instant::now();

        // Clean up old entries
        store.retain(|_, entry| entry.reset_at > now);

        // Get or create entry
        let entry = store.entry(key.to_string()).or_insert_with(|| RateLimitEntry {
            count: 0,
            reset_at: now + Duration::from_secs(60),
        });

        // Check if limit exceeded
        if entry.count >= self.limit {
            warn!("Rate limit exceeded for key: {}", key);
            return Err(StatusCode::TOO_MANY_REQUESTS);
        }

        // Increment counter
        entry.count += 1;

        Ok(())
    }
}

/// Rate limiting middleware
pub async fn rate_limit_middleware(
    State(limiter): State<Arc<RateLimiter>>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract client IP (simplified - in production, use proper IP extraction)
    let client_ip = request
        .headers()
        .get("x-forwarded-for")
        .or_else(|| request.headers().get("x-real-ip"))
        .and_then(|h| h.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or("unknown").trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // Check rate limit
    limiter.check(&client_ip)?;

    Ok(next.run(request).await)
}

