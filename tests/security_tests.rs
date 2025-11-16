//! Security Penetration Testing Suite
//!
//! Automated security tests for authentication, authorization, rate limiting,
//! DDoS protection, and input validation.

use axum::{
    body::Body,
    http::{header, Method, Request, StatusCode},
    Router,
};
use tower::ServiceExt;
use serde_json::json;
use std::sync::Arc;

use jamey_3::api::create_app;
use sqlx::SqlitePool;

/// Create a test router for security testing
async fn create_test_router() -> Router {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    
    // Run migrations
    sqlx::query("CREATE TABLE IF NOT EXISTS api_keys (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        key_hash TEXT NOT NULL UNIQUE,
        name TEXT NOT NULL,
        created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
        expires_at TEXT,
        revoked_at TEXT,
        last_used_at TEXT,
        rate_limit_per_minute INTEGER DEFAULT 60
    )").execute(&pool).await.unwrap();
    
    create_app(pool, None, None).await.unwrap()
}

/// Test authentication bypass attempts
#[tokio::test]
async fn test_auth_bypass_attempts() {
    let app = create_test_router().await;
    
    // Test 1: Missing API key
    let response = app
        .oneshot(
            Request::builder()
                .uri("/evaluate")
                .method(Method::POST)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(json!({"action": "test"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED, "Should reject requests without API key");
    
    // Test 2: Invalid API key format
    let response = app
        .oneshot(
            Request::builder()
                .uri("/evaluate")
                .method(Method::POST)
                .header(header::CONTENT_TYPE, "application/json")
                .header("X-API-Key", "invalid-key")
                .body(Body::from(json!({"action": "test"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED, "Should reject invalid API keys");
    
    // Test 3: Empty API key
    let response = app
        .oneshot(
            Request::builder()
                .uri("/evaluate")
                .method(Method::POST)
                .header(header::CONTENT_TYPE, "application/json")
                .header("X-API-Key", "")
                .body(Body::from(json!({"action": "test"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED, "Should reject empty API keys");
}

/// Test SQL injection attempts
#[tokio::test]
async fn test_sql_injection_attempts() {
    let app = create_test_router().await;
    
    let sql_injection_payloads = vec![
        "'; DROP TABLE api_keys; --",
        "' OR '1'='1",
        "'; SELECT * FROM api_keys; --",
        "1' UNION SELECT * FROM api_keys--",
        "admin'--",
        "admin'/*",
    ];
    
    for payload in sql_injection_payloads {
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/evaluate")
                    .method(Method::POST)
                    .header(header::CONTENT_TYPE, "application/json")
                    .header("X-API-Key", "test-key")
                    .body(Body::from(json!({"action": payload}).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        
        // Should not return 500 (internal server error) - should be handled gracefully
        assert_ne!(
            response.status(),
            StatusCode::INTERNAL_SERVER_ERROR,
            "SQL injection payload should not cause server error: {}",
            payload
        );
    }
}

/// Test XSS attempts
#[tokio::test]
async fn test_xss_attempts() {
    let app = create_test_router().await;
    
    let xss_payloads = vec![
        "<script>alert('XSS')</script>",
        "<img src=x onerror=alert('XSS')>",
        "javascript:alert('XSS')",
        "<svg onload=alert('XSS')>",
        "<body onload=alert('XSS')>",
    ];
    
    for payload in xss_payloads {
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/evaluate")
                    .method(Method::POST)
                    .header(header::CONTENT_TYPE, "application/json")
                    .header("X-API-Key", "test-key")
                    .body(Body::from(json!({"action": payload}).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        
        // Should handle XSS attempts gracefully
        assert_ne!(
            response.status(),
            StatusCode::INTERNAL_SERVER_ERROR,
            "XSS payload should not cause server error: {}",
            payload
        );
    }
}

/// Test rate limiting effectiveness
#[tokio::test]
async fn test_rate_limiting() {
    let app = create_test_router().await;
    
    // Send rapid requests
    let mut success_count = 0;
    let mut rate_limited_count = 0;
    
    for i in 0..150 {
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/")
                    .method(Method::GET)
                    .header("X-API-Key", "test-key")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        
        match response.status() {
            StatusCode::OK => success_count += 1,
            StatusCode::TOO_MANY_REQUESTS => rate_limited_count += 1,
            _ => {}
        }
        
        // Small delay to avoid overwhelming
        if i % 10 == 0 {
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
    }
    
    // Should have rate limited some requests
    assert!(
        rate_limited_count > 0 || success_count < 150,
        "Rate limiting should trigger after many requests"
    );
}

/// Test DDoS protection
#[tokio::test]
async fn test_ddos_protection() {
    let app = create_test_router().await;
    
    // Simulate rapid-fire requests from same IP
    let mut blocked_count = 0;
    
    for _ in 0..200 {
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/")
                    .method(Method::GET)
                    .header("X-Forwarded-For", "192.168.1.100")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        
        if response.status() == StatusCode::TOO_MANY_REQUESTS || 
           response.status() == StatusCode::SERVICE_UNAVAILABLE {
            blocked_count += 1;
        }
    }
    
    // DDoS protection should block some requests
    assert!(
        blocked_count > 0,
        "DDoS protection should block rapid-fire requests"
    );
}

/// Test input validation
#[tokio::test]
async fn test_input_validation() {
    let app = create_test_router().await;
    
    // Test 1: Missing required fields
    let response = app
        .oneshot(
            Request::builder()
                .uri("/evaluate")
                .method(Method::POST)
                .header(header::CONTENT_TYPE, "application/json")
                .header("X-API-Key", "test-key")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUEST,
        "Should reject requests with missing required fields"
    );
    
    // Test 2: Invalid JSON
    let response = app
        .oneshot(
            Request::builder()
                .uri("/evaluate")
                .method(Method::POST)
                .header(header::CONTENT_TYPE, "application/json")
                .header("X-API-Key", "test-key")
                .body(Body::from("invalid json"))
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUEST,
        "Should reject invalid JSON"
    );
    
    // Test 3: Oversized payload
    let large_payload = "x".repeat(2_000_000); // 2MB payload
    let response = app
        .oneshot(
            Request::builder()
                .uri("/evaluate")
                .method(Method::POST)
                .header(header::CONTENT_TYPE, "application/json")
                .header("X-API-Key", "test-key")
                .body(Body::from(json!({"action": large_payload}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    
    // Should reject or handle oversized payloads
    assert!(
        response.status() == StatusCode::PAYLOAD_TOO_LARGE || 
        response.status() == StatusCode::BAD_REQUEST,
        "Should reject oversized payloads"
    );
}

/// Test CORS security
#[tokio::test]
async fn test_cors_security() {
    let app = create_test_router().await;
    
    // Test preflight request
    let response = app
        .oneshot(
            Request::builder()
                .uri("/evaluate")
                .method(Method::OPTIONS)
                .header("Origin", "https://malicious-site.com")
                .header("Access-Control-Request-Method", "POST")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    // CORS should be properly configured (either allow or deny, but not default)
    let headers = response.headers();
    let has_cors_headers = headers.contains_key("access-control-allow-origin") ||
                          headers.contains_key("access-control-allow-methods");
    
    assert!(
        has_cors_headers,
        "CORS headers should be present for preflight requests"
    );
}

/// Test security headers
#[tokio::test]
async fn test_security_headers() {
    let app = create_test_router().await;
    
    let response = app
        .oneshot(
            Request::builder()
                .uri("/")
                .method(Method::GET)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    let headers = response.headers();
    
    // Check for security headers
    assert!(
        headers.contains_key("x-content-type-options"),
        "Should include X-Content-Type-Options header"
    );
    
    assert!(
        headers.contains_key("x-frame-options") || 
        headers.contains_key("x-xss-protection"),
        "Should include security headers"
    );
}

/// Test authorization bypass
#[tokio::test]
async fn test_authorization_bypass() {
    let app = create_test_router().await;
    
    // Test accessing protected endpoint without proper auth
    let response = app
        .oneshot(
            Request::builder()
                .uri("/rules")
                .method(Method::POST)
                .header(header::CONTENT_TYPE, "application/json")
                .header("X-API-Key", "invalid-key")
                .body(Body::from(json!({"rule": "test", "weight": 5.0}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "Should reject unauthorized access to protected endpoints"
    );
}

/// Test path traversal attempts
#[tokio::test]
async fn test_path_traversal() {
    let app = create_test_router().await;
    
    let path_traversal_payloads = vec![
        "../../../etc/passwd",
        "..\\..\\..\\windows\\system32",
        "/etc/passwd",
        "C:\\Windows\\System32",
    ];
    
    for payload in path_traversal_payloads {
        let response = app
            .oneshot(
                Request::builder()
                    .uri(&format!("/{}", payload))
                    .method(Method::GET)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        
        // Should not return 200 for path traversal attempts
        assert_ne!(
            response.status(),
            StatusCode::OK,
            "Path traversal attempt should not succeed: {}",
            payload
        );
    }
}

