use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Model capabilities for task matching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCapabilities {
    /// Strong reasoning capabilities (0.0 to 1.0)
    pub reasoning: f32,
    /// Creative generation capabilities (0.0 to 1.0)
    pub creativity: f32,
    /// Speed/latency score (0.0 to 1.0, higher = faster)
    pub speed: f32,
    /// Tool use and API calling capabilities (0.0 to 1.0)
    pub tool_use: f32,
    /// Multi-modal capabilities (0.0 to 1.0)
    pub multimodal: f32,
    /// Mathematical reasoning (0.0 to 1.0)
    pub math: f32,
    /// Multilingual capabilities (0.0 to 1.0)
    pub multilingual: f32,
}

impl Default for ModelCapabilities {
    fn default() -> Self {
        Self {
            reasoning: 0.5,
            creativity: 0.5,
            speed: 0.5,
            tool_use: 0.5,
            multimodal: 0.0,
            math: 0.5,
            multilingual: 0.5,
        }
    }
}

/// Pricing information for a model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPricing {
    /// Cost per million input tokens (USD)
    pub input_per_million: f64,
    /// Cost per million output tokens (USD)
    pub output_per_million: f64,
}

/// Model metadata and configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    /// OpenRouter model ID
    pub model_id: String,
    /// Human-readable name
    pub name: String,
    /// Maximum context length in tokens
    pub context_length: usize,
    /// Model capabilities
    pub capabilities: ModelCapabilities,
    /// Pricing information
    pub pricing: ModelPricing,
    /// Recommended use cases
    pub use_cases: Vec<String>,
    /// Priority tier (1 = highest, 10 = lowest)
    pub priority_tier: u8,
    /// Whether this model is currently available
    pub available: bool,
}

impl ModelMetadata {
    /// Estimate cost for a request
    pub fn estimate_cost(&self, input_tokens: usize, output_tokens: usize) -> f64 {
        let input_cost = (input_tokens as f64 / 1_000_000.0) * self.pricing.input_per_million;
        let output_cost = (output_tokens as f64 / 1_000_000.0) * self.pricing.output_per_million;
        input_cost + output_cost
    }
}

/// Predefined model configurations for top 10 OpenRouter models
pub struct ModelRegistry;

