# MQTT Architecture for Jamey 3.0

## Overview
This document describes the MQTT integration architecture for Jamey 3.0, providing asynchronous message-based communication for the AI system.

## Architecture Goals
- **Asynchronous Communication**: Enable non-blocking message passing between components
- **Security First**: TLS 1.3 only, mTLS authentication, JWT tokens
- **Reliability**: QoS levels, automatic reconnection, message persistence
- **Integration**: Seamless integration with existing Rust/Tokio/Axum architecture

## Component Architecture

### 1. MQTT Client Module (`src/mqtt/`)
```
mqtt/
├── mod.rs          # Public API and exports
├── config.rs       # MQTT configuration structures
├── client.rs       # Async MQTT client wrapper
├── messages.rs     # Message types and schemas
└── auth.rs         # JWT authentication logic
```

### 2. Security Architecture

#### TLS 1.3 Configuration
- **Transport Security**: TLS 1.3 only (no TLS 1.2 fallback)
- **Port**: 8883 (default MQTT over TLS)
- **Certificate Validation**: Full chain validation with CA certificate
- **mTLS**: Optional client certificate authentication

#### JWT Authentication
- **Token Lifetime**: 5 minutes (configurable)
- **Algorithm**: HS256 (HMAC with SHA-256)
- **Claims**: 
  - `sub`: Client identifier
  - `exp`: Expiration timestamp
  - `iat`: Issued at timestamp
  - `permissions`: Array of allowed topic patterns
- **Refresh Strategy**: Auto-refresh 30 seconds before expiration

#### Access Control
- **Default Policy**: Deny all
- **Topic Permissions**: Explicit allow list per client
- **Rate Limiting**: Client-side throttling to prevent abuse

### 3. Topic Structure

#### Hierarchy
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

#### Topic Patterns
- Read: `jamey/+/query` (subscribe to any query topic)
- Write: `jamey/conscience/evaluate` (specific write endpoint)
- Wildcards: 
  - `+` matches single level
  - `#` matches multiple levels (use sparingly)

### 4. Message Format

#### Standard Envelope
```json
{
  "message_id": "uuid-v4",
  "timestamp": "2024-11-14T20:00:00Z",
  "topic": "jamey/conscience/evaluate",
  "payload": {
    "action": "evaluate",
    "data": { ... }
  },
  "metadata": {
    "client_id": "jamey-instance-1",
    "correlation_id": "uuid-v4"
  }
}
```

#### QoS Levels
- **QoS 0** (At most once): Heartbeats, non-critical status updates
- **QoS 1** (At least once): Operational messages, conscience evaluations, memory operations
- **QoS 2** (Exactly once): Not used (overhead not justified for current use cases)

### 5. Connection Management

#### Connection States
1. **Disconnected**: Initial state, not connected
2. **Connecting**: Connection attempt in progress
3. **Connected**: Established connection, ready for operations
4. **Reconnecting**: Attempting reconnection after disconnect
5. **Error**: Fatal error state requiring manual intervention

#### Reconnection Strategy
- **Initial Retry**: Immediate
- **Backoff**: Exponential with jitter
  - Base delay: 1 second
  - Max delay: 60 seconds
  - Jitter: ±20%
- **Max Attempts**: Unlimited (with exponential backoff)
- **Circuit Breaker**: After 10 consecutive failures, require manual reset

### 6. Integration Points

#### Conscience Engine Integration
```rust
// Publish evaluation results
mqtt_client.publish(
    "jamey/conscience/results",
    serde_json::to_vec(&evaluation)?,
    QoS::AtLeastOnce
).await?;
```

#### Memory System Integration
```rust
// Subscribe to memory queries
mqtt_client.subscribe(
    "jamey/memory/query",
    QoS::AtLeastOnce,
    |message| async move {
        // Handle memory query
    }
).await?;
```

#### API Handler Integration
```rust
// Proxy API requests through MQTT
async fn handle_api_request(
    mqtt: Extension<Arc<MqttClient>>,
    Json(request): Json<ApiRequest>
) -> Result<Json<ApiResponse>> {
    mqtt.request_response(
        "jamey/api/requests",
        request,
        Duration::from_secs(30)
    ).await
}
```

### 7. Configuration

#### Environment Variables
```bash
# Broker connection
MQTT_BROKER_URL=mqtt://localhost
MQTT_PORT=8883

# TLS certificates
MQTT_TLS_CA_CERT=./certs/ca.crt
MQTT_TLS_CLIENT_CERT=./certs/client.crt
MQTT_TLS_CLIENT_KEY=./certs/client.key

# Authentication
MQTT_JWT_SECRET=your-secret-key-here
MQTT_JWT_LIFETIME_SECONDS=300

# Client settings
MQTT_CLIENT_ID=jamey-instance-1
MQTT_KEEP_ALIVE_SECONDS=60
MQTT_MAX_PACKET_SIZE=268435456
```

#### Config Struct
```rust
pub struct MqttConfig {
    pub broker_url: String,
    pub port: u16,
    pub tls_ca_cert: PathBuf,
    pub tls_client_cert: Option<PathBuf>,
    pub tls_client_key: Option<PathBuf>,
    pub jwt_secret: String,
    pub jwt_lifetime: Duration,
    pub client_id: String,
    pub keep_alive: Duration,
}
```

### 8. Error Handling

#### Error Types
- `ConnectionError`: Failed to connect to broker
- `AuthenticationError`: JWT validation or mTLS failure
- `PublishError`: Failed to publish message
- `SubscriptionError`: Failed to subscribe to topic
- `MessageError`: Malformed or invalid message

#### Error Recovery
- **Transient Errors**: Automatic retry with backoff
- **Authentication Errors**: Token refresh and retry
- **Permanent Errors**: Log and notify, require manual intervention

### 9. Testing Strategy

#### Unit Tests
- JWT generation and validation
- Message serialization/deserialization
- Configuration parsing

#### Integration Tests
- Connection establishment with mock broker
- Publish/subscribe operations
- Reconnection logic
- Error handling

#### Performance Tests
- Message throughput
- Connection recovery time
- Memory usage under load

### 10. Deployment Considerations

#### Certificate Management
- Use Let's Encrypt or internal CA for production
- Rotate certificates regularly
- Store certificates securely (not in version control)

#### Broker Selection
- **Mosquitto**: Lightweight, well-tested, good for development
- **EMQX**: Scalable, enterprise-grade, good for production
- **VerneMQ**: Distributed, high availability

#### Monitoring
- Connection state tracking
- Message delivery rates
- Error rates and types
- Latency measurements

## Implementation Phases

### Phase 1: Core Client (Current)
- [x] Basic connection with TLS 1.3
- [x] JWT authentication
- [x] Publish/subscribe operations
- [x] Configuration management

### Phase 2: Integration
- [ ] Conscience engine integration
- [ ] Memory system integration
- [ ] API handler proxying

### Phase 3: Advanced Features
- [ ] Message persistence
- [ ] Request-response pattern
- [ ] Load balancing
- [ ] High availability

## References
- [MQTT 3.1.1 Specification](http://docs.oasis-open.org/mqtt/mqtt/v3.1.1/mqtt-v3.1.1.html)
- [MQTT 5.0 Specification](https://docs.oasis-open.org/mqtt/mqtt/v5.0/mqtt-v5.0.html)
- [rumqttc Documentation](https://docs.rs/rumqttc/)
- [TLS 1.3 RFC](https://tools.ietf.org/html/rfc8446)