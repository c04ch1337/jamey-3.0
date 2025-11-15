//! API module
//! 
//! Contains all API-related functionality including routes, authentication, and rate limiting.

mod auth;
mod key_manager;
mod per_key_rate_limit;

pub use auth::{AuthState, auth_middleware};
pub use key_manager::{ApiKeyManager, ApiKeyInfo};
pub use per_key_rate_limit::{PerKeyRateLimiter, per_key_rate_limit_middleware};

pub mod consciousness;

use axum::{
    extract::State,
    http::{HeaderName, Method, StatusCode, Request},
    response::Json,
    routing::{get, post},
    Router,
    middleware,
    middleware::Next,
};
use crate::conscience::{ConscienceEngine, MoralRule};
use crate::consciousness::ConsciousnessEngine;
use crate::memory::{MemoryLayer, MemorySystem};
use crate::health::{HealthChecker, HealthResponse};
use crate::mqtt::MqttClient;
use crate::metrics::{
    MetricsMiddleware, RateLimitMiddleware, RateLimitConfig,
    init_metrics, record_http_request,
};
// These types are already imported via pub use statements above
use crate::security::JwtAuth;
use crate::security::validation::{ActionInput, RuleInput, validate_input};
use crate::security::auth::{login, jwt_auth_middleware};
use crate::security::headers::security_headers_middleware;
use crate::security::rate_limit::rate_limit_middleware;
use crate::security::{
    DdosProtection, DdosProtectionConfig,
    ThreatDetection, ThreatDetectionConfig,
    IncidentResponse, IncidentResponseConfig,
    security_middleware,
    CsrfProtection, CsrfConfig, csrf_middleware, get_csrf_token,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use sqlx::SqlitePool;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tracing::{error, info, warn};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};

#[derive(Clone)]
pub struct AppState {
    pub conscience: Arc<ConscienceEngine>,
    pub memory: Arc<MemorySystem>,
    pub health: Option<Arc<HealthChecker>>,
    pub mqtt: Option<Arc<MqttClient>>,
    pub key_manager: Option<Arc<ApiKeyManager>>,
    pub per_key_rate_limiter: Option<Arc<PerKeyRateLimiter>>,
    pub consciousness: Option<Arc<ConsciousnessEngine>>,
    pub jwt_auth: Option<Arc<JwtAuth>>,
    pub metrics_handle: Option<PrometheusHandle>,
    pub csrf_protection: Option<Arc<CsrfProtection>>,
}

/// Basic health check (liveness probe)
async fn health(State(state): State<AppState>) -> Json<serde_json::Value> {
    if let Some(health_checker) = &state.health {
        let response = health_checker.check_liveness().await;
        Json(serde_json::json!({
            "status": response.status,
            "service": "Jamey 3.0",
            "version": "3.0.0",
            "timestamp": response.timestamp
        }))
    } else {
        Json(serde_json::json!({
            "status": "ok",
            "service": "Jamey 3.0",
            "version": "3.0.0"
        }))
    }
}

/// Detailed health check with dependency verification
async fn health_detailed(State(state): State<AppState>) -> Json<serde_json::Value> {
    if let Some(health_checker) = &state.health {
        let response = health_checker.check_detailed().await;
        Json(serde_json::json!({
            "status": response.status,
            "service": "Jamey 3.0",
            "version": "3.0.0",
            "timestamp": response.timestamp,
            "dependencies": response.dependencies
        }))
    } else {
        Json(serde_json::json!({
            "status": "ok",
            "service": "Jamey 3.0",
            "version": "3.0.0"
        }))
    }
}

/// Metrics endpoint
async fn metrics(State(state): State<AppState>) -> (StatusCode, String) {
    if let Some(handle) = &state.metrics_handle {
        match handle.render() {
            Ok(metrics) => (StatusCode::OK, metrics),
            Err(e) => {
                error!("Failed to render metrics: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Failed to render metrics".to_string())
            }
        }
    } else {
        match metrics_exporter_prometheus::encode_to_string() {
            Ok(metrics) => (StatusCode::OK, metrics),
            Err(e) => {
                error!("Failed to encode metrics: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Failed to encode metrics".to_string())
            }
        }
    }
}

/// Request body for action evaluation (with optional soul integration)
#[derive(Deserialize)]
struct EvaluateRequest {
    #[serde(flatten)]
    action_input: ActionInput,
    entity_id: Option<String>, // Optional entity for soul integration
}

/// Response for action evaluation
#[derive(Serialize)]
struct EvaluateResponse {
    score: f32,
    action: String,
}

