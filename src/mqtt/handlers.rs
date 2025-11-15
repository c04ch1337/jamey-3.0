use std::sync::Arc;
use tracing::{info, error};

use crate::conscience::ConscienceEngine;
use crate::memory::{MemorySystem, MemoryLayer};
use super::{
    MqttClient, QoS,
    ConscienceEvaluationRequest, ConscienceEvaluationResult,
    MemoryStoreRequest, ApiResponse, ErrorNotification,
};

/// Handle conscience evaluation requests via MQTT
pub async fn handle_conscience_evaluation(
    mqtt: Arc<MqttClient>,
    conscience: Arc<ConscienceEngine>,
    msg: ConscienceEvaluationRequest,
) {
    let correlation_id = msg.metadata.correlation_id;
    let reply_to = msg.metadata.reply_to.unwrap_or_else(|| "jamey/conscience/response".to_string());

    info!(
        action = %msg.action,
        user_id = ?msg.user_id,
        correlation_id = ?correlation_id,
        "Processing MQTT conscience evaluation"
    );

    // Evaluate action with soul integration if user_id provided
    let result = match conscience.evaluate_with_soul(&msg.action, msg.user_id.as_deref()).await {
        Ok((score, emotion)) => {
            // Create success response
            ConscienceEvaluationResult {
                action: msg.action,
                moral_score: score as f64,
                explanation: format!("Action evaluated with score {}", score),
                applied_rules: conscience.get_rules().iter().map(|r| r.name.clone()).collect(),
                approved: score >= 5.0,
            }
        }
        Err(e) => {
            error!(
                error = %e,
                correlation_id = ?correlation_id,
                "Conscience evaluation failed"
            );

            // Publish error notification
            if let Err(e) = mqtt.publish(
                "jamey/errors",
                ErrorNotification {
                    code: "CONSCIENCE_ERROR".into(),
                    message: e.to_string(),
                    component: "conscience".into(),
                    details: None,
                    severity: "error".into(),
                },
                QoS::AtLeastOnce,
            ).await {
                error!("Failed to publish error notification: {}", e);
            }

            return;
        }
    };

    // Publish response
    if let Err(e) = mqtt.publish(&reply_to, result, QoS::AtLeastOnce).await {
        error!(
            error = %e,
            correlation_id = ?correlation_id,
            "Failed to publish conscience evaluation response"
        );
    }
}

/// Handle memory store requests via MQTT
pub async fn handle_memory_store(
    mqtt: Arc<MqttClient>,
    memory: Arc<MemorySystem>,
    msg: MemoryStoreRequest,
) {
    let correlation_id = msg.metadata.correlation_id;
    let reply_to = msg.metadata.reply_to.unwrap_or_else(|| "jamey/memory/response".to_string());

    info!(
        layer = %msg.layer,
        user_id = ?msg.user_id,
        correlation_id = ?correlation_id,
        "Processing MQTT memory store request"
    );

    // Parse memory layer
    let layer = match msg.layer.to_lowercase().as_str() {
        "short_term" => MemoryLayer::ShortTerm,
        "long_term" => MemoryLayer::LongTerm,
        "working" => MemoryLayer::Working,
        "episodic" => MemoryLayer::Episodic,
        "semantic" => MemoryLayer::Semantic,
        _ => {
            error!(
                layer = %msg.layer,
                correlation_id = ?correlation_id,
                "Invalid memory layer specified"
            );

            // Publish error response
            if let Err(e) = mqtt.publish(
                &reply_to,
                ApiResponse {
                    status: 400,
                    headers: Default::default(),
                    body: Some(serde_json::json!({
                        "error": "Invalid memory layer",
                        "valid_layers": [
                            "short_term", "long_term", "working", "episodic", "semantic"
                        ]
                    })),
                },
                QoS::AtLeastOnce,
            ).await {
                error!("Failed to publish error response: {}", e);
            }

            return;
        }
    };

    // Store memory with entity link if user_id provided
    match memory.store_with_entity(layer, msg.content, msg.user_id.as_deref()).await {
        Ok(memory_id) => {
            // Publish success response
            if let Err(e) = mqtt.publish(
                &reply_to,
                ApiResponse {
                    status: 201,
                    headers: Default::default(),
                    body: Some(serde_json::json!({
                        "memory_id": memory_id,
                        "message": "Memory stored successfully"
                    })),
                },
                QoS::AtLeastOnce,
            ).await {
                error!(
                    error = %e,
                    correlation_id = ?correlation_id,
                    "Failed to publish success response"
                );
            }
        }
        Err(e) => {
            error!(
                error = %e,
                correlation_id = ?correlation_id,
                "Failed to store memory"
            );

            // Publish error response
            if let Err(e) = mqtt.publish(
                &reply_to,
                ApiResponse {
                    status: 500,
                    headers: Default::default(),
                    body: Some(serde_json::json!({
                        "error": "Failed to store memory",
                        "details": e.to_string()
                    })),
                },
                QoS::AtLeastOnce,
            ).await {
                error!("Failed to publish error response: {}", e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use crate::mqtt::MessageMetadata;

    #[tokio::test]
    async fn test_memory_store_handler() {
        // Create test components
        let dir = tempdir().unwrap();
        let memory = Arc::new(MemorySystem::new(dir.path().to_path_buf()).await.unwrap());
        
        let mqtt_config = crate::mqtt::MqttConfig::from_env().unwrap();
        let mqtt = Arc::new(MqttClient::new(mqtt_config).await.unwrap());

        // Create test request
        let request = MemoryStoreRequest {
            layer: "short_term".to_string(),
            content: "Test memory".to_string(),
            tags: vec![],
            user_id: Some("test_user".to_string()),
            metadata: MessageMetadata {
                client_id: Some("test_client".to_string()),
                correlation_id: Some(uuid::Uuid::new_v4()),
                reply_to: Some("test/reply".to_string()),
                custom: Default::default(),
            },
        };

        // Handle request
        handle_memory_store(mqtt.clone(), memory.clone(), request).await;

        // Verify memory was stored
        let memories = memory.get_entity_memories("test_user", 10).await.unwrap();
        assert_eq!(memories.len(), 1);
        assert_eq!(memories[0].content, "Test memory");
    }
}