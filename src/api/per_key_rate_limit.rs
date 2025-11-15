//! Per-API-Key Rate Limiting
//!
//! Implements rate limiting on a per-API-key basis with configurable limits.

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

use crate::api::key_manager::ApiKeyManager;

/// Rate limit entry for tracking requests
struct RateLimitEntry {
    count: u32,
    reset_at: Instant,
    limit: u32,
}

/// Per-key rate limiter
#[derive(Clone)]
pub struct PerKeyRateLimiter {
    limits: Arc<Mutex<HashMap<String, RateLimitEntry>>>,
    default_limit: u32,
    key_manager: Option<Arc<ApiKeyManager>>,
}

impl PerKeyRateLimiter {
    /// Create a new per-key rate limiter
    pub fn new(default_limit: u32, key_manager: Option<Arc<ApiKeyManager>>) -> Self {
        Self {
            limits: Arc::new(Mutex::new(HashMap::new())),
            default_limit,
            key_manager,
        }
    }

    /// Check if a request should be allowed for a given key
    pub async fn check(&self, key_id: &str) -> Result<(), StatusCode> {
        // Get rate limit for this key
        let limit = if let Some(ref manager) = self.key_manager {
            // Try to get limit from database
            manager.get_rate_limit(key_id).await
                .ok()
                .flatten()
                .map(|l| l as u32)
                .unwrap_or(self.default_limit)
        } else {
            self.default_limit
        };

        let mut limits = self.limits.lock().unwrap();
        let now = Instant::now();

        // Clean up old entries
        limits.retain(|_, entry| entry.reset_at > now);

        let entry = limits.entry(key_id.to_string())
            .or_insert_with(|| RateLimitEntry {
                count: 0,
                reset_at: now + Duration::from_secs(60),
                limit,
            });

        // Update limit if it changed in database
        if entry.limit != limit {
            entry.limit = limit;
        }

        if entry.count >= entry.limit {
            warn!("Rate limit exceeded for key: {} (limit: {})", key_id, entry.limit);
            return Err(StatusCode::TOO_MANY_REQUESTS);
        }

        entry.count += 1;
        Ok(())
    }

    /// Set a custom rate limit for a key (for testing or manual override)
    pub fn set_limit(&self, key_id: &str, limit: u32) {
        let mut limits = self.limits.lock().unwrap();
        if let Some(entry) = limits.get_mut(key_id) {
            entry.limit = limit;
        }
    }
}

/// Middleware for per-key rate limiting
pub async fn per_key_rate_limit_middleware(
    axum::extract::State(limiter): axum::extract::State<Arc<PerKeyRateLimiter>>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract API key from headers
    let key_id = request
        .headers()
        .get("x-api-key")
        .or_else(|| request.headers().get("authorization"))
        .and_then(|h| h.to_str().ok())
        .map(|s| {
            if s.starts_with("Bearer ") {
                s.strip_prefix("Bearer ").unwrap_or(s).to_string()
            } else {
                s.to_string()
            }
        })
        .unwrap_or_else(|| "anonymous".to_string());

    // Check rate limit
    limiter.check(&key_id).await?;

    Ok(next.run(request).await)
}

