//! Backup creation and management

use crate::backup::BackupConfig;
use crate::db::get_db_path;
use crate::memory::MemoryLayer;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;
use tracing::{info, warn, error};
use uuid::Uuid;

/// Backup manifest containing metadata about a backup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupManifest {
    pub backup_id: Uuid,
    pub timestamp: chrono::DateTime<Utc>,
    pub database_size: u64,
    pub memory_indices_size: u64,
    pub total_size: u64,
    pub version: String,
    pub components: Vec<BackupComponent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupComponent {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub checksum: Option<String>,
}

/// Result of a backup operation
#[derive(Debug)]
pub struct BackupResult {
    pub manifest: BackupManifest,
    pub backup_path: PathBuf,
    pub duration: std::time::Duration,
    pub success: bool,
    pub error: Option<String>,
}

/// Manages backup operations
pub struct BackupManager {
    config: BackupConfig,
    #[allow(dead_code)]
    data_dir: PathBuf,
    memory_dir: PathBuf,
}

impl BackupManager {
    /// Create a new backup manager
    pub fn new(config: BackupConfig) -> Result<Self> {
        // Ensure backup directory exists
        if config.enabled {
            fs::create_dir_all(&config.backup_dir)
                .context("Failed to create backup directory")?;
        }
        
        // Determine data directories
        let current_dir = std::env::current_dir()?;
        let data_dir = current_dir.join("data");
        let memory_dir = data_dir.join("memory");
        
        Ok(Self {
            config,
            data_dir,
            memory_dir,
        })
    }
    
    /// Create a full system backup
    pub async fn create_backup(&self) -> Result<BackupResult> {
        if !self.config.enabled {
            return Ok(BackupResult {
                manifest: BackupManifest {
                    backup_id: Uuid::new_v4(),
                    timestamp: Utc::now(),
                    database_size: 0,
                    memory_indices_size: 0,
                    total_size: 0,
                    version: env!("CARGO_PKG_VERSION").to_string(),
                    components: vec![],
                },
                backup_path: PathBuf::new(),
                duration: std::time::Duration::ZERO,
                success: false,
                error: Some("Backups are disabled".to_string()),
            });
        }
        
        let start = std::time::Instant::now();
        let backup_id = Uuid::new_v4();
        let timestamp = Utc::now();
        
        // Create backup directory
        let backup_path = self.config.backup_dir.join(backup_id.to_string());
        fs::create_dir_all(&backup_path)
            .context("Failed to create backup directory")?;
        
        info!("Starting backup: {}", backup_id);
        
        let mut components = Vec::new();
        let mut total_size = 0u64;
        
        // 1. Backup database
        let db_result = self.backup_database(&backup_path).await;
        match db_result {
            Ok(component) => {
                total_size += component.size;
                components.push(component);
                info!("Database backed up successfully");
            }
            Err(e) => {
                error!("Database backup failed: {}", e);
                return Ok(BackupResult {
                    manifest: BackupManifest {
                        backup_id,
                        timestamp,
                        database_size: 0,
                        memory_indices_size: 0,
                        total_size: 0,
                        version: env!("CARGO_PKG_VERSION").to_string(),
                        components: vec![],
                    },
                    backup_path,
                    duration: start.elapsed(),
                    success: false,
                    error: Some(format!("Database backup failed: {}", e)),
                });
            }
        }
        
        // 2. Backup memory indices
        let memory_result = self.backup_memory_indices(&backup_path).await;
        let memory_size = match memory_result {
            Ok(mut memory_components) => {
                let size: u64 = memory_components.iter().map(|c| c.size).sum();
                components.append(&mut memory_components);
                info!("Memory indices backed up successfully");
                size
            }
            Err(e) => {
                warn!("Memory indices backup failed: {}", e);
                0
            }
        };
        total_size += memory_size;
        
        // 3. Create manifest
        let manifest = BackupManifest {
            backup_id,
            timestamp,
            database_size: components.iter()
                .find(|c| c.name == "database")
                .map(|c| c.size)
                .unwrap_or(0),
            memory_indices_size: memory_size,
            total_size,
            version: env!("CARGO_PKG_VERSION").to_string(),
            components: components.clone(),
        };
        
        // Save manifest
        let manifest_path = backup_path.join("manifest.json");
        let manifest_json = serde_json::to_string_pretty(&manifest)?;
        fs::write(&manifest_path, manifest_json)
            .context("Failed to write backup manifest")?;
        
        let duration = start.elapsed();
        info!("Backup completed: {} ({} bytes in {:?})", 
              backup_id, total_size, duration);
        
        // 4. Cleanup old backups
        if let Err(e) = self.cleanup_old_backups().await {
            warn!("Failed to cleanup old backups: {}", e);
        }
        
        Ok(BackupResult {
            manifest,
            backup_path,
            duration,
            success: true,
            error: None,
        })
    }
    
