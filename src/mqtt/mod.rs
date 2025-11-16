//! MQTT async client module for Jamey 3.0
//!
//! This module provides a secure, asynchronous MQTT client with:
//! - TLS 1.3 transport security
//! - JWT-based authentication
//! - Optional mTLS support
//! - Automatic token refresh
//! - Topic-based permissions
//! - Typed message handling
//!
//! # Example Usage
//!
//! ```no_run
//! use jamey_3::mqtt::{MqttClient, MqttConfig};
//! use rumqttc::QoS;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Load configuration from environment
//!     let config = MqttConfig::from_env()?;
//!     
//!     // Create client
//!     let client = MqttClient::new(config).await?;
//!     
//!     // Subscribe to a topic
//!     client.subscribe("jamey/events/status", QoS::AtLeastOnce, |topic, payload| {
//!         println!("Received message on {}: {:?}", topic, payload);
//!     }).await?;
//!     
//!     // Publish a message
//!     client.publish("jamey/events/status", "Hello MQTT", QoS::AtLeastOnce).await?;
//!     
//!     Ok(())
//! }
//! ```

mod auth;
mod client;
mod config;
mod messages;
pub mod handlers;

// Re-export public API
pub use auth::{AuthError, JwtManager, MqttClaims};
pub use client::{ConnectionState, MqttClient, MqttError};
pub use config::{ConfigError, MqttConfig, ReconnectConfig};
pub use messages::{
    deserialize_message, serialize_message, ApiRequestProxy, ApiResponse,
    ConscienceEvaluationRequest, ConscienceEvaluationResult, ErrorNotification, Heartbeat,
    HeartbeatMetrics, MemoryEntry, MemoryQueryRequest, MemoryQueryResult, MemoryStoreRequest,
    MessageMetadata, MqttMessage, SystemStatus,
};

// Re-export QoS from rumqttc for convenience
pub use rumqttc::QoS;

/// Initialize MQTT client from environment variables
///
/// This is a convenience function that loads configuration from environment
/// and creates a new MQTT client.
///
/// # Example
///
/// ```no_run
/// use jamey_3::mqtt;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let client = mqtt::init().await?;
///     // Use the client...
///     Ok(())
/// }
/// ```
pub async fn init() -> Result<MqttClient, MqttError> {
    let config = MqttConfig::from_env().map_err(|e| MqttError::Connection(e.to_string()))?;
    MqttClient::new(config).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_exports() {
        // This test ensures all the main types are exported
        let _ = std::mem::size_of::<MqttClient>();
        let _ = std::mem::size_of::<MqttConfig>();
        let _ = std::mem::size_of::<MqttError>();
        let _ = std::mem::size_of::<JwtManager>();
    }
}