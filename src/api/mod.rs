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

#[derive(Clone)]
pub struct AppState {
    pub conscience: Arc<ConscienceEngine>,
    pub memory: Arc<MemorySystem>,
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
        }
    }
}

/// Response for action evaluation
#[derive(Serialize)]
struct EvaluateResponse {
    score: f32,
    action: String,
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
pub async fn create_app() -> anyhow::Result<Router> {
    // Initialize metrics endpoint
    let builder = PrometheusBuilder::new();
    let metrics_handle = builder.install_recorder()?;

    // Initialize memory system
    let data_dir = PathBuf::from("data/memory");
    let memory = Arc::new(MemorySystem::new(data_dir).await?);
    
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

    let state = AppState {
        conscience,
        memory,
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

