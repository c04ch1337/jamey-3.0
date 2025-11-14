use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use crate::conscience::{ConscienceEngine, MoralRule};
use crate::memory::{MemoryLayer, MemorySystem};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

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

    let app = Router::new()
        .route("/", get(health))
        .route("/evaluate", post(evaluate_action))
        .route("/rules", get(get_rules))
        .route("/rules", post(add_rule))
        .with_state(state)
        .layer(
            tower_http::cors::CorsLayer::new()
                .allow_origin(tower_http::cors::Any)
                .allow_methods(tower_http::cors::Any)
                .allow_headers(tower_http::cors::Any),
        );

    Ok(app)
}

