use crate::config::Config;
use crate::llm::cost_manager::{CostManager, CostStatistics};
use crate::llm::health::{HealthMonitor, HealthStatus, HealthSummary};
use crate::llm::models::{
    ModelMetadata, ModelRegistry, TaskRequirements,
};
use crate::llm::router::ModelRouter;
use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, error, info, warn};

/// OpenRouter API request/response types
#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
}

#[derive(Serialize, Deserialize, Clone)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
    usage: Option<Usage>,
}

#[derive(Deserialize)]
struct Choice {
    message: ChatMessage,
}

#[derive(Deserialize)]
struct Usage {
    prompt_tokens: usize,
    completion_tokens: usize,
    total_tokens: usize,
}

/// LLM response with metadata
#[derive(Debug, Clone)]
pub struct LLMResponse {
    pub content: String,
    pub model_used: String,
    pub input_tokens: usize,
    pub output_tokens: usize,
    pub cost: f64,
    pub response_time_ms: f64,
}

/// Main redundancy orchestrator for LLM providers
pub struct LLMRedundancyOrchestrator {
    config: Arc<Config>,
    client: Client,
    health_monitor: Arc<HealthMonitor>,
    cost_manager: Arc<CostManager>,
    router: ModelRouter,
    available_models: Vec<ModelMetadata>,
}

impl LLMRedundancyOrchestrator {
    /// Create a new redundancy orchestrator
    pub fn new(config: Arc<Config>, monthly_budget: f64) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .context("Failed to create HTTP client")?;

        let health_monitor = Arc::new(HealthMonitor::new(0.7, 10000.0)); // 70% success rate, 10s max response time
        let cost_manager = Arc::new(CostManager::new(monthly_budget));
        let router = ModelRouter::new(health_monitor.clone(), cost_manager.clone());

        // Load available models
        let available_models = ModelRegistry::get_all_models();

        info!(
            "Initialized LLM Redundancy Orchestrator with {} models, budget: ${}/month",
            available_models.len(),
            monthly_budget
        );

