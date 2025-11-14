# MQTT Client Usage Guide

## Overview

The Jamey 3.0 MQTT client provides secure, asynchronous message-based communication with the following features:

- **TLS 1.3 Transport Security**: Encrypted communication with no TLS 1.2 fallback
- **JWT Authentication**: Short-lived tokens (5 minutes) with automatic refresh
- **Optional mTLS**: Client certificate authentication for enhanced security
- **Topic-based Permissions**: Fine-grained access control using wildcards
- **Typed Message Handling**: Type-safe message serialization/deserialization
- **Automatic Reconnection**: Exponential backoff with jitter

## Quick Start

### 1. Configuration

Add the following to your `.env` file:

```bash
# MQTT Broker Connection
MQTT_BROKER_URL=mqtt://localhost
MQTT_PORT=8883

# TLS Certificates (required)
MQTT_TLS_CA_CERT=./certs/ca.crt
MQTT_TLS_CLIENT_CERT=./certs/client.crt  # Optional for mTLS
MQTT_TLS_CLIENT_KEY=./certs/client.key    # Optional for mTLS

# Authentication
MQTT_JWT_SECRET=your-secret-key-here-minimum-32-characters-required

# Client Settings
MQTT_CLIENT_ID=jamey-instance-1
MQTT_PERMISSIONS=jamey/#,jamey/+/query,jamey/events/#
```

### 2. Basic Usage

```rust
use jamey_3::mqtt::{MqttClient, MqttConfig, QoS};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration from environment
    let config = MqttConfig::from_env()?;
    
    // Create client
    let client = MqttClient::new(config).await?;
    
    // Subscribe to a topic
    client.subscribe("jamey/events/status", QoS::AtLeastOnce, |topic, payload| {
        println!("Received on {}: {:?}", topic, payload);
    }).await?;
    
    // Publish a message
    client.publish(
        "jamey/events/status",
        "System online",
        QoS::AtLeastOnce
    ).await?;
    
    // Keep running...
    tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    
    // Disconnect
    client.disconnect().await?;
    
    Ok(())
}
```

## Advanced Usage

### Typed Message Handling

Use strongly-typed messages for better safety:

```rust
use jamey_3::mqtt::messages::{
    ConscienceEvaluationRequest,
    ConscienceEvaluationResult,
    MqttMessage,
};

// Subscribe with typed handler
client.subscribe_typed::<ConscienceEvaluationResult, _>(
    "jamey/conscience/results",
    QoS::AtLeastOnce,
    |message: MqttMessage<ConscienceEvaluationResult>| {
        println!("Moral score: {}", message.payload.moral_score);
        println!("Explanation: {}", message.payload.explanation);
    }
).await?;

// Publish with typed payload
let request = ConscienceEvaluationRequest {
    action: "Delete user data".to_string(),
    context: "User requested account deletion".to_string(),
    user_id: Some("user123".to_string()),
};

client.publish(
    "jamey/conscience/evaluate",
    request,
    QoS::AtLeastOnce
).await?;
```

### Message Metadata

Add correlation IDs and custom metadata:

```rust
use jamey_3::mqtt::messages::{MqttMessage, MessageMetadata};
use uuid::Uuid;

let metadata = MessageMetadata::with_client_id("jamey-instance-1".to_string())
    .with_correlation_id(Uuid::new_v4())
    .with_reply_to("jamey/responses/client1".to_string());

let message = MqttMessage::with_metadata(
    "jamey/memory/store".to_string(),
    my_payload,
    metadata
);
```

### Connection State Monitoring

```rust
use jamey_3::mqtt::ConnectionState;

// Check connection state
let state = client.state().await;
match state {
    ConnectionState::Connected => println!("Connected to broker"),
    ConnectionState::Reconnecting => println!("Reconnecting..."),
    ConnectionState::Error => println!("Connection error"),
    _ => println!("Other state: {:?}", state),
}
```

## Topic Structure

The MQTT client uses a hierarchical topic structure:

```
jamey/
├── conscience/
│   ├── evaluate      # Conscience evaluation requests
│   └── results       # Evaluation results
├── memory/
│   ├── store         # Memory storage requests
│   ├── query         # Memory queries
│   └── updates       # Memory update notifications
├── events/
│   ├── status        # System status updates
│   ├── heartbeat     # Keep-alive messages
│   └── errors        # Error notifications
└── api/
    ├── requests      # API request proxying
    └── responses     # API responses
```

## Topic Permissions

Topic permissions use MQTT wildcard patterns:

- `+` - Single-level wildcard (matches one level)
- `#` - Multi-level wildcard (matches all remaining levels)

Examples:
```bash
jamey/#                # All jamey topics
jamey/+/query          # jamey/memory/query, jamey/conscience/query, etc.
jamey/events/#         # All event topics
jamey/conscience/evaluate  # Specific topic only
```

