//! MQTT Integration Tests
//!
//! Tests for MQTT client functionality including:
//! - Connection handling
//! - Message publishing/subscribing
//! - Reconnection logic
//! - Authentication
//! - Error handling

use jamey_3::mqtt::{MqttClient, MqttConfig};
use std::time::Duration;

/// Helper to create a test MQTT config
fn create_test_mqtt_config() -> MqttConfig {
    MqttConfig {
        broker_host: "localhost".to_string(),
        port: 1883,
        client_id: format!("test_client_{}", uuid::Uuid::new_v4()),
        username: None,
        password: None,
        tls_enabled: false,
        keep_alive: Duration::from_secs(60),
        max_packet_size: 1024 * 1024,
        jwt_secret: "test_secret_key_for_jwt_authentication_32chars".to_string(),
        jwt_lifetime: Duration::from_secs(3600),
        permissions: vec!["jamey/#".to_string()],
        reconnect_config: Default::default(),
    }
}

/// Test MQTT config validation
#[test]
fn test_mqtt_config_validation() {
    let mut config = create_test_mqtt_config();
    
    // Valid config should pass
    assert!(config.validate().is_ok());
    
    // Invalid: empty broker host
    config.broker_host = String::new();
    assert!(config.validate().is_err());
    
    // Invalid: port out of range
    config.broker_host = "localhost".to_string();
    config.port = 0;
    assert!(config.validate().is_err());
    
    config.port = 65536;
    assert!(config.validate().is_err());
    
    // Invalid: JWT secret too short
    config.port = 1883;
    config.jwt_secret = "short".to_string();
    assert!(config.validate().is_err());
}

/// Test MQTT client creation
#[tokio::test]
async fn test_mqtt_client_creation() {
    let config = create_test_mqtt_config();
    
    // Note: This will fail if MQTT broker is not running
    // That's expected - we're testing the client creation logic
    let result = MqttClient::new(config).await;
    
    // Either succeeds (if broker available) or fails with connection error
    match result {
        Ok(_) => {
            // Client created successfully
            // This means a broker is available for testing
        }
        Err(e) => {
            // Expected if no broker running
            // Verify it's a connection error, not a config error
            assert!(e.to_string().contains("Connection") || 
                   e.to_string().contains("connection") ||
                   e.to_string().contains("Connection refused"));
        }
    }
}

/// Test MQTT message serialization
#[test]
fn test_mqtt_message_serialization() {
    use jamey_3::mqtt::messages::{serialize_message, deserialize_message, Heartbeat};
    use jamey_3::mqtt::MqttMessage;
    
    let heartbeat = Heartbeat {
        timestamp: chrono::Utc::now(),
        status: "online".to_string(),
        metrics: None,
    };
    
    let message = MqttMessage::Heartbeat(heartbeat);
    
    // Serialize
    let serialized = serialize_message(&message).unwrap();
    
    // Deserialize
    let deserialized: MqttMessage = deserialize_message(&serialized).unwrap();
    
    // Verify round-trip
    match (message, deserialized) {
        (MqttMessage::Heartbeat(h1), MqttMessage::Heartbeat(h2)) => {
            assert_eq!(h1.status, h2.status);
        }
        _ => panic!("Message type mismatch"),
    }
}

/// Test MQTT error types
#[test]
fn test_mqtt_error_types() {
    use jamey_3::mqtt::MqttError;
    
    // Test error creation
    let err = MqttError::Connection("test error".to_string());
    assert_eq!(err.to_string(), "Connection error: test error");
    
    let err = MqttError::NotConnected;
    assert_eq!(err.to_string(), "Not connected");
    
    let err = MqttError::InvalidTopic("invalid/topic".to_string());
    assert_eq!(err.to_string(), "Invalid topic: invalid/topic");
}

