use std::env;
<<<<<<< HEAD
=======
use std::time::Duration;
>>>>>>> origin/main
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
<<<<<<< HEAD
        let default_trust = env::var("SOUL_DEFAULT_TRUST")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.5);
        tracing::debug!("Soul default trust: {}", default_trust);

        let base_decay_rate = env::var("SOUL_BASE_DECAY_RATE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.01);
        tracing::debug!("Soul base decay rate: {}", base_decay_rate);

        let prune_threshold = env::var("SOUL_PRUNE_THRESHOLD")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.1);
        tracing::debug!("Soul prune threshold: {}", prune_threshold);

        let empathy_threshold = env::var("SOUL_EMPATHY_THRESHOLD")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.7);
        tracing::debug!("Soul empathy threshold: {}", empathy_threshold);

        let auto_record_emotions = env::var("SOUL_AUTO_RECORD")
            .ok()
            .map(|v| v == "true")
            .unwrap_or(true);
        tracing::debug!("Soul auto record emotions: {}", auto_record_emotions);

        Self {
            default_trust,
            base_decay_rate,
            prune_threshold,
            empathy_threshold,
            auto_record_emotions,
=======
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
>>>>>>> origin/main
        }
    }
}

<<<<<<< HEAD
/// Security configuration
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    /// Allowed CORS origins (comma-separated)
    pub allowed_origins: Vec<String>,
    /// API key for authentication (optional, if not set, no auth required)
    pub api_key: Option<String>,
    /// Rate limit: requests per minute per IP
    pub rate_limit_per_minute: u32,
    /// Maximum length for action field in evaluate endpoint
    pub max_action_length: usize,
    /// Maximum length for rule name
    pub max_rule_name_length: usize,
    /// Maximum length for rule description
    pub max_rule_description_length: usize,
    /// Minimum rule weight
    pub min_rule_weight: f32,
    /// Maximum rule weight
    pub max_rule_weight: f32,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            allowed_origins: vec!["http://localhost:5173".to_string(), "http://localhost:3000".to_string()],
            api_key: None,
            rate_limit_per_minute: 60,
            max_action_length: 10_000,
            max_rule_name_length: 100,
            max_rule_description_length: 500,
            min_rule_weight: 0.0,
            max_rule_weight: 100.0,
=======
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
>>>>>>> origin/main
        }
    }
}

<<<<<<< HEAD
impl SecurityConfig {
    pub fn from_env() -> Self {
        let allowed_origins = env::var("ALLOWED_ORIGINS")
            .ok()
            .map(|v| v.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_else(|| {
                // Default to localhost for development
                vec!["http://localhost:5173".to_string(), "http://localhost:3000".to_string()]
            });

        let api_key = env::var("API_KEY").ok();

        Self {
            allowed_origins,
            api_key,
            rate_limit_per_minute: env::var("RATE_LIMIT_PER_MINUTE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(60),
            max_action_length: env::var("MAX_ACTION_LENGTH")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10_000),
            max_rule_name_length: env::var("MAX_RULE_NAME_LENGTH")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(100),
            max_rule_description_length: env::var("MAX_RULE_DESCRIPTION_LENGTH")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(500),
            min_rule_weight: env::var("MIN_RULE_WEIGHT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0.0),
            max_rule_weight: env::var("MAX_RULE_WEIGHT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(100.0),
        }
    }
}

/// Core configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreConfig {
    pub openrouter_api_key: String,
    pub openrouter_model: String,
    pub openrouter_api_url: String,
    pub database_url: Option<String>,
    pub data_dir: Option<String>,
}

/// Operational settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationalConfig {
    pub port: u16,
    pub host: String,
    pub dev_mode: bool,
    pub enable_test_features: bool,
    pub log_level: String,
}

impl Default for OperationalConfig {
    fn default() -> Self {
        Self {
            port: 3000,
            host: "0.0.0.0".to_string(),
            dev_mode: false,
            enable_test_features: false,
            log_level: "info".to_string(),
        }
    }
}

/// Phoenix backup system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhoenixConfig {
    pub enabled: bool,
    pub backup_dir: String,
    pub encryption_key: String,
    pub auto_backup_hours: u32,
    pub max_backups: u32,
}