impl ModelRegistry {
    /// Get all predefined models
    pub fn get_all_models() -> Vec<ModelMetadata> {
        vec![
            // 1. Claude 3 Opus - Best for complex reasoning
            ModelMetadata {
                model_id: "anthropic/claude-3-opus".to_string(),
                name: "Claude 3 Opus".to_string(),
                context_length: 200_000,
                capabilities: ModelCapabilities {
                    reasoning: 1.0,
                    creativity: 0.95,
                    speed: 0.4,
                    tool_use: 0.9,
                    multimodal: 0.8,
                    math: 0.95,
                    multilingual: 0.85,
                },
                pricing: ModelPricing {
                    input_per_million: 15.0,
                    output_per_million: 75.0,
                },
                use_cases: vec![
                    "Complex reasoning".to_string(),
                    "Strategic planning".to_string(),
                    "Consciousness simulation".to_string(),
                    "Jamey 3.0 Core Consciousness".to_string(),
                ],
                priority_tier: 1,
                available: true,
            },
            // 2. GPT-4 Turbo - General intelligence
            ModelMetadata {
                model_id: "openai/gpt-4-turbo".to_string(),
                name: "GPT-4 Turbo".to_string(),
                context_length: 128_000,
                capabilities: ModelCapabilities {
                    reasoning: 0.95,
                    creativity: 0.95,
                    speed: 0.6,
                    tool_use: 0.95,
                    multimodal: 0.9,
                    math: 0.9,
                    multilingual: 0.8,
                },
                pricing: ModelPricing {
                    input_per_million: 10.0,
                    output_per_million: 30.0,
                },
                use_cases: vec![
                    "General intelligence".to_string(),
                    "Creative tasks".to_string(),
                    "Multi-domain knowledge".to_string(),
                    "Phoenix.Marie Core Processing".to_string(),
                ],
                priority_tier: 1,
                available: true,
            },
            // 3. Claude 3 Sonnet - Cost-effective advanced reasoning
            ModelMetadata {
                model_id: "anthropic/claude-3-sonnet".to_string(),
                name: "Claude 3 Sonnet".to_string(),
                context_length: 200_000,
                capabilities: ModelCapabilities {
                    reasoning: 0.85,
                    creativity: 0.85,
                    speed: 0.7,
                    tool_use: 0.85,
                    multimodal: 0.8,
                    math: 0.85,
                    multilingual: 0.85,
                },
                pricing: ModelPricing {
                    input_per_million: 3.0,
                    output_per_million: 15.0,
                },
                use_cases: vec![
                    "Cost-effective advanced reasoning".to_string(),
                    "Operational intelligence".to_string(),
                    "ORCH Command Network".to_string(),
                ],
                priority_tier: 2,
                available: true,
            },
            // 4. Gemini Pro 1.5 - Massive context
            ModelMetadata {
                model_id: "google/gemini-pro-1.5".to_string(),
                name: "Gemini Pro 1.5".to_string(),
                context_length: 1_000_000,
                capabilities: ModelCapabilities {
                    reasoning: 0.9,
                    creativity: 0.85,
                    speed: 0.5,
                    tool_use: 0.8,
                    multimodal: 0.9,
                    math: 0.9,
                    multilingual: 0.9,
                },
                pricing: ModelPricing {
                    input_per_million: 1.25,
                    output_per_million: 5.0,
                },
                use_cases: vec![
                    "Massive context".to_string(),
                    "Multi-modal reasoning".to_string(),
                    "Lifelong learning systems".to_string(),
                    "Massive memory integration".to_string(),
                ],
                priority_tier: 2,
                available: true,
            },
            // 5. GPT-4 Vision - Multi-modal
            ModelMetadata {
                model_id: "openai/gpt-4-vision-preview".to_string(),
                name: "GPT-4 Vision".to_string(),
                context_length: 128_000,
                capabilities: ModelCapabilities {
                    reasoning: 0.9,
                    creativity: 0.9,
                    speed: 0.5,
                    tool_use: 0.85,
                    multimodal: 1.0,
                    math: 0.85,
                    multilingual: 0.75,
                },
                pricing: ModelPricing {
                    input_per_million: 10.0,
                    output_per_million: 30.0,
                },
                use_cases: vec![
                    "Multi-modal understanding".to_string(),
                    "Visual reasoning".to_string(),
                    "Embodiment".to_string(),
                ],
                priority_tier: 3,
                available: true,
            },
            // 6. Claude 3 Haiku - Real-time processing
            ModelMetadata {
                model_id: "anthropic/claude-3-haiku".to_string(),
                name: "Claude 3 Haiku".to_string(),
                context_length: 200_000,
                capabilities: ModelCapabilities {
                    reasoning: 0.7,
                    creativity: 0.7,
                    speed: 1.0,
                    tool_use: 0.7,
                    multimodal: 0.7,
                    math: 0.7,
                    multilingual: 0.75,
                },
                pricing: ModelPricing {
                    input_per_million: 0.25,
                    output_per_million: 1.25,
                },
                use_cases: vec![
                    "Real-time processing".to_string(),
                    "Low-latency responses".to_string(),
                    "Operational tasks".to_string(),
                    "Voice processing".to_string(),
                ],
                priority_tier: 3,
                available: true,
            },
            // 7. Mixtral 8x22B - Open-weight alternative
            ModelMetadata {
                model_id: "mistralai/mixtral-8x22b".to_string(),
                name: "Mixtral 8x22B".to_string(),
                context_length: 64_000,
                capabilities: ModelCapabilities {
                    reasoning: 0.8,
                    creativity: 0.8,
                    speed: 0.6,
                    tool_use: 0.75,
                    multimodal: 0.0,
                    math: 0.8,
                    multilingual: 0.8,
                },
                pricing: ModelPricing {
                    input_per_million: 2.0,
                    output_per_million: 6.0,
                },
                use_cases: vec![
                    "Open-weight alternative".to_string(),
                    "Cost-effective scale".to_string(),
                    "Experimental architectures".to_string(),
                ],
                priority_tier: 4,
                available: true,
            },
            // 8. Command R+ - Tool use
            ModelMetadata {
                model_id: "cohere/command-r-plus".to_string(),
                name: "Command R+".to_string(),
                context_length: 128_000,
                capabilities: ModelCapabilities {
                    reasoning: 0.75,
                    creativity: 0.7,
                    speed: 0.7,
                    tool_use: 1.0,
                    multimodal: 0.0,
                    math: 0.7,
                    multilingual: 0.8,
                },
                pricing: ModelPricing {
                    input_per_million: 3.0,
                    output_per_million: 15.0,
                },
                use_cases: vec![
                    "Tool use".to_string(),
                    "API calling".to_string(),
                    "Operational automation".to_string(),
                    "System automation".to_string(),
                ],
                priority_tier: 4,
                available: true,
            },
            // 9. Llama 3 70B - Open-source foundation
            ModelMetadata {
                model_id: "meta-llama/llama-3-70b-instruct".to_string(),
                name: "Llama 3 70B".to_string(),
                context_length: 8_000,
                capabilities: ModelCapabilities {
                    reasoning: 0.75,
                    creativity: 0.75,
                    speed: 0.7,
                    tool_use: 0.7,
                    multimodal: 0.0,
                    math: 0.75,
                    multilingual: 0.7,
                },
                pricing: ModelPricing {
                    input_per_million: 0.9,
                    output_per_million: 0.9,
                },
                use_cases: vec![
                    "Open-source foundation".to_string(),
                    "Customizable".to_string(),
                    "Cost-effective".to_string(),
                    "Experimental features".to_string(),
                ],
                priority_tier: 5,
                available: true,
            },
            // 10. Qwen 2 72B - Multilingual & math
            ModelMetadata {
                model_id: "qwen/qwen-2-72b-instruct".to_string(),
                name: "Qwen 2 72B".to_string(),
                context_length: 32_000,
                capabilities: ModelCapabilities {
                    reasoning: 0.8,
                    creativity: 0.75,
                    speed: 0.7,
                    tool_use: 0.75,
                    multimodal: 0.0,
                    math: 0.9,
                    multilingual: 0.95,
                },
                pricing: ModelPricing {
                    input_per_million: 1.5,
                    output_per_million: 1.5,
                },
                use_cases: vec![
                    "Multilingual capabilities".to_string(),
                    "Mathematical reasoning".to_string(),
                    "Analytical tasks".to_string(),
                    "International operations".to_string(),
                ],
                priority_tier: 5,
                available: true,
            },
        ]
    }

