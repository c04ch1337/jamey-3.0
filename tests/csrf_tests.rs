//! CSRF Protection Tests
//!
//! Tests for CSRF protection including:
//! - Token generation
//! - Token validation
//! - Middleware enforcement
//! - Cookie and header matching

use axum::{
    body::Body,
    http::{header, Method, Request, StatusCode},
    Router,
};
use axum_test::TestServer;
use jamey_3::security::{CsrfProtection, CsrfConfig, csrf_middleware, get_csrf_token};
use std::sync::Arc;
use tower::ServiceExt;

/// Helper to create a test router with CSRF protection
fn create_test_router() -> Router {
    let csrf = Arc::new(CsrfProtection::new(CsrfConfig {
        enabled: true,
        token_expiration: std::time::Duration::from_secs(3600),
        cookie_name: "csrf-token".to_string(),
        header_name: "x-csrf-token".to_string(),
        secret: None,
    }));

    Router::new()
        .route("/csrf-token", axum::routing::get(get_csrf_token))
        .route("/test-post", axum::routing::post(|| async { "OK" }))
        .layer(axum::middleware::from_fn_with_state(
            csrf.clone(),
            csrf_middleware,
        ))
        .with_state(csrf)
}

/// Test CSRF token generation
#[tokio::test]
async fn test_csrf_token_generation() {
    let csrf = CsrfProtection::new(CsrfConfig::default());
    
    let token1 = csrf.generate_token();
    let token2 = csrf.generate_token();
    
    // Tokens should be unique
    assert_ne!(token1, token2);
    
    // Tokens should start with "csrf_"
    assert!(token1.starts_with("csrf_"));
    assert!(token2.starts_with("csrf_"));
    
    // Tokens should be reasonably long
    assert!(token1.len() > 20);
    assert!(token2.len() > 20);
}

/// Test CSRF token storage and validation
#[tokio::test]
async fn test_csrf_token_storage_validation() {
    let csrf = CsrfProtection::new(CsrfConfig::default());
    
    let token = csrf.generate_token();
    
    // Store token
    csrf.store_token(token.clone()).await.unwrap();
    
    // Validate token
    assert!(csrf.validate_token(&token).await.is_ok());
    
    // Invalid token should fail
    assert!(csrf.validate_token("invalid_token").await.is_err());
    
    // Remove token
    csrf.remove_token(&token).await;
    
    // Token should no longer be valid
    assert!(csrf.validate_token(&token).await.is_err());
}

/// Test CSRF token endpoint
#[tokio::test]
async fn test_csrf_token_endpoint() {
    let app = create_test_router();
    let server = TestServer::new(app).unwrap();
    
    let response = server.get("/csrf-token").await;
    
    assert_eq!(response.status_code(), StatusCode::OK);
    
    let body: serde_json::Value = response.json();
    assert!(body["token"].is_string());
    assert_eq!(body["enabled"], true);
}

/// Test CSRF middleware blocks requests without token
#[tokio::test]
async fn test_csrf_middleware_blocks_no_token() {
    let app = create_test_router();
    let server = TestServer::new(app).unwrap();
    
    // POST request without CSRF token should be blocked
    let response = server.post("/test-post").await;
    
    assert_eq!(response.status_code(), StatusCode::FORBIDDEN);
}

/// Test CSRF middleware allows GET requests
#[tokio::test]
async fn test_csrf_middleware_allows_get() {
    let csrf = Arc::new(CsrfProtection::new(CsrfConfig::default()));
    
    let app = Router::new()
        .route("/test-get", axum::routing::get(|| async { "OK" }))
        .layer(axum::middleware::from_fn_with_state(
            csrf.clone(),
            csrf_middleware,
        ))
        .with_state(csrf);
    
    let server = TestServer::new(app).unwrap();
    
    // GET request should be allowed (not state-changing)
    let response = server.get("/test-get").await;
    
    assert_eq!(response.status_code(), StatusCode::OK);
}

/// Test CSRF middleware requires matching header and cookie
#[tokio::test]
async fn test_csrf_middleware_requires_matching_tokens() {
    let app = create_test_router();
    let server = TestServer::new(app).unwrap();
    
    // Get a CSRF token
    let token_response = server.get("/csrf-token").await;
    let token_body: serde_json::Value = token_response.json();
    let token = token_body["token"].as_str().unwrap();
    
    // POST with only header token (no cookie) should fail
    let response = server
        .post("/test-post")
        .add_header(header::HeaderName::from_static("x-csrf-token"), token)
        .await;
    
    assert_eq!(response.status_code(), StatusCode::FORBIDDEN);
    
    // POST with mismatched tokens should fail
    let response = server
        .post("/test-post")
        .add_header(header::HeaderName::from_static("x-csrf-token"), "different_token")
        .add_header(header::HeaderName::from_static("cookie"), format!("csrf-token={}", token))
        .await;
    
    assert_eq!(response.status_code(), StatusCode::FORBIDDEN);
}

/// Test CSRF middleware allows valid requests
#[tokio::test]
async fn test_csrf_middleware_allows_valid_request() {
    let app = create_test_router();
    let server = TestServer::new(app).unwrap();
    
    // Get a CSRF token
    let token_response = server.get("/csrf-token").await;
    let token_body: serde_json::Value = token_response.json();
    let token = token_body["token"].as_str().unwrap();
    
    // POST with matching header and cookie should succeed
    let response = server
        .post("/test-post")
        .add_header(header::HeaderName::from_static("x-csrf-token"), token)
        .add_header(header::HeaderName::from_static("cookie"), format!("csrf-token={}", token))
        .await;
    
    assert_eq!(response.status_code(), StatusCode::OK);
}

/// Test CSRF can be disabled
#[tokio::test]
async fn test_csrf_can_be_disabled() {
    let csrf = Arc::new(CsrfProtection::new(CsrfConfig {
        enabled: false,
        ..Default::default()
    }));
    
    let app = Router::new()
        .route("/test-post", axum::routing::post(|| async { "OK" }))
        .layer(axum::middleware::from_fn_with_state(
            csrf.clone(),
            csrf_middleware,
        ))
        .with_state(csrf);
    
    let server = TestServer::new(app).unwrap();
    
    // POST without token should succeed when CSRF is disabled
    let response = server.post("/test-post").await;
    
    assert_eq!(response.status_code(), StatusCode::OK);
}

/// Test expired CSRF token is rejected
#[tokio::test]
async fn test_expired_csrf_token() {
    let csrf = CsrfProtection::new(CsrfConfig {
        token_expiration: std::time::Duration::from_secs(1),
        ..Default::default()
    });
    
    let token = csrf.generate_token();
    csrf.store_token(token.clone()).await.unwrap();
    
    // Wait for token to expire
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    
    // Expired token should be rejected
    assert!(csrf.validate_token(&token).await.is_err());
}