/// Evaluate an action's morality (with input validation and optional soul integration)
async fn evaluate_action(
    State(state): State<AppState>,
    Json(req): Json<EvaluateRequest>,
) -> Result<Json<EvaluateResponse>, StatusCode> {
    // Validate input
    if let Err(errors) = validate_input(&req.action_input) {
        warn!("Input validation failed for evaluate_action: {:?}", errors);
        return Err(StatusCode::BAD_REQUEST);
    }

    // Use evaluate_with_soul when entity_id is provided, otherwise use standard evaluation
    let score = if let Some(entity_id) = &req.entity_id {
        match state.conscience.evaluate_with_soul(&req.action_input.action, Some(entity_id)).await {
            Ok((s, _)) => s,
            Err(e) => {
                error!("Failed to evaluate action with soul: {}", e);
                // Fallback to standard evaluation
                state.conscience.evaluate(&req.action_input.action)
            }
        }
    } else {
        state.conscience.evaluate(&req.action_input.action)
    };

    // Store in short-term memory with entity link if available
    let memory_result = if let Some(entity_id) = &req.entity_id {
        state
            .memory
            .store_with_entity(
                MemoryLayer::ShortTerm,
                format!("Action: {} | Score: {}", req.action_input.action, score),
                Some(entity_id),
            )
            .await
    } else {
        state
            .memory
            .store(
                MemoryLayer::ShortTerm,
                format!("Action: {} | Score: {}", req.action_input.action, score),
            )
            .await
    };

    if let Err(e) = memory_result {
        error!("Failed to store memory: {}", e);
    }

    info!("Action evaluated: {} with score: {}", req.action_input.action, score);
    Ok(Json(EvaluateResponse {
        score,
        action: req.action_input.action,
    }))
}

/// Get all moral rules
async fn get_rules(State(state): State<AppState>) -> Json<Vec<MoralRule>> {
    info!("Retrieved all moral rules");
    Json(state.conscience.get_rules())
}

/// Add a new moral rule (with input validation)
async fn add_rule(
    State(state): State<AppState>,
    Json(req): Json<RuleInput>,
) -> Result<StatusCode, StatusCode> {
    // Validate input
    if let Err(errors) = validate_input(&req) {
        warn!("Input validation failed for add_rule: {:?}", errors);
        return Err(StatusCode::BAD_REQUEST);
    }

    let rule = MoralRule {
        name: req.name,
        description: req.description,
        weight: req.weight,
    };
    state.conscience.add_rule(rule.clone());
    
    info!("Added new moral rule: {} with weight: {}", rule.name, rule.weight);
    Ok(StatusCode::CREATED)
}

