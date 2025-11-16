use crate::llm::models::ModelMetadata;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tracing::{debug, warn, error};

/// Health status of a model
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// Model is healthy and available
    Healthy,
    /// Model is degraded but usable
    Degraded,
    /// Model is unavailable
    Unavailable,
    /// Model status is unknown
    Unknown,
}

/// Health metrics for a model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelHealth {
    /// Current health status
    pub status: HealthStatus,
    /// Last successful request timestamp
    pub last_success: Option<DateTime<Utc>>,
    /// Last failed request timestamp
    pub last_failure: Option<DateTime<Utc>>,
    /// Number of consecutive failures
    pub consecutive_failures: u32,
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
    /// Total requests
    pub total_requests: u64,
    /// Successful requests
    pub successful_requests: u64,
    /// Last health check timestamp
    pub last_check: Option<DateTime<Utc>>,
}

impl Default for ModelHealth {
    fn default() -> Self {
        Self {
            status: HealthStatus::Unknown,
            last_success: None,
            last_failure: None,
            consecutive_failures: 0,
            avg_response_time_ms: 0.0,
            success_rate: 1.0,
            total_requests: 0,
            successful_requests: 0,
            last_check: None,
        }
    }
}

impl ModelHealth {
    /// Update health after a successful request
    pub fn record_success(&mut self, response_time_ms: f64) {
        self.status = HealthStatus::Healthy;
        self.last_success = Some(Utc::now());
        self.consecutive_failures = 0;
        self.total_requests += 1;
        self.successful_requests += 1;
        
        // Update average response time (exponential moving average)
        if self.avg_response_time_ms == 0.0 {
            self.avg_response_time_ms = response_time_ms;
        } else {
            self.avg_response_time_ms = (self.avg_response_time_ms * 0.9) + (response_time_ms * 0.1);
        }
        
        // Update success rate
        self.success_rate = self.successful_requests as f64 / self.total_requests as f64;
        self.last_check = Some(Utc::now());
    }

    /// Update health after a failed request
    pub fn record_failure(&mut self, _error: &str) {
        self.last_failure = Some(Utc::now());
        self.consecutive_failures += 1;
        self.total_requests += 1;
        
        // Update success rate
        self.success_rate = self.successful_requests as f64 / self.total_requests as f64;
        
        // Update status based on failure pattern
        if self.consecutive_failures >= 5 {
            self.status = HealthStatus::Unavailable;
            error!("Model marked as unavailable after {} consecutive failures", self.consecutive_failures);
        } else if self.consecutive_failures >= 3 {
            self.status = HealthStatus::Degraded;
            warn!("Model marked as degraded after {} consecutive failures", self.consecutive_failures);
        } else if self.success_rate < 0.5 && self.total_requests >= 10 {
            self.status = HealthStatus::Degraded;
            warn!("Model marked as degraded due to low success rate: {:.1}%", self.success_rate * 100.0);
        }
        
        self.last_check = Some(Utc::now());
    }

    /// Check if model should be retried after a failure
    pub fn should_retry(&self) -> bool {
        match self.status {
            HealthStatus::Healthy => true,
            HealthStatus::Degraded => {
                // Retry degraded models with exponential backoff
                if let Some(last_failure) = self.last_failure {
                    let time_since_failure = Utc::now().signed_duration_since(last_failure);
                    let backoff_seconds = 2_u64.pow(self.consecutive_failures.min(5));
                    time_since_failure.num_seconds() >= backoff_seconds as i64
                } else {
                    true
                }
            }
            HealthStatus::Unavailable => {
                // Only retry unavailable models after significant backoff
                if let Some(last_failure) = self.last_failure {
                    let time_since_failure = Utc::now().signed_duration_since(last_failure);
                    time_since_failure.num_minutes() >= 15
                } else {
                    false
                }
            }
            HealthStatus::Unknown => true,
        }
    }

    /// Get health score (0.0 to 1.0, higher is better)
    pub fn health_score(&self) -> f64 {
        match self.status {
            HealthStatus::Healthy => {
                // Factor in success rate and response time
                let success_factor = self.success_rate;
                let response_factor = if self.avg_response_time_ms > 0.0 {
                    // Normalize response time (assume 5000ms is slow)
                    (1.0 - (self.avg_response_time_ms / 5000.0).min(1.0)).max(0.0)
                } else {
                    1.0
                };
                (success_factor * 0.7 + response_factor * 0.3).min(1.0)
            }
            HealthStatus::Degraded => 0.5,
            HealthStatus::Unavailable => 0.0,
            HealthStatus::Unknown => 0.5,
        }
    }
}

