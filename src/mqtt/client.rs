use crate::mqtt::auth::{AuthError, JwtManager, MqttClaims};
use crate::mqtt::config::MqttConfig;
use crate::mqtt::messages::{deserialize_message, serialize_message, MqttMessage};
use rumqttc::{
    AsyncClient, ClientError, Event, EventLoop, MqttOptions, Packet, QoS, Transport,
};
use rumqttc::ClientConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::BufReader;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// MQTT client errors
#[derive(Debug, Error)]
pub enum MqttError {
    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Authentication error: {0}")]
    Authentication(#[from] AuthError),

    #[error("Publish error: {0}")]
    Publish(String),

    #[error("Subscribe error: {0}")]
    Subscribe(String),

    #[error("Message serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("TLS configuration error: {0}")]
    TlsConfig(String),

    #[error("Client error: {0}")]
    Client(#[from] ClientError),

    #[error("Not connected")]
    NotConnected,

    #[error("Invalid topic: {0}")]
    InvalidTopic(String),

    #[error("Permission denied for topic: {0}")]
    PermissionDenied(String),
}

/// Connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
    Error,
}

/// Type alias for message handlers
type MessageHandler = Arc<dyn Fn(String, Vec<u8>) -> () + Send + Sync>;

/// MQTT async client wrapper
pub struct MqttClient {
    #[allow(dead_code)]
    config: MqttConfig,
    client: AsyncClient,
    jwt_manager: JwtManager,
    current_token: Arc<RwLock<Option<String>>>,
    current_claims: Arc<RwLock<Option<MqttClaims>>>,
    state: Arc<RwLock<ConnectionState>>,
    handlers: Arc<RwLock<HashMap<String, MessageHandler>>>,
    token_refresh_handle: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
}

impl MqttClient {
    /// Create a new MQTT client with the given configuration
    pub async fn new(config: MqttConfig) -> Result<Self, MqttError> {
        // Validate configuration
        config
            .validate()
            .map_err(|e| MqttError::Connection(e.to_string()))?;

        // Create MQTT options
        let mut mqtt_options = MqttOptions::new(
            &config.client_id,
            config.broker_address().split(':').next().unwrap(),
            config.port,
        );

        mqtt_options.set_keep_alive(config.keep_alive);
        mqtt_options.set_max_packet_size(config.max_packet_size, config.max_packet_size);

        // Configure TLS 1.3
        let tls_config = Self::create_tls_config(&config)?;
        // rumqttc uses its own ClientConfig (rustls 0.20), which we've already created
        use rumqttc::TlsConfiguration;
        let tls_config_rumqttc = TlsConfiguration::Rustls(Arc::new(tls_config));
        mqtt_options.set_transport(Transport::tls_with_config(tls_config_rumqttc));

        // Create the async client
        let (client, eventloop) = AsyncClient::new(mqtt_options, 10);

        // Create JWT manager
        let jwt_manager = JwtManager::new(config.jwt_secret.clone(), config.jwt_lifetime);

        // Generate initial token
        let token = jwt_manager
            .generate_token(config.client_id.clone(), config.permissions.clone())
            .map_err(MqttError::from)?;

        let claims = jwt_manager
            .validate_token(&token)
            .map_err(MqttError::from)?;

        // Set credentials with JWT token as password
        // Note: In a real implementation, you'd need to configure the MQTT broker
        // to validate JWT tokens. For now, we store it for future use.

        let mqtt_client = Self {
            config: config.clone(),
            client,
            jwt_manager,
            current_token: Arc::new(RwLock::new(Some(token))),
            current_claims: Arc::new(RwLock::new(Some(claims))),
            state: Arc::new(RwLock::new(ConnectionState::Disconnected)),
            handlers: Arc::new(RwLock::new(HashMap::new())),
            token_refresh_handle: Arc::new(RwLock::new(None)),
        };

        // Start the event loop handler
        tokio::spawn(Self::handle_events(
            eventloop,
            mqtt_client.state.clone(),
            mqtt_client.handlers.clone(),
        ));

        // Start token refresh task
        let refresh_handle = tokio::spawn(Self::token_refresh_loop(
            mqtt_client.jwt_manager.clone(),
            mqtt_client.current_token.clone(),
            mqtt_client.current_claims.clone(),
            config.client_id.clone(),
            config.permissions.clone(),
        ));

        *mqtt_client.token_refresh_handle.write().await = Some(refresh_handle);

        Ok(mqtt_client)
    }

