//! Rate limiting middleware for Jamey 3.0
//!
//! Provides configurable rate limiting for API endpoints using the governor crate.

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{Response, IntoResponse},
};
use governor::{
    clock::QuantaClock,
    state::InMemoryState,
    Quota, RateLimiter,
};
use std::{
    num::NonZeroU32,
    sync::Arc,
};
use tracing::{info, warn};

/// Rate limiting middleware function
pub async fn rate_limit_middleware(request: Request, next: Next) -> Response {
    // Simple in-memory rate limiter for demonstration
    // In production, you'd want a more sophisticated solution with Redis or similar
    static RATE_LIMITER: std::sync::LazyLock<Arc<RateLimiter<governor::state::NotKeyed, InMemoryState, QuantaClock, governor::middleware::NoOpMiddleware>>> = 
        std::sync::LazyLock::new(|| {
            let quota = Quota::per_second(NonZeroU32::new(10).unwrap())
                .allow_burst(NonZeroU32::new(20).unwrap());
            Arc::new(RateLimiter::direct(quota))
        });

    // Check rate limit
    match RATE_LIMITER.check() {
        Ok(_) => {
            info!("Rate limit check passed for path: {}", request.uri().path());
            next.run(request).await
        }
        Err(_) => {
            warn!("Rate limit exceeded for path: {}", request.uri().path());
            let mut response = StatusCode::TOO_MANY_REQUESTS.into_response();
            response.headers_mut().insert(
                "Retry-After",
                "60".parse().unwrap(),
            );
            response
        }
    }
}

/// Rate limiting configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub max_requests: u32,
    pub window_seconds: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 100,
            window_seconds: 60,
        }
    }
}

impl RateLimitConfig {
    pub fn new(max_requests: u32, window_seconds: u64) -> Self {
        Self {
            max_requests,
            window_seconds,
        }
    }

    pub fn from_env() -> Self {
        let max_requests = std::env::var("RATE_LIMIT_MAX_REQUESTS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(100);

        let window_seconds = std::env::var("RATE_LIMIT_WINDOW_SECONDS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(60);

        Self {
            max_requests,
            window_seconds,
        }
    }
}

/// Rate limiter for authentication endpoints (stricter limits)
pub struct AuthRateLimitLayer;

impl AuthRateLimitLayer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for AuthRateLimitLayer {
    fn default() -> Self {
        Self::new()
    }
}

/// Auth rate limiting middleware function
pub async fn auth_rate_limit_middleware(request: Request, next: Next) -> Response {
    // Stricter rate limiting for auth endpoints: 5 requests per minute
    static AUTH_RATE_LIMITER: std::sync::LazyLock<Arc<RateLimiter<governor::state::NotKeyed, InMemoryState, QuantaClock, governor::middleware::NoOpMiddleware>>> = 
        std::sync::LazyLock::new(|| {
            let quota = Quota::per_minute(NonZeroU32::new(5).unwrap())
                .allow_burst(NonZeroU32::new(5).unwrap());
            Arc::new(RateLimiter::direct(quota))
        });

    // Check rate limit
    match AUTH_RATE_LIMITER.check() {
        Ok(_) => {
            info!("Auth rate limit check passed for path: {}", request.uri().path());
            next.run(request).await
        }
        Err(_) => {
            warn!("Auth rate limit exceeded for path: {}", request.uri().path());
            let mut response = StatusCode::TOO_MANY_REQUESTS.into_response();
            response.headers_mut().insert(
                "Retry-After",
                "60".parse().unwrap(),
            );
            response
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_config_default() {
        let config = RateLimitConfig::default();
        assert_eq!(config.max_requests, 100);
        assert_eq!(config.window_seconds, 60);
    }

    #[test]
    fn test_rate_limit_config_new() {
        let config = RateLimitConfig::new(50, 30);
        assert_eq!(config.max_requests, 50);
        assert_eq!(config.window_seconds, 30);
    }
}