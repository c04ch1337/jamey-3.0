//! MQTT message handlers for Jamey 3.0
//!
//! This module provides handler functions for processing MQTT messages
//! and integrating with the conscience and memory systems.

use std::sync::Arc;
use crate::conscience::ConscienceEngine;
use crate::memory::{MemorySystem, MemoryLayer};
use super::{MqttClient, ConscienceEvaluationRequest, MemoryStoreRequest, ConscienceEvaluationResult};
use tracing::{info, error};

/// Handle conscience evaluation requests from MQTT
pub async fn handle_conscience_evaluation(
    mqtt: Arc<MqttClient>,
    conscience: Arc<ConscienceEngine>,
    msg: ConscienceEvaluationRequest,
) {
    info!("Processing conscience evaluation request: {}", msg.action);
    
    let score = conscience.evaluate(&msg.action);
    
    // Publish result back
    let result = ConscienceEvaluationResult {
        action: msg.action.clone(),
        moral_score: score as f64,
        explanation: format!("Action evaluated with score: {}", score),
        applied_rules: vec![],
        approved: score > 0.0,
    };
    
    // Serialize and publish
    if let Ok(payload) = serde_json::to_string(&result) {
        if let Err(e) = mqtt.publish("jamey/conscience/result", payload.as_bytes(), super::QoS::AtLeastOnce).await {
            error!("Failed to publish conscience result: {}", e);
        }
    }
}

/// Handle memory store requests from MQTT
pub async fn handle_memory_store(
    _mqtt: Arc<MqttClient>,
    memory: Arc<MemorySystem>,
    msg: MemoryStoreRequest,
) {
    info!("Processing memory store request");
    
    // Parse layer string to MemoryLayer enum
    let layer = match msg.layer.to_lowercase().as_str() {
        "episodic" => MemoryLayer::Episodic,
        "semantic" => MemoryLayer::Semantic,
        "working" => MemoryLayer::Working,
        "short_term" | "shortterm" => MemoryLayer::ShortTerm,
        "long_term" | "longterm" => MemoryLayer::LongTerm,
        _ => {
            error!("Invalid memory layer: {}. Valid layers are: episodic, semantic, working, short_term, long_term", msg.layer);
            return;
        }
    };
    
    let result = memory.store(
        layer,
        msg.content.clone(),
    ).await;
    
    match result {
        Ok(_) => {
            info!("Memory stored successfully");
            // Optionally publish success confirmation
        }
        Err(e) => {
            error!("Failed to store memory: {}", e);
            // Optionally publish error notification
        }
    }
}