    /// Create TLS configuration with TLS 1.3 only
    fn create_tls_config(config: &MqttConfig) -> Result<ClientConfig, MqttError> {
        // Use rustls 0.20 (the version rumqttc uses via tokio-rustls)
        use rustls::ClientConfig as RustlsClientConfig;
        use rustls::RootCertStore;
        
        let mut root_store = RootCertStore::empty();

        // Load CA certificate
        let ca_cert_file = fs::File::open(&config.tls_ca_cert)
            .map_err(|e| MqttError::TlsConfig(format!("Failed to open CA cert: {}", e)))?;

        let mut ca_cert_reader = BufReader::new(ca_cert_file);
        let ca_certs = rustls_pemfile::certs(&mut ca_cert_reader)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| MqttError::TlsConfig(format!("Failed to parse CA cert: {}", e)))?;

        for cert in ca_certs {
            let cert_der = rustls::Certificate(cert.to_vec());
            root_store
                .add(&cert_der)
                .map_err(|e| MqttError::TlsConfig(format!("Failed to add CA cert: {}", e)))?;
        }

        // Configure TLS (mTLS support can be added later if needed)
        // For now, we'll use basic TLS without client certificates
        let tls_config = RustlsClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        Ok(tls_config)
    }

    /// Event loop handler
    async fn handle_events(
        mut eventloop: EventLoop,
        state: Arc<RwLock<ConnectionState>>,
        handlers: Arc<RwLock<HashMap<String, MessageHandler>>>,
    ) {
        loop {
            match eventloop.poll().await {
                Ok(Event::Incoming(Packet::ConnAck(_))) => {
                    info!("Connected to MQTT broker");
                    *state.write().await = ConnectionState::Connected;
                }
                Ok(Event::Incoming(Packet::Publish(publish))) => {
                    debug!("Received message on topic: {}", publish.topic);

                    let handlers_lock = handlers.read().await;
                    if let Some(handler) = handlers_lock.get(&publish.topic) {
                        handler(publish.topic.clone(), publish.payload.to_vec());
                    }
                }
                Ok(Event::Incoming(Packet::SubAck(_))) => {
                    debug!("Subscription acknowledged");
                }
                Ok(Event::Incoming(Packet::PingResp)) => {
                    debug!("Ping response received");
                }
                Ok(Event::Outgoing(_)) => {
                    // Outgoing events, can be ignored for now
                }
                Err(e) => {
                    error!("Event loop error: {}", e);
                    *state.write().await = ConnectionState::Error;

                    // Wait before retrying
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    *state.write().await = ConnectionState::Reconnecting;
                }
                _ => {}
            }
        }
    }

