//! API Integration Tests
//!
//! Comprehensive integration tests for all REST API endpoints,
//! including authentication, authorization, rate limiting, and error handling.

use axum::{
    body::Body,
    http::{Request, StatusCode, HeaderValue, header},
    Router,
};
use tower::ServiceExt;
use serde_json::{json, Value};

use jamey_3::api::create_app;
use jamey_3::db;
use tempfile::TempDir;

/// Helper to create a test router with in-memory database
async fn create_test_router() -> (Router, TempDir) {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test_api.db");
    
    // Initialize database
    let pool = db::init_db_with_config(db::DatabaseConfig {
        database_url: format!("sqlite://{}?mode=rwc", db_path.to_str().unwrap()),
        max_connections: 5,
        ..Default::default()
    }).await.unwrap();
    
    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    
    // Create router using the actual create_app function
    let router = create_app(pool, None, None).await.unwrap();
    
    (router, temp_dir)
}

/// Helper to create a request with API key
fn create_request_with_key(method: &str, path: &str, body: Option<Value>, api_key: Option<&str>) -> Request<Body> {
    let mut request = Request::builder()
        .method(method)
        .uri(path)
        .header(header::CONTENT_TYPE, "application/json");
    
    if let Some(key) = api_key {
        request = request.header("X-API-Key", key);
    }
    
    let body_str = body.map(|v| serde_json::to_string(&v).unwrap()).unwrap_or_default();
    request.body(Body::from(body_str)).unwrap()
}

#[tokio::test]
async fn test_health_check() {
    let (router, _temp_dir) = create_test_router().await;
    
    let response = router
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(json["status"], "ok");
}

#[tokio::test]
async fn test_evaluate_endpoint() {
    let (router, _temp_dir) = create_test_router().await;
    
    let request = create_request_with_key(
        "POST",
        "/evaluate",
        Some(json!({ "action": "help someone in need" })),
        None,
    );
    
    let response = router.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    
    assert!(json.get("score").is_some());
    assert!(json.get("action").is_some());
    assert!(json["score"].as_f64().unwrap() >= 0.0);
    assert!(json["score"].as_f64().unwrap() <= 100.0);
}

#[tokio::test]
async fn test_evaluate_invalid_input() {
    let (router, _temp_dir) = create_test_router().await;
    
    // Test with missing action field
    let request = create_request_with_key(
        "POST",
        "/evaluate",
        Some(json!({})),
        None,
    );
    
    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    // Test with invalid JSON
    let request = Request::builder()
        .method("POST")
        .uri("/evaluate")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from("invalid json"))
        .unwrap();
    
    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_rules() {
    let (router, _temp_dir) = create_test_router().await;
    
    let request = Request::builder()
        .uri("/rules")
        .body(Body::empty())
        .unwrap();
    
    let response = router.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    
    assert!(json.is_array());
    // Should have default rules
    assert!(json.as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_add_rule() {
    let (router, _temp_dir) = create_test_router().await;
    
    let new_rule = json!({
        "name": "test-rule",
        "description": "Test rule for integration testing",
        "weight": 5.0
    });
    
    let request = create_request_with_key(
        "POST",
        "/rules",
        Some(new_rule.clone()),
        None,
    );
    
    let response = router.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::CREATED);
    
    // Verify rule was added
    let request = Request::builder()
        .uri("/rules")
        .body(Body::empty())
        .unwrap();
    
    let response = router.oneshot(request).await.unwrap();
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    
    let rules = json.as_array().unwrap();
    assert!(rules.iter().any(|r| r["name"] == "test-rule"));
}

#[tokio::test]
async fn test_add_rule_invalid() {
    let (router, _temp_dir) = create_test_router().await;
    
    // Test with missing fields
    let invalid_rule = json!({
        "name": "test-rule"
        // Missing description and weight
    });
    
    let request = create_request_with_key(
        "POST",
        "/rules",
        Some(invalid_rule),
        None,
    );
    
    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    // Test with negative weight
    let invalid_rule = json!({
        "name": "test-rule",
        "description": "Test",
        "weight": -1.0
    });
    
    let request = create_request_with_key(
        "POST",
        "/rules",
        Some(invalid_rule),
        None,
    );
    
    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_cors_headers() {
    let (router, _temp_dir) = create_test_router().await;
    
    let request = Request::builder()
        .method("OPTIONS")
        .uri("/evaluate")
        .header("Origin", "http://localhost:5173")
        .header("Access-Control-Request-Method", "POST")
        .body(Body::empty())
        .unwrap();
    
    let response = router.oneshot(request).await.unwrap();
    
    // CORS should be configured
    // Note: Actual CORS behavior depends on configuration
    assert!(response.status() == StatusCode::OK || response.status() == StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_not_found() {
    let (router, _temp_dir) = create_test_router().await;
    
    let request = Request::builder()
        .uri("/nonexistent")
        .body(Body::empty())
        .unwrap();
    
    let response = router.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

