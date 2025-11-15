<<<<<<< HEAD
//! API module
//! 
//! Contains all API-related functionality including routes, authentication, and rate limiting.

mod auth;
mod key_manager;
mod per_key_rate_limit;

pub use auth::{AuthState, auth_middleware};
pub use key_manager::{ApiKeyManager, ApiKeyInfo};
pub use per_key_rate_limit::{PerKeyRateLimiter, per_key_rate_limit_middleware};

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use crate::conscience::{ConscienceEngine, MoralRule};
use crate::memory::{MemoryLayer, MemorySystem};
use crate::health::{HealthChecker, HealthResponse};
use crate::mqtt::MqttClient;
use crate::metrics::{
    MetricsMiddleware, RateLimitMiddleware, RateLimitConfig,
    init_metrics, record_http_request,
};
use crate::api::key_manager::ApiKeyManager;
use crate::api::per_key_rate_limit::PerKeyRateLimiter;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use sqlx::SqlitePool;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing::info;
=======
use axum::{
    extract::State,
    http::{HeaderName, Method, StatusCode},
    response::Json,
    routing::{get, post},
    Router,
    middleware,
};
use crate::conscience::{ConscienceEngine, MoralRule};
use crate::consciousness::ConsciousnessEngine;
use crate::memory::{MemoryLayer, MemorySystem};
use crate::security::JwtAuth;
use crate::security::validation::{ActionInput, RuleInput, validate_input};
use crate::security::auth::{login, jwt_auth_middleware};
use crate::security::headers::security_headers_middleware;
use crate::security::rate_limit::rate_limit_middleware;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tracing::{error, info, warn};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};

pub mod consciousness;
>>>>>>> origin/main

#[derive(Clone)]
pub struct AppState {
    pub conscience: Arc<ConscienceEngine>,
    pub memory: Arc<MemorySystem>,
<<<<<<< HEAD
    pub health: Arc<HealthChecker>,
    pub mqtt: Option<Arc<MqttClient>>,
    pub key_manager: Option<Arc<ApiKeyManager>>,
    pub per_key_rate_limiter: Option<Arc<PerKeyRateLimiter>>,
}

/// Basic health check (liveness probe)
async fn health(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(state.health.check_liveness().await)
}

/// Detailed health check with dependency verification
async fn health_detailed(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(state.health.check_detailed().await)
}

/// Metrics endpoint
async fn metrics() -> (StatusCode, String) {
    match metrics_exporter_prometheus::encode_to_string() {
        Ok(metrics) => (StatusCode::OK, metrics),
        Err(e) => {
            tracing::error!("Failed to encode metrics: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to encode metrics".into())
=======
    pub consciousness: Arc<ConsciousnessEngine>,
    pub jwt_auth: Arc<JwtAuth>,
    pub metrics_handle: PrometheusHandle,
}

/// Health check endpoint
async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "Jamey 3.0",
        "version": "3.0.0"
    }))
}

async fn get_metrics(
    State(state): State<AppState>,
) -> (StatusCode, String) {
    match state.metrics_handle.render() {
        Ok(metrics) => (StatusCode::OK, metrics),
        Err(e) => {
            error!("Failed to render metrics: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to render metrics".to_string())
>>>>>>> origin/main
        }
    }
}

<<<<<<< HEAD
/// Request body for action evaluation
#[derive(Deserialize)]
struct EvaluateRequest {
    action: String,
    entity_id: Option<String>, // Optional entity for soul integration
}

=======
>>>>>>> origin/main
/// Response for action evaluation
#[derive(Serialize)]
struct EvaluateResponse {
    score: f32,
    action: String,
<<<<<<< HEAD
    emotion: Option<String>, // Added for soul integration
}

