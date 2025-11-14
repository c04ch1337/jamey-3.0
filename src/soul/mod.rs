//! Soul Knowledge Base module
//! 
//! Tracks entities with emotional states and trust scores.

pub mod emotion;
pub mod entity;
pub mod storage;
pub mod trust;
pub mod empathy;
pub mod integration;

pub use emotion::Emotion;
pub use entity::SoulEntity;
pub use storage::SoulStorage;
pub use trust::TrustCalculator;
pub use empathy::EmpathyScorer;
pub use integration::*;