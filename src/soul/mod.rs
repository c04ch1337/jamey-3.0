//! Soul Knowledge Base module
<<<<<<< HEAD
//!
=======
//! 
>>>>>>> origin/main
//! Tracks entities with emotional states and trust scores.

pub mod emotion;
pub mod entity;
pub mod storage;
pub mod trust;
pub mod empathy;
pub mod integration;
<<<<<<< HEAD
pub mod config;
=======
>>>>>>> origin/main

pub use emotion::Emotion;
pub use entity::SoulEntity;
pub use storage::SoulStorage;
pub use trust::TrustCalculator;
pub use empathy::EmpathyScorer;
<<<<<<< HEAD
pub use integration::*;
pub use config::SoulConfig;

#[derive(Debug, thiserror::Error)]
pub enum SoulError {
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Storage error: {0}")]
    Storage(#[from] sqlx::Error),
    
    #[error("Invalid value: {0}")]
    InvalidValue(String),
}
=======
pub use integration::*;
>>>>>>> origin/main
