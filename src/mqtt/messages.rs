use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Standard MQTT message envelope for all messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MqttMessage<T> {
    /// Unique message identifier
    pub message_id: Uuid,
    
    /// Timestamp when the message was created
    pub timestamp: DateTime<Utc>,
    
    /// MQTT topic this message was sent to/received from
    pub topic: String,
    
    /// The actual message payload
    pub payload: T,
    
    /// Optional metadata
    #[serde(default)]
    pub metadata: MessageMetadata,
}

impl<T> MqttMessage<T> {
    /// Create a new message with the given topic and payload
    pub fn new(topic: String, payload: T) -> Self {
        Self {
            message_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            topic,
            payload,
            metadata: MessageMetadata::default(),
        }
    }
    
    /// Create a new message with metadata
    pub fn with_metadata(topic: String, payload: T, metadata: MessageMetadata) -> Self {
        Self {
            message_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            topic,
            payload,
            metadata,
        }
    }
}

/// Metadata for message tracking and correlation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MessageMetadata {
    /// Client ID that sent the message
    pub client_id: Option<String>,
    
    /// Correlation ID for request-response tracking
    pub correlation_id: Option<Uuid>,
    
    /// Reply-to topic for responses
    pub reply_to: Option<String>,
    
    /// Additional custom fields
    #[serde(flatten)]
    pub custom: std::collections::HashMap<String, serde_json::Value>,
}

impl MessageMetadata {
    /// Create new metadata with a client ID
    pub fn with_client_id(client_id: String) -> Self {
        Self {
            client_id: Some(client_id),
            ..Default::default()
        }
    }
    
    /// Add a correlation ID for request tracking
    pub fn with_correlation_id(mut self, correlation_id: Uuid) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }
    
    /// Add a reply-to topic
    pub fn with_reply_to(mut self, reply_to: String) -> Self {
        self.reply_to = Some(reply_to);
        self
    }
}

/// Conscience evaluation request payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConscienceEvaluationRequest {
    /// The action to evaluate
    pub action: String,
    
    /// Context for the evaluation
    pub context: String,
    
    /// Optional user ID for personalized evaluation
    pub user_id: Option<String>,
}

/// Conscience evaluation result payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConscienceEvaluationResult {
    /// The original action that was evaluated
    pub action: String,
    
    /// Moral score (-1.0 to 1.0)
    pub moral_score: f64,
    
    /// Explanation of the evaluation
    pub explanation: String,
    
    /// Applied moral rules
    pub applied_rules: Vec<String>,
    
    /// Whether the action was approved
    pub approved: bool,
}

/// Memory store request payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStoreRequest {
    /// Memory layer (episodic, semantic, procedural, working, sensory)
    pub layer: String,
    
    /// Memory content
    pub content: String,
    
    /// Associated tags for retrieval
    pub tags: Vec<String>,
    
    /// Optional user ID
    pub user_id: Option<String>,
}

/// Memory query request payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryQueryRequest {
    /// Query text for semantic search
    pub query: String,
    
    /// Optional memory layer to search in
    pub layer: Option<String>,
    
    /// Maximum number of results
    #[serde(default = "default_limit")]
    pub limit: usize,
    
    /// Optional user ID
    pub user_id: Option<String>,
}

fn default_limit() -> usize {
    10
}

/// Memory query result payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryQueryResult {
    /// Found memories
    pub memories: Vec<MemoryEntry>,
    
    /// Total number of matches
    pub total: usize,
}

/// A single memory entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    /// Memory ID
    pub id: Uuid,
    
    /// Memory layer
    pub layer: String,
    
    /// Memory content
    pub content: String,
    
    /// Associated tags
    pub tags: Vec<String>,
    
    /// Relevance score (0.0 to 1.0)
    pub score: f64,
    
    /// When the memory was created
    pub created_at: DateTime<Utc>,
}

/// System status update payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatus {
    /// Status type (online, offline, error, warning)
    pub status: String,
    
    /// Human-readable message
    pub message: String,
    
    /// Component name
    pub component: String,
    
    /// Additional status data
    #[serde(flatten)]
    pub data: std::collections::HashMap<String, serde_json::Value>,
}

/// Heartbeat payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Heartbeat {
    /// Client ID
    pub client_id: String,
    
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    
    /// Optional health metrics
    #[serde(default)]
    pub metrics: HeartbeatMetrics,
}

/// Health metrics in heartbeat
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HeartbeatMetrics {
    /// Memory usage in bytes
    pub memory_usage: Option<u64>,
    
    /// CPU usage percentage
    pub cpu_usage: Option<f64>,
    
    /// Number of active connections
    pub active_connections: Option<u32>,
    
    /// Uptime in seconds
    pub uptime: Option<u64>,
}

/// Error notification payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorNotification {
    /// Error code
    pub code: String,
    
    /// Error message
    pub message: String,
    
    /// Component that generated the error
    pub component: String,
    
    /// Stack trace or additional details
    pub details: Option<String>,
    
    /// Severity level (critical, error, warning)
    pub severity: String,
}

/// API request proxy payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiRequestProxy {
    /// HTTP method
    pub method: String,
    
    /// Request path
    pub path: String,
    
    /// Request headers
    #[serde(default)]
    pub headers: std::collections::HashMap<String, String>,
    
    /// Request body
    pub body: Option<serde_json::Value>,
}

/// API response payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse {
    /// HTTP status code
    pub status: u16,
    
    /// Response headers
    #[serde(default)]
    pub headers: std::collections::HashMap<String, String>,
    
    /// Response body
    pub body: Option<serde_json::Value>,
}

/// Helper to serialize a message to JSON bytes
pub fn serialize_message<T: Serialize>(message: &MqttMessage<T>) -> Result<Vec<u8>, serde_json::Error> {
    serde_json::to_vec(message)
}

/// Helper to deserialize a message from JSON bytes
pub fn deserialize_message<T: for<'de> Deserialize<'de>>(
    bytes: &[u8],
) -> Result<MqttMessage<T>, serde_json::Error> {
    serde_json::from_slice(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_message_serialization() {
        let payload = ConscienceEvaluationRequest {
            action: "test_action".to_string(),
            context: "test_context".to_string(),
            user_id: Some("user123".to_string()),
        };
        
        let message = MqttMessage::new("jamey/conscience/evaluate".to_string(), payload);
        
        let serialized = serialize_message(&message).expect("Failed to serialize");
        let deserialized: MqttMessage<ConscienceEvaluationRequest> =
            deserialize_message(&serialized).expect("Failed to deserialize");
        
        assert_eq!(message.topic, deserialized.topic);
        assert_eq!(message.payload.action, deserialized.payload.action);
    }
    
    #[test]
    fn test_metadata_builder() {
        let metadata = MessageMetadata::with_client_id("client1".to_string())
            .with_correlation_id(Uuid::new_v4())
            .with_reply_to("jamey/responses/test".to_string());
        
        assert_eq!(metadata.client_id, Some("client1".to_string()));
        assert!(metadata.correlation_id.is_some());
        assert_eq!(metadata.reply_to, Some("jamey/responses/test".to_string()));
    }
}