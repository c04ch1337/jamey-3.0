use std::env;
use std::time::Duration;
use crate::mqtt::MqttConfig;
use serde::{Deserialize, Serialize};

/// Soul Knowledge Base configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoulConfig {
    /// Default trust score for new entities
    pub default_trust: f32,
    /// Base decay rate per day
    pub base_decay_rate: f32,
    /// Minimum trust score before pruning
    pub prune_threshold: f32,
    /// Minimum empathy threshold for positive interactions
    pub empathy_threshold: f32,
    /// Whether to enable automatic emotion recording from conscience
    pub auto_record_emotions: bool,
}

impl Default for SoulConfig {
    fn default() -> Self {
        Self {
            default_trust: 0.5,
            base_decay_rate: 0.01,
            prune_threshold: 0.1,
            empathy_threshold: 0.7,
            auto_record_emotions: true,
        }
    }
}

impl SoulConfig {
    /// Load soul configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            default_trust: env::var("SOUL_DEFAULT_TRUST")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0.5),
            base_decay_rate: env::var("SOUL_BASE_DECAY_RATE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0.01),
            prune_threshold: env::var("SOUL_PRUNE_THRESHOLD")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0.1),
            empathy_threshold: env::var("SOUL_EMPATHY_THRESHOLD")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0.7),
            auto_record_emotions: env::var("SOUL_AUTO_RECORD")
                .ok()
                .map(|v| v == "true")
                .unwrap_or(true),
        }
    }
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Maximum number of connections in the pool
    pub max_connections: u32,
    /// Connection timeout in seconds
    pub connect_timeout_secs: u64,
    /// Query timeout in seconds
    pub query_timeout_secs: u64,
    /// Enable performance metrics
    pub enable_metrics: bool,
    /// Enable WAL journal mode
    pub enable_wal: bool,
    /// Connection pool idle timeout in seconds
    pub idle_timeout_secs: u64,
    /// Connection pool max lifetime in seconds
    pub max_lifetime_secs: u64,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            max_connections: 10,
            connect_timeout_secs: 30,
            query_timeout_secs: 10,
            enable_metrics: true,
            enable_wal: true,
            idle_timeout_secs: 600, // 10 minutes
            max_lifetime_secs: 1800, // 30 minutes
        }
    }
}

