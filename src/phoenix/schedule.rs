use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::{info, error};

use super::vault::PhoenixVault;

/// Automatic backup scheduler for Phoenix Vault
pub struct BackupScheduler {
    /// Reference to Phoenix Vault
    vault: Arc<PhoenixVault>,

    /// Backup interval in hours
    interval_hours: u64,

    /// Whether the scheduler is running
    running: bool,
}

impl BackupScheduler {
    /// Create a new backup scheduler
    pub fn new(vault: Arc<PhoenixVault>, interval_hours: u64) -> Self {
        Self {
            vault,
            interval_hours,
            running: false,
        }
    }

    /// Start automatic backup schedule
    pub async fn start(mut self) {
        if self.running {
            error!("Backup scheduler already running");
            return;
        }

        self.running = true;
        let mut interval = interval(Duration::from_secs(self.interval_hours * 3600));

        info!(
            "Starting automatic backup scheduler (interval: {} hours)",
            self.interval_hours
        );

        loop {
            interval.tick().await;

            info!("Starting scheduled backup...");
            match self.vault.create_backup().await {
                Ok(manifest) => {
                    info!(
                        "Scheduled backup completed successfully: {} ({} bytes)",
                        manifest.backup_id,
                        manifest.size_bytes
                    );

                    // Log components backed up
                    for component in &manifest.components {
                        info!("  - Backed up: {}", component);
                    }
                }
                Err(e) => {
                    error!("Scheduled backup failed: {}", e);
                }
            }
        }
    }

    /// Stop the scheduler
    pub fn stop(&mut self) {
        if self.running {
            self.running = false;
            info!("Backup scheduler stopped");
        }
    }
}

impl Drop for BackupScheduler {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_scheduler_creation() {
        let dir = tempdir().unwrap();
        let key = [0u8; 32];
        
        let vault = Arc::new(
            PhoenixVault::new(
                dir.path().to_path_buf(),
                key,
                true,
                5
            ).unwrap()
        );

        let scheduler = BackupScheduler::new(vault, 24);
        assert_eq!(scheduler.interval_hours, 24);
        assert!(!scheduler.running);
    }

    #[tokio::test]
    async fn test_scheduler_start_stop() {
        let dir = tempdir().unwrap();
        let key = [0u8; 32];
        
        let vault = Arc::new(
            PhoenixVault::new(
                dir.path().to_path_buf(),
                key,
                true,
                5
            ).unwrap()
        );

        let mut scheduler = BackupScheduler::new(vault, 24);
        
        // Start in background task
        let scheduler_handle = {
            let scheduler = scheduler.clone();
            tokio::spawn(async move {
                scheduler.start().await;
            })
        };

        // Let it run briefly
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Stop
        scheduler.stop();
        
        // Should exit
        scheduler_handle.abort();
    }
}