    /// Get model by ID
    pub fn get_model(model_id: &str) -> Option<ModelMetadata> {
        Self::get_all_models()
            .into_iter()
            .find(|m| m.model_id == model_id)
    }

    /// Get models by priority tier
    pub fn get_models_by_tier(tier: u8) -> Vec<ModelMetadata> {
        Self::get_all_models()
            .into_iter()
            .filter(|m| m.priority_tier == tier)
            .collect()
    }

    /// Get models suitable for a specific use case
    pub fn get_models_for_use_case(use_case: &str) -> Vec<ModelMetadata> {
        Self::get_all_models()
            .into_iter()
            .filter(|m| {
                m.use_cases
                    .iter()
                    .any(|uc| uc.to_lowercase().contains(&use_case.to_lowercase()))
            })
            .collect()
    }
}

/// Task requirements for model selection
#[derive(Debug, Clone)]
pub struct TaskRequirements {
    /// Requires strong reasoning
    pub requires_reasoning: bool,
    /// Requires creativity
    pub requires_creativity: bool,
    /// Requires speed/low latency
    pub requires_speed: bool,
    /// Requires tool use capabilities
    pub requires_tool_use: bool,
    /// Requires multi-modal capabilities
    pub requires_multimodal: bool,
    /// Requires mathematical reasoning
    pub requires_math: bool,
    /// Requires multilingual capabilities
    pub requires_multilingual: bool,
    /// Estimated context length needed
    pub context_length: usize,
    /// Maximum budget for this request (USD)
    pub max_budget: Option<f64>,
    /// Task type identifier
    pub task_type: String,
}

impl Default for TaskRequirements {
    fn default() -> Self {
        Self {
            requires_reasoning: false,
            requires_creativity: false,
            requires_speed: false,
            requires_tool_use: false,
            requires_multimodal: false,
            requires_math: false,
            requires_multilingual: false,
            context_length: 4_000,
            max_budget: None,
            task_type: "general".to_string(),
        }
    }
}

/// Predefined task types for common use cases
impl TaskRequirements {
    /// Jamey 3.0 Core Consciousness task
    pub fn jamey_core_consciousness() -> Self {
        Self {
            requires_reasoning: true,
            requires_creativity: true,
            requires_speed: false,
            requires_tool_use: false,
            requires_multimodal: false,
            requires_math: false,
            requires_multilingual: false,
            context_length: 16_000,
            max_budget: Some(0.50),
            task_type: "jamey_core_consciousness".to_string(),
        }
    }

    /// Phoenix.Marie Core Processing task
    pub fn phoenix_core_processing() -> Self {
        Self {
            requires_reasoning: true,
            requires_creativity: true,
            requires_speed: false,
            requires_tool_use: false,
            requires_multimodal: false,
            requires_math: false,
            requires_multilingual: false,
            context_length: 16_000,
            max_budget: Some(0.50),
            task_type: "phoenix_core_processing".to_string(),
        }
    }

    /// ORCH Command Network task
    pub fn orch_command_network() -> Self {
        Self {
            requires_reasoning: true,
            requires_creativity: false,
            requires_speed: true,
            requires_tool_use: true,
            requires_multimodal: false,
            requires_math: false,
            requires_multilingual: false,
            context_length: 8_000,
            max_budget: Some(0.20),
            task_type: "orch_command_network".to_string(),
        }
    }

    /// Real-time voice processing task
    pub fn realtime_voice() -> Self {
        Self {
            requires_reasoning: false,
            requires_creativity: false,
            requires_speed: true,
            requires_tool_use: false,
            requires_multimodal: false,
            requires_math: false,
            requires_multilingual: false,
            context_length: 4_000,
            max_budget: Some(0.05),
            task_type: "realtime_voice".to_string(),
        }
    }

    /// Massive context memory integration
    pub fn massive_context() -> Self {
        Self {
            requires_reasoning: true,
            requires_creativity: false,
            requires_speed: false,
            requires_tool_use: false,
            requires_multimodal: false,
            requires_math: false,
            requires_multilingual: false,
            context_length: 500_000,
            max_budget: Some(1.00),
            task_type: "massive_context".to_string(),
        }
    }
}

