use std::path::PathBuf;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// MQTT configuration errors
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Missing required configuration: {0}")]
    MissingConfig(String),
    
    #[error("Invalid configuration value for {0}: {1}")]
    InvalidValue(String, String),
    
    #[error("Failed to read certificate file: {0}")]
    CertificateRead(#[from] std::io::Error),
}

/// MQTT client configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MqttConfig {
    /// MQTT broker URL (e.g., "mqtt://localhost" or "mqtts://broker.example.com")
    pub broker_url: String,
    
    /// MQTT broker port (typically 8883 for TLS)
    pub port: u16,
    
    /// Path to CA certificate for TLS
    pub tls_ca_cert: PathBuf,
    
    /// Path to client certificate for mTLS (optional)
    pub tls_client_cert: Option<PathBuf>,
    
    /// Path to client private key for mTLS (optional)
    pub tls_client_key: Option<PathBuf>,
    
    /// JWT secret for token generation
    pub jwt_secret: String,
    
    /// JWT token lifetime
    pub jwt_lifetime: Duration,
    
    /// MQTT client ID
    pub client_id: String,
    
    /// Keep-alive interval
    pub keep_alive: Duration,
    
    /// Maximum packet size
    pub max_packet_size: usize,
    
    /// Connection timeout
    pub connection_timeout: Duration,
    
    /// Reconnection settings
    pub reconnect: ReconnectConfig,
    
    /// Topic permissions for this client
    pub permissions: Vec<String>,
}

/// Reconnection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconnectConfig {
    /// Initial retry delay
    pub initial_delay: Duration,
    
    /// Maximum retry delay
    pub max_delay: Duration,
    
    /// Exponential backoff multiplier
    pub backoff_multiplier: f64,
    
    /// Random jitter percentage (0.0 to 1.0)
    pub jitter: f64,
    
    /// Maximum number of consecutive failures before circuit breaker
    pub max_failures: u32,
}

impl Default for ReconnectConfig {
    fn default() -> Self {
        Self {
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
            backoff_multiplier: 2.0,
            jitter: 0.2,
            max_failures: 10,
        }
    }
}

impl MqttConfig {
    /// Create a new MQTT configuration with default values
    pub fn new() -> Self {
        Self {
            broker_url: "mqtt://localhost".to_string(),
            port: 8883,
            tls_ca_cert: PathBuf::from("./certs/ca.crt"),
            tls_client_cert: None,
            tls_client_key: None,
            jwt_secret: String::new(),
            jwt_lifetime: Duration::from_secs(300),
            client_id: format!("jamey-{}", uuid::Uuid::new_v4()),
            keep_alive: Duration::from_secs(60),
            max_packet_size: 268_435_456,
            connection_timeout: Duration::from_secs(30),
            reconnect: ReconnectConfig::default(),
            permissions: vec!["jamey/#".to_string()],
        }
    }

