pub mod api;
pub mod cli;
pub mod config;
pub mod conscience;
pub mod db;
pub mod health;
pub mod llm;
pub mod memory;
pub mod metrics;
pub mod mqtt;
pub mod phoenix;
pub mod soul;

// Re-export commonly used types
pub use conscience::{ConscienceEngine, MoralRule};
pub use memory::{MemoryLayer, MemorySystem};
pub use config::{Config, SecurityConfig};
pub use llm::OpenRouterClient;
pub use mqtt::{MqttClient, MqttConfig, MqttError};
pub use soul::{Emotion, SoulEntity, SoulStorage, TrustCalculator, EmpathyScorer};