    /// Backup the SQLite database
    async fn backup_database(&self, backup_path: &Path) -> Result<BackupComponent> {
        let db_path = get_db_path()?;
        
        if !db_path.exists() {
            anyhow::bail!("Database file not found: {}", db_path.display());
        }
        
        let backup_db_path = backup_path.join("jamey.db");
        
        // Copy database file
        fs::copy(&db_path, &backup_db_path)
            .context("Failed to copy database file")?;
        
        let size = fs::metadata(&backup_db_path)?.len();
        
        Ok(BackupComponent {
            name: "database".to_string(),
            path: backup_db_path,
            size,
            checksum: None, // Could add checksum calculation if needed
        })
    }
    
    /// Backup memory indices (all 5 layers)
    async fn backup_memory_indices(&self, backup_path: &Path) -> Result<Vec<BackupComponent>> {
        let mut components = Vec::new();
        let backup_memory_dir = backup_path.join("memory");
        fs::create_dir_all(&backup_memory_dir)?;
        
        // Backup each memory layer
        for layer in [
            MemoryLayer::ShortTerm,
            MemoryLayer::LongTerm,
            MemoryLayer::Working,
            MemoryLayer::Episodic,
            MemoryLayer::Semantic,
        ] {
            let layer_dir = self.memory_dir.join(layer.as_str());
            
            if !layer_dir.exists() {
                continue; // Skip if layer doesn't exist yet
            }
            
            let backup_layer_dir = backup_memory_dir.join(layer.as_str());
            
            // Copy entire directory recursively
            copy_dir_all(&layer_dir, &backup_layer_dir)
                .with_context(|| format!("Failed to backup {} layer", layer.as_str()))?;
            
            let size = dir_size(&backup_layer_dir)?;
            
            components.push(BackupComponent {
                name: format!("memory_{}", layer.as_str()),
                path: backup_layer_dir,
                size,
                checksum: None,
            });
        }
        
        Ok(components)
    }
    
    /// Cleanup old backups based on retention policy
    async fn cleanup_old_backups(&self) -> Result<usize> {
        let mut deleted = 0;
        let retention_cutoff = Utc::now() - chrono::Duration::days(self.config.retention_days as i64);
        
        // Get all backup directories
        let entries = fs::read_dir(&self.config.backup_dir)?;
        let mut backups: Vec<(PathBuf, DateTime<Utc>)> = Vec::new();
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                // Try to parse as UUID and load manifest
                if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                    if Uuid::parse_str(dir_name).is_ok() {
                        let manifest_path = path.join("manifest.json");
                        if let Ok(manifest_json) = fs::read_to_string(&manifest_path) {
                            if let Ok(manifest) = serde_json::from_str::<BackupManifest>(&manifest_json) {
                                backups.push((path, manifest.timestamp));
                            }
                        }
                    }
                }
            }
        }
        
        // Sort by timestamp (oldest first)
        backups.sort_by_key(|(_, ts)| *ts);
        
        // Delete old backups
        for (path, timestamp) in &backups {
            if *timestamp < retention_cutoff {
                info!("Deleting old backup: {} (from {})", path.display(), timestamp);
                fs::remove_dir_all(path)?;
                deleted += 1;
            }
        }
        
        // Also enforce max_backups limit
        if backups.len() - deleted > self.config.max_backups {
            let to_delete = backups.len() - deleted - self.config.max_backups;
            for (path, timestamp) in backups.iter().take(to_delete) {
                info!("Deleting backup to enforce limit: {} (from {})", path.display(), timestamp);
                fs::remove_dir_all(path)?;
                deleted += 1;
            }
        }
        
        if deleted > 0 {
            info!("Cleaned up {} old backup(s)", deleted);
        }
        
        Ok(deleted)
    }
    
    /// List all available backups
    pub async fn list_backups(&self) -> Result<Vec<BackupManifest>> {
        let mut backups = Vec::new();
        
        if !self.config.backup_dir.exists() {
            return Ok(backups);
        }
        
        let entries = fs::read_dir(&self.config.backup_dir)?;
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                let manifest_path = path.join("manifest.json");
                if manifest_path.exists() {
                    if let Ok(manifest_json) = fs::read_to_string(&manifest_path) {
                        if let Ok(manifest) = serde_json::from_str::<BackupManifest>(&manifest_json) {
                            backups.push(manifest);
                        }
                    }
                }
            }
        }
        
        // Sort by timestamp (newest first)
        backups.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        
        Ok(backups)
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

/// Calculate total size of directory
fn dir_size(path: &Path) -> Result<u64> {
    let mut size = 0u64;
    
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                size += dir_size(&path)?;
            } else {
                size += fs::metadata(&path)?.len();
            }
        }
    } else {
        size = fs::metadata(path)?.len();
    }
    
    Ok(size)
}

