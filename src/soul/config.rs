use serde::{Deserialize, Serialize};
use std::env;
use super::SoulError;

/// Soul system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoulConfig {
    /// Whether the Soul system is enabled
    pub enabled: bool,

    /// Default trust score for new entities (0.0 to 1.0)
    pub default_trust: f32,

    /// Base decay rate per day (0.0 to 1.0)
    pub base_decay_rate: f32,

    /// Minimum trust score before pruning (0.0 to 1.0)
    pub prune_threshold: f32,

    /// Minimum empathy threshold for positive interactions (0.0 to 1.0)
    pub empathy_threshold: f32,

    /// Whether to enable automatic emotion recording from conscience
    pub auto_record_emotions: bool,

    /// Maximum number of emotions to track per entity
    pub max_emotions_per_entity: usize,

    /// Maximum number of linked memories per entity
    pub max_memories_per_entity: usize,

    /// Trust boost factor for positive interactions (0.0 to 1.0)
    pub trust_boost_factor: f32,

    /// Trust penalty factor for negative interactions (0.0 to 1.0)
    pub trust_penalty_factor: f32,
}

impl Default for SoulConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_trust: 0.5,
            base_decay_rate: 0.01,
            prune_threshold: 0.1,
            empathy_threshold: 0.7,
            auto_record_emotions: true,
            max_emotions_per_entity: 100,
            max_memories_per_entity: 1000,
            trust_boost_factor: 0.5,
            trust_penalty_factor: 0.3,
        }
    }
}

impl SoulConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, SoulError> {
        let mut config = Self::default();

        if let Ok(val) = env::var("SOUL_ENABLED") {
            config.enabled = val == "true";
        }

        if let Ok(val) = env::var("SOUL_DEFAULT_TRUST") {
            config.default_trust = val.parse().map_err(|_| 
                SoulError::Config("Invalid SOUL_DEFAULT_TRUST value".to_string()))?;
            if !(0.0..=1.0).contains(&config.default_trust) {
                return Err(SoulError::Config("SOUL_DEFAULT_TRUST must be between 0.0 and 1.0".to_string()));
            }
        }

        if let Ok(val) = env::var("SOUL_BASE_DECAY_RATE") {
            config.base_decay_rate = val.parse().map_err(|_|
                SoulError::Config("Invalid SOUL_BASE_DECAY_RATE value".to_string()))?;
            if !(0.0..=1.0).contains(&config.base_decay_rate) {
                return Err(SoulError::Config("SOUL_BASE_DECAY_RATE must be between 0.0 and 1.0".to_string()));
            }
        }

        if let Ok(val) = env::var("SOUL_PRUNE_THRESHOLD") {
            config.prune_threshold = val.parse().map_err(|_|
                SoulError::Config("Invalid SOUL_PRUNE_THRESHOLD value".to_string()))?;
            if !(0.0..=1.0).contains(&config.prune_threshold) {
                return Err(SoulError::Config("SOUL_PRUNE_THRESHOLD must be between 0.0 and 1.0".to_string()));
            }
        }

        if let Ok(val) = env::var("SOUL_EMPATHY_THRESHOLD") {
            config.empathy_threshold = val.parse().map_err(|_|
                SoulError::Config("Invalid SOUL_EMPATHY_THRESHOLD value".to_string()))?;
            if !(0.0..=1.0).contains(&config.empathy_threshold) {
                return Err(SoulError::Config("SOUL_EMPATHY_THRESHOLD must be between 0.0 and 1.0".to_string()));
            }
        }

        if let Ok(val) = env::var("SOUL_AUTO_RECORD") {
            config.auto_record_emotions = val == "true";
        }

        if let Ok(val) = env::var("SOUL_MAX_EMOTIONS") {
            config.max_emotions_per_entity = val.parse().map_err(|_|
                SoulError::Config("Invalid SOUL_MAX_EMOTIONS value".to_string()))?;
        }

        if let Ok(val) = env::var("SOUL_MAX_MEMORIES") {
            config.max_memories_per_entity = val.parse().map_err(|_|
                SoulError::Config("Invalid SOUL_MAX_MEMORIES value".to_string()))?;
        }

        if let Ok(val) = env::var("SOUL_TRUST_BOOST_FACTOR") {
            config.trust_boost_factor = val.parse().map_err(|_|
                SoulError::Config("Invalid SOUL_TRUST_BOOST_FACTOR value".to_string()))?;
            if !(0.0..=1.0).contains(&config.trust_boost_factor) {
                return Err(SoulError::Config("SOUL_TRUST_BOOST_FACTOR must be between 0.0 and 1.0".to_string()));
            }
        }

        if let Ok(val) = env::var("SOUL_TRUST_PENALTY_FACTOR") {
            config.trust_penalty_factor = val.parse().map_err(|_|
                SoulError::Config("Invalid SOUL_TRUST_PENALTY_FACTOR value".to_string()))?;
            if !(0.0..=1.0).contains(&config.trust_penalty_factor) {
                return Err(SoulError::Config("SOUL_TRUST_PENALTY_FACTOR must be between 0.0 and 1.0".to_string()));
            }
        }

        Ok(config)
    }

    /// Validate configuration values
    pub fn validate(&self) -> Result<(), SoulError> {
        if !self.enabled {
            return Ok(());
        }

        if !(0.0..=1.0).contains(&self.default_trust) {
            return Err(SoulError::InvalidValue("default_trust must be between 0.0 and 1.0".to_string()));
        }

        if !(0.0..=1.0).contains(&self.base_decay_rate) {
            return Err(SoulError::InvalidValue("base_decay_rate must be between 0.0 and 1.0".to_string()));
        }

        if !(0.0..=1.0).contains(&self.prune_threshold) {
            return Err(SoulError::InvalidValue("prune_threshold must be between 0.0 and 1.0".to_string()));
        }

        if !(0.0..=1.0).contains(&self.empathy_threshold) {
            return Err(SoulError::InvalidValue("empathy_threshold must be between 0.0 and 1.0".to_string()));
        }

        if !(0.0..=1.0).contains(&self.trust_boost_factor) {
            return Err(SoulError::InvalidValue("trust_boost_factor must be between 0.0 and 1.0".to_string()));
        }

        if !(0.0..=1.0).contains(&self.trust_penalty_factor) {
            return Err(SoulError::InvalidValue("trust_penalty_factor must be between 0.0 and 1.0".to_string()));
        }

        if self.max_emotions_per_entity == 0 {
            return Err(SoulError::InvalidValue("max_emotions_per_entity must be greater than 0".to_string()));
        }

        if self.max_memories_per_entity == 0 {
            return Err(SoulError::InvalidValue("max_memories_per_entity must be greater than 0".to_string()));
        }

        Ok(())
    }
}