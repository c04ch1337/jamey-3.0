use std::path::PathBuf;
use tokio::fs;
use tracing::{info, warn};
use sqlx::SqlitePool;

use super::vault::{PhoenixError, BackupManifest};

/// Implements backup operations for PhoenixVault
impl super::vault::PhoenixVault {
    /// Backup SQLite database
    pub(crate) async fn backup_database(
        &self,
        backup_path: &PathBuf,
        manifest: &mut BackupManifest,
    ) -> Result<(), PhoenixError> {
        info!("Backing up SQLite database...");

        // Get database path from environment or default
        let db_path = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "data/jamey.db".to_string())
            .trim_start_matches("sqlite://")
            .to_string();

        let db_backup_path = backup_path.join("jamey.db.enc");

        // Ensure source exists
        if !PathBuf::from(&db_path).exists() {
            warn!("Database file not found at {}", db_path);
            return Ok(());
        }

        // Encrypt and copy
        let size = self.encryptor
            .encrypt_file(
                &PathBuf::from(&db_path),
                &db_backup_path
            )
            .await
            .map_err(|e| PhoenixError::Encryption(e.to_string()))?;

        manifest.components.push("database".to_string());
        manifest.size_bytes += size;

        info!("Database backup completed ({} bytes)", size);
        Ok(())
    }

    /// Backup Tantivy memory indices
    pub(crate) async fn backup_memory_indices(
        &self,
        backup_path: &PathBuf,
        manifest: &mut BackupManifest,
    ) -> Result<(), PhoenixError> {
        info!("Backing up memory indices...");

        let memory_dir = PathBuf::from("data/memory");
        if !memory_dir.exists() {
            warn!("Memory directory not found");
            return Ok(());
        }

        let memory_backup_dir = backup_path.join("memory");
        fs::create_dir_all(&memory_backup_dir).await?;

        // Backup each memory layer
        let layers = ["short_term", "long_term", "working", "episodic", "semantic"];
        
        for layer in layers {
            let layer_dir = memory_dir.join(layer);
            if !layer_dir.exists() {
                continue;
            }

            let layer_backup_dir = memory_backup_dir.join(layer);
            fs::create_dir_all(&layer_backup_dir).await?;

            // Encrypt and copy each file in the layer
            let mut entries = fs::read_dir(&layer_dir).await?;
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                if path.is_file() {
                    if let Some(filename) = path.file_name() {
                        let dest_path = layer_backup_dir.join(filename).with_extension("enc");
                        
                        let size = self.encryptor
                            .encrypt_file(&path, &dest_path)
                            .await
                            .map_err(|e| PhoenixError::Encryption(e.to_string()))?;

                        manifest.size_bytes += size;
                    }
                }
            }

            manifest.components.push(format!("memory_{}", layer));
        }

        info!("Memory indices backup completed");
        Ok(())
    }

    /// Save backup manifest
    pub(crate) async fn save_manifest(
        &self,
        backup_path: &PathBuf,
        manifest: &BackupManifest,
    ) -> Result<(), PhoenixError> {
        let manifest_path = backup_path.join("manifest.json.enc");

        // Serialize manifest
        let manifest_json = serde_json::to_vec(manifest)
            .map_err(|e| PhoenixError::InvalidManifest(e.to_string()))?;

        // Encrypt and save
        let encrypted = self.encryptor
            .encrypt(&manifest_json)
            .map_err(|e| PhoenixError::Encryption(e.to_string()))?;

        fs::write(manifest_path, encrypted).await?;

        Ok(())
    }

    /// Load backup manifest
    pub(crate) async fn load_manifest(
        &self,
        backup_path: &PathBuf,
    ) -> Result<BackupManifest, PhoenixError> {
        let manifest_path = backup_path.join("manifest.json.enc");

        // Read and decrypt
        let encrypted = fs::read(manifest_path).await?;
        let decrypted = self.encryptor
            .decrypt(&encrypted)
            .map_err(|e| PhoenixError::Encryption(e.to_string()))?;

        // Deserialize
        serde_json::from_slice(&decrypted)
            .map_err(|e| PhoenixError::InvalidManifest(e.to_string()))
    }

    /// Clean up old backups beyond max_backups
    pub(crate) async fn cleanup_old_backups(&self) -> Result<(), PhoenixError> {
        let mut backups = self.list_backups().await?;
        
        // Sort by timestamp (oldest first)
        backups.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        // Remove oldest backups beyond max_backups
        while backups.len() > self.max_backups {
            if let Some(oldest) = backups.first() {
                let backup_id = oldest.backup_id;
                if let Err(e) = self.delete_backup(backup_id).await {
                    warn!("Failed to delete old backup {}: {}", backup_id, e);
                }
                backups.remove(0);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use chrono::Utc;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_manifest_roundtrip() {
        let dir = tempdir().unwrap();
        let key = [0u8; 32];
        
        let vault = super::super::vault::PhoenixVault::new(
            dir.path().to_path_buf(),
            key,
            true,
            5
        ).unwrap();

        let manifest = BackupManifest {
            backup_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            components: vec!["test".to_string()],
            size_bytes: 42,
            description: Some("Test backup".to_string()),
        };

        // Save and load
        vault.save_manifest(&dir.path().to_path_buf(), &manifest).await.unwrap();
        let loaded = vault.load_manifest(&dir.path().to_path_buf()).await.unwrap();

        assert_eq!(manifest.backup_id, loaded.backup_id);
        assert_eq!(manifest.components, loaded.components);
        assert_eq!(manifest.size_bytes, loaded.size_bytes);
        assert_eq!(manifest.description, loaded.description);
    }
}