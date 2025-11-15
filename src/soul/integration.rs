//! Integration helpers for connecting soul KB with other systems

use anyhow::Result;
use uuid::Uuid;
<<<<<<< HEAD
use super::{Emotion, SoulEntity, SoulStorage};

/// Map a conscience evaluation score (0.0-1.0) to an emotion
pub fn score_to_emotion(score: f32) -> Emotion {
    match score {
        s if s > 0.7 => Emotion::Love,
        s if s > 0.4 => Emotion::Joy,
        s if s > 0.3 => Emotion::Neutral,
        s if s > 0.1 => Emotion::Sadness,
        _ => Emotion::Anger,
=======
use chrono::Utc;
use super::emotion::{Emotion, EmotionType};
use super::{SoulEntity, SoulStorage};

/// Map a conscience evaluation score (0.0-1.0) to an emotion
pub fn score_to_emotion(score: f32) -> Emotion {
    let emotion_type = match score {
        s if s > 0.7 => EmotionType::PaternalLove,
        s if s > 0.4 => EmotionType::Joy,
        s if s > 0.3 => EmotionType::Calm,
        s if s > 0.1 => EmotionType::ProtectiveConcern,
        _ => EmotionType::Focus,
    };
    Emotion {
        id: Uuid::new_v4(),
        emotion_type,
        intensity: score as f64,
        target: None,
        timestamp: Utc::now(),
        duration: 0.0,
>>>>>>> origin/main
    }
}

/// Record an emotion from a conscience evaluation
/// Returns the entity ID
pub async fn record_conscience_emotion(
    storage: &SoulStorage,
    entity_name: &str,
    evaluation_score: f32,
    default_trust: f32,
) -> Result<i64> {
    let emotion = score_to_emotion(evaluation_score);
    
    // Get or create entity
    let mut entity = storage.get_entity(entity_name).await?
        .unwrap_or_else(|| {
            let mut e = SoulEntity::new(entity_name.to_string());
            e.trust_score = default_trust;
            e
        });
    
    // Record emotion and boost trust
<<<<<<< HEAD
    entity.record_emotion(emotion);
=======
    entity.record_emotion(emotion.clone());
>>>>>>> origin/main
    entity.boost_trust();
    
    // Save to database
    let entity_id = storage.upsert_entity(&entity).await?;
    storage.record_emotion(entity_id, emotion, 1).await?;
    
    Ok(entity_id)
}

/// Link a memory to an entity in the soul KB
pub async fn link_memory_to_soul(
    storage: &SoulStorage,
    entity_name: &str,
    memory_id: Uuid,
) -> Result<()> {
    if let Some(entity) = storage.get_entity(entity_name).await? {
        storage.link_memory(entity.id, memory_id).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_score_to_emotion_mapping() {
<<<<<<< HEAD
        assert_eq!(score_to_emotion(0.9), Emotion::Love);
        assert_eq!(score_to_emotion(0.6), Emotion::Joy);
        assert_eq!(score_to_emotion(0.35), Emotion::Neutral);
        assert_eq!(score_to_emotion(0.2), Emotion::Sadness);
        assert_eq!(score_to_emotion(0.05), Emotion::Anger);
=======
        let high = score_to_emotion(0.9);
        assert_eq!(high.emotion_type, EmotionType::PaternalLove);
        
        let mid = score_to_emotion(0.6);
        assert_eq!(mid.emotion_type, EmotionType::Joy);
        
        let neutral = score_to_emotion(0.35);
        assert_eq!(neutral.emotion_type, EmotionType::Calm);
        
        let low = score_to_emotion(0.2);
        assert_eq!(low.emotion_type, EmotionType::ProtectiveConcern);
        
        let very_low = score_to_emotion(0.05);
        assert_eq!(very_low.emotion_type, EmotionType::Focus);
>>>>>>> origin/main
    }
}