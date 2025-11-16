use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs::{File, OpenOptions};
use tokio::io::AsyncWriteExt;
use uuid::Uuid;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Serialize, Deserialize)]
pub enum AuditEventType {
    Authentication,
    Authorization,
    DataAccess,
    Configuration,
    SystemOperation,
    SecurityAlert,
    UserAction,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Severity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub event_type: AuditEventType,
    pub severity: Severity,
    pub user_id: Option<Uuid>,
    pub ip_address: Option<String>,
    pub resource: String,
    pub action: String,
    pub status: String,
    pub details: serde_json::Value,
}

pub struct AuditLogger {
    log_dir: PathBuf,
    current_log_file: Arc<Mutex<File>>,
    max_file_size: u64,
    max_files: usize,
}

impl AuditLogger {
    pub async fn new(log_dir: PathBuf, max_file_size: u64, max_files: usize) -> Result<Self, std::io::Error> {
        std::fs::create_dir_all(&log_dir)?;
        
        let current_file_path = log_dir.join(format!("audit_{}.log", Utc::now().format("%Y%m%d_%H%M%S")));
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&current_file_path)
            .await?;

        Ok(Self {
            log_dir,
            current_log_file: Arc::new(Mutex::new(file)),
            max_file_size,
            max_files,
        })
    }

    pub async fn log_event(&self, event: AuditEvent) -> Result<(), std::io::Error> {
        let log_entry = serde_json::to_string(&event)?;
        let mut file = self.current_log_file.lock().await;

        // Check if rotation is needed
        let metadata = file.metadata().await?;
        if metadata.len() >= self.max_file_size {
            self.rotate_logs().await?;
        }

        file.write_all(log_entry.as_bytes()).await?;
        file.write_all(b"\n").await?;
        file.sync_all().await?;

        Ok(())
    }

    async fn rotate_logs(&self) -> Result<(), std::io::Error> {
        // List existing log files
        let mut log_files: Vec<_> = std::fs::read_dir(&self.log_dir)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry.path()
                    .extension()
                    .map(|ext| ext == "log")
                    .unwrap_or(false)
            })
            .collect();

        // Sort by creation time
        log_files.sort_by_key(|entry| entry.metadata().unwrap().created().unwrap());

        // Remove oldest files if we exceed max_files
        while log_files.len() >= self.max_files {
            if let Some(oldest) = log_files.first() {
                std::fs::remove_file(oldest.path())?;
                log_files.remove(0);
            }
        }

        // Create new log file
        let new_file_path = self.log_dir.join(format!(
            "audit_{}.log",
            Utc::now().format("%Y%m%d_%H%M%S")
        ));

        let new_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&new_file_path)
            .await?;

        // Replace the current file
        let mut current_file = self.current_log_file.lock().await;
        *current_file = new_file;

        Ok(())
    }

    pub async fn log_sensitive_operation(
        &self,
        user_id: Uuid,
        ip_address: String,
        resource: String,
        action: String,
        details: serde_json::Value,
    ) -> Result<(), std::io::Error> {
        let event = AuditEvent {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event_type: AuditEventType::SystemOperation,
            severity: Severity::Critical,
            user_id: Some(user_id),
            ip_address: Some(ip_address),
            resource,
            action,
            status: "completed".to_string(),
            details,
        };

        self.log_event(event).await
    }

    pub async fn log_user_action(
        &self,
        user_id: Uuid,
        ip_address: String,
        action: String,
        details: serde_json::Value,
    ) -> Result<(), std::io::Error> {
        let event = AuditEvent {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event_type: AuditEventType::UserAction,
            severity: Severity::Info,
            user_id: Some(user_id),
            ip_address: Some(ip_address),
            resource: "user_action".to_string(),
            action,
            status: "completed".to_string(),
            details,
        };

        self.log_event(event).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_audit_logging() {
        let temp_dir = tempdir().unwrap();
        let logger = AuditLogger::new(
            temp_dir.path().to_path_buf(),
            1024, // 1KB max file size
            5,    // Keep 5 files max
        )
        .await
        .unwrap();

        // Test basic logging
        let event = AuditEvent {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event_type: AuditEventType::Authentication,
            severity: Severity::Info,
            user_id: Some(Uuid::new_v4()),
            ip_address: Some("127.0.0.1".to_string()),
            resource: "login".to_string(),
            action: "authenticate".to_string(),
            status: "success".to_string(),
            details: serde_json::json!({"method": "password"}),
        };

        logger.log_event(event).await.unwrap();

        // Verify log file exists and contains data
        let log_files: Vec<_> = std::fs::read_dir(temp_dir.path())
            .unwrap()
            .filter_map(|entry| entry.ok())
            .collect();
        assert_eq!(log_files.len(), 1);
    }

    #[tokio::test]
    async fn test_log_rotation() {
        let temp_dir = tempdir().unwrap();
        let logger = AuditLogger::new(
            temp_dir.path().to_path_buf(),
            10, // Very small max file size to trigger rotation
            3,  // Keep 3 files max
        )
        .await
        .unwrap();

        // Generate enough logs to trigger multiple rotations
        for _ in 0..10 {
            let event = AuditEvent {
                id: Uuid::new_v4(),
                timestamp: Utc::now(),
                event_type: AuditEventType::Authentication,
                severity: Severity::Info,
                user_id: Some(Uuid::new_v4()),
                ip_address: Some("127.0.0.1".to_string()),
                resource: "test".to_string(),
                action: "test".to_string(),
                status: "test".to_string(),
                details: serde_json::json!({}),
            };

            logger.log_event(event).await.unwrap();
        }

        // Verify number of log files
        let log_files: Vec<_> = std::fs::read_dir(temp_dir.path())
            .unwrap()
            .filter_map(|entry| entry.ok())
            .collect();
        assert!(log_files.len() <= 3);
    }
}