    /// Load MQTT configuration from environment variables
    pub fn from_env() -> Result<Self, ConfigError> {
        tracing::debug!("Loading MQTT configuration from environment");
        let mut config = Self::new();

        // Load broker URL and port
        if let Ok(url) = std::env::var("MQTT_BROKER_URL") {
            config.broker_url = url.clone();
            tracing::debug!("MQTT broker URL: {}", url);
        } else {
            tracing::debug!("Using default broker URL: {}", config.broker_url);
        }

        if let Ok(port_str) = std::env::var("MQTT_PORT") {
            match port_str.parse() {
                Ok(port) => {
                    config.port = port;
                    tracing::debug!("MQTT port: {}", port);
                }
                Err(_) => {
                    tracing::warn!("Invalid MQTT port value: {}", port_str);
                    return Err(ConfigError::InvalidValue("MQTT_PORT".to_string(), "Invalid port number".to_string()));
                }
            }
        } else {
            tracing::debug!("Using default port: {}", config.port);
        }

        // Load TLS configuration
        if let Ok(ca_cert) = std::env::var("MQTT_TLS_CA_CERT") {
            config.tls_ca_cert = PathBuf::from(ca_cert.clone());
            tracing::debug!("CA certificate path: {}", ca_cert);
        } else {
            tracing::debug!("Using default CA certificate path: {:?}", config.tls_ca_cert);
        }

        if let Ok(client_cert) = std::env::var("MQTT_TLS_CLIENT_CERT") {
            config.tls_client_cert = Some(PathBuf::from(client_cert.clone()));
            tracing::debug!("Client certificate path: {}", client_cert);
        }

        if let Ok(client_key) = std::env::var("MQTT_TLS_CLIENT_KEY") {
            config.tls_client_key = Some(PathBuf::from(client_key.clone()));
            tracing::debug!("Client key path: {}", client_key);
        }

        // Load JWT configuration
        match std::env::var("MQTT_JWT_SECRET") {
            Ok(secret) => {
                config.jwt_secret = secret;
                tracing::debug!("JWT secret loaded: {}", if config.jwt_secret.is_empty() { "empty" } else { "present" });
            }
            Err(_) => {
                tracing::error!("Required MQTT_JWT_SECRET not found");
                return Err(ConfigError::MissingConfig("MQTT_JWT_SECRET".to_string()));
            }
        }

        if let Ok(lifetime_str) = std::env::var("MQTT_JWT_LIFETIME_SECONDS") {
            match lifetime_str.parse() {
                Ok(lifetime) => {
                    config.jwt_lifetime = Duration::from_secs(lifetime);
                    tracing::debug!("JWT lifetime: {} seconds", lifetime);
                }
                Err(_) => {
                    tracing::warn!("Invalid JWT lifetime value: {}", lifetime_str);
                    return Err(ConfigError::InvalidValue("MQTT_JWT_LIFETIME_SECONDS".to_string(), "Invalid duration".to_string()));
                }
            }
        } else {
            tracing::debug!("Using default JWT lifetime: {} seconds", config.jwt_lifetime.as_secs());
        }

        // Load client settings
        if let Ok(client_id) = std::env::var("MQTT_CLIENT_ID") {
            config.client_id = client_id.clone();
            tracing::debug!("Client ID: {}", client_id);
        } else {
            tracing::debug!("Using generated client ID: {}", config.client_id);
        }

        if let Ok(keep_alive_str) = std::env::var("MQTT_KEEP_ALIVE_SECONDS") {
            match keep_alive_str.parse() {
                Ok(keep_alive) => {
                    config.keep_alive = Duration::from_secs(keep_alive);
                    tracing::debug!("Keep-alive interval: {} seconds", keep_alive);
                }
                Err(_) => {
                    tracing::warn!("Invalid keep-alive value: {}", keep_alive_str);
                    return Err(ConfigError::InvalidValue("MQTT_KEEP_ALIVE_SECONDS".to_string(), "Invalid duration".to_string()));
                }
            }
        }

        if let Ok(packet_size_str) = std::env::var("MQTT_MAX_PACKET_SIZE") {
            match packet_size_str.parse() {
                Ok(packet_size) => {
                    config.max_packet_size = packet_size;
                    tracing::debug!("Maximum packet size: {} bytes", packet_size);
                }
                Err(_) => {
                    tracing::warn!("Invalid packet size value: {}", packet_size_str);
                    return Err(ConfigError::InvalidValue("MQTT_MAX_PACKET_SIZE".to_string(), "Invalid size".to_string()));
                }
            }
        }

        if let Ok(timeout_str) = std::env::var("MQTT_CONNECTION_TIMEOUT_SECONDS") {
            match timeout_str.parse() {
                Ok(timeout) => {
                    config.connection_timeout = Duration::from_secs(timeout);
                    tracing::debug!("Connection timeout: {} seconds", timeout);
                }
                Err(_) => {
                    tracing::warn!("Invalid connection timeout value: {}", timeout_str);
                    return Err(ConfigError::InvalidValue("MQTT_CONNECTION_TIMEOUT_SECONDS".to_string(), "Invalid duration".to_string()));
                }
            }
        }

        // Load permissions
        if let Ok(perms) = std::env::var("MQTT_PERMISSIONS") {
            config.permissions = perms
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            tracing::debug!("Topic permissions: {:?}", config.permissions);
        } else {
            tracing::debug!("Using default permissions: {:?}", config.permissions);
        }

        // Validate configuration
        tracing::debug!("Validating MQTT configuration...");
        config.validate()?;
        tracing::debug!("MQTT configuration validated successfully");
        
        Ok(config)
    }
    
