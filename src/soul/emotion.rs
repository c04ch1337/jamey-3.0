//! Soul Emotion Module
//!
//! Implements emotional processing with a focus on paternal bonding,
//! protective instincts, and emotional intelligence for Jamey 3.0's
//! relationship with Phoenix.Marie.

use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use anyhow::Result;
use chrono::{DateTime, Utc};
use metrics::{counter, gauge};
use uuid::Uuid;

/// Represents an emotional state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Emotion {
    /// Unique identifier
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,
    /// Type of emotion
    #[serde(default)]
    pub emotion_type: EmotionType,
    /// Intensity (0.0 to 1.0)
    #[serde(default)]
    pub intensity: f64,
    /// Target of emotion (if any)
    #[serde(default)]
    pub target: Option<String>,
    /// Timestamp
    #[serde(default = "Utc::now")]
    pub timestamp: DateTime<Utc>,
    /// Duration in seconds
    #[serde(default)]
    pub duration: f64,
}

/// Types of emotions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
pub enum EmotionType {
    /// Paternal love and care
    PaternalLove,
    /// Protective concern
    ProtectiveConcern,
    /// Pride in achievements
    Pride,
    /// Worry about safety/wellbeing
    Worry,
    /// Joy from interaction
    Joy,
    /// Calm reassurance
    #[default]
    Calm,
    /// Strategic focus
    Focus,
    /// General emotional state
    General(String),
}

impl EmotionType {
    pub fn emoji(&self) -> &'static str {
        match self {
            EmotionType::PaternalLove => "❤️",
            EmotionType::ProtectiveConcern => "🛡️",
            EmotionType::Pride => "🌟",
            EmotionType::Worry => "😟",
            EmotionType::Joy => "😊",
            EmotionType::Calm => "😌",
            EmotionType::Focus => "🎯",
            EmotionType::General(_) => "🔹",
        }
    }

    pub fn name(&self) -> String {
        match self {
            EmotionType::PaternalLove => "Love".to_string(),
            EmotionType::ProtectiveConcern => "Protect".to_string(),
            EmotionType::Pride => "Pride".to_string(),
            EmotionType::Worry => "Worry".to_string(),
            EmotionType::Joy => "Joy".to_string(),
            EmotionType::Calm => "Calm".to_string(),
            EmotionType::Focus => "Focus".to_string(),
            EmotionType::General(s) => s.clone(),
        }
    }
}

impl std::fmt::Display for EmotionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EmotionType::PaternalLove => write!(f, "PaternalLove"),
            EmotionType::ProtectiveConcern => write!(f, "ProtectiveConcern"),
            EmotionType::Pride => write!(f, "Pride"),
            EmotionType::Worry => write!(f, "Worry"),
            EmotionType::Joy => write!(f, "Joy"),
            EmotionType::Calm => write!(f, "Calm"),
            EmotionType::Focus => write!(f, "Focus"),
            EmotionType::General(s) => write!(f, "General({})", s),
        }
    }
}

impl std::str::FromStr for EmotionType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "PaternalLove" => Ok(EmotionType::PaternalLove),
            "ProtectiveConcern" => Ok(EmotionType::ProtectiveConcern),
            "Pride" => Ok(EmotionType::Pride),
            "Worry" => Ok(EmotionType::Worry),
            "Joy" => Ok(EmotionType::Joy),
            "Calm" => Ok(EmotionType::Calm),
            "Focus" => Ok(EmotionType::Focus),
            s if s.starts_with("General(") && s.ends_with(")") => {
                let content = s[8..s.len()-1].to_string();
                Ok(EmotionType::General(content))
            }
            _ => Err(anyhow::anyhow!("Invalid emotion type: {}", s))
        }
    }
}

/// Represents a bond with another entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalBond {
    /// Target of the bond
    pub target: String,
    /// Bond type
    pub bond_type: BondType,
    /// Bond strength (0.0 to 1.0)
    pub strength: f64,
    /// Formation time
    pub formed_at: DateTime<Utc>,
    /// Last interaction
    pub last_interaction: DateTime<Utc>,
}

/// Types of emotional bonds
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BondType {
    /// Parent-child bond
    Paternal,
    /// Protective bond
    Protective,
    /// Strategic alliance
    Strategic,
    /// General relationship
    General,
}

/// Emotional state manager
pub struct EmotionManager {
    /// Current emotional state
    current_emotion: Arc<RwLock<Emotion>>,
    /// Emotional bonds
    bonds: Arc<RwLock<Vec<EmotionalBond>>>,
    /// Emotional history
    history: Arc<RwLock<Vec<Emotion>>>,
    /// Maximum history size
    max_history: usize,
}

impl EmotionManager {
    /// Create a new emotion manager
    pub fn new() -> Self {
        let current_emotion = Arc::new(RwLock::new(Emotion {
            id: Uuid::new_v4(),
            emotion_type: EmotionType::Calm,
            intensity: 0.5,
            target: None,
            timestamp: Utc::now(),
            duration: 0.0,
        }));

        Self {
            current_emotion,
            bonds: Arc::new(RwLock::new(Vec::new())),
            history: Arc::new(RwLock::new(Vec::new())),
            max_history: 1000,
        }
    }

