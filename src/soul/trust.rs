use super::emotion::Emotion;
use std::collections::HashMap;

/// Trust calculator for soul entities
pub struct TrustCalculator;

impl TrustCalculator {
    /// Calculate empathy score from emotion counts
    /// Returns a value between 0.0 and 1.0
    pub fn calculate_empathy(emotions: &HashMap<Emotion, u32>) -> f32 {
        if emotions.is_empty() {
            return 0.5; // Neutral empathy
        }
        
        let total: u32 = emotions.values().sum();
        if total == 0 {
            return 0.5;
        }
        
        let mut weighted_sum = 0.0;
        for (emotion, count) in emotions {
            let weight = emotion.score();
            weighted_sum += weight * (*count as f32);
        }
        
        weighted_sum / (total as f32)
    }
    
    /// Calculate trust boost based on empathy score
    pub fn trust_boost_from_empathy(empathy: f32) -> f32 {
        // High empathy (>0.7) gives positive boost
        if empathy > 0.7 {
            (empathy - 0.7) * 0.5 // Up to +0.15 boost
        } else if empathy < 0.3 {
            (empathy - 0.3) * 0.3 // Negative adjustment
        } else {
            0.0
        }
    }
    
    /// Calculate decay rate based on emotional state
    /// Positive emotions = slower decay, negative = faster decay
    pub fn calculate_decay_rate(emotions: &HashMap<Emotion, u32>) -> f32 {
        let empathy = Self::calculate_empathy(emotions);
        
        // Base decay rate
        let base_rate = 0.01;
        
        // Adjust based on empathy
        if empathy > 0.7 {
            base_rate * 0.5 // Slower decay for positive relationships
        } else if empathy < 0.3 {
            base_rate * 2.0 // Faster decay for negative relationships
        } else {
            base_rate
        }
    }
}