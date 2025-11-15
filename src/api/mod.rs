use axum::{
    extract::State,
    http::{HeaderName, Method, StatusCode},
    response::Json,
    routing::{get, post},
    Router,
};
use crate::conscience::{ConscienceEngine, MoralRule};
use crate::memory::{MemoryLayer, MemorySystem};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::cors::{AllowOrigin, CorsLayer};

#[derive(Clone)]
pub struct AppState {
    pub conscience: Arc<ConscienceEngine>,
    pub memory: Arc<MemorySystem>,
}

/// Health check endpoint
async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "Jamey 3.0",
        "version": "3.0.0"
    }))
}

/// Request body for action evaluation
#[derive(Deserialize)]
struct EvaluateRequest {
    action: String,
}

/// Response for action evaluation
#[derive(Serialize)]
struct EvaluateResponse {
    score: f32,
    action: String,
}

/// Evaluate an action's morality
async fn evaluate_action(
    State(state): State<AppState>,
    Json(req): Json<EvaluateRequest>,
) -> Result<Json<EvaluateResponse>, StatusCode> {
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
        tracing::error!("Failed to store memory: {}", e);
    }

    Ok(Json(EvaluateResponse {
        score,
        action: req.action,
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
pub async fn create_app() -> anyhow::Result<Router> {
    // Initialize conscience engine
    let conscience = Arc::new(ConscienceEngine::new());

    // Initialize memory system
    let data_dir = PathBuf::from("data/memory");
    let memory = Arc::new(MemorySystem::new(data_dir).await?);

    let state = AppState { conscience, memory };

    // Configure CORS from environment variables
    let cors_layer = create_cors_layer();

    let app = Router::new()
        .route("/", get(health))
        .route("/evaluate", post(evaluate_action))
        .route("/rules", get(get_rules))
        .route("/rules", post(add_rule))
        .with_state(state)
        .layer(cors_layer);

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

