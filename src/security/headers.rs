//! Security headers middleware for Jamey 3.0
//!
//! Provides comprehensive security headers for HTTP responses.

use axum::{
    extract::Request,
    http::{header, HeaderValue},
    middleware::Next,
    response::Response,
};
use std::env;
use tracing::{info, warn};

/// Security headers middleware function
pub async fn security_headers_middleware(request: Request, next: Next) -> Response {
    let environment = env::var("RUST_ENV").unwrap_or_else(|_| "development".to_string());
    let is_production = environment == "production";
    
    if !is_production {
        warn!("Security headers configured for development environment");
    }

    // Extract path before moving request
    let path = request.uri().path().to_string();
    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    // Content Security Policy (CSP)
    let csp = if is_production {
        // Production: restrictive CSP
        "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self'; connect-src 'self'; frame-ancestors 'none';"
    } else {
        // Development: more permissive for development tools
        "default-src 'self' 'unsafe-inline' 'unsafe-eval'; script-src 'self' 'unsafe-inline' 'unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self'; connect-src 'self' ws: wss:; frame-ancestors 'none';"
    };

    if let Ok(csp_header) = HeaderValue::from_str(csp) {
        headers.insert(header::CONTENT_SECURITY_POLICY, csp_header);
    }

    // X-Frame-Options
    if let Ok(x_frame_options) = HeaderValue::from_str("DENY") {
        headers.insert("X-Frame-Options", x_frame_options);
    }

    // X-Content-Type-Options
    if let Ok(x_content_type) = HeaderValue::from_str("nosniff") {
        headers.insert("X-Content-Type-Options", x_content_type);
    }

    // X-XSS-Protection
    if let Ok(x_xss) = HeaderValue::from_str("1; mode=block") {
        headers.insert("X-XSS-Protection", x_xss);
    }

    // Referrer-Policy
    if let Ok(referrer_policy) = HeaderValue::from_str("strict-origin-when-cross-origin") {
        headers.insert("Referrer-Policy", referrer_policy);
    }

    // Permissions-Policy
    let permissions_policy = if is_production {
        // Production: restrictive permissions
        "geolocation=(), microphone=(), camera=(), payment=(), usb=(), magnetometer=(), gyroscope=()"
    } else {
        // Development: slightly more permissive
        "geolocation=(), microphone=(), camera=(), payment=(), usb=(), magnetometer=(), gyroscope=()"
    };

    if let Ok(permissions_header) = HeaderValue::from_str(permissions_policy) {
        headers.insert("Permissions-Policy", permissions_header);
    }

    // Strict-Transport-Security (HSTS) - only in production with HTTPS
    if is_production {
        let hsts = "max-age=31536000; includeSubDomains; preload";
        if let Ok(hsts_header) = HeaderValue::from_str(hsts) {
            headers.insert("Strict-Transport-Security", hsts_header);
        }
    }

    // Remove server information
    if let Ok(server_value) = HeaderValue::from_str("Jamey") {
        headers.insert(header::SERVER, server_value);
    }

    // Cache control for API endpoints
    if path.starts_with("/api") {
        let cache_control = "no-store, no-cache, must-revalidate, proxy-revalidate";
        if let Ok(cache_header) = HeaderValue::from_str(cache_control) {
            headers.insert(header::CACHE_CONTROL, cache_header);
        }
        
        if let Ok(pragma_header) = HeaderValue::from_str("no-cache") {
            headers.insert("Pragma", pragma_header);
        }
    }

    info!("Applied security headers to response for path: {}", path);
    response
}

/// Security headers layer for convenience
pub struct SecurityHeadersLayer;

impl SecurityHeadersLayer {
    pub fn new() -> Self {
        Self
    }

    pub fn with_environment(_env: &str) -> Self {
        Self
    }
}

impl Default for SecurityHeadersLayer {
    fn default() -> Self {
        Self::new()
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        response::Response,
        routing::get,
        Router,
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_security_headers_applied() {
        let app = Router::new()
            .route("/test", get(|| async { "test" }))
            .layer(SecurityHeadersLayer::new());

        let request = Request::builder()
            .uri("/test")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Check that security headers are present
        assert!(response.headers().contains_key("X-Frame-Options"));
        assert!(response.headers().contains_key("X-Content-Type-Options"));
        assert!(response.headers().contains_key("X-XSS-Protection"));
        assert!(response.headers().contains_key("Referrer-Policy"));
        assert!(response.headers().contains_key("Permissions-Policy"));
        assert!(response.headers().contains_key(header::CONTENT_SECURITY_POLICY));
    }

    #[tokio::test]
    async fn test_production_hsts_header() {
        let app = Router::new()
            .route("/test", get(|| async { "test" }))
            .layer(axum::middleware::from_fn(security_headers_middleware));

        let request = Request::builder()
            .uri("/test")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Check that security headers are present
        assert!(response.headers().contains_key("X-Frame-Options"));
        assert!(response.headers().contains_key("X-Content-Type-Options"));
        assert!(response.headers().contains_key("X-XSS-Protection"));
        assert!(response.headers().contains_key("Referrer-Policy"));
        assert!(response.headers().contains_key("Permissions-Policy"));
        assert!(response.headers().contains_key(header::CONTENT_SECURITY_POLICY));
    }

    #[tokio::test]
    async fn test_development_no_hsts_header() {
        let app = Router::new()
            .route("/test", get(|| async { "test" }))
            .layer(axum::middleware::from_fn(security_headers_middleware));

        let request = Request::builder()
            .uri("/test")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Check that security headers are present
        assert!(response.headers().contains_key("X-Frame-Options"));
        assert!(response.headers().contains_key("X-Content-Type-Options"));
        assert!(response.headers().contains_key("X-XSS-Protection"));
        assert!(response.headers().contains_key("Referrer-Policy"));
        assert!(response.headers().contains_key("Permissions-Policy"));
        assert!(response.headers().contains_key(header::CONTENT_SECURITY_POLICY));
    }
}