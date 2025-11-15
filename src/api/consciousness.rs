//! Consciousness System API endpoints
//!
//! Provides HTTP endpoints for monitoring and controlling the consciousness system.
//!
//! # Consciousness System API
//!
//! This module provides HTTP endpoints for monitoring and controlling the consciousness system:
//!
//! - `GET /consciousness/metrics` - Get current consciousness metrics (Φ, workspace activity, etc.)
//! - `GET /consciousness/config` - Get current consciousness configuration
//! - `POST /consciousness/toggle` - Toggle consciousness subsystems on/off
//! - `POST /consciousness/process` - Process information through the consciousness system
//!
//! These endpoints allow external systems to monitor the consciousness state and
//! control its behavior at runtime.

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error};

use crate::api::AppState;
use crate::config::ConsciousnessConfig;
use crate::security::validation::{ContentInput, ToggleSubsystemInput, validate_input};

/// Response for consciousness metrics endpoint
#[derive(Serialize)]
pub struct ConsciousnessMetricsResponse {
    /// Phi value (integrated information)
    pub phi_value: f64,
    /// Global workspace activity level
    pub workspace_activity: f64,
    /// Current metacognition level
    pub metacognition_level: f64,
    /// Current attention focus
    pub attention_focus: String,
    /// Timestamp of metrics collection
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Request for toggling consciousness subsystems
#[derive(Deserialize)]
pub struct ToggleSubsystemRequest {
    /// Whether to enable the higher-order thought system
    pub enable_higher_order: Option<bool>,
    /// Whether to enable the predictive processing system
    pub enable_predictive: Option<bool>,
    /// Whether to enable the attention schema
    pub enable_attention: Option<bool>,
}

/// Response for toggling consciousness subsystems
#[derive(Serialize)]
pub struct ToggleSubsystemResponse {
    /// Current state of the higher-order thought system
    pub higher_order_enabled: bool,
    /// Current state of the predictive processing system
    pub predictive_enabled: bool,
    /// Current state of the attention schema
    pub attention_enabled: bool,
    /// Success message
    pub message: String,
}

/// Get current consciousness metrics
pub async fn get_metrics(
    State(state): State<AppState>,
) -> Result<Json<ConsciousnessMetricsResponse>, StatusCode> {
    let metrics = state.consciousness.get_metrics().await;
    
    info!("Retrieved consciousness metrics: Φ={}", metrics.phi_value);
    
    Ok(Json(ConsciousnessMetricsResponse {
        phi_value: metrics.phi_value,
        workspace_activity: metrics.workspace_activity,
        metacognition_level: metrics.metacognition_level,
        attention_focus: metrics.attention_focus,
        timestamp: chrono::Utc::now(),
    }))
}

/// Get current consciousness configuration
pub async fn get_config(
    State(_state): State<AppState>,
) -> Result<Json<ConsciousnessConfig>, StatusCode> {
    // Note: This method needs to be implemented in ConsciousnessEngine
    // For now, we'll return a 501 Not Implemented status
    warn!("Consciousness config endpoint not yet implemented");
    Err(StatusCode::NOT_IMPLEMENTED)
}

/// Toggle consciousness subsystems
pub async fn toggle_subsystems(
    State(_state): State<AppState>,
    Json(request): Json<ToggleSubsystemInput>,
) -> Result<Json<ToggleSubsystemResponse>, StatusCode> {
    // Validate input
    if let Err(errors) = validate_input(&request) {
        warn!("Input validation failed for toggle_subsystems: {:?}", errors);
        return Err(StatusCode::BAD_REQUEST);
    }

    // Note: This method needs to be implemented in ConsciousnessEngine
    // For now, we'll return a 501 Not Implemented status
    warn!("Consciousness toggle subsystems endpoint not yet implemented");
    Err(StatusCode::NOT_IMPLEMENTED)
}

/// Response for processing information
#[derive(Serialize)]
pub struct ProcessResponse {
    /// Phi value after processing
    pub phi_value: f64,
    /// Attention focus after processing
    pub attention_focus: String,
    /// Success message
    pub message: String,
}

/// Process information through consciousness system
pub async fn process_information(
    State(state): State<AppState>,
    Json(request): Json<ContentInput>,
) -> Result<Json<ProcessResponse>, StatusCode> {
    // Validate input
    if let Err(errors) = validate_input(&request) {
        warn!("Input validation failed for process_information: {:?}", errors);
        return Err(StatusCode::BAD_REQUEST);
    }

    match state.consciousness.process_information(&request.content).await {
        Ok(_) => {
            let metrics = state.consciousness.get_metrics().await;
            info!("Processed information through consciousness system: Φ={}", metrics.phi_value);
            Ok(Json(ProcessResponse {
                phi_value: metrics.phi_value,
                attention_focus: metrics.attention_focus,
                message: "Information processed successfully".to_string(),
            }))
        }
        Err(e) => {
            error!("Failed to process information through consciousness system: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// NOTE: The following methods need to be added to ConsciousnessEngine:
//
// pub async fn get_config(&self) -> ConsciousnessConfig {
//     // Return a copy of the current configuration
// }
//
// pub async fn toggle_subsystems(
//     &self,
//     enable_higher_order: Option<bool>,
//     enable_predictive: Option<bool>,
//     enable_attention: Option<bool>,
// ) -> Result<ConsciousnessConfig> {
//     // Update the enabled flags if provided
//     // Return the updated configuration
// }

// NOTE: The following tests should be added once the ConsciousnessEngine methods are implemented:
//
// #[tokio::test]
// async fn test_get_metrics_endpoint() {
//     // Create a test app with a mock ConsciousnessEngine
//     // Call the /consciousness/metrics endpoint
//     // Verify the response contains the expected metrics
// }
//
// #[tokio::test]
// async fn test_toggle_subsystems_endpoint() {
//     // Create a test app with a mock ConsciousnessEngine
//     // Call the /consciousness/toggle endpoint with various combinations
//     // Verify the response reflects the requested changes
// }