    /// Validate the configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate required fields
        if self.broker_url.is_empty() {
            return Err(ConfigError::MissingConfig("broker_url".to_string()));
        }

        if self.jwt_secret.is_empty() {
            return Err(ConfigError::MissingConfig("jwt_secret".to_string()));
        }

        if self.client_id.is_empty() {
            return Err(ConfigError::MissingConfig("client_id".to_string()));
        }

        // Validate value constraints
        if self.jwt_secret.len() < 32 {
            return Err(ConfigError::InvalidValue(
                "jwt_secret".to_string(),
                "JWT secret must be at least 32 characters".to_string(),
            ));
        }

        if self.port == 0 {
            return Err(ConfigError::InvalidValue(
                "port".to_string(),
                "Port number cannot be 0".to_string(),
            ));
        }

        // Validate TLS configuration
        if !self.tls_ca_cert.exists() {
            return Err(ConfigError::InvalidValue(
                "tls_ca_cert".to_string(),
                format!("Certificate file not found: {:?}", self.tls_ca_cert),
            ));
        }

        // Validate mTLS configuration
        if let (Some(cert), Some(key)) = (&self.tls_client_cert, &self.tls_client_key) {
            if !cert.exists() {
                return Err(ConfigError::InvalidValue(
                    "tls_client_cert".to_string(),
                    format!("Client certificate not found: {:?}", cert),
                ));
            }
            if !key.exists() {
                return Err(ConfigError::InvalidValue(
                    "tls_client_key".to_string(),
                    format!("Client key not found: {:?}", key),
                ));
            }
        } else if self.tls_client_cert.is_some() || self.tls_client_key.is_some() {
            return Err(ConfigError::InvalidValue(
                "mTLS".to_string(),
                "Both client certificate and key must be provided for mTLS".to_string(),
            ));
        }

        // Validate durations
        if self.jwt_lifetime.as_secs() < 60 {
            return Err(ConfigError::InvalidValue(
                "jwt_lifetime".to_string(),
                "JWT lifetime must be at least 60 seconds".to_string(),
            ));
        }

        if self.keep_alive.as_secs() < 10 {
            return Err(ConfigError::InvalidValue(
                "keep_alive".to_string(),
                "Keep-alive must be at least 10 seconds".to_string(),
            ));
        }

        if self.connection_timeout.as_secs() < 5 {
            return Err(ConfigError::InvalidValue(
                "connection_timeout".to_string(),
                "Connection timeout must be at least 5 seconds".to_string(),
            ));
        }

        // Validate packet size
        if self.max_packet_size == 0 {
            return Err(ConfigError::InvalidValue(
                "max_packet_size".to_string(),
                "Maximum packet size cannot be 0".to_string(),
            ));
        }

        Ok(())
    }
    
    /// Get the broker address as host:port
    pub fn broker_address(&self) -> String {
        format!("{}:{}", self.broker_url.trim_start_matches("mqtt://").trim_start_matches("mqtts://"), self.port)
    }
    
    /// Check if mTLS is configured
    pub fn has_mtls(&self) -> bool {
        self.tls_client_cert.is_some() && self.tls_client_key.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_reconnect_config() {
        let config = ReconnectConfig::default();
        assert_eq!(config.initial_delay, Duration::from_secs(1));
        assert_eq!(config.max_delay, Duration::from_secs(60));
        assert_eq!(config.backoff_multiplier, 2.0);
    }
    
    #[test]
    fn test_broker_address() {
        let config = MqttConfig {
            broker_url: "mqtt://localhost".to_string(),
            port: 8883,
            tls_ca_cert: PathBuf::from("./certs/ca.crt"),
            tls_client_cert: None,
            tls_client_key: None,
            jwt_secret: "test-secret-that-is-long-enough-for-validation".to_string(),
            jwt_lifetime: Duration::from_secs(300),
            client_id: "test-client".to_string(),
            keep_alive: Duration::from_secs(60),
            max_packet_size: 268_435_456,
            connection_timeout: Duration::from_secs(30),
            reconnect: ReconnectConfig::default(),
            permissions: vec!["jamey/#".to_string()],
        };
        
        assert_eq!(config.broker_address(), "localhost:8883");
    }
}