/// Evaluate an action's morality
async fn evaluate_action(
    State(state): State<AppState>,
    Json(req): Json<EvaluateRequest>,
) -> Result<Json<EvaluateResponse>, StatusCode> {
    // Use evaluate_with_soul when entity_id is provided
    let (score, emotion) = match state.conscience.evaluate_with_soul(&req.action, req.entity_id.as_deref()).await {
        Ok((s, e)) => (s, e),
        Err(e) => {
            tracing::error!("Failed to evaluate action: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Store in short-term memory with entity link
    if let Err(e) = state
        .memory
        .store_with_entity(
            MemoryLayer::ShortTerm,
            format!("Action: {} | Score: {}", req.action, score),
            req.entity_id.as_deref(),
        )
        .await
    {
        tracing::error!("Failed to store memory: {}", e);
    }

    Ok(Json(EvaluateResponse {
        score,
        action: req.action,
        emotion: emotion.map(|e| e.to_string()),
    }))
}

/// Get all moral rules
async fn get_rules(State(state): State<AppState>) -> Json<Vec<MoralRule>> {
    Json(state.conscience.get_rules())
}

/// Add a new moral rule
#[derive(Deserialize)]
struct AddRuleRequest {
    name: String,
    description: String,
    weight: f32,
}

async fn add_rule(
    State(state): State<AppState>,
    Json(req): Json<AddRuleRequest>,
) -> Result<StatusCode, StatusCode> {
=======
}

/// Evaluate an action's morality (with authentication and validation)
async fn evaluate_action(
    State(state): State<AppState>,
    Json(req): Json<ActionInput>,
) -> Result<Json<EvaluateResponse>, StatusCode> {
    // Validate input
    if let Err(errors) = validate_input(&req) {
        warn!("Input validation failed for evaluate_action: {:?}", errors);
        return Err(StatusCode::BAD_REQUEST);
    }

    let score = state.conscience.evaluate(&req.action);

    // Store in short-term memory
    if let Err(e) = state
        .memory
        .store(
            MemoryLayer::ShortTerm,
            format!("Action: {} | Score: {}", req.action, score),
        )
        .await
    {
        error!("Failed to store memory: {}", e);
    }

    info!("Action evaluated: {} with score: {}", req.action, score);
    Ok(Json(EvaluateResponse {
        score,
        action: req.action,
    }))
}

/// Get all moral rules (with authentication)
async fn get_rules(State(state): State<AppState>) -> Json<Vec<MoralRule>> {
    info!("Retrieved all moral rules");
    Json(state.conscience.get_rules())
}

/// Add a new moral rule (with authentication and validation)
async fn add_rule(
    State(state): State<AppState>,
    Json(req): Json<RuleInput>,
) -> Result<StatusCode, StatusCode> {
    // Validate input
    if let Err(errors) = validate_input(&req) {
        warn!("Input validation failed for add_rule: {:?}", errors);
        return Err(StatusCode::BAD_REQUEST);
    }

>>>>>>> origin/main
    let rule = MoralRule {
        name: req.name,
        description: req.description,
        weight: req.weight,
    };
<<<<<<< HEAD
    state.conscience.add_rule(rule);
    Ok(StatusCode::CREATED)
}

/// Create the Axum application
pub async fn create_app(
    pool: SqlitePool,
    mqtt: Option<Arc<MqttClient>>,
    soul: Option<Arc<crate::soul::SoulStorage>>,
) -> anyhow::Result<Router> {
    // Initialize metrics
    init_metrics().await?;

    // Initialize conscience engine
    let conscience = Arc::new(ConscienceEngine::new());
=======
    state.conscience.add_rule(rule.clone());
    
    info!("Added new moral rule: {} with weight: {}", rule.name, rule.weight);
    Ok(StatusCode::CREATED)
}

/// Create the Axum application with comprehensive security
pub async fn create_app() -> anyhow::Result<Router> {
    // Initialize metrics endpoint
    let builder = PrometheusBuilder::new();
    let metrics_handle = builder.install_recorder()?;
>>>>>>> origin/main

    // Initialize memory system
    let data_dir = PathBuf::from("data/memory");
    let memory = Arc::new(MemorySystem::new(data_dir).await?);
<<<<<<< HEAD

    // Initialize API key manager
    let pool_arc = Arc::new(pool.clone());
    let key_manager = Arc::new(ApiKeyManager::new(pool_arc.clone()));
    
    // Initialize per-key rate limiter
    let per_key_rate_limiter = Arc::new(PerKeyRateLimiter::new(
        60, // Default 60 requests per minute
        Some(key_manager.clone()),
    ));

    // Initialize health checker
    let health = Arc::new(HealthChecker::new(
        pool,
        memory.clone(),
        mqtt.clone(),
    ));
=======
    
    // Initialize conscience engine
    let conscience = Arc::new(ConscienceEngine::new());
    
    // Initialize consciousness engine
    let consciousness = Arc::new(ConsciousnessEngine::new(memory.clone()).await?);

    // Initialize JWT authentication
    let jwt_auth = match JwtAuth::new() {
        Ok(auth) => Arc::new(auth),
        Err(e) => {
            error!("Failed to initialize JWT authentication: {}", e);
            return Err(anyhow::anyhow!("JWT authentication initialization failed: {}", e));
        }
    };
>>>>>>> origin/main

    let state = AppState {
        conscience,
        memory,
<<<<<<< HEAD
        health,
        mqtt,
        key_manager: Some(key_manager.clone()),
        per_key_rate_limiter: Some(per_key_rate_limiter.clone()),
    };

    // Configure global rate limiting (fallback)
    let rate_limit = RateLimitConfig {
        requests_per_second: 50,  // Adjust based on your needs
        burst_size: 100,
    };

    // Build middleware stack
    let middleware = ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .layer(tower::layer::layer_fn(|s| MetricsMiddleware::new(s)))
        // Global rate limiting (fallback)
        .layer(tower::layer::layer_fn(move |s| RateLimitMiddleware::new(s, rate_limit.clone())))
        .into_inner();

    let app = Router::new()
        .route("/", get(health))
        .route("/health", get(health_detailed))
        .route("/metrics", get(metrics))
        .route("/evaluate", post(evaluate_action))
        .route("/rules", get(get_rules))
        .route("/rules", post(add_rule))
        .with_state(state)
        // Per-key rate limiting (applied to all routes)
        .layer(axum::middleware::from_fn_with_state(
            per_key_rate_limiter.clone(),
            crate::api::per_key_rate_limit::per_key_rate_limit_middleware,
        ))
        .layer(middleware)
        .layer(
            tower_http::cors::CorsLayer::new()
                .allow_origin(tower_http::cors::Any)
                .allow_methods(tower_http::cors::Any)
                .allow_headers(tower_http::cors::Any),
        );

    info!("API routes configured with metrics, per-key rate limiting, and API key management");
    Ok(app)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request;
    use axum::body::Body;
    use tower::ServiceExt;
    use serde_json::json;

    #[tokio::test]
    async fn test_health_endpoints() {
        // Set up test app
        let pool = sqlx::sqlite::SqlitePool::connect("sqlite::memory:").await.unwrap();
        let app = create_app(pool, None).await.unwrap();

        // Test basic health check
        let response = app
            .clone()
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Test detailed health check
        let response = app
            .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let health: HealthResponse = serde_json::from_slice(&body).unwrap();
        assert!(matches!(health.status, "ok" | "degraded"));
    }

    #[tokio::test]
    async fn test_metrics_endpoint() {
        let pool = sqlx::sqlite::SqlitePool::connect("sqlite::memory:").await.unwrap();
        let app = create_app(pool, None).await.unwrap();

        let response = app
            .oneshot(Request::builder().uri("/metrics").body(Body::empty()).unwrap())
            .await
            .unwrap();
        
        assert_eq!(response.status(), StatusCode::OK);
        
        // Metrics should be Prometheus format
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let metrics = String::from_utf8(body.to_vec()).unwrap();
        assert!(metrics.contains("# HELP"));
    }
}
=======
        consciousness,
        jwt_auth,
        metrics_handle,
    };

    // Configure CORS from environment variables
    let cors_layer = create_cors_layer();

    // Build the application with security layers
    let app = Router::new()
        // Public endpoints (no authentication required)
        .route("/", get(health))
        .route("/login", post(login))
        .nest("/public", Router::new()
             .route("/health", get(health))
             .route("/metrics", get(get_metrics))
        )
        // Protected endpoints (authentication required)
        .route("/evaluate", post(evaluate_action))
        .route("/rules", get(get_rules))
        .route("/rules", post(add_rule))
        .route("/consciousness/metrics", get(consciousness::get_metrics))
        .route("/consciousness/config", get(consciousness::get_config))
        .route("/consciousness/toggle", post(consciousness::toggle_subsystems))
        .route("/consciousness/process", post(consciousness::process_information))
        .layer(middleware::from_fn_with_state(state.jwt_auth.clone(), jwt_auth_middleware))
        .with_state(state)
        // Apply global security layers
        .layer(middleware::from_fn(security_headers_middleware))
        .layer(middleware::from_fn(rate_limit_middleware))
        .layer(cors_layer);

    info!("Application created with comprehensive security layers");
    Ok(app)
}

/// Create CORS layer based on environment variables
fn create_cors_layer() -> CorsLayer {
    use std::env;
    
    // Get allowed origins from environment (comma-separated)
    let allowed_origins = env::var("CORS_ALLOWED_ORIGINS")
        .unwrap_or_else(|_| "http://localhost:5173".to_string())
        .split(',')
        .map(|s| s.trim().to_string())
        .collect::<Vec<String>>();

    // In development, allow localhost origins
    // In production, use explicit list from environment
    let cors_builder = if allowed_origins.contains(&"*".to_string()) || allowed_origins.is_empty() {
        // Fallback: allow localhost in development
        tracing::warn!("CORS_ALLOWED_ORIGINS not set or contains '*'. Allowing localhost only.");
        CorsLayer::new()
            .allow_origin(AllowOrigin::list([
                "http://localhost:5173".parse().unwrap(),
                "http://localhost:3000".parse().unwrap(),
            ]))
    } else {
        // Production: use explicit origins
        let origins: Vec<_> = allowed_origins
            .iter()
            .filter_map(|origin| origin.parse().ok())
            .collect();
        
        if origins.is_empty() {
            tracing::warn!("No valid CORS origins found. Defaulting to localhost.");
            CorsLayer::new()
                .allow_origin(AllowOrigin::list([
                    "http://localhost:5173".parse().unwrap(),
                ]))
        } else {
            CorsLayer::new()
                .allow_origin(AllowOrigin::list(origins))
        }
    };

    // Get allowed methods from environment
    let allowed_methods = env::var("CORS_ALLOWED_METHODS")
        .unwrap_or_else(|_| "GET,POST,OPTIONS".to_string())
        .split(',')
        .map(|s| s.trim())
        .filter_map(|m| match m {
            "GET" => Some(Method::GET),
            "POST" => Some(Method::POST),
            "PUT" => Some(Method::PUT),
            "DELETE" => Some(Method::DELETE),
            "OPTIONS" => Some(Method::OPTIONS),
            "PATCH" => Some(Method::PATCH),
            _ => None,
        })
        .collect::<Vec<_>>();

    let methods = if allowed_methods.is_empty() {
        vec![Method::GET, Method::POST, Method::OPTIONS]
    } else {
        allowed_methods
    };

    // Get allowed headers from environment
    let allowed_headers_str = env::var("CORS_ALLOWED_HEADERS")
        .unwrap_or_else(|_| "Content-Type,Authorization".to_string());
    
    let headers: Vec<HeaderName> = allowed_headers_str
        .split(',')
        .map(|s| s.trim())
        .filter_map(|h| {
            // Convert header names to lowercase and try to parse
            let lower = h.to_lowercase();
            HeaderName::from_bytes(lower.as_bytes()).ok()
        })
        .collect();

    let headers = if headers.is_empty() {
        vec![
            HeaderName::from_static("content-type"),
            HeaderName::from_static("authorization"),
        ]
    } else {
        headers
    };

    cors_builder
        .allow_methods(methods)
        .allow_headers(headers)
        .allow_credentials(false) // Set to true if cookies/auth needed
}

>>>>>>> origin/main
