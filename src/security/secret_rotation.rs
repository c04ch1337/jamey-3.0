//! Secret Rotation Framework for Jamey 3.0
//!
//! Provides automated secret rotation for API keys, JWT secrets, and other sensitive credentials.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};
use uuid::Uuid;

/// Secret type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SecretType {
    ApiKey,
    JwtSecret,
    EncryptionKey,
    DatabasePassword,
    MqttPassword,
    Other(String),
}

/// Secret rotation policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationPolicy {
    /// Secret type
    pub secret_type: SecretType,
    /// Rotation interval in days
    pub rotation_interval_days: u32,
    /// Grace period in days (old secret remains valid)
    pub grace_period_days: u32,
    /// Warning days before expiration
    pub warning_days: u32,
    /// Enable automatic rotation
    pub auto_rotate: bool,
}

impl Default for RotationPolicy {
    fn default() -> Self {
        Self {
            secret_type: SecretType::ApiKey,
            rotation_interval_days: 90,
            grace_period_days: 7,
            warning_days: 14,
            auto_rotate: false,
        }
    }
}

/// Secret metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretMetadata {
    /// Secret identifier
    pub id: String,
    /// Secret type
    pub secret_type: SecretType,
    /// Current version
    pub current_version: u32,
    /// Created at
    pub created_at: DateTime<Utc>,
    /// Expires at
    pub expires_at: DateTime<Utc>,
    /// Last rotated at
    pub last_rotated_at: Option<DateTime<Utc>>,
    /// Next rotation due
    pub next_rotation_due: DateTime<Utc>,
    /// Rotation policy
    pub policy: RotationPolicy,
}

/// Secret rotation event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationEvent {
    /// Event ID
    pub id: String,
    /// Secret ID
    pub secret_id: String,
    /// Event type
    pub event_type: RotationEventType,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Success status
    pub success: bool,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Details
    pub details: String,
}

/// Rotation event type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RotationEventType {
    RotationScheduled,
    RotationStarted,
    RotationCompleted,
    RotationFailed,
    WarningSent,
    ExpirationImminent,
}

/// Secret rotation configuration
#[derive(Debug, Clone)]
pub struct SecretRotationConfig {
    /// Enable secret rotation
    pub enabled: bool,
    /// Check interval in seconds
    pub check_interval_secs: u64,
    /// Default rotation policies
    pub default_policies: HashMap<SecretType, RotationPolicy>,
}

impl Default for SecretRotationConfig {
    fn default() -> Self {
        let mut policies = HashMap::new();
        
        // API Key policy: 90 days, 7 day grace period
        policies.insert(
            SecretType::ApiKey,
            RotationPolicy {
                secret_type: SecretType::ApiKey,
                rotation_interval_days: 90,
                grace_period_days: 7,
                warning_days: 14,
                auto_rotate: false,
            },
        );
        
        // JWT Secret policy: 180 days, 14 day grace period
        policies.insert(
            SecretType::JwtSecret,
            RotationPolicy {
                secret_type: SecretType::JwtSecret,
                rotation_interval_days: 180,
                grace_period_days: 14,
                warning_days: 30,
                auto_rotate: false,
            },
        );
        
        // Encryption key policy: 365 days, 30 day grace period
        policies.insert(
            SecretType::EncryptionKey,
            RotationPolicy {
                secret_type: SecretType::EncryptionKey,
                rotation_interval_days: 365,
                grace_period_days: 30,
                warning_days: 60,
                auto_rotate: false,
            },
        );
        
        Self {
            enabled: true,
            check_interval_secs: 3600, // 1 hour
            default_policies: policies,
        }
    }
}

impl SecretRotationConfig {
    /// Create config from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();
        
        if let Ok(enabled) = std::env::var("SECRET_ROTATION_ENABLED") {
            config.enabled = enabled.parse().unwrap_or(true);
        }
        
        if let Ok(interval) = std::env::var("SECRET_ROTATION_CHECK_INTERVAL_SECS") {
            if let Ok(parsed) = interval.parse::<u64>() {
                config.check_interval_secs = parsed;
            }
        }
        
        // Configure API key rotation
        if let Ok(days) = std::env::var("SECRET_ROTATION_API_KEY_DAYS") {
            if let Ok(parsed) = days.parse::<u32>() {
                if let Some(policy) = config.default_policies.get_mut(&SecretType::ApiKey) {
                    policy.rotation_interval_days = parsed;
                }
            }
        }
        
        // Configure JWT secret rotation
        if let Ok(days) = std::env::var("SECRET_ROTATION_JWT_DAYS") {
            if let Ok(parsed) = days.parse::<u32>() {
                if let Some(policy) = config.default_policies.get_mut(&SecretType::JwtSecret) {
                    policy.rotation_interval_days = parsed;
                }
            }
        }
        
        // Enable auto-rotation
        if let Ok(enabled) = std::env::var("SECRET_ROTATION_AUTO_ROTATE") {
            let auto = enabled.parse().unwrap_or(false);
            for policy in config.default_policies.values_mut() {
                policy.auto_rotate = auto;
            }
        }
        
