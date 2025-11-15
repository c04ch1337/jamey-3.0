pub mod api;
pub mod cli;
pub mod config;
pub mod conscience;
pub mod consciousness;
pub mod db;
pub mod llm;
pub mod memory;
pub mod mqtt;
pub mod soul;

// Re-export commonly used types
pub use conscience::{ConscienceEngine, MoralRule};
pub use consciousness::ConsciousnessEngine;
pub use memory::{MemoryLayer, MemorySystem};
pub use config::{Config, ConsciousnessConfig, SoulConfig};
pub use llm::OpenRouterClient;
pub use mqtt::{MqttClient, MqttConfig, MqttError};
pub use soul::{Emotion, SoulEntity, SoulStorage, TrustCalculator, EmpathyScorer};

