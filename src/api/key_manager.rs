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
            // Update last_used_at
            sqlx::query!(
                "UPDATE api_keys SET last_used_at = datetime('now') WHERE id = ?",
                key_info.id
            )
            .execute(self.pool.as_ref())
            .await?;
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

    /// Rotate an API key (revoke old, create new)
    pub async fn rotate_key(
        &self,
        old_key: &str,
        new_key_name: &str,
    ) -> Result<String, sqlx::Error> {
        let old_hash = Self::hash_key(old_key);

        // Revoke old key
        sqlx::query!(
            "UPDATE api_keys SET revoked_at = datetime('now') WHERE key_hash = ?",
            old_hash
        )
        .execute(self.pool.as_ref())
        .await?;

        // Get rate limit from old key
        let old_key_info = sqlx::query!(
            "SELECT rate_limit_per_minute FROM api_keys WHERE key_hash = ?",
            old_hash
        )
        .fetch_optional(self.pool.as_ref())
        .await?;

        let rate_limit = old_key_info
            .map(|k| k.rate_limit_per_minute)
            .flatten()
            .unwrap_or(60);

        // Create new key
        let new_key = self.create_key(new_key_name, None, Some(rate_limit)).await?;

        info!("Rotated API key: {} -> {}", old_hash, new_key_name);
        Ok(new_key)
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
    pub id: i64,
    pub key_hash: String,
    pub name: String,
    pub created_at: String,
    pub expires_at: Option<String>,
    pub revoked_at: Option<String>,
    pub last_used_at: Option<String>,
    pub rate_limit_per_minute: Option<i64>,
}

