use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, Instant, interval};
use chrono::{DateTime, Utc, Timelike};
use serde::{Serialize, Deserialize};
use tracing::{info, warn, error};

use super::BackupManager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleConfig {
    /// Daily backup hour (0-23)
    pub daily_backup_hour: u32,
    /// Weekly backup day (0 = Sunday, 6 = Saturday)
    pub weekly_backup_day: u32,
    /// Monthly backup day (1-28)
    pub monthly_backup_day: u32,
    /// Whether to enable daily backups
    pub daily_enabled: bool,
    /// Whether to enable weekly backups
    pub weekly_enabled: bool,
    /// Whether to enable monthly backups
    pub monthly_enabled: bool,
}

impl Default for ScheduleConfig {
    fn default() -> Self {
        Self {
            daily_backup_hour: 2, // 2 AM
            weekly_backup_day: 0, // Sunday
            monthly_backup_day: 1, // 1st of month
            daily_enabled: true,
            weekly_enabled: true,
            monthly_enabled: true,
        }
    }
}

#[derive(Debug)]
pub struct BackupScheduler {
    manager: Arc<BackupManager>,
    config: Arc<RwLock<ScheduleConfig>>,
    last_daily: Arc<RwLock<Option<DateTime<Utc>>>>,
    last_weekly: Arc<RwLock<Option<DateTime<Utc>>>>,
    last_monthly: Arc<RwLock<Option<DateTime<Utc>>>>,
}

impl BackupScheduler {
    pub fn new(manager: Arc<BackupManager>, config: ScheduleConfig) -> Self {
        Self {
            manager,
            config: Arc::new(RwLock::new(config)),
            last_daily: Arc::new(RwLock::new(None)),
            last_weekly: Arc::new(RwLock::new(None)),
            last_monthly: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn update_config(&self, new_config: ScheduleConfig) {
        *self.config.write().await = new_config;
    }

    pub async fn start(&self) {
        info!("Starting backup scheduler");
        
        let check_interval = Duration::from_secs(60); // Check every minute
        let mut interval = interval(check_interval);

        loop {
            interval.tick().await;
            self.check_schedules().await;
        }
    }

    async fn check_schedules(&self) {
        let now = Utc::now();
        let config = self.config.read().await;

        // Check daily backup
        if config.daily_enabled {
            self.check_daily_backup(now).await;
        }

        // Check weekly backup
        if config.weekly_enabled {
            self.check_weekly_backup(now).await;
        }

        // Check monthly backup
        if config.monthly_enabled {
            self.check_monthly_backup(now).await;
        }
    }

    async fn check_daily_backup(&self, now: DateTime<Utc>) {
        let config = self.config.read().await;
        let mut last_daily = self.last_daily.write().await;

        if let Some(last) = *last_daily {
            if last.date() == now.date() {
                return; // Already ran today
            }
        }

        if now.hour() == config.daily_backup_hour {
            info!("Starting scheduled daily backup");
            match self.manager.create_backup(Some("Daily backup".to_string())).await {
                Ok(backup_id) => {
                    info!("Daily backup completed successfully: {}", backup_id);
                    *last_daily = Some(now);
                }
                Err(e) => {
                    error!("Daily backup failed: {}", e);
                }
            }
        }
    }

    async fn check_weekly_backup(&self, now: DateTime<Utc>) {
        let config = self.config.read().await;
        let mut last_weekly = self.last_weekly.write().await;

        if let Some(last) = *last_weekly {
            if last.date() == now.date() {
                return; // Already ran today
            }
        }

        if now.weekday().num_days_from_sunday() == config.weekly_backup_day 
           && now.hour() == config.daily_backup_hour {
            info!("Starting scheduled weekly backup");
            match self.manager.create_backup(Some("Weekly backup".to_string())).await {
                Ok(backup_id) => {
                    info!("Weekly backup completed successfully: {}", backup_id);
                    *last_weekly = Some(now);
                }
                Err(e) => {
                    error!("Weekly backup failed: {}", e);
                }
            }
        }
    }

    async fn check_monthly_backup(&self, now: DateTime<Utc>) {
        let config = self.config.read().await;
        let mut last_monthly = self.last_monthly.write().await;

        if let Some(last) = *last_monthly {
            if last.date() == now.date() {
                return; // Already ran today
            }
        }

        if now.day() == config.monthly_backup_day 
           && now.hour() == config.daily_backup_hour {
            info!("Starting scheduled monthly backup");
            match self.manager.create_backup(Some("Monthly backup".to_string())).await {
                Ok(backup_id) => {
                    info!("Monthly backup completed successfully: {}", backup_id);
                    *last_monthly = Some(now);
                }
                Err(e) => {
                    error!("Monthly backup failed: {}", e);
                }
            }
        }
    }

    pub async fn get_next_scheduled_backups(&self) -> ScheduledBackups {
        let config = self.config.read().await;
        let now = Utc::now();

        let daily = if config.daily_enabled {
            let next = if now.hour() >= config.daily_backup_hour {
                // Next day at backup hour
                now.date().succ().and_hms_opt(config.daily_backup_hour, 0, 0)
            } else {
                // Today at backup hour
                now.date().and_hms_opt(config.daily_backup_hour, 0, 0)
            };
            next
        } else {
            None
        };

        let weekly = if config.weekly_enabled {
            let days_until_next = (7 + config.weekly_backup_day - now.weekday().num_days_from_sunday()) % 7;
            Some(now.date().succ_opt().unwrap().and_hms_opt(config.daily_backup_hour, 0, 0).unwrap() + chrono::Duration::days(days_until_next as i64))
        } else {
            None
        };

        let monthly = if config.monthly_enabled {
            let next_month = if now.day() >= config.monthly_backup_day {
                now.date().succ_opt().unwrap()
            } else {
                now.date()
            };
            Some(next_month.with_day(config.monthly_backup_day).unwrap().and_hms_opt(config.daily_backup_hour, 0, 0).unwrap())
        } else {
            None
        };

        ScheduledBackups {
            next_daily: daily,
            next_weekly: weekly,
            next_monthly: monthly,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScheduledBackups {
    pub next_daily: Option<DateTime<Utc>>,
    pub next_weekly: Option<DateTime<Utc>>,
    pub next_monthly: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use crate::backup::{BackupConfig, BackupManager};

    #[tokio::test]
    async fn test_scheduler_config() {
        let temp_dir = tempfile::tempdir().unwrap();
        let backup_config = BackupConfig {
            backup_dir: temp_dir.path().to_path_buf(),
            max_backups: 5,
            encryption_key: [0u8; 32],
            components: vec!["test".to_string()],
        };

        let manager = Arc::new(BackupManager::new(backup_config).unwrap());
        
        let schedule_config = ScheduleConfig {
            daily_backup_hour: 3,
            weekly_backup_day: 1,
            monthly_backup_day: 15,
            daily_enabled: true,
            weekly_enabled: true,
            monthly_enabled: true,
        };

        let scheduler = BackupScheduler::new(manager, schedule_config.clone());
        
        // Update config
        let new_config = ScheduleConfig {
            daily_backup_hour: 4,
            ..schedule_config
        };
        
        scheduler.update_config(new_config.clone()).await;
        
        let current_config = scheduler.config.read().await;
        assert_eq!(current_config.daily_backup_hour, 4);
    }
}