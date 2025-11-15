pub mod api;
<<<<<<< HEAD
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
=======
pub mod backup;
pub mod telemetry;
pub mod cli;
pub mod config;
pub mod conscience;
pub mod consciousness;
pub mod db;
pub mod llm;
pub mod memory;
pub mod mqtt;
pub mod security;
>>>>>>> origin/main
pub mod soul;

// Re-export commonly used types
pub use conscience::{ConscienceEngine, MoralRule};
<<<<<<< HEAD
pub use memory::{MemoryLayer, MemorySystem};
pub use config::{Config, SecurityConfig};
pub use llm::OpenRouterClient;
pub use mqtt::{MqttClient, MqttConfig, MqttError};
pub use soul::{Emotion, SoulEntity, SoulStorage, TrustCalculator, EmpathyScorer};
=======
pub use consciousness::ConsciousnessEngine;
pub use memory::{MemoryLayer, MemorySystem};
pub use config::{Config, ConsciousnessConfig, SoulConfig};
pub use llm::OpenRouterClient;
pub use mqtt::{MqttClient, MqttConfig, MqttError};
pub use soul::{Emotion, SoulEntity, SoulStorage, TrustCalculator, EmpathyScorer};
pub use security::{JwtAuth, JwtClaims, AuthError, SecurityHeadersLayer, RateLimitLayer};
pub use backup::{BackupManager, BackupConfig, BackupManifest, BackupResult, RestoreManager, RestoreResult, BackupScheduler, ScheduleConfig};
>>>>>>> origin/main

