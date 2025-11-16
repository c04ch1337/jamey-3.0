use std::path::PathBuf;
use std::time::Instant;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::fs;
use tracing::{info, error};

use super::encryption::Encryptor;
use crate::metrics;

#[derive(Debug, Error)]
pub enum PhoenixError {
    #[error("Phoenix Vault is disabled")]
    Disabled,

    #[error("Backup not found: {0}")]
    BackupNotFound(Uuid),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Invalid backup manifest: {0}")]
    InvalidManifest(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

/// Metadata about a backup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupManifest {
    /// Unique backup identifier
    pub backup_id: Uuid,

    /// When the backup was created
    pub timestamp: DateTime<Utc>,

    /// List of backed up components
    pub components: Vec<String>,

    /// Total size in bytes
    pub size_bytes: u64,

    /// Optional description
    pub description: Option<String>,
}

impl BackupManifest {
    fn new(backup_id: Uuid, timestamp: DateTime<Utc>) -> Self {
        Self {
            backup_id,
            timestamp,
            components: Vec::new(),
            size_bytes: 0,
            description: None,
        }
    }
}

/// Phoenix Vault for backup and recovery
pub struct PhoenixVault {
    /// Backup directory
    backup_dir: PathBuf,

    /// Encryption key (32 bytes for AES-256)
    #[allow(dead_code)]
    encryption_key: [u8; 32],

    /// Whether vault is enabled
    pub(crate) enabled: bool,

    /// Maximum backups to retain
    pub(crate) max_backups: usize,

    /// Encryptor instance
    pub(crate) encryptor: Encryptor,
}

impl PhoenixVault {
    /// Create a new Phoenix Vault
    pub fn new(
        backup_dir: PathBuf,
        #[allow(dead_code)]
    encryption_key: [u8; 32],
        enabled: bool,
        max_backups: usize,
    ) -> anyhow::Result<Self> {
        if enabled {
            // Use synchronous filesystem operation here since this constructor is not async.
            // Async filesystem operations are used in the async backup/restore methods.
            std::fs::create_dir_all(&backup_dir)?;
        }

        Ok(Self {
            backup_dir,
            encryption_key,
            enabled,
            max_backups,
            encryptor: Encryptor::new(&encryption_key),
        })
    }

    /// Create a full system backup
    pub async fn create_backup(&self) -> Result<BackupManifest, PhoenixError> {
        if !self.enabled {
            return Err(PhoenixError::Disabled);
        }

        let start = Instant::now();
        let timestamp = Utc::now();
        let backup_id = Uuid::new_v4();
        
        info!("Starting backup {}", backup_id);

        // Create backup directory
        let backup_path = self.backup_dir.join(backup_id.to_string());
        fs::create_dir_all(&backup_path).await?;

        // Initialize manifest
        let mut manifest = BackupManifest::new(backup_id, timestamp);

        // 1. Backup SQLite database
        self.backup_database(&backup_path, &mut manifest).await?;

        // 2. Backup Tantivy indices
        self.backup_memory_indices(&backup_path, &mut manifest).await?;

        // 3. Save manifest
        self.save_manifest(&backup_path, &manifest).await?;

        // 4. Cleanup old backups
        self.cleanup_old_backups().await?;

        let duration = start.elapsed();
        
        // Record metrics
        metrics::record_backup_operation(
            "success",
            "vault",
            Some(duration),
            Some(manifest.size_bytes),
        );

        info!("Backup {} completed successfully ({} bytes, {}s)", 
            backup_id, manifest.size_bytes, duration.as_secs());
        Ok(manifest)
    }

    /// Restore from a backup
    pub async fn restore_backup(&self, backup_id: Uuid) -> Result<(), PhoenixError> {
        if !self.enabled {
            return Err(PhoenixError::Disabled);
        }

        let backup_path = self.backup_dir.join(backup_id.to_string());
        if !backup_path.exists() {
            return Err(PhoenixError::BackupNotFound(backup_id));
        }

        info!("Starting restore from backup {}", backup_id);

        // Load manifest
        let manifest = self.load_manifest(&backup_path).await?;

        // Restore components in reverse order of criticality
        // 1. Database first (most critical)
        self.restore_database(&backup_path).await?;

        // 2. Memory indices
        self.restore_memory_indices(&backup_path).await?;

        // 3. Verify restoration
        self.verify_restoration(&manifest).await?;

        info!("Restore from backup {} completed successfully", backup_id);
        Ok(())
    }

    /// List available backups
    pub async fn list_backups(&self) -> Result<Vec<BackupManifest>, PhoenixError> {
        if !self.enabled {
            return Ok(Vec::new());
        }

        let mut backups = Vec::new();
        let mut entries = fs::read_dir(&self.backup_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            if entry.file_type().await?.is_dir() {
                if let Ok(backup_id) = Uuid::parse_str(&entry.file_name().to_string_lossy()) {
                    let backup_path = entry.path();
                    match self.load_manifest(&backup_path).await {
                        Ok(manifest) => backups.push(manifest),
                        Err(e) => error!("Failed to load manifest for backup {}: {}", backup_id, e),
                    }
                }
            }
        }

        // Sort by timestamp (newest first)
        backups.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        Ok(backups)
    }

    /// Delete a backup
    pub async fn delete_backup(&self, backup_id: Uuid) -> Result<(), PhoenixError> {
        if !self.enabled {
            return Err(PhoenixError::Disabled);
        }

        let backup_path = self.backup_dir.join(backup_id.to_string());
        if !backup_path.exists() {
            return Err(PhoenixError::BackupNotFound(backup_id));
        }

        fs::remove_dir_all(backup_path).await?;
        info!("Deleted backup {}", backup_id);
        Ok(())
    }

    // Helper methods are implemented for PhoenixVault in:
    // - phoenix::backup (database + memory index backup + manifest IO + cleanup)
    // - phoenix::restore (database + memory index restore + verification)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_vault_creation() {
        let dir = tempdir().unwrap();
        let key = [0u8; 32];
        
        let vault = PhoenixVault::new(
            dir.path().to_path_buf(),
            key,
            true,
            5
        ).unwrap();

        assert!(vault.enabled);
        assert_eq!(vault.max_backups, 5);
        assert!(dir.path().exists());
    }

    #[tokio::test]
    async fn test_disabled_vault() {
        let dir = tempdir().unwrap();
        let key = [0u8; 32];
        
        let vault = PhoenixVault::new(
            dir.path().to_path_buf(),
            key,
            false,
            5
        ).unwrap();

        match vault.create_backup().await {
            Err(PhoenixError::Disabled) => (),
            _ => panic!("Expected Disabled error"),
        }
    }
}