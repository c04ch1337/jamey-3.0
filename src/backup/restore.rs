use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tracing::{info, warn, error};
use uuid::Uuid;
use serde::{Serialize, Deserialize};

use crate::metrics::{self, increment_counter, record_histogram};
use crate::phoenix::vault::PhoenixVault;
use super::{BackupMetadata, BackupStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreOptions {
    pub components: Vec<String>,
    pub force: bool,
    pub verify_integrity: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreResult {
    pub backup_id: Uuid,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub restored_components: Vec<String>,
    pub failed_components: Vec<String>,
    pub duration_secs: u64,
}

pub struct RestoreManager {
    vault: Arc<PhoenixVault>,
    backup_dir: PathBuf,
}

impl RestoreManager {
    pub fn new(vault: Arc<PhoenixVault>, backup_dir: PathBuf) -> Self {
        Self {
            vault,
            backup_dir,
        }
    }

    pub async fn restore_from_backup(
        &self,
        backup_id: Uuid,
        options: RestoreOptions,
    ) -> Result<RestoreResult, Box<dyn std::error::Error + Send + Sync>> {
        let start_time = chrono::Utc::now();
        info!("Starting restore from backup {}", backup_id);
        increment_counter!("restore_operations_total", "operation" => "start");

        let backup_path = self.backup_dir.join(backup_id.to_string());
        if !backup_path.exists() {
            return Err(format!("Backup {} not found", backup_id).into());
        }

        // Verify backup integrity if requested
        if options.verify_integrity {
            self.verify_backup_integrity(&backup_path).await?;
        }

        let mut restored_components = Vec::new();
        let mut failed_components = Vec::new();

        // Restore database if requested
        if options.components.contains(&"database".to_string()) {
            match self.restore_database(&backup_path, options.force).await {
                Ok(()) => {
                    info!("Database restored successfully");
                    restored_components.push("database".to_string());
                }
                Err(e) => {
                    error!("Failed to restore database: {}", e);
                    failed_components.push("database".to_string());
                }
            }
        }

        // Restore memory indices if requested
        if options.components.contains(&"memory".to_string()) {
            match self.restore_memory_indices(&backup_path, options.force).await {
                Ok(()) => {
                    info!("Memory indices restored successfully");
                    restored_components.push("memory".to_string());
                }
                Err(e) => {
                    error!("Failed to restore memory indices: {}", e);
                    failed_components.push("memory".to_string());
                }
            }
        }

        let duration = chrono::Utc::now()
            .signed_duration_since(start_time)
            .num_seconds() as u64;

        record_histogram!("restore_duration_seconds", duration as f64);

        let result = RestoreResult {
            backup_id,
            timestamp: start_time,
            restored_components,
            failed_components,
            duration_secs: duration,
        };

        match result.failed_components.is_empty() {
            true => {
                info!("Restore completed successfully");
                increment_counter!("restore_operations_total", "operation" => "success");
            }
            false => {
                warn!("Restore completed with failures");
                increment_counter!("restore_operations_total", "operation" => "partial_failure");
            }
        }

        Ok(result)
    }

    async fn verify_backup_integrity(&self, backup_path: &PathBuf) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Verifying backup integrity");

        // Check manifest
        let manifest_path = backup_path.join("manifest.json.enc");
        if !manifest_path.exists() {
            return Err("Backup manifest not found".into());
        }

        // Verify database backup if it exists
        let db_backup_path = backup_path.join("jamey.db.enc");
        if db_backup_path.exists() {
            self.vault.verify_file(&db_backup_path)
                .await
                .map_err(|e| format!("Database backup verification failed: {}", e))?;
        }

        // Verify memory indices if they exist
        let memory_backup_dir = backup_path.join("memory");
        if memory_backup_dir.exists() {
            let mut entries = fs::read_dir(&memory_backup_dir).await?;
            while let Some(entry) = entries.next_entry().await? {
                if entry.path().extension().map_or(false, |ext| ext == "enc") {
                    self.vault.verify_file(&entry.path())
                        .await
                        .map_err(|e| format!("Memory index verification failed: {}", e))?;
                }
            }
        }

        info!("Backup integrity verification completed");
        Ok(())
    }

    async fn restore_database(&self, backup_path: &PathBuf, force: bool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let db_backup_path = backup_path.join("jamey.db.enc");
        let target_path = PathBuf::from("data/jamey.db");

        if target_path.exists() && !force {
            return Err("Database file already exists and force option not specified".into());
        }

        // Create parent directory if it doesn't exist
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        // Decrypt and restore
        self.vault.decrypt_file(&db_backup_path, &target_path)
            .await
            .map_err(|e| format!("Failed to decrypt database: {}", e))?;

        Ok(())
    }

    async fn restore_memory_indices(&self, backup_path: &PathBuf, force: bool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let memory_backup_dir = backup_path.join("memory");
        let target_dir = PathBuf::from("data/memory");

        if target_dir.exists() && !force {
            return Err("Memory indices directory already exists and force option not specified".into());
        }

        // Create target directory
        fs::create_dir_all(&target_dir).await?;

        // Restore each memory layer
        let layers = ["short_term", "long_term", "working", "episodic", "semantic"];
        
        for layer in layers {
            let layer_backup_dir = memory_backup_dir.join(layer);
            if !layer_backup_dir.exists() {
                continue;
            }

            let layer_target_dir = target_dir.join(layer);
            fs::create_dir_all(&layer_target_dir).await?;

            let mut entries = fs::read_dir(&layer_backup_dir).await?;
            while let Some(entry) = entries.next_entry().await? {
                if entry.path().extension().map_or(false, |ext| ext == "enc") {
                    let file_name = entry.file_name();
                    let target_path = layer_target_dir.join(file_name)
                        .with_extension("");
                    
                    self.vault.decrypt_file(&entry.path(), &target_path)
                        .await
                        .map_err(|e| format!("Failed to decrypt memory index: {}", e))?;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_restore_manager() {
        let temp_dir = tempdir().unwrap();
        let key = [0u8; 32];
        
        let vault = Arc::new(PhoenixVault::new(
            temp_dir.path().to_path_buf(),
            key,
            true,
            5,
        ).unwrap());

        let restore_manager = RestoreManager::new(
            vault,
            temp_dir.path().to_path_buf(),
        );

        // Create test backup
        let backup_id = Uuid::new_v4();
        let backup_dir = temp_dir.path().join(backup_id.to_string());
        fs::create_dir_all(&backup_dir).await.unwrap();

        // Test restore with non-existent backup
        let result = restore_manager.restore_from_backup(
            Uuid::new_v4(),
            RestoreOptions {
                components: vec!["database".to_string()],
                force: false,
                verify_integrity: true,
            },
        ).await;

        assert!(result.is_err());
    }
}
