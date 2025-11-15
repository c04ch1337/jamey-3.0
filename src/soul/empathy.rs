use super::entity::SoulEntity;
use super::emotion::Emotion;
use super::trust::TrustCalculator;
use std::collections::HashMap;

/// Empathy scorer for soul entities
pub struct EmpathyScorer;

impl EmpathyScorer {
    /// Score an interaction and update entity trust
    pub fn score_interaction(entity: &mut SoulEntity, new_emotions: HashMap<Emotion, u32>) {
        // Merge new emotions with existing
        for (emotion, count) in new_emotions {
            *entity.emotions.entry(emotion).or_insert(0) += count;
        }
        
        // Calculate empathy and adjust trust
        let empathy = TrustCalculator::calculate_empathy(&entity.emotions);
        let trust_boost = TrustCalculator::trust_boost_from_empathy(empathy);
        
        // Apply trust boost
        entity.trust_score = (entity.trust_score + trust_boost).clamp(0.0, 1.0);
        
        // Update decay rate based on emotional state
        entity.decay_rate = TrustCalculator::calculate_decay_rate(&entity.emotions);
    }
    
    /// Check if entity has high empathy (threshold: 0.7)
    pub fn has_high_empathy(entity: &SoulEntity) -> bool {
        TrustCalculator::calculate_empathy(&entity.emotions) > 0.7
    }
    
    /// Get empathy level description
    pub fn empathy_description(empathy: f32) -> &'static str {
        if empathy > 0.8 {
            "Very High"
        } else if empathy > 0.6 {
            "High"
        } else if empathy > 0.4 {
            "Moderate"
        } else if empathy > 0.2 {
            "Low"
        } else {
            "Very Low"
        }
    }
}