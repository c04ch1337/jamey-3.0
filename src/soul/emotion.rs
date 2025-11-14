use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
#[sqlx(rename_all = "lowercase")]
pub enum Emotion {
    Joy,
    Sadness,
    Anger,
    Neutral,
    Love,
}

impl Emotion {
    /// Get emoji representation
    pub fn emoji(&self) -> &'static str {
        match self {
            Emotion::Joy => "😊",
            Emotion::Sadness => "😢",
            Emotion::Anger => "😡",
            Emotion::Neutral => "😐",
            Emotion::Love => "😍",
        }
    }
    
    /// Get default score for this emotion
    pub fn score(&self) -> f32 {
        match self {
            Emotion::Joy => 0.8,
            Emotion::Sadness => 0.2,
            Emotion::Anger => 0.1,
            Emotion::Neutral => 0.5,
            Emotion::Love => 1.0,
        }
    }
    
    /// Parse emotion from string (case-insensitive, supports both text and emoji)
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "joy" | "happy" | "😊" => Some(Emotion::Joy),
            "sadness" | "sad" | "😢" => Some(Emotion::Sadness),
            "anger" | "angry" | "😡" => Some(Emotion::Anger),
            "neutral" | "😐" => Some(Emotion::Neutral),
            "love" | "😍" => Some(Emotion::Love),
            _ => None,
        }
    }
    
    /// Get emotion name
    pub fn name(&self) -> &'static str {
        match self {
            Emotion::Joy => "joy",
            Emotion::Sadness => "sadness",
            Emotion::Anger => "anger",
            Emotion::Neutral => "neutral",
            Emotion::Love => "love",
        }
    }
}

impl std::fmt::Display for Emotion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.emoji(), self.name())
    }
}