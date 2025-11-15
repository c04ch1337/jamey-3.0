use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::collections::HashMap;
use uuid::Uuid;
use super::emotion::{Emotion, EmotionType};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SoulEntity {
    pub id: i64,
    pub entity_name: String,
    pub trust_score: f32,
    pub decay_rate: f32,
    pub last_interaction: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    
    #[sqlx(skip)]
    pub emotions: HashMap<EmotionType, u32>,
    
    #[sqlx(skip)]
    pub linked_memories: Vec<Uuid>,
}

impl SoulEntity {
    /// Create a new soul entity with default values
    pub fn new(entity_name: String) -> Self {
        Self {
            id: 0, // Will be set by database
            entity_name,
            trust_score: 0.5,
            decay_rate: 0.01,
            last_interaction: Utc::now(),
            created_at: Utc::now(),
            emotions: HashMap::new(),
            linked_memories: Vec::new(),
        }
    }
    
    /// Record an emotion occurrence (increments count)
    pub fn record_emotion(&mut self, emotion: Emotion) {
        *self.emotions.entry(emotion.emotion_type).or_insert(0) += 1;
        self.last_interaction = Utc::now();
    }
    
    /// Calculate empathy score based on emotion distribution
    pub fn empathy_score(&self) -> f32 {
        if self.emotions.is_empty() {
            return 0.5; // Neutral empathy
        }
        
        let total: u32 = self.emotions.values().sum();
        if total == 0 {
            return 0.5;
        }
        
        let mut weighted_sum = 0.0;
        for (emotion_type, count) in &self.emotions {
            // Use intensity 0.5 as default weight for emotion types
            let weight = match emotion_type {
                EmotionType::PaternalLove => 1.0,
                EmotionType::Joy => 0.8,
                EmotionType::Calm => 0.5,
                EmotionType::ProtectiveConcern => 0.6,
                EmotionType::Pride => 0.7,
                EmotionType::Focus => 0.4,
                EmotionType::Worry => 0.3,
                EmotionType::General(_) => 0.5,
            };
            weighted_sum += weight * (*count as f32);
        }
        
        weighted_sum / (total as f32)
    }
    
    /// Get the dominant emotion (most frequent)
    pub fn dominant_emotion(&self) -> Option<EmotionType> {
        self.emotions
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(emotion_type, _)| emotion_type.clone())
    }
    
    /// Apply trust decay based on time elapsed since last interaction
    pub fn apply_decay(&mut self, days: f64) {
        let decay = self.decay_rate * days as f32;
        self.trust_score = (self.trust_score - decay).max(0.0);
    }
    
    /// Boost trust based on empathy score
    pub fn boost_trust(&mut self) {
        let empathy = self.empathy_score();
        
        // High empathy (>0.7) boosts trust
        if empathy > 0.7 {
            let boost = (empathy - 0.7) * 0.5;
            self.trust_score = (self.trust_score + boost).min(1.0);
        }
        // Low empathy (<0.3) decreases trust
        else if empathy < 0.3 {
            let penalty = (0.3 - empathy) * 0.3;
            self.trust_score = (self.trust_score - penalty).max(0.0);
        }
        
        // Adjust decay rate based on empathy
        if empathy > 0.7 {
            self.decay_rate = 0.005; // Slower decay for positive relationships
        } else if empathy < 0.3 {
            self.decay_rate = 0.02; // Faster decay for negative relationships
        } else {
            self.decay_rate = 0.01; // Normal decay
        }
    }
}