        Ok(Self {
            config,
            client,
            health_monitor,
            cost_manager,
            router,
            available_models,
        })
    }

    /// Send a chat completion request with automatic failover
    pub async fn chat(
        &self,
        messages: Vec<(String, String)>,
        task: Option<TaskRequirements>,
        temperature: Option<f32>,
    ) -> Result<LLMResponse> {
        self.chat_with_provider(messages, task, temperature, None).await
    }

    /// Send a chat completion request with a specific provider preference
    /// If preferred_provider is Some, it will try that model first before falling back
    pub async fn chat_with_provider(
        &self,
        messages: Vec<(String, String)>,
        task: Option<TaskRequirements>,
        temperature: Option<f32>,
        preferred_provider: Option<&str>,
    ) -> Result<LLMResponse> {
        let task = task.unwrap_or_else(TaskRequirements::default);
        let chat_messages: Vec<ChatMessage> = messages
            .into_iter()
            .map(|(role, content)| ChatMessage { role, content })
            .collect();

        // If preferred provider is specified, try to use it first
        let primary_model = if let Some(provider_id) = preferred_provider {
            // Try to find the preferred model in available models
            self.available_models
                .iter()
                .find(|m| m.model_id == provider_id)
                .cloned()
                .unwrap_or_else(|| {
                    // Fall back to routing if preferred model not found
                    self.router
                        .route_to_optimal_model(&task, &self.available_models)
                        .unwrap_or_else(|| self.available_models[0].clone())
                })
        } else {
            // Select optimal model using router
            self.router
                .route_to_optimal_model(&task, &self.available_models)
                .ok_or_else(|| anyhow::anyhow!("No available models for routing"))?
        };

        // Get fallback chain
        let fallback_chain = self.router.get_fallback_chain(&primary_model, &self.available_models);
        let mut models_to_try = vec![primary_model];
        models_to_try.extend(fallback_chain);

        info!("Attempting request with {} models in fallback chain", models_to_try.len());

        // Try models in order
        let mut last_error = None;
        for (attempt, model) in models_to_try.iter().enumerate() {
            debug!("Attempt {}: Trying model {}", attempt + 1, model.model_id);

            match self.try_model(&model, &chat_messages, temperature).await {
                Ok(response) => {
                    // Record success
                    self.health_monitor
                        .record_success(&model.model_id, response.response_time_ms);
                    
                    // Record cost
                    self.cost_manager.record_cost(
                        &model.model_id,
                        response.input_tokens,
                        response.output_tokens,
                        &model.pricing,
                    );

                    info!(
                        "Success with model {} (attempt {}): ${:.4}, {:.2}ms",
                        model.model_id, attempt + 1, response.cost, response.response_time_ms
                    );

                    return Ok(response);
                }
                Err(e) => {
                    // Record failure
                    self.health_monitor.record_failure(&model.model_id, &e.to_string());
                    last_error = Some(e);
                    warn!("Model {} failed: {}", model.model_id, last_error.as_ref().unwrap());
                    
                    // Continue to next model
                    continue;
                }
            }
        }

        // All models failed
        error!("All {} models failed", models_to_try.len());
        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("All models failed")))
    }

    /// Try a specific model
    async fn try_model(
        &self,
        model: &ModelMetadata,
        messages: &[ChatMessage],
        temperature: Option<f32>,
    ) -> Result<LLMResponse> {
        let start = Instant::now();

        let request = ChatRequest {
            model: model.model_id.clone(),
            messages: messages.to_vec(),
            temperature,
            max_tokens: Some(4000),
        };

        let url = format!("{}/chat/completions", self.config.core.openrouter_api_url);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.core.openrouter_api_key))
            .header("Content-Type", "application/json")
            .header("HTTP-Referer", "https://github.com/TransformArmyAI/Jamey-3.0")
            .header("X-Title", "Jamey 3.0 - General & Guardian")
            .json(&request)
            .send()
            .await
            .context("Failed to send request to OpenRouter")?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("OpenRouter API error: {} - {}", status, error_text);
        }

        let chat_response: ChatResponse = response
            .json()
            .await
            .context("Failed to parse response from OpenRouter")?;

        let response_time_ms = start.elapsed().as_millis() as f64;

        if let Some(choice) = chat_response.choices.first() {
            let usage = chat_response.usage.unwrap_or_else(|| Usage {
                prompt_tokens: 0,
                completion_tokens: 0,
                total_tokens: 0,
            });

            let cost = model.estimate_cost(usage.prompt_tokens, usage.completion_tokens);

            Ok(LLMResponse {
                content: choice.message.content.clone(),
                model_used: model.model_id.clone(),
                input_tokens: usage.prompt_tokens,
                output_tokens: usage.completion_tokens,
                cost,
                response_time_ms,
            })
        } else {
            anyhow::bail!("No response from OpenRouter API");
        }
    }

    /// Send a simple prompt
    pub async fn prompt(
        &self,
        prompt: &str,
        task: Option<TaskRequirements>,
    ) -> Result<LLMResponse> {
        self.chat(
            vec![("user".to_string(), prompt.to_string())],
            task,
            Some(0.7),
        )
        .await
    }

    /// Process a document using its preferred LLM provider (if specified)
    /// This method respects the document's preferred_llm_provider field
    pub async fn process_document(
        &self,
        document_content: &str,
        preferred_provider: Option<&str>,
        task: Option<TaskRequirements>,
        temperature: Option<f32>,
    ) -> Result<LLMResponse> {
        let messages = vec![
            ("system".to_string(), "You are processing a document. Analyze and respond appropriately.".to_string()),
            ("user".to_string(), document_content.to_string()),
        ];
        
        self.chat_with_provider(messages, task, temperature, preferred_provider).await
    }

    /// Get cost statistics
    pub fn get_cost_statistics(&self) -> CostStatistics {
        self.cost_manager.get_statistics()
    }

    /// Get health summary
    pub fn get_health_summary(&self) -> HealthSummary {
        self.health_monitor.get_health_summary()
    }

    /// Get available models
    pub fn get_available_models(&self) -> &[ModelMetadata] {
        &self.available_models
    }

    /// Check budget warnings
    pub fn check_budget_warnings(&self) -> Vec<crate::llm::cost_manager::BudgetWarning> {
        self.cost_manager.check_budget_warnings()
    }

    /// Get recommended model for a task type
    pub fn get_recommended_model(&self, task_type: &str) -> Option<ModelMetadata> {
        let recommended = self
            .router
            .get_recommended_models(task_type, &self.available_models);
        recommended.first().cloned()
    }
}

/// Convenience function to create orchestrator from config
pub fn create_orchestrator(config: Arc<Config>) -> Result<LLMRedundancyOrchestrator> {
    // Default monthly budget: $1000, can be overridden via env var
    let monthly_budget = std::env::var("LLM_MONTHLY_BUDGET")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1000.0);

    LLMRedundancyOrchestrator::new(config, monthly_budget)
}