## QoS Levels

Choose appropriate QoS levels for your use case:

- **QoS 0 (At Most Once)**: Use for heartbeats, non-critical status updates
  ```rust
  client.publish("jamey/events/heartbeat", heartbeat, QoS::AtMostOnce).await?;
  ```

- **QoS 1 (At Least Once)**: Use for operational messages, conscience evaluations, memory operations
  ```rust
  client.publish("jamey/conscience/evaluate", request, QoS::AtLeastOnce).await?;
  ```

- **QoS 2 (Exactly Once)**: Not used (overhead not justified for current use cases)

## Security Best Practices

### 1. Certificate Management

Generate certificates for development:

```bash
# Create certs directory
mkdir -p certs

# Generate CA certificate
openssl req -x509 -newkey rsa:4096 -keyout certs/ca.key -out certs/ca.crt -days 365 -nodes

# Generate client certificate
openssl req -newkey rsa:4096 -keyout certs/client.key -out certs/client.csr -nodes
openssl x509 -req -in certs/client.csr -CA certs/ca.crt -CAkey certs/ca.key -CAcreateserial -out certs/client.crt -days 365
```

**Production**: Use Let's Encrypt or your organization's CA.

### 2. JWT Secret

Generate a strong JWT secret:

```bash
# Generate a random 64-character secret
openssl rand -hex 32
```

Add to `.env`:
```bash
MQTT_JWT_SECRET=<generated-secret>
```

### 3. Topic Permissions

Follow the principle of least privilege:

```bash
# Allow only specific topics needed
MQTT_PERMISSIONS=jamey/memory/query,jamey/events/status

# Avoid overly broad permissions in production
# MQTT_PERMISSIONS=jamey/#  # Too broad!
```

## Integration Examples

### Conscience Engine Integration

```rust
use jamey_3::mqtt::{MqttClient, QoS};
use jamey_3::mqtt::messages::ConscienceEvaluationResult;

async fn publish_evaluation(
    mqtt: &MqttClient,
    result: ConscienceEvaluationResult
) -> Result<(), Box<dyn std::error::Error>> {
    mqtt.publish(
        "jamey/conscience/results",
        result,
        QoS::AtLeastOnce
    ).await?;
    Ok(())
}
```

### Memory System Integration

```rust
use jamey_3::mqtt::{MqttClient, QoS};
use jamey_3::mqtt::messages::{MemoryStoreRequest, MemoryQueryRequest};

async fn store_memory(
    mqtt: &MqttClient,
    layer: String,
    content: String,
    tags: Vec<String>
) -> Result<(), Box<dyn std::error::Error>> {
    let request = MemoryStoreRequest {
        layer,
        content,
        tags,
        user_id: None,
    };
    
    mqtt.publish("jamey/memory/store", request, QoS::AtLeastOnce).await?;
    Ok(())
}
```

### API Handler Integration

```rust
use axum::{Extension, Json};
use std::sync::Arc;
use jamey_3::mqtt::MqttClient;

async fn api_handler(
    Extension(mqtt): Extension<Arc<MqttClient>>,
    Json(payload): Json<serde_json::Value>
) -> Result<Json<serde_json::Value>, String> {
    mqtt.publish("jamey/api/requests", payload, QoS::AtLeastOnce)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(Json(serde_json::json!({"status": "queued"})))
}
```

## Troubleshooting

### Connection Issues

1. **Certificate errors**: Verify certificate paths and permissions
   ```bash
   ls -la certs/
   # Files should be readable
   ```

2. **Port blocked**: Ensure port 8883 is not blocked by firewall
   ```bash
   nc -zv localhost 8883
   ```

3. **JWT secret too short**: Must be at least 32 characters
   ```bash
   # Check length
   echo -n "$MQTT_JWT_SECRET" | wc -c
   ```

### Permission Denied

Check your topic permissions match the topics you're trying to access:

```rust
// Check current permissions
if let Some(claims) = client.current_claims().await {
    println!("Permissions: {:?}", claims.permissions);
}
```

### Token Expiration

Tokens are automatically refreshed, but you can check status:

```rust
if let Some(claims) = client.current_claims().await {
    if let Some(time_left) = claims.time_until_expiry() {
        println!("Token expires in: {:?}", time_left);
    }
}
```

## Testing

Run the included tests:

```bash
# Run all MQTT tests
cargo test --lib mqtt

# Run specific test
cargo test --lib mqtt::auth::tests::test_jwt_generation_and_validation
```

## Further Reading

- [MQTT 3.1.1 Specification](http://docs.oasis-open.org/mqtt/mqtt/v3.1.1/mqtt-v3.1.1.html)
- [rumqttc Documentation](https://docs.rs/rumqttc/)
- [TLS 1.3 RFC](https://tools.ietf.org/html/rfc8446)
- [JWT Best Practices](https://tools.ietf.org/html/rfc8725)