impl DatabaseConfig {
    /// Load database configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            max_connections: env::var("DB_MAX_CONNECTIONS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10),
            connect_timeout_secs: env::var("DB_CONNECT_TIMEOUT_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(30),
            query_timeout_secs: env::var("DB_QUERY_TIMEOUT_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10),
            enable_metrics: env::var("DB_ENABLE_METRICS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(true),
            enable_wal: env::var("DB_ENABLE_WAL")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(true),
            idle_timeout_secs: env::var("DB_IDLE_TIMEOUT_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(600),
            max_lifetime_secs: env::var("DB_MAX_LIFETIME_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1800),
        }
    }

    /// Get connect timeout as Duration
    pub fn connect_timeout(&self) -> Duration {
        Duration::from_secs(self.connect_timeout_secs)
    }

    /// Get query timeout as Duration
    pub fn query_timeout(&self) -> Duration {
        Duration::from_secs(self.query_timeout_secs)
    }

    /// Get idle timeout as Duration
    pub fn idle_timeout(&self) -> Duration {
        Duration::from_secs(self.idle_timeout_secs)
    }

    /// Get max lifetime as Duration
    pub fn max_lifetime(&self) -> Duration {
        Duration::from_secs(self.max_lifetime_secs)
    }
}

/// Consciousness System configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsciousnessConfig {
    /// Competition threshold for global workspace broadcast (0.0 to 1.0)
    pub competition_threshold: f64,
    /// Broadcast channel size for workspace
    pub broadcast_channel_size: usize,
    /// Broadcast factor for activity level calculation
    pub broadcast_factor: f64,
    /// Competition divisor for activity level calculation
    pub competition_divisor: f64,
    /// Maximum competition factor for activity level
    pub competition_max_factor: f64,
    /// Maximum content length for priority calculation
    pub priority_max_length: f64,
    /// Φ threshold for consciousness check (0.0 to 1.0)
    pub phi_threshold: f64,
    /// Epsilon value for Φ calculations
    pub phi_epsilon: f64,
    /// Maximum content length for feature extraction
    pub feature_max_length: f64,
    /// Maximum word count for feature extraction
    pub feature_max_words: f64,
    /// Whether to enable higher-order thought processing
    pub enable_higher_order: bool,
    pub enable_predictive: bool,
    pub enable_attention: bool,
}

impl Default for ConsciousnessConfig {
    fn default() -> Self {
        Self {
            competition_threshold: 0.7,
            broadcast_channel_size: 100,
            broadcast_factor: 0.5,
            competition_divisor: 10.0,
            competition_max_factor: 0.5,
            priority_max_length: 100.0,
            phi_threshold: 0.85,
            phi_epsilon: 1e-6,
            feature_max_length: 100.0,
            feature_max_words: 50.0,
            enable_higher_order: true,
            enable_predictive: true,
            enable_attention: true,
        }
    }
}

impl ConsciousnessConfig {
    /// Load consciousness configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            competition_threshold: env::var("CONSCIOUSNESS_COMPETITION_THRESHOLD")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0.7),
            broadcast_channel_size: env::var("CONSCIOUSNESS_BROADCAST_CHANNEL_SIZE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(100),
            broadcast_factor: env::var("CONSCIOUSNESS_BROADCAST_FACTOR")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0.5),
            competition_divisor: env::var("CONSCIOUSNESS_COMPETITION_DIVISOR")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10.0),
            competition_max_factor: env::var("CONSCIOUSNESS_COMPETITION_MAX_FACTOR")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0.5),
            priority_max_length: env::var("CONSCIOUSNESS_PRIORITY_MAX_LENGTH")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(100.0),
            phi_threshold: env::var("CONSCIOUSNESS_PHI_THRESHOLD")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0.85),
            phi_epsilon: env::var("CONSCIOUSNESS_PHI_EPSILON")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1e-6),
            feature_max_length: env::var("CONSCIOUSNESS_FEATURE_MAX_LENGTH")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(100.0),
            feature_max_words: env::var("CONSCIOUSNESS_FEATURE_MAX_WORDS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(50.0),
            enable_higher_order: env::var("CONSCIOUSNESS_ENABLE_HIGHER_ORDER")
                .ok()
                .map(|v| v == "true")
                .unwrap_or(true),
            enable_predictive: env::var("CONSCIOUSNESS_ENABLE_PREDICTIVE")
                .ok()
                .map(|v| v == "true")
                .unwrap_or(true),
            enable_attention: env::var("CONSCIOUSNESS_ENABLE_ATTENTION")
                .ok()
                .map(|v| v == "true")
                .unwrap_or(true),
        }
    }
}

/// Application configuration loaded from environment variables
#[derive(Debug, Clone)]
pub struct Config {
    pub openrouter_api_key: String,
    pub openrouter_model: String,
    pub openrouter_api_url: String,
    pub database_url: Option<String>,
    pub mqtt: Option<MqttConfig>,
    pub soul: SoulConfig,
    pub consciousness: ConsciousnessConfig,
    pub database: DatabaseConfig,
}