        config
    }
}

/// Secret rotation manager
#[derive(Clone)]
pub struct SecretRotationManager {
    config: SecretRotationConfig,
    secrets: Arc<Mutex<HashMap<String, SecretMetadata>>>,
    rotation_events: Arc<Mutex<Vec<RotationEvent>>>,
    rotation_callbacks: Arc<Mutex<HashMap<SecretType, Box<dyn Fn(&str) -> Result<String, String> + Send + Sync>>>>,
}

impl SecretRotationManager {
    /// Create new secret rotation manager
    pub fn new(config: SecretRotationConfig) -> Self {
        let manager = Self {
            config,
            secrets: Arc::new(Mutex::new(HashMap::new())),
            rotation_events: Arc::new(Mutex::new(Vec::new())),
            rotation_callbacks: Arc::new(Mutex::new(HashMap::new())),
        };
        
        // Start rotation check task
        if manager.config.enabled {
            let check_manager = manager.clone();
            tokio::spawn(async move {
                check_manager.rotation_check_task().await;
            });
        }
        
        manager
    }
    
    /// Register a secret for rotation tracking
    pub fn register_secret(
        &self,
        id: String,
        secret_type: SecretType,
        created_at: DateTime<Utc>,
    ) -> Result<(), String> {
        let policy = self.config.default_policies
            .get(&secret_type)
            .cloned()
            .unwrap_or_else(|| {
                warn!("No policy found for secret type {:?}, using defaults", secret_type);
                RotationPolicy {
                    secret_type: secret_type.clone(),
                    ..Default::default()
                }
            });
        
        let next_rotation = created_at + chrono::Duration::days(policy.rotation_interval_days as i64);
        let expires_at = next_rotation + chrono::Duration::days(policy.grace_period_days as i64);
        
        let metadata = SecretMetadata {
            id: id.clone(),
            secret_type,
            current_version: 1,
            created_at,
            expires_at,
            last_rotated_at: None,
            next_rotation_due: next_rotation,
            policy,
        };
        
        let mut secrets = self.secrets.lock().unwrap();
        secrets.insert(id, metadata);
        
        info!("Registered secret for rotation tracking");
        Ok(())
    }
    
    /// Register a rotation callback for a secret type
    pub fn register_rotation_callback<F>(&self, secret_type: SecretType, callback: F)
    where
        F: Fn(&str) -> Result<String, String> + Send + Sync + 'static,
    {
        let mut callbacks = self.rotation_callbacks.lock().unwrap();
        callbacks.insert(secret_type, Box::new(callback));
        info!("Registered rotation callback for secret type");
    }
    
    /// Rotate a secret
    pub async fn rotate_secret(&self, secret_id: &str) -> Result<String, String> {
        let mut secrets = self.secrets.lock().unwrap();
        let metadata = secrets.get_mut(secret_id)
            .ok_or_else(|| format!("Secret not found: {}", secret_id))?;
        
        // Record rotation start
        self.record_event(
            secret_id,
            RotationEventType::RotationStarted,
            true,
            None,
            "Rotation started".to_string(),
        );
        
        // Get rotation callback
        let callbacks = self.rotation_callbacks.lock().unwrap();
        let callback = callbacks.get(&metadata.secret_type)
            .ok_or_else(|| format!("No rotation callback for secret type: {:?}", metadata.secret_type))?;
        
        // Perform rotation
        let new_secret = match callback(secret_id) {
            Ok(secret) => secret,
            Err(e) => {
                self.record_event(
                    secret_id,
                    RotationEventType::RotationFailed,
                    false,
                    Some(e.clone()),
                    "Rotation failed".to_string(),
                );
                return Err(e);
            }
        };
        
        // Update metadata
        let now = Utc::now();
        metadata.current_version += 1;
        metadata.last_rotated_at = Some(now);
        metadata.next_rotation_due = now + chrono::Duration::days(metadata.policy.rotation_interval_days as i64);
        metadata.expires_at = metadata.next_rotation_due + chrono::Duration::days(metadata.policy.grace_period_days as i64);
        
        // Record success
        self.record_event(
            secret_id,
            RotationEventType::RotationCompleted,
            true,
            None,
            format!("Rotated to version {}", metadata.current_version),
        );
        
        info!("Secret rotated successfully: {} (version {})", secret_id, metadata.current_version);
        Ok(new_secret)
    }
    
    /// Check if secret needs rotation
    pub fn needs_rotation(&self, secret_id: &str) -> bool {
        let secrets = self.secrets.lock().unwrap();
        if let Some(metadata) = secrets.get(secret_id) {
            let now = Utc::now();
            now >= metadata.next_rotation_due
        } else {
            false
        }
    }
    
    /// Check if secret is expiring soon
    pub fn is_expiring_soon(&self, secret_id: &str) -> bool {
        let secrets = self.secrets.lock().unwrap();
        if let Some(metadata) = secrets.get(secret_id) {
            let now = Utc::now();
            let warning_date = metadata.expires_at - chrono::Duration::days(metadata.policy.warning_days as i64);
            now >= warning_date && now < metadata.expires_at
        } else {
            false
        }
    }
    
