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

#[derive(Clone)]
pub struct AppState {
    pub conscience: Arc<ConscienceEngine>,
    pub memory: Arc<MemorySystem>,
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
        }
    }
}

/// Request body for action evaluation
#[derive(Deserialize)]
struct EvaluateRequest {
    action: String,
    entity_id: Option<String>, // Optional entity for soul integration
}

/// Response for action evaluation
#[derive(Serialize)]
struct EvaluateResponse {
    score: f32,
    action: String,
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
    let rule = MoralRule {
        name: req.name,
        description: req.description,
        weight: req.weight,
    };
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

    // Initialize memory system
    let data_dir = PathBuf::from("data/memory");
    let memory = Arc::new(MemorySystem::new(data_dir).await?);

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

    let state = AppState {
        conscience,
        memory,
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
