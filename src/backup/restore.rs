//! Backup restore functionality

use crate::backup::{BackupConfig, BackupManifest};
use crate::db::get_db_path;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::fs;
use tracing::{info, warn, error};
use uuid::Uuid;

/// Result of a restore operation
#[derive(Debug)]
pub struct RestoreResult {
    pub backup_id: Uuid,
    pub restored_components: Vec<String>,
    pub duration: std::time::Duration,
    pub success: bool,
    pub error: Option<String>,
}

/// Manages restore operations
pub struct RestoreManager {
    config: BackupConfig,
    data_dir: PathBuf,
    memory_dir: PathBuf,
}

impl RestoreManager {
    /// Create a new restore manager
    pub fn new(config: BackupConfig) -> Result<Self> {
        let current_dir = std::env::current_dir()?;
        let data_dir = current_dir.join("data");
        let memory_dir = data_dir.join("memory");
        
        Ok(Self {
            config,
            data_dir,
            memory_dir,
        })
    }
    
    /// Restore from a backup
    pub async fn restore_backup(&self, backup_id: Uuid) -> Result<RestoreResult> {
        let start = std::time::Instant::now();
        let backup_path = self.config.backup_dir.join(backup_id.to_string());
        
        if !backup_path.exists() {
            anyhow::bail!("Backup not found: {}", backup_id);
        }
        
        // Load manifest
        let manifest_path = backup_path.join("manifest.json");
        let manifest_json = fs::read_to_string(&manifest_path)
            .context("Failed to read backup manifest")?;
        let manifest: BackupManifest = serde_json::from_str(&manifest_json)
            .context("Failed to parse backup manifest")?;
        
        info!("Restoring backup: {} (from {})", backup_id, manifest.timestamp);
        
        let mut restored_components = Vec::new();
        
        // 1. Restore database
        match self.restore_database(&backup_path).await {
            Ok(_) => {
                restored_components.push("database".to_string());
                info!("Database restored successfully");
            }
            Err(e) => {
                error!("Database restore failed: {}", e);
                return Ok(RestoreResult {
                    backup_id,
                    restored_components,
                    duration: start.elapsed(),
                    success: false,
                    error: Some(format!("Database restore failed: {}", e)),
                });
            }
        }
        
        // 2. Restore memory indices
        match self.restore_memory_indices(&backup_path).await {
            Ok(_) => {
                restored_components.push("memory_indices".to_string());
                info!("Memory indices restored successfully");
            }
            Err(e) => {
                warn!("Memory indices restore failed: {}", e);
                // Don't fail the entire restore if memory indices fail
            }
        }
        
        let duration = start.elapsed();
        info!("Restore completed: {} (restored {} components in {:?})",
              backup_id, restored_components.len(), duration);
        
        Ok(RestoreResult {
            backup_id,
            restored_components,
            duration,
            success: true,
            error: None,
        })
    }
    
    /// Restore the database
    async fn restore_database(&self, backup_path: &Path) -> Result<()> {
        let backup_db_path = backup_path.join("jamey.db");
        
        if !backup_db_path.exists() {
            anyhow::bail!("Backup database file not found");
        }
        
        let db_path = get_db_path()?;
        
        // Ensure data directory exists
        if let Some(parent) = db_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // Copy backup database to data directory
        fs::copy(&backup_db_path, &db_path)
            .context("Failed to copy database file")?;
        
        info!("Database restored to: {}", db_path.display());
        Ok(())
    }
    
    /// Restore memory indices
    async fn restore_memory_indices(&self, backup_path: &Path) -> Result<()> {
        let backup_memory_dir = backup_path.join("memory");
        
        if !backup_memory_dir.exists() {
            warn!("No memory indices in backup, skipping restore");
            return Ok(());
        }
        
        // Ensure memory directory exists
        fs::create_dir_all(&self.memory_dir)?;
        
        // Restore each layer
        for entry in fs::read_dir(&backup_memory_dir)? {
            let entry = entry?;
            let backup_layer_dir = entry.path();
            
            if backup_layer_dir.is_dir() {
                if let Some(layer_name) = backup_layer_dir.file_name() {
                    let restore_layer_dir = self.memory_dir.join(layer_name);
                    
                    // Remove existing layer directory if it exists
                    if restore_layer_dir.exists() {
                        fs::remove_dir_all(&restore_layer_dir)?;
                    }
                    
                    // Copy layer directory
                    copy_dir_all(&backup_layer_dir, &restore_layer_dir)
                        .with_context(|| format!("Failed to restore layer: {:?}", layer_name))?;
                    
                    info!("Restored memory layer: {:?}", layer_name);
                }
            }
        }
        
        Ok(())
    }
    
    /// Verify backup integrity
    pub async fn verify_backup(&self, backup_id: Uuid) -> Result<bool> {
        let backup_path = self.config.backup_dir.join(backup_id.to_string());
        
        if !backup_path.exists() {
            return Ok(false);
        }
        
        // Check manifest exists
        let manifest_path = backup_path.join("manifest.json");
        if !manifest_path.exists() {
            return Ok(false);
        }
        
        // Try to parse manifest
        let manifest_json = fs::read_to_string(&manifest_path)?;
        let _manifest: BackupManifest = serde_json::from_str(&manifest_json)?;
        
        // Check database exists
        let db_path = backup_path.join("jamey.db");
        if !db_path.exists() {
            return Ok(false);
        }
        
        Ok(true)
    }
}

/// Copy directory recursively
fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name();
        let dst_path = dst.join(&file_name);
        
        if path.is_dir() {
            copy_dir_all(&path, &dst_path)?;
        } else {
            fs::copy(&path, &dst_path)?;
        }
    }
    
    Ok(())
}