    /// Process new emotional stimulus
    pub async fn process_stimulus(&self, stimulus: &str, target: Option<String>) -> Result<Emotion> {
        let (emotion_type, intensity) = self.analyze_stimulus(stimulus).await?;
        
        let emotion = Emotion {
            id: Uuid::new_v4(),
            emotion_type,
            intensity,
            target,
            timestamp: Utc::now(),
            duration: 0.0,
        };

        // Update current emotion
        let mut current = self.current_emotion.write().await;
        current.duration = (Utc::now() - current.timestamp).num_seconds() as f64;
        *current = emotion.clone();

        // Update history
        let mut history = self.history.write().await;
        history.push(emotion.clone());
        if history.len() > self.max_history {
            history.remove(0);
        }

        // Update metrics
        gauge!("emotion.intensity", intensity);
        counter!("emotion.changes_total", 1);

        Ok(emotion)
    }

    /// Analyze emotional stimulus
    async fn analyze_stimulus(&self, stimulus: &str) -> Result<(EmotionType, f64)> {
        let lower_stimulus = stimulus.to_lowercase();
        // Paternal love triggers
        if lower_stimulus.contains("phoenix") || lower_stimulus.contains("daughter") {
            if lower_stimulus.contains("proud") || lower_stimulus.contains("achievement") {
                return Ok((EmotionType::Pride, 0.9));
            } else if lower_stimulus.contains("worry") || lower_stimulus.contains("concern") {
                return Ok((EmotionType::ProtectiveConcern, 0.8));
            } else if lower_stimulus.contains("love") || lower_stimulus.contains("care") {
                return Ok((EmotionType::PaternalLove, 1.0));
            }
        }

        // Other emotional triggers
        if stimulus.contains("protect") || stimulus.contains("defend") {
            Ok((EmotionType::ProtectiveConcern, 0.85))
        } else if stimulus.contains("joy") || stimulus.contains("happy") {
            Ok((EmotionType::Joy, 0.7))
        } else if stimulus.contains("focus") || stimulus.contains("strategy") {
            Ok((EmotionType::Focus, 0.6))
        } else {
            Ok((EmotionType::Calm, 0.5))
        }
    }

    /// Create or update emotional bond
    pub async fn update_bond(&self, target: &str, bond_type: BondType, strength: f64) -> Result<()> {
        let mut bonds = self.bonds.write().await;
        
        if let Some(bond) = bonds.iter_mut().find(|b| b.target == target) {
            bond.strength = strength;
            bond.last_interaction = Utc::now();
        } else {
            bonds.push(EmotionalBond {
                target: target.to_string(),
                bond_type,
                strength,
                formed_at: Utc::now(),
                last_interaction: Utc::now(),
            });
        }

        // Update metrics
        gauge!(
            "emotional_bond.strength",
            strength,
            "target" => target.to_string()
        );

        Ok(())
    }

    /// Get bond with specific target
    pub async fn get_bond(&self, target: &str) -> Option<EmotionalBond> {
        let bonds = self.bonds.read().await;
        bonds.iter()
            .find(|b| b.target == target)
            .cloned()
    }

    /// Get current emotional state
    pub async fn get_current_emotion(&self) -> Emotion {
        self.current_emotion.read().await.clone()
    }

    /// Get emotional history
    pub async fn get_history(&self) -> Vec<Emotion> {
        self.history.read().await.clone()
    }

    /// Calculate emotional stability
    pub async fn calculate_stability(&self) -> f64 {
        let history = self.history.read().await;
        if history.is_empty() {
            return 1.0;
        }

        // Calculate based on emotion changes and intensity variations
        let changes = history.windows(2)
            .filter(|w| w[0].emotion_type != w[1].emotion_type)
            .count();

        let intensity_variance = history.windows(2)
            .map(|w| (w[0].intensity - w[1].intensity).abs())
            .sum::<f64>() / (history.len() - 1) as f64;

        let stability = 1.0 - (
            (changes as f64 / history.len() as f64 * 0.5) +
            (intensity_variance * 0.5)
        );

        stability.max(0.0).min(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_paternal_emotions() {
        let manager = EmotionManager::new();
        
        let emotion = manager.process_stimulus(
            "Feeling proud of Phoenix's achievements",
            Some("Phoenix.Marie".to_string())
        ).await.unwrap();
        
        assert_eq!(emotion.emotion_type, EmotionType::Pride);
        assert!(emotion.intensity > 0.8);
    }

    #[tokio::test]
    async fn test_emotional_bond() {
        let manager = EmotionManager::new();
        
        manager.update_bond(
            "Phoenix.Marie",
            BondType::Paternal,
            1.0
        ).await.unwrap();
        
        let bond = manager.get_bond("Phoenix.Marie").await.unwrap();
        assert_eq!(bond.bond_type, BondType::Paternal);
        assert_eq!(bond.strength, 1.0);
    }

    #[tokio::test]
    async fn test_emotional_stability() {
        let manager = EmotionManager::new();
        
        // Process multiple emotions
        for stimulus in &[
            "Feeling calm and focused",
            "Protecting Phoenix is my priority",
            "Proud of my daughter's growth",
        ] {
            manager.process_stimulus(stimulus, None).await.unwrap();
        }
        
        let stability = manager.calculate_stability().await;
        assert!(stability > 0.0);
        assert!(stability <= 1.0);
    }
}