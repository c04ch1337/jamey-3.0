//! API Key Manager
//! 
//! Manages API keys with rotation support, expiration, and per-key rate limiting.

use sha2::{Sha256, Digest};
use sqlx::SqlitePool;
use std::sync::Arc;
use tracing::{info, warn};
use chrono::Utc;
use uuid::Uuid;

/// API Key Manager for handling key validation and rotation
pub struct ApiKeyManager {
    pool: Arc<SqlitePool>,
}

impl ApiKeyManager {
    /// Create a new API Key Manager
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }

    /// Hash an API key using SHA-256
    fn hash_key(key: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Validate an API key
    pub async fn validate_key(&self, key: &str) -> Result<Option<ApiKeyInfo>, sqlx::Error> {
        let hash = Self::hash_key(key);
        
        let result = sqlx::query_as!(
            ApiKeyInfo,
            r#"
            SELECT
                id,
                key_hash,
                name,
                created_at,
                expires_at,
                revoked_at,
                last_used_at,
                rate_limit_per_minute
            FROM api_keys
            WHERE key_hash = ?
            AND (expires_at IS NULL OR expires_at > datetime('now'))
            AND revoked_at IS NULL
            "#,
            hash
        )
        .fetch_optional(self.pool.as_ref())
        .await?;

        if let Some(ref key_info) = result {
            if let Some(id) = key_info.id {
                // Update last_used_at
                sqlx::query!(
                    "UPDATE api_keys SET last_used_at = datetime('now') WHERE id = ?",
                    id
                )
                .execute(self.pool.as_ref())
                .await?;
            } else {
                warn!("ApiKeyInfo.id is NULL for key_hash {}", hash);
            }
        }

        Ok(result)
    }

    /// Create a new API key
    pub async fn create_key(
        &self,
        name: &str,
        expires_at: Option<chrono::DateTime<Utc>>,
        rate_limit_per_minute: Option<i64>,
    ) -> Result<String, sqlx::Error> {
        // Generate a secure random key
        let key = format!("jamey_{}", uuid::Uuid::new_v4());
        let hash = Self::hash_key(&key);

        let expires_str = expires_at.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string());
        let rate_limit = rate_limit_per_minute.unwrap_or(60);

        sqlx::query!(
            r#"
            INSERT INTO api_keys (key_hash, name, created_at, expires_at, rate_limit_per_minute)
            VALUES (?, ?, datetime('now'), ?, ?)
            "#,
            hash,
            name,
            expires_str,
            rate_limit
        )
        .execute(self.pool.as_ref())
        .await?;

        info!("Created new API key: {}", name);
        Ok(key)
    }

    /// Rotate an API key (revoke old, create new) with grace period
    pub async fn rotate_key(
        &self,
        old_key: &str,
        new_key_name: &str,
        grace_period_days: Option<u32>,
    ) -> Result<String, sqlx::Error> {
        let old_hash = Self::hash_key(old_key);

        // Get old key info
        let old_key_info = sqlx::query!(
            "SELECT id, rate_limit_per_minute, created_at FROM api_keys WHERE key_hash = ?",
            old_hash
        )
        .fetch_optional(self.pool.as_ref())
        .await?;

        let old_key_id = old_key_info
            .as_ref()
            .ok_or_else(|| sqlx::Error::RowNotFound)?
            .id
            .expect("api_keys.id must not be NULL");

        let rate_limit = old_key_info
            .map(|k| k.rate_limit_per_minute)
            .flatten()
            .unwrap_or(60);

        // Create new key first
        let new_key = self.create_key(new_key_name, None, Some(rate_limit)).await?;

        // Set grace period expiration for old key (default 7 days)
        let grace_days = grace_period_days.unwrap_or(7);
        let grace_expiry = Utc::now() + chrono::Duration::days(grace_days as i64);
        let grace_expiry_str = grace_expiry.format("%Y-%m-%d %H:%M:%S").to_string();

        // Update old key with grace period instead of immediate revocation
        sqlx::query!(
            "UPDATE api_keys SET expires_at = ?, revoked_at = NULL WHERE key_hash = ?",
            grace_expiry_str,
            old_hash
        )
        .execute(self.pool.as_ref())
        .await?;

        // Log rotation event
        self.log_rotation_event(old_key_id, &old_hash, &new_key_name, grace_days).await?;

        info!("Rotated API key: {} -> {} (grace period: {} days)", old_hash, new_key_name, grace_days);
        Ok(new_key)
    }

    /// Log rotation event for audit trail
    async fn log_rotation_event(
        &self,
        key_id: i64,
        old_hash: &str,
        new_key_name: &str,
        grace_period_days: u32,
    ) -> Result<(), sqlx::Error> {
        // Create rotation log entry
        let grace_period_days_i64 = grace_period_days as i64;
        sqlx::query!(
            r#"
            INSERT INTO api_key_rotations (key_id, old_key_hash, new_key_name, grace_period_days, rotated_at)
            VALUES (?, ?, ?, ?, datetime('now'))
            "#,
            key_id,
            old_hash,
            new_key_name,
            grace_period_days_i64
        )
        .execute(self.pool.as_ref())
        .await?;

        Ok(())
    }

    /// Get rotation history for a key
    pub async fn get_rotation_history(&self, key_hash: &str) -> Result<Vec<RotationHistory>, sqlx::Error> {
        let key_id = sqlx::query!(
            "SELECT id FROM api_keys WHERE key_hash = ?",
            key_hash
        )
        .fetch_optional(self.pool.as_ref())
        .await?
        .map(|k| k.id)
        .flatten()
        .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let rotations = sqlx::query_as!(
            RotationHistory,
            r#"
            SELECT 
                id,
                key_id,
                old_key_hash,
                new_key_name,
                grace_period_days,
                rotated_at
            FROM api_key_rotations
            WHERE key_id = ?
            ORDER BY rotated_at DESC
            "#,
            key_id
        )
        .fetch_all(self.pool.as_ref())
        .await?;

        Ok(rotations)
    }

    /// Revoke an API key
    pub async fn revoke_key(&self, key: &str) -> Result<(), sqlx::Error> {
        let hash = Self::hash_key(key);

        sqlx::query!(
            "UPDATE api_keys SET revoked_at = datetime('now') WHERE key_hash = ?",
            hash
        )
        .execute(self.pool.as_ref())
        .await?;

        info!("Revoked API key: {}", hash);
        Ok(())
    }

    /// Get rate limit for a key
    pub async fn get_rate_limit(&self, key: &str) -> Result<Option<i64>, sqlx::Error> {
        let hash = Self::hash_key(key);

        let result = sqlx::query!(
            "SELECT rate_limit_per_minute FROM api_keys WHERE key_hash = ? AND revoked_at IS NULL",
            hash
        )
        .fetch_optional(self.pool.as_ref())
        .await?;

        Ok(result.and_then(|r| r.rate_limit_per_minute))
    }
}

/// API Key information
#[derive(Debug)]
pub struct ApiKeyInfo {
    pub id: Option<i64>,
    pub key_hash: String,
    pub name: String,
    pub created_at: String,
    pub expires_at: Option<String>,
    pub revoked_at: Option<String>,
    pub last_used_at: Option<String>,
    pub rate_limit_per_minute: Option<i64>,
}

/// Rotation history entry
#[derive(Debug)]
pub struct RotationHistory {
    pub id: Option<i64>,
    pub key_id: Option<i64>,
    pub old_key_hash: String,
    pub new_key_name: String,
    pub grace_period_days: i64,
    pub rotated_at: String,
}