/// Health monitor for tracking model availability and performance
pub struct HealthMonitor {
    /// Health data for each model
    health_data: Arc<RwLock<HashMap<String, ModelHealth>>>,
    /// Minimum success rate to consider healthy
    min_success_rate: f64,
    /// Maximum response time to consider healthy (ms)
    max_response_time_ms: f64,
}

impl HealthMonitor {
    /// Create a new health monitor
    pub fn new(min_success_rate: f64, max_response_time_ms: f64) -> Self {
        Self {
            health_data: Arc::new(RwLock::new(HashMap::new())),
            min_success_rate,
            max_response_time_ms,
        }
    }

    /// Get or create health record for a model
    fn get_or_create_health(&self, model_id: &str) -> ModelHealth {
        let health_data = self.health_data.read().unwrap();
        health_data.get(model_id).cloned().unwrap_or_default()
    }

    /// Update health record
    fn update_health(&self, model_id: &str, health: ModelHealth) {
        let mut health_data = self.health_data.write().unwrap();
        health_data.insert(model_id.to_string(), health);
    }

    /// Record a successful request
    pub fn record_success(&self, model_id: &str, response_time_ms: f64) {
        let mut health = self.get_or_create_health(model_id);
        health.record_success(response_time_ms);
        self.update_health(model_id, health);
        debug!("Recorded success for {}: {:.2}ms", model_id, response_time_ms);
    }

    /// Record a failed request
    pub fn record_failure(&self, model_id: &str, error: &str) {
        let mut health = self.get_or_create_health(model_id);
        health.record_failure(error);
        self.update_health(model_id, health);
        warn!("Recorded failure for {}: {}", model_id, error);
    }

    /// Get health status for a model
    pub fn get_health(&self, model_id: &str) -> Option<ModelHealth> {
        let health_data = self.health_data.read().unwrap();
        health_data.get(model_id).cloned()
    }

    /// Check if a model is available
    pub fn is_available(&self, model_id: &str) -> bool {
        if let Some(health) = self.get_health(model_id) {
            health.status != HealthStatus::Unavailable && health.should_retry()
        } else {
            // Unknown models are assumed available
            true
        }
    }

    /// Get available models from a list
    pub fn filter_available(&self, models: &[ModelMetadata]) -> Vec<ModelMetadata> {
        models
            .iter()
            .filter(|m| {
                m.available && self.is_available(&m.model_id)
            })
            .cloned()
            .collect()
    }

    /// Get health score for a model
    pub fn get_health_score(&self, model_id: &str) -> f64 {
        self.get_health(model_id)
            .map(|h| h.health_score())
            .unwrap_or(0.5) // Unknown models get neutral score
    }

    /// Get all health data
    pub fn get_all_health(&self) -> HashMap<String, ModelHealth> {
        self.health_data.read().unwrap().clone()
    }

    /// Reset health for a model (for testing or manual intervention)
    pub fn reset_health(&self, model_id: &str) {
        let mut health_data = self.health_data.write().unwrap();
        health_data.insert(model_id.to_string(), ModelHealth::default());
        debug!("Reset health for model: {}", model_id);
    }

    /// Get models sorted by health score (best first)
    pub fn sort_by_health(&self, models: &mut [ModelMetadata]) {
        models.sort_by(|a, b| {
            let score_a = self.get_health_score(&a.model_id);
            let score_b = self.get_health_score(&b.model_id);
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    /// Check if model meets health criteria
    pub fn meets_health_criteria(&self, model_id: &str) -> bool {
        if let Some(health) = self.get_health(model_id) {
            health.status == HealthStatus::Healthy
                && health.success_rate >= self.min_success_rate
                && (health.avg_response_time_ms == 0.0 || health.avg_response_time_ms <= self.max_response_time_ms)
        } else {
            // Unknown models are assumed to meet criteria
            true
        }
    }

    /// Get health summary for all models
    pub fn get_health_summary(&self) -> HealthSummary {
        let health_data = self.health_data.read().unwrap();
        let mut healthy = 0;
        let mut degraded = 0;
        let mut unavailable = 0;
        let mut unknown = 0;

        for health in health_data.values() {
            match health.status {
                HealthStatus::Healthy => healthy += 1,
                HealthStatus::Degraded => degraded += 1,
                HealthStatus::Unavailable => unavailable += 1,
                HealthStatus::Unknown => unknown += 1,
            }
        }

        HealthSummary {
            total_models: health_data.len(),
            healthy,
            degraded,
            unavailable,
            unknown,
        }
    }
}

/// Health summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthSummary {
    pub total_models: usize,
    pub healthy: usize,
    pub degraded: usize,
    pub unavailable: usize,
    pub unknown: usize,
}