/// Create the Axum application with comprehensive security
/// 
/// # Arguments
/// * `pool` - Database connection pool
/// * `mqtt` - Optional MQTT client
/// * `soul` - Optional Soul storage (for entity tracking)
pub async fn create_app(
    pool: SqlitePool,
    mqtt: Option<Arc<MqttClient>>,
    soul: Option<Arc<crate::soul::SoulStorage>>,
) -> anyhow::Result<Router> {
    // Initialize metrics
    let metrics_handle = init_metrics().await?;
    let prometheus_handle = PrometheusBuilder::new().install_recorder().ok();

    // Initialize memory system
    let data_dir = PathBuf::from("data/memory");
    let memory_system = MemorySystem::new(data_dir).await?;
    
    // Wire Soul storage to Memory system if provided
    let memory_system = if let Some(ref soul_storage) = soul {
        memory_system.with_soul_storage(soul_storage.clone())
    } else {
        memory_system
    };
    let memory = Arc::new(memory_system);

    // Initialize conscience engine
    let conscience_engine = ConscienceEngine::new();
    
    // Wire Soul storage to Conscience engine if provided
    let conscience_engine = if let Some(ref soul_storage) = soul {
        conscience_engine.with_soul_storage(soul_storage.clone())
    } else {
        conscience_engine
    };
    let conscience = Arc::new(conscience_engine);
    
    // Initialize consciousness engine (optional)
    let consciousness = ConsciousnessEngine::new(memory.clone()).await.ok().map(Arc::new);

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
        pool.clone(),
        memory.clone(),
        mqtt.clone(),
    ));

    // Initialize JWT authentication (optional, for advanced auth)
    let jwt_auth = JwtAuth::new().ok().map(Arc::new);

    // Initialize security systems
    let ddos_protection = Arc::new(DdosProtection::new(DdosProtectionConfig::from_env()));
    let threat_detection = Arc::new(ThreatDetection::new(ThreatDetectionConfig::from_env()));
    let incident_response = Arc::new(IncidentResponse::new(IncidentResponseConfig::from_env()));
    let csrf_protection = Arc::new(CsrfProtection::new(CsrfConfig::from_env()));

    let state = AppState {
        conscience,
        memory,
        health: Some(health),
        mqtt,
        key_manager: Some(key_manager.clone()),
        per_key_rate_limiter: Some(per_key_rate_limiter.clone()),
        consciousness,
        jwt_auth,
        metrics_handle: prometheus_handle,
        csrf_protection: Some(csrf_protection.clone()),
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

    // Configure CORS from environment variables (SECURE - fixes vulnerability)
    let cors_layer = create_cors_layer();

    // Build the application with security layers
    let mut app = Router::new()
        // Public endpoints (no authentication required)
        .route("/", get(health))
        .route("/health", get(health_detailed))
        .route("/metrics", get(metrics))
        // CSRF token endpoint (public, for getting tokens)
        .route("/csrf-token", get(get_csrf_token))
        // Protected endpoints
        .route("/evaluate", post(evaluate_action))
        .route("/rules", get(get_rules))
        .route("/rules", post(add_rule));

    // Add consciousness endpoints if available
    if state.consciousness.is_some() {
        app = app
            .route("/consciousness/metrics", get(consciousness::get_metrics))
            .route("/consciousness/config", get(consciousness::get_config))
            .route("/consciousness/toggle", post(consciousness::toggle_subsystems))
            .route("/consciousness/process", post(consciousness::process_information));
    }

    // Add JWT login endpoint if JWT auth is available
    if state.jwt_auth.is_some() {
        app = app.route("/login", post(login));
    }

    // Apply middleware layers (order matters - security first)
    let app = app
        .with_state(state)
        // Add security components to request extensions for middleware access
        .layer(axum::middleware::from_fn(move |mut request: Request, next: Next| {
            let ddos = ddos_protection.clone();
            let threat = threat_detection.clone();
            let incident = incident_response.clone();
            let csrf = csrf_protection.clone();
            
            request.extensions_mut().insert(ddos);
            request.extensions_mut().insert(threat);
            request.extensions_mut().insert(incident);
            request.extensions_mut().insert(csrf);
            
            async move {
                next.run(request).await
            }
        }))
        // CSRF protection (for state-changing operations)
        .layer(axum::middleware::from_fn_with_state(
            csrf_protection.clone(),
            csrf_middleware,
        ))
        // Combined security middleware (DDoS + Threat Detection + Incident Response)
        .layer(middleware::from_fn(security_middleware))
        // Per-key rate limiting (applied to all routes)
        .layer(axum::middleware::from_fn_with_state(
            per_key_rate_limiter.clone(),
            crate::api::per_key_rate_limit::per_key_rate_limit_middleware,
        ))
        .layer(middleware)
        // Security headers
        .layer(middleware::from_fn(security_headers_middleware))
        // Global rate limiting (fallback)
        .layer(middleware::from_fn(rate_limit_middleware))
        // SECURE CORS (fixes vulnerability - no longer allows all origins)
        .layer(cors_layer);

    info!("API routes configured with comprehensive security: DDoS protection, threat detection, incident response, per-key rate limiting, API key management, and secure CORS");
    Ok(app)
}

/// Create CORS layer based on environment variables
/// SECURITY: This fixes the vulnerability by restricting origins instead of allowing all
fn create_cors_layer() -> CorsLayer {
    use std::env;
    
    // Get allowed origins from environment (comma-separated)
    let allowed_origins = env::var("CORS_ALLOWED_ORIGINS")
        .or_else(|_| env::var("ALLOWED_ORIGINS"))
        .unwrap_or_else(|_| "http://localhost:5173,http://localhost:3000".to_string())
        .split(',')
        .map(|s| s.trim().to_string())
        .collect::<Vec<String>>();

    // In development, allow localhost origins
    // In production, use explicit list from environment
    let cors_builder = if allowed_origins.contains(&"*".to_string()) || allowed_origins.is_empty() {
        // Fallback: allow localhost in development only
        warn!("CORS_ALLOWED_ORIGINS not set or contains '*'. Allowing localhost only (development mode).");
        warn!("⚠️  SECURITY: Set CORS_ALLOWED_ORIGINS in production to restrict origins!");
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
            warn!("No valid CORS origins found. Defaulting to localhost.");
            CorsLayer::new()
                .allow_origin(AllowOrigin::list([
                    "http://localhost:5173".parse().unwrap(),
                ]))
        } else {
            info!("CORS configured with {} allowed origin(s)", origins.len());
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
        .unwrap_or_else(|_| "Content-Type,Authorization,x-api-key".to_string());
    
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
            HeaderName::from_static("x-api-key"),
        ]
    } else {
        headers
    };

    cors_builder
        .allow_methods(methods)
        .allow_headers(headers)
        .allow_credentials(false) // Set to true if cookies/auth needed
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
