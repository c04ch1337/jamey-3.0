use std::path::PathBuf;
use tokio::fs;
use tracing::{info, warn};
use sqlx::SqlitePool;

use super::vault::{PhoenixError, BackupManifest};

/// Implements restore operations for PhoenixVault
impl super::vault::PhoenixVault {
    /// Restore SQLite database
    pub(crate) async fn restore_database(
        &self,
        backup_path: &PathBuf,
    ) -> Result<(), PhoenixError> {
        info!("Restoring SQLite database...");

        let db_backup_path = backup_path.join("jamey.db.enc");
        if !db_backup_path.exists() {
            warn!("Database backup not found");
            return Ok(());
        }

        // Get target database path
        let db_path = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "data/jamey.db".to_string())
            .trim_start_matches("sqlite://")
            .to_string();

        // Create parent directory if needed
        if let Some(parent) = PathBuf::from(&db_path).parent() {
            fs::create_dir_all(parent).await?;
        }

        // Decrypt and restore
        self.encryptor
            .decrypt_file(
                &db_backup_path,
                &PathBuf::from(&db_path)
            )
            .await
            .map_err(|e| PhoenixError::Encryption(e.to_string()))?;

        info!("Database restore completed");
        Ok(())
    }

    /// Restore Tantivy memory indices
    pub(crate) async fn restore_memory_indices(
        &self,
        backup_path: &PathBuf,
    ) -> Result<(), PhoenixError> {
        info!("Restoring memory indices...");

        let memory_backup_dir = backup_path.join("memory");
        if !memory_backup_dir.exists() {
            warn!("Memory backup directory not found");
            return Ok(());
        }

        let memory_dir = PathBuf::from("data/memory");

        // Restore each memory layer
        let layers = ["short_term", "long_term", "working", "episodic", "semantic"];
        
        for layer in layers {
            let layer_backup_dir = memory_backup_dir.join(layer);
            if !layer_backup_dir.exists() {
                continue;
            }

            let layer_dir = memory_dir.join(layer);
            fs::create_dir_all(&layer_dir).await?;

            // Decrypt and restore each file
            let mut entries = fs::read_dir(&layer_backup_dir).await?;
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                if path.is_file() && path.extension().map_or(false, |ext| ext == "enc") {
                    if let Some(filename) = path.file_stem() {
                        let dest_path = layer_dir.join(filename);
                        
                        self.encryptor
                            .decrypt_file(&path, &dest_path)
                            .await
                            .map_err(|e| PhoenixError::Encryption(e.to_string()))?;
                    }
                }
            }
        }

        info!("Memory indices restore completed");
        Ok(())
    }

    /// Verify restoration against manifest
    pub(crate) async fn verify_restoration(
        &self,
        manifest: &BackupManifest,
    ) -> Result<(), PhoenixError> {
        info!("Verifying restoration...");

        // Check database
        if manifest.components.contains(&"database".to_string()) {
            let db_path = std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "data/jamey.db".to_string())
                .trim_start_matches("sqlite://")
                .to_string();

            if !PathBuf::from(&db_path).exists() {
                return Err(PhoenixError::InvalidManifest(
                    "Database file not restored".into()
                ));
            }

            // Try to open database
            let pool = SqlitePool::connect(&format!("sqlite:{}", db_path))
                .await
                .map_err(PhoenixError::Database)?;

            // Simple test query
            sqlx::query("SELECT 1")
                .fetch_one(&pool)
                .await
                .map_err(PhoenixError::Database)?;

            pool.close().await;
        }

        // Check memory indices
        for component in &manifest.components {
            if component.starts_with("memory_") {
                let layer = component.trim_start_matches("memory_");
                let layer_dir = PathBuf::from("data/memory").join(layer);

                if !layer_dir.exists() {
                    return Err(PhoenixError::InvalidManifest(
                        format!("Memory layer {} not restored", layer)
                    ));
                }

                // Check if directory has any files
                let mut entries = fs::read_dir(&layer_dir).await?;
                if entries.next_entry().await?.is_none() {
                    return Err(PhoenixError::InvalidManifest(
                        format!("Memory layer {} is empty", layer)
                    ));
                }
            }
        }

        info!("Restoration verification completed successfully");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use uuid::Uuid;
    use chrono::Utc;

    #[tokio::test]
    async fn test_verify_restoration() {
        let dir = tempdir().unwrap();
        let key = [0u8; 32];
        
        let vault = super::super::vault::PhoenixVault::new(
            dir.path().to_path_buf(),
            key,
            true,
            5
        ).unwrap();

        // Create test manifest
        let manifest = BackupManifest {
            backup_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            components: vec![
                "database".to_string(),
                "memory_short_term".to_string()
            ],
            size_bytes: 0,
            description: None,
        };

        // Should fail verification (files don't exist)
        assert!(vault.verify_restoration(&manifest).await.is_err());
    }
}