impl Config {
    /// Load configuration from environment variables
    /// Returns None if OPENROUTER_API_KEY is not set (LLM features will be unavailable)
    pub fn from_env() -> anyhow::Result<Option<Self>> {
        // Load .env file if it exists (dotenvy handles this)
        dotenvy::dotenv().ok();

        // OpenRouter API key is optional - only needed for LLM features
        let openrouter_api_key = match env::var("OPENROUTER_API_KEY") {
            Ok(key) if !key.is_empty() => key,
            _ => {
                tracing::warn!("OPENROUTER_API_KEY not set. LLM features will be unavailable.");
                tracing::warn!("Create a .env file with OPENROUTER_API_KEY=your-key to enable LLM features.");
                return Ok(None);
            }
        };

        let openrouter_model = env::var("OPENROUTER_MODEL")
            .unwrap_or_else(|_| "deepseek/deepseek-chat".to_string());

        let openrouter_api_url = env::var("OPENROUTER_API_URL")
            .unwrap_or_else(|_| "https://openrouter.ai/api/v1".to_string());

        let database_url = env::var("DATABASE_URL").ok();

        // Try to load MQTT configuration (optional)
        let mqtt = MqttConfig::from_env().ok();
        if mqtt.is_none() {
            tracing::info!("MQTT configuration not found or incomplete. MQTT features will be unavailable.");
        }

        Ok(Some(Config {
            openrouter_api_key,
            openrouter_model,
            openrouter_api_url,
            database_url,
            mqtt,
            soul: SoulConfig::from_env(),
            consciousness: ConsciousnessConfig::from_env(),
            database: DatabaseConfig::from_env(),
        }))
    }

    /// Load configuration from environment variables (required)
    /// Fails if OPENROUTER_API_KEY is not set
    pub fn from_env_required() -> anyhow::Result<Self> {
        Self::from_env()?
            .ok_or_else(|| anyhow::anyhow!("OPENROUTER_API_KEY environment variable is required. Create a .env file with your API key."))
    }

    /// Validate that required configuration is present
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.openrouter_api_key.is_empty() {
            anyhow::bail!("OPENROUTER_API_KEY cannot be empty");
        }
        if self.openrouter_model.is_empty() {
            anyhow::bail!("OPENROUTER_MODEL cannot be empty");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consciousness_config_default() {
        let config = ConsciousnessConfig::default();
        assert_eq!(config.competition_threshold, 0.7);
        assert_eq!(config.phi_threshold, 0.85);
        assert_eq!(config.broadcast_channel_size, 100);
        assert_eq!(config.phi_epsilon, 1e-6);
        assert!(config.enable_higher_order);
        assert!(config.enable_predictive);
        assert!(config.enable_attention);
    }

    #[test]
    fn test_consciousness_config_from_env() {
        // Test that from_env works even without env vars (uses defaults)
        let config = ConsciousnessConfig::from_env();
        assert!(config.competition_threshold > 0.0);
        assert!(config.phi_threshold > 0.0);
        assert!(config.broadcast_channel_size > 0);
        assert!(config.enable_higher_order);
        assert!(config.enable_predictive);
        assert!(config.enable_attention);
    }

    #[test]
    fn test_database_config_default() {
        let config = DatabaseConfig::default();
        assert_eq!(config.max_connections, 10);
        assert_eq!(config.connect_timeout_secs, 30);
        assert_eq!(config.query_timeout_secs, 10);
        assert!(config.enable_metrics);
        assert!(config.enable_wal);
        assert_eq!(config.idle_timeout_secs, 600);
        assert_eq!(config.max_lifetime_secs, 1800);
    }

    #[test]
    fn test_database_config_from_env() {
        // Test that from_env works even without env vars (uses defaults)
        let config = DatabaseConfig::from_env();
        assert!(config.max_connections > 0);
        assert!(config.connect_timeout_secs > 0);
        assert!(config.query_timeout_secs > 0);
        assert!(config.enable_metrics);
        assert!(config.enable_wal);
    }

    #[test]
    fn test_database_config_durations() {
        let config = DatabaseConfig::default();
        assert_eq!(config.connect_timeout(), Duration::from_secs(30));
        assert_eq!(config.query_timeout(), Duration::from_secs(10));
        assert_eq!(config.idle_timeout(), Duration::from_secs(600));
        assert_eq!(config.max_lifetime(), Duration::from_secs(1800));
    }
}