    /// Get secrets needing rotation
    pub fn get_secrets_needing_rotation(&self) -> Vec<String> {
        let secrets = self.secrets.lock().unwrap();
        secrets.iter()
            .filter(|(_, metadata)| {
                let now = Utc::now();
                now >= metadata.next_rotation_due
            })
            .map(|(id, _)| id.clone())
            .collect()
    }
    
    /// Get secrets expiring soon
    pub fn get_secrets_expiring_soon(&self) -> Vec<String> {
        let secrets = self.secrets.lock().unwrap();
        secrets.iter()
            .filter(|(_, metadata)| {
                let now = Utc::now();
                let warning_date = metadata.expires_at - chrono::Duration::days(metadata.policy.warning_days as i64);
                now >= warning_date && now < metadata.expires_at
            })
            .map(|(id, _)| id.clone())
            .collect()
    }
    
    /// Record rotation event
    fn record_event(
        &self,
        secret_id: &str,
        event_type: RotationEventType,
        success: bool,
        error: Option<String>,
        details: String,
    ) {
        let event_type_clone = event_type.clone();
        let event = RotationEvent {
            id: Uuid::new_v4().to_string(),
            secret_id: secret_id.to_string(),
            event_type: event_type_clone.clone(),
            timestamp: Utc::now(),
            success,
            error,
            details,
        };
        
        let mut events = self.rotation_events.lock().unwrap();
        events.push(event.clone());
        
        // Keep only last 1000 events
        let len = events.len();
        if len > 1000 {
            events.drain(0..len - 1000);
        }
        
        match event_type {
            RotationEventType::RotationFailed => {
                error!("Secret rotation failed: {} - {}", secret_id, event.details);
            }
            RotationEventType::RotationCompleted => {
                info!("Secret rotation completed: {} - {}", secret_id, event.details);
            }
            RotationEventType::WarningSent => {
                warn!("Secret rotation warning: {} - {}", secret_id, event.details);
            }
            _ => {
                info!("Secret rotation event: {} - {}", secret_id, event.details);
            }
        }
    }
    
    /// Rotation check task
    async fn rotation_check_task(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(self.config.check_interval_secs));
        
        loop {
            interval.tick().await;
            
            if !self.config.enabled {
                continue;
            }
            
            // Check for secrets needing rotation
            let secrets_to_rotate = self.get_secrets_needing_rotation();
            for secret_id in secrets_to_rotate {
                let metadata = {
                    let secrets = self.secrets.lock().unwrap();
                    secrets.get(&secret_id).cloned()
                };
                
                if let Some(metadata) = metadata {
                    if metadata.policy.auto_rotate {
                        // Auto-rotate
                        if let Err(e) = self.rotate_secret(&secret_id).await {
                            error!("Failed to auto-rotate secret {}: {}", secret_id, e);
                        }
                    } else {
                        // Schedule rotation
                        self.record_event(
                            &secret_id,
                            RotationEventType::RotationScheduled,
                            true,
                            None,
                            format!("Rotation scheduled for {}", metadata.next_rotation_due),
                        );
                    }
                }
            }
            
            // Check for secrets expiring soon
            let expiring_secrets = self.get_secrets_expiring_soon();
            for secret_id in expiring_secrets {
                self.record_event(
                    &secret_id,
                    RotationEventType::WarningSent,
                    true,
                    None,
                    "Secret expiring soon - rotation recommended".to_string(),
                );
            }
        }
    }
    
    /// Get rotation events
    pub fn get_rotation_events(&self, limit: usize) -> Vec<RotationEvent> {
        let events = self.rotation_events.lock().unwrap();
        events.iter().rev().take(limit).cloned().collect()
    }
    
    /// Get secret metadata
    pub fn get_secret_metadata(&self, secret_id: &str) -> Option<SecretMetadata> {
        let secrets = self.secrets.lock().unwrap();
        secrets.get(secret_id).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rotation_policy_default() {
        let policy = RotationPolicy::default();
        assert_eq!(policy.rotation_interval_days, 90);
        assert_eq!(policy.grace_period_days, 7);
    }
    
    #[test]
    fn test_secret_registration() {
        let config = SecretRotationConfig::default();
        let manager = SecretRotationManager::new(config);
        
        let result = manager.register_secret(
            "test-secret".to_string(),
            SecretType::ApiKey,
            Utc::now(),
        );
        
        assert!(result.is_ok());
        assert!(manager.get_secret_metadata("test-secret").is_some());
    }
    
    #[test]
    fn test_needs_rotation() {
        let config = SecretRotationConfig::default();
        let manager = SecretRotationManager::new(config);
        
        // Register secret with past rotation date
        let past_date = Utc::now() - chrono::Duration::days(100);
        manager.register_secret(
            "old-secret".to_string(),
            SecretType::ApiKey,
            past_date,
        ).unwrap();
        
        assert!(manager.needs_rotation("old-secret"));
    }
}

