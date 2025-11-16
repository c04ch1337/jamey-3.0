use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use tracing::{info, warn, error};
use uuid::Uuid;

use crate::phoenix::vault::PhoenixVault;
use crate::metrics::{self, increment_counter, record_histogram};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    pub backup_dir: PathBuf,
    pub max_backups: usize,
    pub encryption_key: [u8; 32],
    pub components: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupMetadata {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub components: Vec<String>,
    pub size_bytes: u64,
    pub duration_secs: u64,
    pub status: BackupStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackupStatus {
    Success,
    Partial { failed_components: Vec<String> },
    Failed { reason: String },
}

pub struct BackupManager {
    config: BackupConfig,
    vault: Arc<PhoenixVault>,
    metadata_store: Arc<RwLock<Vec<BackupMetadata>>>,
}

impl BackupManager {
    pub fn new(config: BackupConfig) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let vault = PhoenixVault::new(
            config.backup_dir.clone(),
            config.encryption_key,
            true,
            config.max_backups,
        )?;

        Ok(Self {
            config,
            vault: Arc::new(vault),
            metadata_store: Arc::new(RwLock::new(Vec::new())),
        })
    }

    pub async fn create_backup(&self, description: Option<String>) -> Result<Uuid, Box<dyn std::error::Error + Send + Sync>> {
        let backup_id = Uuid::new_v4();
        let start_time = Utc::now();
        
        info!("Starting backup {}", backup_id);
        increment_counter!("backup_operations_total", "operation" => "start");

        let backup_path = self.config.backup_dir.join(backup_id.to_string());
        tokio::fs::create_dir_all(&backup_path).await?;

        let mut failed_components = Vec::new();
        let mut total_size = 0;

        // Backup database
        if self.config.components.contains(&"database".to_string()) {
            match self.vault.backup_database(&backup_path, &mut self.vault.create_manifest()).await {
                Ok(()) => {
                    info!("Database backup successful");
                    total_size += tokio::fs::metadata(backup_path.join("jamey.db.enc")).await?.len();
                }
                Err(e) => {
                    error!("Database backup failed: {}", e);
                    failed_components.push("database".to_string());
                }
            }
        }

        // Backup memory indices
        if self.config.components.contains(&"memory".to_string()) {
            match self.vault.backup_memory_indices(&backup_path, &mut self.vault.create_manifest()).await {
                Ok(()) => {
                    info!("Memory indices backup successful");
                    let memory_size = walkdir::WalkDir::new(backup_path.join("memory"))
                        .into_iter()
                        .filter_map(|e| e.ok())
                        .filter_map(|e| e.metadata().ok())
                        .filter(|m| m.is_file())
                        .map(|m| m.len())
                        .sum::<u64>();
                    total_size += memory_size;
                }
                Err(e) => {
                    error!("Memory indices backup failed: {}", e);
                    failed_components.push("memory".to_string());
                }
            }
        }

        // Calculate duration
        let duration = Utc::now().signed_duration_since(start_time).num_seconds() as u64;
        record_histogram!("backup_duration_seconds", duration as f64);
        record_histogram!("backup_size_bytes", total_size as f64);

        // Create metadata
        let status = if failed_components.is_empty() {
            BackupStatus::Success
        } else if failed_components.len() < self.config.components.len() {
            BackupStatus::Partial { failed_components: failed_components.clone() }
        } else {
            BackupStatus::Failed { reason: "All components failed".to_string() }
        };

        let metadata = BackupMetadata {
            id: backup_id,
            timestamp: start_time,
            components: self.config.components.clone(),
            size_bytes: total_size,
            duration_secs: duration,
            status,
        };

        // Store metadata
        self.metadata_store.write().await.push(metadata.clone());

        // Cleanup old backups
        self.cleanup_old_backups().await?;

        match status {
            BackupStatus::Success => {
                info!("Backup {} completed successfully", backup_id);
                increment_counter!("backup_operations_total", "operation" => "success");
            }
            BackupStatus::Partial { .. } => {
                warn!("Backup {} partially completed with failures: {:?}", backup_id, failed_components);
                increment_counter!("backup_operations_total", "operation" => "partial");
            }
            BackupStatus::Failed { .. } => {
                error!("Backup {} failed completely", backup_id);
                increment_counter!("backup_operations_total", "operation" => "failure");
            }
        }

        Ok(backup_id)
    }

    async fn cleanup_old_backups(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut metadata = self.metadata_store.write().await;
        
        // Sort by timestamp (oldest first)
        metadata.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        // Remove oldest backups beyond max_backups
        while metadata.len() > self.config.max_backups {
            if let Some(oldest) = metadata.first() {
                let backup_id = oldest.id;
                let backup_path = self.config.backup_dir.join(backup_id.to_string());
                
                if let Err(e) = tokio::fs::remove_dir_all(&backup_path).await {
                    warn!("Failed to delete old backup {}: {}", backup_id, e);
                } else {
                    info!("Deleted old backup {}", backup_id);
                }
                
                metadata.remove(0);
            }
        }

        Ok(())
    }

    pub async fn list_backups(&self) -> Vec<BackupMetadata> {
        self.metadata_store.read().await.clone()
    }

    pub async fn get_backup(&self, id: Uuid) -> Option<BackupMetadata> {
        self.metadata_store.read().await
            .iter()
            .find(|b| b.id == id)
            .cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_backup_manager() {
        let temp_dir = tempdir().unwrap();
        let config = BackupConfig {
            backup_dir: temp_dir.path().to_path_buf(),
            max_backups: 2,
            encryption_key: [0u8; 32],
            components: vec!["database".to_string(), "memory".to_string()],
        };

        let manager = BackupManager::new(config).unwrap();

        // Create test backups
        let backup1 = manager.create_backup(None).await.unwrap();
        let backup2 = manager.create_backup(None).await.unwrap();
        let backup3 = manager.create_backup(None).await.unwrap();

        // Verify only latest backups are kept
        let backups = manager.list_backups().await;
        assert_eq!(backups.len(), 2);
        assert!(!backups.iter().any(|b| b.id == backup1));
        assert!(backups.iter().any(|b| b.id == backup2));
        assert!(backups.iter().any(|b| b.id == backup3));
    }
}