impl Default for PhoenixConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            backup_dir: "data/phoenix".to_string(),
            encryption_key: String::new(),
            auto_backup_hours: 24,
            max_backups: 10,
=======
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
>>>>>>> origin/main
        }
    }
}

/// Application configuration loaded from environment variables
#[derive(Debug, Clone)]
pub struct Config {
<<<<<<< HEAD
    pub core: CoreConfig,
    pub security: SecurityConfig,
    pub soul: SoulConfig,
    pub mqtt: Option<MqttConfig>,
    pub phoenix: Option<PhoenixConfig>,
    pub operational: OperationalConfig,
=======
    pub openrouter_api_key: String,
    pub openrouter_model: String,
    pub openrouter_api_url: String,
    pub database_url: Option<String>,
    pub mqtt: Option<MqttConfig>,
    pub soul: SoulConfig,
    pub consciousness: ConsciousnessConfig,
    pub database: DatabaseConfig,
>>>>>>> origin/main
}

impl Config {
    /// Load configuration from environment variables
<<<<<<< HEAD
    pub fn from_env() -> anyhow::Result<Option<Self>> {
        // Load .env file if it exists
        if let Ok(path) = dotenvy::dotenv() {
            tracing::debug!("Loaded .env file from: {}", path.display());
        }

        // Load core configuration
        let core = Self::load_core_config()?;
        tracing::debug!("Core config loaded: {:?}", core);
        
        if core.openrouter_api_key.is_empty() {
            tracing::warn!("OPENROUTER_API_KEY not set. LLM features will be unavailable.");
            return Ok(None);
        }

        // Load other configurations
        let security = SecurityConfig::from_env();
        tracing::debug!("Security config loaded: {:?}", security);
        
        let soul = SoulConfig::from_env();
        tracing::debug!("Soul config loaded: {:?}", soul);
        
        let mqtt = MqttConfig::from_env().ok();
        tracing::debug!("MQTT config loaded: {:?}", mqtt);
        
        let phoenix = Self::load_phoenix_config();
        tracing::debug!("Phoenix config loaded: {:?}", phoenix);
        
        let operational = Self::load_operational_config();
        tracing::debug!("Operational config loaded: {:?}", operational);

        Ok(Some(Config {
            core,
            security,
            soul,
            mqtt,
            phoenix,
            operational,
        }))
    }

    /// Load core configuration settings
    fn load_core_config() -> anyhow::Result<CoreConfig> {
        let api_key = env::var("OPENROUTER_API_KEY").unwrap_or_default();
        tracing::debug!("OpenRouter API key loaded: {}", if api_key.is_empty() { "empty" } else { "present" });

        let model = env::var("OPENROUTER_MODEL")
            .unwrap_or_else(|_| "anthropic/claude-3.5-sonnet".to_string());
        tracing::debug!("OpenRouter model: {}", model);

        let api_url = env::var("OPENROUTER_API_URL")
            .unwrap_or_else(|_| "https://openrouter.ai/api/v1".to_string());
        tracing::debug!("OpenRouter API URL: {}", api_url);

        let database_url = env::var("DATABASE_URL").ok();
        tracing::debug!("Database URL: {:?}", database_url);

        let data_dir = env::var("DATA_DIR").ok();
        tracing::debug!("Data directory: {:?}", data_dir);

        Ok(CoreConfig {
            openrouter_api_key: api_key,
            openrouter_model: model,
            openrouter_api_url: api_url,
            database_url,
            data_dir,
        })
    }

