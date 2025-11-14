use std::path::PathBuf;
use std::time::Duration;
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
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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
    /// Load MQTT configuration from environment variables
    pub fn from_env() -> Result<Self, ConfigError> {
        dotenvy::dotenv().ok();
        
        let broker_url = std::env::var("MQTT_BROKER_URL")
            .unwrap_or_else(|_| "mqtt://localhost".to_string());
        
        let port = std::env::var("MQTT_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8883);
        
        let tls_ca_cert = std::env::var("MQTT_TLS_CA_CERT")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("./certs/ca.crt"));
        
        let tls_client_cert = std::env::var("MQTT_TLS_CLIENT_CERT")
            .ok()
            .map(PathBuf::from);
        
        let tls_client_key = std::env::var("MQTT_TLS_CLIENT_KEY")
            .ok()
            .map(PathBuf::from);
        
        let jwt_secret = std::env::var("MQTT_JWT_SECRET")
            .map_err(|_| ConfigError::MissingConfig("MQTT_JWT_SECRET".to_string()))?;
        
        let jwt_lifetime_seconds = std::env::var("MQTT_JWT_LIFETIME_SECONDS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(300); // Default 5 minutes
        
        let client_id = std::env::var("MQTT_CLIENT_ID")
            .unwrap_or_else(|_| format!("jamey-{}", uuid::Uuid::new_v4()));
        
        let keep_alive_seconds = std::env::var("MQTT_KEEP_ALIVE_SECONDS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(60);
        
        let max_packet_size = std::env::var("MQTT_MAX_PACKET_SIZE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(268_435_456); // 256 MB default
        
        let connection_timeout_seconds = std::env::var("MQTT_CONNECTION_TIMEOUT_SECONDS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(30);
        
        // Parse permissions from comma-separated list
        let permissions = std::env::var("MQTT_PERMISSIONS")
            .ok()
            .map(|p| {
                p.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_else(|| vec![
                "jamey/#".to_string(), // Default: allow all jamey topics
            ]);
        
        Ok(Self {
            broker_url,
            port,
            tls_ca_cert,
            tls_client_cert,
            tls_client_key,
            jwt_secret,
            jwt_lifetime: Duration::from_secs(jwt_lifetime_seconds),
            client_id,
            keep_alive: Duration::from_secs(keep_alive_seconds),
            max_packet_size,
            connection_timeout: Duration::from_secs(connection_timeout_seconds),
            reconnect: ReconnectConfig::default(),
            permissions,
        })
    }
    
    /// Validate the configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Check broker URL
        if self.broker_url.is_empty() {
            return Err(ConfigError::MissingConfig("broker_url".to_string()));
        }
        
        // Check JWT secret
        if self.jwt_secret.is_empty() {
            return Err(ConfigError::MissingConfig("jwt_secret".to_string()));
        }
        
        if self.jwt_secret.len() < 32 {
            return Err(ConfigError::InvalidValue(
                "jwt_secret".to_string(),
                "JWT secret must be at least 32 characters".to_string(),
            ));
        }
        
        // Check client ID
        if self.client_id.is_empty() {
            return Err(ConfigError::MissingConfig("client_id".to_string()));
        }
        
        // Check CA certificate exists
        if !self.tls_ca_cert.exists() {
            return Err(ConfigError::InvalidValue(
                "tls_ca_cert".to_string(),
                format!("Certificate file not found: {:?}", self.tls_ca_cert),
            ));
        }
        
        // Check client certificate and key (if mTLS is configured)
        if self.tls_client_cert.is_some() || self.tls_client_key.is_some() {
            if self.tls_client_cert.is_none() || self.tls_client_key.is_none() {
                return Err(ConfigError::InvalidValue(
                    "mTLS".to_string(),
                    "Both client certificate and key must be provided for mTLS".to_string(),
                ));
            }
            
            if let Some(ref cert) = self.tls_client_cert {
                if !cert.exists() {
                    return Err(ConfigError::InvalidValue(
                        "tls_client_cert".to_string(),
                        format!("Certificate file not found: {:?}", cert),
                    ));
                }
            }
            
            if let Some(ref key) = self.tls_client_key {
                if !key.exists() {
                    return Err(ConfigError::InvalidValue(
                        "tls_client_key".to_string(),
                        format!("Key file not found: {:?}", key),
                    ));
                }
            }
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