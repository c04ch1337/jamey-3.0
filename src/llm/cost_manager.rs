use crate::llm::models::{ModelMetadata, ModelPricing};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tracing::{debug, warn};

/// Daily cost tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyCost {
    pub date: DateTime<Utc>,
    pub total_spend: f64,
    pub request_count: u64,
    pub model_breakdown: HashMap<String, f64>, // model_id -> cost
}

/// Cost manager for tracking and managing LLM API costs
pub struct CostManager {
    /// Monthly budget in USD
    monthly_budget: f64,
    /// Daily spending records
    daily_costs: Arc<RwLock<Vec<DailyCost>>>,
    /// Current day's spending
    current_daily_spend: Arc<RwLock<f64>>,
    /// Current day's date
    current_date: Arc<RwLock<DateTime<Utc>>>,
    /// Per-model cost tracking
    model_costs: Arc<RwLock<HashMap<String, f64>>>,
}

impl CostManager {
    /// Create a new cost manager
    pub fn new(monthly_budget: f64) -> Self {
        let now = Utc::now();
        Self {
            monthly_budget,
            daily_costs: Arc::new(RwLock::new(Vec::new())),
            current_daily_spend: Arc::new(RwLock::new(0.0)),
            current_date: Arc::new(RwLock::new(now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc())),
            model_costs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Check if we can afford a model for a given task
    pub fn can_afford_model(
        &self,
        model: &ModelMetadata,
        estimated_input_tokens: usize,
        estimated_output_tokens: usize,
    ) -> bool {
        let estimated_cost = model.estimate_cost(estimated_input_tokens, estimated_output_tokens);
        self.can_afford(estimated_cost)
    }

    /// Check if we can afford a cost amount
    pub fn can_afford(&self, cost: f64) -> bool {
        self.check_and_reset_daily_if_needed();
        
        let daily_budget = self.monthly_budget / 30.0;
        let current_spend = *self.current_daily_spend.read().unwrap();
        let projected_daily = current_spend + cost;
        
        // Allow 10% overage buffer
        projected_daily <= daily_budget * 1.1
    }

    /// Record a cost for a model
    pub fn record_cost(
        &self,
        model_id: &str,
        input_tokens: usize,
        output_tokens: usize,
        pricing: &ModelPricing,
    ) {
        self.check_and_reset_daily_if_needed();
        
        let input_cost = (input_tokens as f64 / 1_000_000.0) * pricing.input_per_million;
        let output_cost = (output_tokens as f64 / 1_000_000.0) * pricing.output_per_million;
        let total_cost = input_cost + output_cost;

        {
            let mut current_spend = self.current_daily_spend.write().unwrap();
            *current_spend += total_cost;
        }

        {
            let mut model_costs = self.model_costs.write().unwrap();
            *model_costs.entry(model_id.to_string()).or_insert(0.0) += total_cost;
        }

        debug!(
            "Recorded cost: ${:.4} for model {} ({} input + {} output tokens)",
            total_cost, model_id, input_tokens, output_tokens
        );
    }

    /// Get current daily spend
    pub fn get_daily_spend(&self) -> f64 {
        self.check_and_reset_daily_if_needed();
        *self.current_daily_spend.read().unwrap()
    }

    /// Get daily budget
    pub fn get_daily_budget(&self) -> f64 {
        self.monthly_budget / 30.0
    }

    /// Get monthly budget
    pub fn get_monthly_budget(&self) -> f64 {
        self.monthly_budget
    }

    /// Get remaining daily budget
    pub fn get_remaining_daily_budget(&self) -> f64 {
        let daily_budget = self.get_daily_budget();
        let current_spend = self.get_daily_spend();
        (daily_budget - current_spend).max(0.0)
    }

    /// Get cost breakdown by model
    pub fn get_model_breakdown(&self) -> HashMap<String, f64> {
        self.model_costs.read().unwrap().clone()
    }

    /// Get cost-effective alternative model from hierarchy
    pub fn get_cost_effective_alternative(
        &self,
        current_model: &ModelMetadata,
        hierarchy: &[&str],
    ) -> Option<String> {
        let current_cost = current_model.pricing.input_per_million;
        
        for model_id in hierarchy {
            if *model_id == current_model.model_id {
                continue;
            }
            
            // In a real implementation, we'd look up the model metadata
            // For now, we'll return the first alternative in hierarchy
            if let Some(alt_model) = crate::llm::models::ModelRegistry::get_model(model_id) {
                if alt_model.pricing.input_per_million < current_cost {
                    return Some(model_id.to_string());
                }
            }
        }
        
        None
    }

    /// Get spending statistics
    pub fn get_statistics(&self) -> CostStatistics {
        self.check_and_reset_daily_if_needed();
        
        let daily_spend = self.get_daily_spend();
        let daily_budget = self.get_daily_budget();
        let monthly_budget = self.get_monthly_budget();
        let remaining_daily = self.get_remaining_daily_budget();
        let model_breakdown = self.get_model_breakdown();
        
        // Estimate monthly projection
        let daily_avg = daily_spend;
        let projected_monthly = daily_avg * 30.0;
        let monthly_remaining = monthly_budget - projected_monthly;

        CostStatistics {
            daily_spend,
            daily_budget,
            remaining_daily,
            monthly_budget,
            projected_monthly,
            monthly_remaining,
            model_breakdown,
        }
    }

    /// Check if we need to reset daily tracking (new day)
    fn check_and_reset_daily_if_needed(&self) {
        let now = Utc::now();
        let today = now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();
        
        let mut current_date = self.current_date.write().unwrap();
        if today > *current_date {
            // New day - archive yesterday and reset
            let yesterday_spend = *self.current_daily_spend.read().unwrap();
            let model_breakdown = self.model_costs.read().unwrap().clone();
            
            {
                let mut daily_costs = self.daily_costs.write().unwrap();
                daily_costs.push(DailyCost {
                    date: *current_date,
                    total_spend: yesterday_spend,
                    request_count: 0, // Would need to track this separately
                    model_breakdown: model_breakdown.clone(),
                });
                
                // Keep only last 30 days
                if daily_costs.len() > 30 {
                    daily_costs.remove(0);
                }
            }
            
            // Reset for new day
            *self.current_daily_spend.write().unwrap() = 0.0;
            *self.model_costs.write().unwrap() = HashMap::new();
            *current_date = today;
            
            debug!("Reset daily cost tracking for new day: {}", today);
        }
    }

    /// Get historical daily costs
    pub fn get_historical_costs(&self, days: usize) -> Vec<DailyCost> {
        let daily_costs = self.daily_costs.read().unwrap();
        let start = daily_costs.len().saturating_sub(days);
        daily_costs[start..].to_vec()
    }

    /// Warn if approaching budget limits
    pub fn check_budget_warnings(&self) -> Vec<BudgetWarning> {
        let stats = self.get_statistics();
        let mut warnings = Vec::new();

        // Daily budget warnings
        let daily_usage_percent = (stats.daily_spend / stats.daily_budget) * 100.0;
        if daily_usage_percent >= 90.0 {
            warnings.push(BudgetWarning {
                level: WarningLevel::Critical,
                message: format!(
                    "Daily budget nearly exhausted: ${:.2}/{:.2} ({:.1}%)",
                    stats.daily_spend, stats.daily_budget, daily_usage_percent
                ),
            });
        } else if daily_usage_percent >= 75.0 {
            warnings.push(BudgetWarning {
                level: WarningLevel::Warning,
                message: format!(
                    "Daily budget at {}%: ${:.2}/{:.2}",
                    daily_usage_percent as u32, stats.daily_spend, stats.daily_budget
                ),
            });
        }

        // Monthly projection warnings
        let monthly_usage_percent = (stats.projected_monthly / stats.monthly_budget) * 100.0;
        if monthly_usage_percent >= 100.0 {
            warnings.push(BudgetWarning {
                level: WarningLevel::Critical,
                message: format!(
                    "Projected monthly spend exceeds budget: ${:.2} (projected) vs ${:.2} (budget)",
                    stats.projected_monthly, stats.monthly_budget
                ),
            });
        } else if monthly_usage_percent >= 90.0 {
            warnings.push(BudgetWarning {
                level: WarningLevel::Warning,
                message: format!(
                    "Projected monthly spend at {}%: ${:.2} (projected) vs ${:.2} (budget)",
                    monthly_usage_percent as u32, stats.projected_monthly, stats.monthly_budget
                ),
            });
        }

        warnings
    }
}

/// Cost statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostStatistics {
    pub daily_spend: f64,
    pub daily_budget: f64,
    pub remaining_daily: f64,
    pub monthly_budget: f64,
    pub projected_monthly: f64,
    pub monthly_remaining: f64,
    pub model_breakdown: HashMap<String, f64>,
}

/// Budget warning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetWarning {
    pub level: WarningLevel,
    pub message: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum WarningLevel {
    Info,
    Warning,
    Critical,
}

