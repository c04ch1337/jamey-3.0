use crate::llm::cost_manager::CostManager;
use crate::llm::health::HealthMonitor;
use crate::llm::models::{ModelMetadata, TaskRequirements};
use std::sync::Arc;
use tracing::{debug, warn};

/// Model fitness score for task matching
#[derive(Debug, Clone)]
pub struct ModelFitness {
    pub model: ModelMetadata,
    pub score: f64,
    pub breakdown: FitnessBreakdown,
}

#[derive(Debug, Clone)]
pub struct FitnessBreakdown {
    pub capability_score: f64,
    pub context_score: f64,
    pub cost_score: f64,
    pub health_score: f64,
}

/// Intelligent model router for selecting optimal models
pub struct ModelRouter {
    health_monitor: Arc<HealthMonitor>,
    cost_manager: Arc<CostManager>,
}

impl ModelRouter {
    /// Create a new model router
    pub fn new(health_monitor: Arc<HealthMonitor>, cost_manager: Arc<CostManager>) -> Self {
        Self {
            health_monitor,
            cost_manager,
        }
    }

    /// Route to optimal model based on task requirements
    pub fn route_to_optimal_model(
        &self,
        task: &TaskRequirements,
        available_models: &[ModelMetadata],
    ) -> Option<ModelMetadata> {
        if available_models.is_empty() {
            warn!("No available models for routing");
            return None;
        }

        // Filter by health and availability
        let healthy_models = self.health_monitor.filter_available(available_models);
        if healthy_models.is_empty() {
            warn!("No healthy models available, falling back to all models");
            // Fall back to all models if none are healthy
            return available_models.first().cloned();
        }

        // Score all models
        let mut fitness_scores: Vec<ModelFitness> = healthy_models
            .iter()
            .map(|model| self.calculate_fitness(model, task))
            .collect();

        // Sort by score (highest first)
        fitness_scores.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        if let Some(best) = fitness_scores.first() {
            debug!(
                "Selected model: {} with fitness score: {:.2}",
                best.model.model_id, best.score
            );
            debug!(
                "Fitness breakdown: capability={:.2}, context={:.2}, cost={:.2}, health={:.2}",
                best.breakdown.capability_score,
                best.breakdown.context_score,
                best.breakdown.cost_score,
                best.breakdown.health_score
            );
            Some(best.model.clone())
        } else {
            None
        }
    }

    /// Calculate fitness score for a model given task requirements
    fn calculate_fitness(&self, model: &ModelMetadata, task: &TaskRequirements) -> ModelFitness {
        let capability_score = self.calculate_capability_score(model, task);
        let context_score = self.calculate_context_score(model, task);
        let cost_score = self.calculate_cost_score(model, task);
        let health_score = self.health_monitor.get_health_score(&model.model_id);

        // Weighted combination
        let total_score = (capability_score * 0.40)
            + (context_score * 0.15)
            + (cost_score * 0.25)
            + (health_score * 0.20);

        ModelFitness {
            model: model.clone(),
            score: total_score,
            breakdown: FitnessBreakdown {
                capability_score,
                context_score,
                cost_score,
                health_score,
            },
        }
    }

    /// Calculate capability matching score
    fn calculate_capability_score(&self, model: &ModelMetadata, task: &TaskRequirements) -> f64 {
        let mut score = 0.0;
        let mut weight_sum = 0.0;

        if task.requires_reasoning {
            score += model.capabilities.reasoning * 40.0;
            weight_sum += 40.0;
        }
        if task.requires_creativity {
            score += model.capabilities.creativity * 30.0;
            weight_sum += 30.0;
        }
        if task.requires_speed {
            score += model.capabilities.speed * 20.0;
            weight_sum += 20.0;
        }
        if task.requires_tool_use {
            score += model.capabilities.tool_use * 25.0;
            weight_sum += 25.0;
        }
        if task.requires_multimodal {
            score += model.capabilities.multimodal * 20.0;
            weight_sum += 20.0;
        }
        if task.requires_math {
            score += model.capabilities.math * 15.0;
            weight_sum += 15.0;
        }
        if task.requires_multilingual {
            score += model.capabilities.multilingual * 10.0;
            weight_sum += 10.0;
        }

        if weight_sum > 0.0 {
            score / weight_sum
        } else {
            // If no specific requirements, give neutral score
            0.5
        }
    }