    /// Load Phoenix backup configuration if enabled
    fn load_phoenix_config() -> Option<PhoenixConfig> {
        let enabled = env::var("PHOENIX_ENABLED").map(|v| v != "false").unwrap_or(true);
        tracing::debug!("Phoenix backup system enabled: {}", enabled);

        if enabled {
            let backup_dir = env::var("PHOENIX_BACKUP_DIR")
                .unwrap_or_else(|_| "data/phoenix".to_string());
            tracing::debug!("Phoenix backup directory: {}", backup_dir);

            let encryption_key = env::var("PHOENIX_ENCRYPTION_KEY").unwrap_or_default();
            tracing::debug!("Phoenix encryption key: {}",
                if encryption_key.is_empty() { "not set" } else { "present" });

            let auto_backup_hours = env::var("PHOENIX_AUTO_BACKUP_HOURS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(24);
            tracing::debug!("Phoenix auto backup interval: {} hours", auto_backup_hours);

            let max_backups = env::var("PHOENIX_MAX_BACKUPS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10);
            tracing::debug!("Phoenix max backups: {}", max_backups);

            Some(PhoenixConfig {
                enabled: true,
                backup_dir,
                encryption_key,
                auto_backup_hours,
                max_backups,
            })
        } else {
            tracing::debug!("Phoenix backup system disabled");
            None
        }
    }

    /// Load operational configuration
    fn load_operational_config() -> OperationalConfig {
        OperationalConfig {
            port: env::var("PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3000),
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            dev_mode: env::var("DEV_MODE").map(|v| v == "true").unwrap_or(false),
            enable_test_features: env::var("ENABLE_TEST_FEATURES")
                .map(|v| v == "true")
                .unwrap_or(false),
            log_level: env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
        }
    }

    /// Load configuration from environment variables (required)
    pub fn from_env_required() -> anyhow::Result<Self> {
        Self::from_env()?
            .ok_or_else(|| anyhow::anyhow!("OPENROUTER_API_KEY environment variable is required"))
    }

    /// Validate configuration
    pub fn validate(&self) -> anyhow::Result<()> {
        tracing::debug!("Validating configuration...");

        // Validate core config
        if self.core.openrouter_api_key.is_empty() {
            tracing::error!("OPENROUTER_API_KEY validation failed: empty value");
            anyhow::bail!("OPENROUTER_API_KEY cannot be empty");
        }
        if self.core.openrouter_model.is_empty() {
            tracing::error!("OPENROUTER_MODEL validation failed: empty value");
            anyhow::bail!("OPENROUTER_MODEL cannot be empty");
        }
        tracing::debug!("Core configuration validated successfully");

        // Validate Phoenix config if enabled
        if let Some(phoenix) = &self.phoenix {
            if phoenix.enabled {
                if phoenix.encryption_key.is_empty() {
                    tracing::error!("Phoenix validation failed: encryption key missing");
                    anyhow::bail!("PHOENIX_ENCRYPTION_KEY is required when Phoenix backup is enabled");
                }
                if phoenix.auto_backup_hours == 0 {
                    tracing::warn!("Phoenix auto_backup_hours is 0 - automatic backups disabled");
                }
                tracing::debug!("Phoenix configuration validated successfully");
            }
        }

        // Validate Soul config
        if self.soul.default_trust < 0.0 || self.soul.default_trust > 1.0 {
            tracing::error!("Soul validation failed: default_trust out of range [0.0, 1.0]");
            anyhow::bail!("SOUL_DEFAULT_TRUST must be between 0.0 and 1.0");
        }
        if self.soul.base_decay_rate < 0.0 || self.soul.base_decay_rate > 1.0 {
            tracing::error!("Soul validation failed: base_decay_rate out of range [0.0, 1.0]");
            anyhow::bail!("SOUL_BASE_DECAY_RATE must be between 0.0 and 1.0");
        }
        tracing::debug!("Soul configuration validated successfully");

        // Validate security config in production
        if !self.operational.dev_mode {
            if self.security.api_key.is_none() {
                tracing::error!("Security validation failed: API_KEY required in production");
                anyhow::bail!("API_KEY is required in production mode");
            }
            if self.security.allowed_origins.is_empty() {
                tracing::error!("Security validation failed: ALLOWED_ORIGINS required in production");
                anyhow::bail!("ALLOWED_ORIGINS must be configured in production mode");
            }
            tracing::debug!("Security configuration validated successfully");
        }

        tracing::debug!("All configuration validated successfully");
=======
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
>>>>>>> origin/main
        Ok(())
    }
}

<<<<<<< HEAD
=======
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

>>>>>>> origin/main
