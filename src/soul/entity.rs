use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::collections::HashMap;
use uuid::Uuid;
<<<<<<< HEAD
use super::emotion::Emotion;
use super::config::SoulConfig;
=======
use super::emotion::{Emotion, EmotionType};
>>>>>>> origin/main

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SoulEntity {
    pub id: i64,
    pub entity_name: String,
    pub trust_score: f32,
    pub decay_rate: f32,
    pub last_interaction: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    
    #[sqlx(skip)]
<<<<<<< HEAD
    pub emotions: HashMap<Emotion, u32>,
=======
    pub emotions: HashMap<EmotionType, u32>,
>>>>>>> origin/main
    
    #[sqlx(skip)]
    pub linked_memories: Vec<Uuid>,
}

impl SoulEntity {
<<<<<<< HEAD
    /// Create a new soul entity with configuration values
    pub fn new(entity_name: String, config: &SoulConfig) -> Self {
        Self {
            id: 0, // Will be set by database
            entity_name,
            trust_score: config.default_trust,
            decay_rate: config.base_decay_rate,
=======
    /// Create a new soul entity with default values
    pub fn new(entity_name: String) -> Self {
        Self {
            id: 0, // Will be set by database
            entity_name,
            trust_score: 0.5,
            decay_rate: 0.01,
>>>>>>> origin/main
            last_interaction: Utc::now(),
            created_at: Utc::now(),
            emotions: HashMap::new(),
            linked_memories: Vec::new(),
        }
    }
    
    /// Record an emotion occurrence (increments count)
<<<<<<< HEAD
    pub fn record_emotion(&mut self, emotion: Emotion, config: &SoulConfig) -> bool {
        if self.emotions.len() >= config.max_emotions_per_entity && !self.emotions.contains_key(&emotion) {
            return false;
        }
        *self.emotions.entry(emotion).or_insert(0) += 1;
        self.last_interaction = Utc::now();
        true
=======
    pub fn record_emotion(&mut self, emotion: Emotion) {
        *self.emotions.entry(emotion.emotion_type).or_insert(0) += 1;
        self.last_interaction = Utc::now();
>>>>>>> origin/main
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
<<<<<<< HEAD
        for (emotion, count) in &self.emotions {
            let weight = emotion.score();
=======
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
>>>>>>> origin/main
            weighted_sum += weight * (*count as f32);
        }
        
        weighted_sum / (total as f32)
    }
    
    /// Get the dominant emotion (most frequent)
<<<<<<< HEAD
    pub fn dominant_emotion(&self) -> Option<Emotion> {
        self.emotions
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(emotion, _)| *emotion)
=======
    pub fn dominant_emotion(&self) -> Option<EmotionType> {
        self.emotions
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(emotion_type, _)| emotion_type.clone())
>>>>>>> origin/main
    }
    
    /// Apply trust decay based on time elapsed since last interaction
    pub fn apply_decay(&mut self, days: f64) {
        let decay = self.decay_rate * days as f32;
        self.trust_score = (self.trust_score - decay).max(0.0);
    }
    
<<<<<<< HEAD
    /// Boost trust based on empathy score using configuration values
    pub fn boost_trust(&mut self, config: &SoulConfig) {
        let empathy = self.empathy_score();
        
        // High empathy boosts trust
        if empathy > config.empathy_threshold {
            let boost = (empathy - config.empathy_threshold) * config.trust_boost_factor;
            self.trust_score = (self.trust_score + boost).min(1.0);
        }
        // Low empathy decreases trust
        else if empathy < (1.0 - config.empathy_threshold) {
            let penalty = ((1.0 - config.empathy_threshold) - empathy) * config.trust_penalty_factor;
=======
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
>>>>>>> origin/main
            self.trust_score = (self.trust_score - penalty).max(0.0);
        }
        
        // Adjust decay rate based on empathy
<<<<<<< HEAD
        if empathy > config.empathy_threshold {
            self.decay_rate = config.base_decay_rate * 0.5; // Slower decay for positive relationships
        } else if empathy < (1.0 - config.empathy_threshold) {
            self.decay_rate = config.base_decay_rate * 2.0; // Faster decay for negative relationships
        } else {
            self.decay_rate = config.base_decay_rate; // Normal decay
        }
    }

    /// Link a memory to the entity
    pub fn link_memory(&mut self, memory_id: Uuid, config: &SoulConfig) -> bool {
        if self.linked_memories.len() >= config.max_memories_per_entity {
            return false;
        }
        if !self.linked_memories.contains(&memory_id) {
            self.linked_memories.push(memory_id);
            true
        } else {
            false
=======
        if empathy > 0.7 {
            self.decay_rate = 0.005; // Slower decay for positive relationships
        } else if empathy < 0.3 {
            self.decay_rate = 0.02; // Faster decay for negative relationships
        } else {
            self.decay_rate = 0.01; // Normal decay
>>>>>>> origin/main
        }
    }
}