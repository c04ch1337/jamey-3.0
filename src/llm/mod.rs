use crate::config::Config;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// OpenRouter API client for LLM interactions
pub struct OpenRouterClient {
    config: Arc<Config>,
    client: reqwest::Client,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

#[derive(Serialize, Deserialize, Clone)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ChatMessage,
}

impl OpenRouterClient {
    /// Create a new OpenRouter client
    pub fn new(config: Arc<Config>) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .expect("Failed to create HTTP client");

        Self { config, client }
    }

    /// Send a chat completion request to OpenRouter
    pub async fn chat(
        &self,
        messages: Vec<(String, String)>, // (role, content)
        temperature: Option<f32>,
    ) -> anyhow::Result<String> {
        let chat_messages: Vec<ChatMessage> = messages
            .into_iter()
            .map(|(role, content)| ChatMessage { role, content })
            .collect();

        let request = ChatRequest {
            model: self.config.openrouter_model.clone(),
            messages: chat_messages,
            temperature,
        };

        let url = format!("{}/chat/completions", self.config.openrouter_api_url);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.openrouter_api_key))
            .header("Content-Type", "application/json")
            .header("HTTP-Referer", "https://github.com/TransformArmyAI/Jamey-3.0")
            .header("X-Title", "Jamey 3.0 - General & Guardian")
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("OpenRouter API error: {} - {}", status, error_text);
        }

        let chat_response: ChatResponse = response.json().await?;

        if let Some(choice) = chat_response.choices.first() {
            Ok(choice.message.content.clone())
        } else {
            anyhow::bail!("No response from OpenRouter API");
        }
    }

    /// Send a simple prompt and get a response
    pub async fn prompt(&self, prompt: &str) -> anyhow::Result<String> {
        self.chat(
            vec![("user".to_string(), prompt.to_string())],
            Some(0.7),
        )
        .await
    }
}