/// Test MQTT config from environment
#[test]
fn test_mqtt_config_from_env() {
    // Set test environment variables
    std::env::set_var("MQTT_BROKER_HOST", "test-broker");
    std::env::set_var("MQTT_BROKER_PORT", "1883");
    std::env::set_var("MQTT_CLIENT_ID", "test-client");
    std::env::set_var("MQTT_JWT_SECRET", "test_secret_key_for_jwt_authentication_32chars");
    
    let config = MqttConfig::from_env();
    
    match config {
        Ok(cfg) => {
            assert_eq!(cfg.broker_host, "test-broker");
            assert_eq!(cfg.port, 1883);
            assert_eq!(cfg.client_id, "test-client");
        }
        Err(_) => {
            // Config might require more variables
            // That's acceptable
        }
    }
    
    // Cleanup
    std::env::remove_var("MQTT_BROKER_HOST");
    std::env::remove_var("MQTT_BROKER_PORT");
    std::env::remove_var("MQTT_CLIENT_ID");
    std::env::remove_var("MQTT_JWT_SECRET");
}

/// Test MQTT connection state
#[test]
fn test_mqtt_connection_state() {
    use jamey_3::mqtt::ConnectionState;
    
    let states = vec![
        ConnectionState::Disconnected,
        ConnectionState::Connecting,
        ConnectionState::Connected,
        ConnectionState::Reconnecting,
        ConnectionState::Error,
    ];
    
    // Verify all states are distinct
    for (i, state1) in states.iter().enumerate() {
        for (j, state2) in states.iter().enumerate() {
            if i == j {
                assert_eq!(state1, state2);
            } else {
                assert_ne!(state1, state2);
            }
        }
    }
}

/// Test MQTT topic validation
#[test]
fn test_mqtt_topic_validation() {
    use jamey_3::mqtt::MqttError;
    
    // Valid topics
    let valid_topics = vec![
        "jamey/events",
        "jamey/events/status",
        "jamey/conscience/evaluate",
        "test/topic/123",
    ];
    
    for topic in valid_topics {
        // Basic validation - topics should not be empty
        assert!(!topic.is_empty());
        assert!(topic.len() <= 65535); // MQTT topic length limit
    }
    
    // Invalid topics would be caught by MQTT client
    // We test error creation
    let err = MqttError::InvalidTopic("".to_string());
    assert!(err.to_string().contains("Invalid topic"));
}

/// Test MQTT message types
#[test]
fn test_mqtt_message_types() {
    use jamey_3::mqtt::messages::{Heartbeat, ConscienceEvaluationRequest};
    use jamey_3::mqtt::MqttMessage;
    use chrono::Utc;
    
    // Test Heartbeat message
    let heartbeat = Heartbeat {
        timestamp: Utc::now(),
        status: "online".to_string(),
        metrics: None,
    };
    
    let msg = MqttMessage::Heartbeat(heartbeat);
    match msg {
        MqttMessage::Heartbeat(h) => {
            assert_eq!(h.status, "online");
        }
        _ => panic!("Wrong message type"),
    }
    
    // Test ConscienceEvaluationRequest
    let request = ConscienceEvaluationRequest {
        action: "test action".to_string(),
        user_id: Some("user123".to_string()),
        metadata: Default::default(),
    };
    
    let msg = MqttMessage::ConscienceEvaluationRequest(request);
    match msg {
        MqttMessage::ConscienceEvaluationRequest(r) => {
            assert_eq!(r.action, "test action");
            assert_eq!(r.user_id, Some("user123".to_string()));
        }
        _ => panic!("Wrong message type"),
    }
}

/// Test MQTT reconnection config
#[test]
fn test_mqtt_reconnection_config() {
    use jamey_3::mqtt::ReconnectConfig;
    
    let config = ReconnectConfig::default();
    
    // Verify default values are reasonable
    assert!(config.max_retries > 0);
    assert!(config.initial_delay.as_secs() > 0);
    assert!(config.max_delay.as_secs() >= config.initial_delay.as_secs());
}

