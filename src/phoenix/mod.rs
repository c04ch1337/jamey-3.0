//! Phoenix Vault backup and recovery system
//! 
//! Provides encrypted backup and recovery for critical system data:
//! - SQLite database
//! - Memory indices (all 5 layers)
//! - Configuration files
//! 
//! Uses AES-256-GCM encryption and supports scheduled backups.

mod vault;
mod backup;
mod restore;
mod encryption;
mod schedule;

pub use vault::{PhoenixVault, BackupManifest};
pub use schedule::BackupScheduler;
pub use encryption::Encryptor;

// Re-export error types
pub use vault::PhoenixError;