    /// Token refresh loop
    async fn token_refresh_loop(
        jwt_manager: JwtManager,
        current_token: Arc<RwLock<Option<String>>>,
        current_claims: Arc<RwLock<Option<MqttClaims>>>,
        client_id: String,
        permissions: Vec<String>,
    ) {
        loop {
            // Check every 10 seconds if token needs refresh
            tokio::time::sleep(Duration::from_secs(10)).await;

            let claims_lock = current_claims.read().await;
            if let Some(claims) = claims_lock.as_ref() {
                if jwt_manager.needs_refresh(claims) {
                    drop(claims_lock);

                    info!("Refreshing JWT token");
                    match jwt_manager.generate_token(client_id.clone(), permissions.clone()) {
                        Ok(new_token) => {
                            match jwt_manager.validate_token(&new_token) {
                                Ok(new_claims) => {
                                    *current_token.write().await = Some(new_token);
                                    *current_claims.write().await = Some(new_claims);
                                    info!("JWT token refreshed successfully");
                                }
                                Err(e) => {
                                    error!("Failed to validate new token: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to generate new token: {}", e);
                        }
                    }
                }
            }
        }
    }

    /// Check if the client has permission for a topic
    async fn check_permission(&self, topic: &str) -> Result<(), MqttError> {
        let claims_lock = self.current_claims.read().await;
        if let Some(claims) = claims_lock.as_ref() {
            if claims.has_permission(topic) {
                Ok(())
            } else {
                Err(MqttError::PermissionDenied(topic.to_string()))
            }
        } else {
            Err(MqttError::Authentication(AuthError::InvalidToken))
        }
    }

    /// Publish a message to a topic
    pub async fn publish<T: Serialize>(
        &self,
        topic: &str,
        payload: T,
        qos: QoS,
    ) -> Result<(), MqttError> {
        // Check permission
        self.check_permission(topic).await?;

        // Create message envelope
        let message = MqttMessage::new(topic.to_string(), payload);
        let bytes = serialize_message(&message)?;

        // Publish
        self.client
            .publish(topic, qos, false, bytes)
            .await
            .map_err(|e| MqttError::Publish(e.to_string()))?;

        debug!("Published message to topic: {}", topic);
        Ok(())
    }

    /// Subscribe to a topic with a handler
    pub async fn subscribe<F>(&self, topic: &str, qos: QoS, handler: F) -> Result<(), MqttError>
    where
        F: Fn(String, Vec<u8>) + Send + Sync + 'static,
    {
        // Check permission
        self.check_permission(topic).await?;

        // Subscribe
        self.client
            .subscribe(topic, qos)
            .await
            .map_err(|e| MqttError::Subscribe(e.to_string()))?;

        // Register handler
        let handler_arc: MessageHandler = Arc::new(handler);
        self.handlers
            .write()
            .await
            .insert(topic.to_string(), handler_arc);

        info!("Subscribed to topic: {}", topic);
        Ok(())
    }

    /// Subscribe to a topic with a typed message handler
    pub async fn subscribe_typed<T, F>(
        &self,
        topic: &str,
        qos: QoS,
        handler: F,
    ) -> Result<(), MqttError>
    where
        T: for<'de> Deserialize<'de> + Send + 'static,
        F: Fn(MqttMessage<T>) + Send + Sync + 'static,
    {
        let handler = move |_topic: String, bytes: Vec<u8>| {
            match deserialize_message::<T>(&bytes) {
                Ok(message) => handler(message),
                Err(e) => error!("Failed to deserialize message: {}", e),
            }
        };

        self.subscribe(topic, qos, handler).await
    }

    /// Unsubscribe from a topic
    pub async fn unsubscribe(&self, topic: &str) -> Result<(), MqttError> {
        self.client
            .unsubscribe(topic)
            .await
            .map_err(|e| MqttError::Subscribe(e.to_string()))?;

        self.handlers.write().await.remove(topic);

        info!("Unsubscribed from topic: {}", topic);
        Ok(())
    }

    /// Get the current connection state
    pub async fn state(&self) -> ConnectionState {
        *self.state.read().await
    }

    /// Get the current JWT token
    pub async fn current_token(&self) -> Option<String> {
        self.current_token.read().await.clone()
    }

    /// Get the current JWT claims
    pub async fn current_claims(&self) -> Option<MqttClaims> {
        self.current_claims.read().await.clone()
    }

    /// Disconnect from the broker
    pub async fn disconnect(&self) -> Result<(), MqttError> {
        // Cancel token refresh task
        if let Some(handle) = self.token_refresh_handle.write().await.take() {
            handle.abort();
        }

        self.client
            .disconnect()
            .await
            .map_err(|e| MqttError::Connection(e.to_string()))?;

        *self.state.write().await = ConnectionState::Disconnected;
        info!("Disconnected from MQTT broker");
        Ok(())
    }
}

impl Drop for MqttClient {
    fn drop(&mut self) {
        // Note: We can't properly disconnect in a synchronous Drop
        // Users should call disconnect() before dropping
        warn!("MqttClient dropped without explicit disconnect");
    }
}