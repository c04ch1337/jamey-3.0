//! Automated Backup and Disaster Recovery System
//!
//! Provides production-grade backup and restore functionality for:
//! - SQLite database
//! - Tantivy memory indices (5 layers)
//! - Configuration files
//!
//! Features:
//! - Automated scheduled backups
//! - Retention policies
//! - Backup verification
//! - Disaster recovery restore

pub mod backup;
pub mod restore;
pub mod schedule;

pub use backup::{BackupManager, BackupManifest, BackupResult};
pub use restore::{RestoreManager, RestoreResult};
pub use schedule::{BackupScheduler, ScheduleConfig};

use std::path::PathBuf;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Backup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    /// Backup directory path
    pub backup_dir: PathBuf,
    /// Maximum number of backups to retain
    pub max_backups: usize,
    /// Enable automatic backups
    pub enabled: bool,
    /// Backup interval in hours
    pub interval_hours: u64,
    /// Retention period in days
    pub retention_days: u64,
    /// Compress backups
    pub compress: bool,
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            backup_dir: PathBuf::from("data/backups"),
            max_backups: 10,
            enabled: true,
            interval_hours: 24, // Daily backups
            retention_days: 30, // Keep for 30 days
            compress: true,
        }
    }
}

impl BackupConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        use std::env;
        
        let backup_dir = env::var("BACKUP_DIR")
            .map(|v| PathBuf::from(v))
            .unwrap_or_else(|_| PathBuf::from("data/backups"));
        
        Self {
            backup_dir,
            max_backups: env::var("BACKUP_MAX_BACKUPS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10),
            enabled: env::var("BACKUP_ENABLED")
                .ok()
                .map(|v| v == "true" || v == "1")
                .unwrap_or(true),
            interval_hours: env::var("BACKUP_INTERVAL_HOURS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(24),
            retention_days: env::var("BACKUP_RETENTION_DAYS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(30),
            compress: env::var("BACKUP_COMPRESS")
                .ok()
                .map(|v| v == "true" || v == "1")
                .unwrap_or(true),
        }
    }
    
    /// Get backup interval as Duration
    pub fn interval(&self) -> Duration {
        Duration::from_secs(self.interval_hours * 3600)
    }
    
    /// Get retention period as Duration
    pub fn retention(&self) -> Duration {
        Duration::from_secs(self.retention_days * 86400)
    }
}