    /// Calculate context length adequacy score
    fn calculate_context_score(&self, model: &ModelMetadata, task: &TaskRequirements) -> f64 {
        if model.context_length >= task.context_length {
            // Bonus for having more context than needed
            let excess = model.context_length - task.context_length;
            let excess_ratio = (excess as f64 / task.context_length as f64).min(1.0);
            1.0 + (excess_ratio * 0.2) // Up to 20% bonus
        } else {
            // Penalty for insufficient context
            let shortfall = task.context_length - model.context_length;
            let shortfall_ratio = (shortfall as f64 / task.context_length as f64).min(1.0);
            (1.0 - shortfall_ratio).max(0.0)
        }
    }

    /// Calculate cost efficiency score
    fn calculate_cost_score(&self, model: &ModelMetadata, task: &TaskRequirements) -> f64 {
        // Estimate token usage (rough approximation)
        let estimated_input_tokens = task.context_length;
        let estimated_output_tokens = estimated_input_tokens / 4; // Assume 1/4 output ratio

        let estimated_cost = model.estimate_cost(estimated_input_tokens, estimated_output_tokens);

        // Check if we can afford it
        let can_afford = self.cost_manager.can_afford_model(
            model,
            estimated_input_tokens,
            estimated_output_tokens,
        );

        if !can_afford {
            return 0.0; // Can't afford = 0 score
        }

        // Check against budget if specified
        if let Some(max_budget) = task.max_budget {
            if estimated_cost > max_budget {
                // Penalty for exceeding task budget
                let overage_ratio = (estimated_cost / max_budget).min(2.0);
                return (1.0 / overage_ratio).max(0.0);
            }
        }

        // Score based on cost efficiency (lower cost = higher score)
        // Normalize against most expensive model (assume $100/M tokens as max)
        let max_cost_per_million = 100.0;
        let normalized_input_cost = model.pricing.input_per_million / max_cost_per_million;
        let normalized_output_cost = model.pricing.output_per_million / max_cost_per_million;
        let avg_normalized_cost = (normalized_input_cost + normalized_output_cost) / 2.0;

        // Invert so lower cost = higher score
        1.0 - (avg_normalized_cost.min(1.0))
    }

    /// Get fallback models in priority order
    pub fn get_fallback_chain(
        &self,
        primary_model: &ModelMetadata,
        all_models: &[ModelMetadata],
    ) -> Vec<ModelMetadata> {
        let mut fallbacks = Vec::new();
        let mut available = self.health_monitor.filter_available(all_models);

        // Remove primary model
        available.retain(|m| m.model_id != primary_model.model_id);

        // Sort by priority tier and health
        available.sort_by(|a, b| {
            // First by priority tier
            let tier_cmp = a.priority_tier.cmp(&b.priority_tier);
            if tier_cmp != std::cmp::Ordering::Equal {
                return tier_cmp;
            }
            // Then by health score
            let health_a = self.health_monitor.get_health_score(&a.model_id);
            let health_b = self.health_monitor.get_health_score(&b.model_id);
            health_b.partial_cmp(&health_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        fallbacks.extend(available);
        fallbacks
    }

    /// Get recommended models for a task type
    pub fn get_recommended_models(
        &self,
        task_type: &str,
        all_models: &[ModelMetadata],
    ) -> Vec<ModelMetadata> {
        let healthy_models = self.health_monitor.filter_available(all_models);
        
        // Filter by use case
        let mut matching: Vec<ModelMetadata> = healthy_models
            .into_iter()
            .filter(|m| {
                m.use_cases
                    .iter()
                    .any(|uc| uc.to_lowercase().contains(&task_type.to_lowercase()))
            })
            .collect();

        // Sort by priority tier and health
        matching.sort_by(|a, b| {
            let tier_cmp = a.priority_tier.cmp(&b.priority_tier);
            if tier_cmp != std::cmp::Ordering::Equal {
                return tier_cmp;
            }
            let health_a = self.health_monitor.get_health_score(&a.model_id);
            let health_b = self.health_monitor.get_health_score(&b.model_id);
            health_b.partial_cmp(&health_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        matching
    }
}

