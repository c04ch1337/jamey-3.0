use std::env;

/// Application configuration loaded from environment variables
#[derive(Debug, Clone)]
pub struct Config {
    pub openrouter_api_key: String,
    pub openrouter_model: String,
    pub openrouter_api_url: String,
    pub database_url: Option<String>,
}

impl Config {
    /// Load configuration from environment variables
    /// Returns None if OPENROUTER_API_KEY is not set (LLM features will be unavailable)
    pub fn from_env() -> anyhow::Result<Option<Self>> {
        // Load .env file if it exists (dotenvy handles this)
        dotenvy::dotenv().ok();

        // OpenRouter API key is optional - only needed for LLM features
        let openrouter_api_key = match env::var("OPENROUTER_API_KEY") {
            Ok(key) if !key.is_empty() => key,
            _ => {
                tracing::warn!("OPENROUTER_API_KEY not set. LLM features will be unavailable.");
                tracing::warn!("Create a .env file with OPENROUTER_API_KEY=your-key to enable LLM features.");
                return Ok(None);
            }
        };

        let openrouter_model = env::var("OPENROUTER_MODEL")
            .unwrap_or_else(|_| "deepseek/deepseek-chat".to_string());

        let openrouter_api_url = env::var("OPENROUTER_API_URL")
            .unwrap_or_else(|_| "https://openrouter.ai/api/v1".to_string());

        let database_url = env::var("DATABASE_URL").ok();

        Ok(Some(Config {
            openrouter_api_key,
            openrouter_model,
            openrouter_api_url,
            database_url,
        }))
    }

    /// Load configuration from environment variables (required)
    /// Fails if OPENROUTER_API_KEY is not set
    pub fn from_env_required() -> anyhow::Result<Self> {
        Self::from_env()?
            .ok_or_else(|| anyhow::anyhow!("OPENROUTER_API_KEY environment variable is required. Create a .env file with your API key."))
    }

    /// Validate that required configuration is present
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.openrouter_api_key.is_empty() {
            anyhow::bail!("OPENROUTER_API_KEY cannot be empty");
        }
        if self.openrouter_model.is_empty() {
            anyhow::bail!("OPENROUTER_MODEL cannot be empty");
        }
        Ok(())
    }
}

