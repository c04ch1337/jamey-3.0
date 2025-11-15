//! Automated backup scheduling

use crate::backup::{BackupConfig, BackupManager};
use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::{interval, sleep};
use tracing::{info, warn, error};

/// Backup scheduler configuration
#[derive(Debug, Clone)]
pub struct ScheduleConfig {
    /// Backup interval
    pub interval: Duration,
    /// Whether scheduling is enabled
    pub enabled: bool,
}

impl Default for ScheduleConfig {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(86400), // 24 hours
            enabled: true,
        }
    }
}

/// Manages automated backup scheduling
pub struct BackupScheduler {
    manager: Arc<BackupManager>,
    config: Arc<RwLock<ScheduleConfig>>,
    running: Arc<RwLock<bool>>,
}

impl BackupScheduler {
    /// Create a new backup scheduler
    pub fn new(manager: BackupManager, config: ScheduleConfig) -> Self {
        Self {
            manager: Arc::new(manager),
            config: Arc::new(RwLock::new(config)),
            running: Arc::new(RwLock::new(false)),
        }
    }
    
    /// Start the backup scheduler
    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.write().await;
        if *running {
            warn!("Backup scheduler is already running");
            return Ok(());
        }
        
        *running = true;
        drop(running);
        
        let manager = Arc::clone(&self.manager);
        let config = Arc::clone(&self.config);
        let running_flag = Arc::clone(&self.running);
        
        tokio::spawn(async move {
            info!("Backup scheduler started");
            
            loop {
                // Check if still running
                {
                    let running = running_flag.read().await;
                    if !*running {
                        info!("Backup scheduler stopped");
                        break;
                    }
                }
                
                // Get current config
                let interval = {
                    let config = config.read().await;
                    if !config.enabled {
                        // Wait a bit before checking again
                        sleep(Duration::from_secs(60)).await;
                        continue;
                    }
                    config.interval
                };
                
                // Wait for interval
                sleep(interval).await;
                
                // Check if still running before backup
                {
                    let running = running_flag.read().await;
                    if !*running {
                        break;
                    }
                }
                
                // Perform backup
                info!("Starting scheduled backup");
                match manager.create_backup().await {
                    Ok(result) => {
                        if result.success {
                            info!("Scheduled backup completed successfully: {} ({} bytes in {:?})",
                                  result.manifest.backup_id,
                                  result.manifest.total_size,
                                  result.duration);
                        } else {
                            error!("Scheduled backup failed: {:?}", result.error);
                        }
                    }
                    Err(e) => {
                        error!("Scheduled backup error: {}", e);
                    }
                }
            }
        });
        
        Ok(())
    }
    
    /// Stop the backup scheduler
    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
        info!("Backup scheduler stop requested");
    }
    
    /// Check if scheduler is running
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }
    
    /// Update scheduler configuration
    pub async fn update_config(&self, config: ScheduleConfig) {
        let mut current = self.config.write().await;
        *current = config;
        info!("Backup scheduler configuration updated");
    }
    
    /// Trigger an immediate backup (manual trigger)
    pub async fn trigger_backup(&self) -> Result<crate::backup::BackupResult> {
        info!("Manual backup triggered");
        self.manager.create_backup().await
    }
}

