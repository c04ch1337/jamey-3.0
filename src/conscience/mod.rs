use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
<<<<<<< HEAD
use tracing::info;

use crate::soul::{Emotion, SoulStorage};
=======
>>>>>>> origin/main

/// A moral rule with a weight and description
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoralRule {
    pub name: String,
    pub description: String,
    pub weight: f32,
}

/// Conscience Engine that evaluates actions against moral rules
#[derive(Clone)]
pub struct ConscienceEngine {
<<<<<<< HEAD
    /// Thread-safe storage of moral rules
    rules: Arc<DashMap<String, MoralRule>>,

    /// Optional Soul KB integration
    soul_storage: Option<Arc<SoulStorage>>,
=======
    rules: Arc<DashMap<String, MoralRule>>,
>>>>>>> origin/main
}

impl ConscienceEngine {
    /// Create a new Conscience Engine with default rules
    pub fn new() -> Self {
        let rules = Arc::new(DashMap::new());
        
        // Add default rules
        rules.insert(
            "no-harm".to_string(),
            MoralRule {
                name: "no-harm".to_string(),
                description: "Do not cause physical or emotional harm".to_string(),
                weight: 10.0,
            },
        );
        
        rules.insert(
            "truth".to_string(),
            MoralRule {
                name: "truth".to_string(),
                description: "Be honest and truthful".to_string(),
                weight: 8.0,
            },
        );

<<<<<<< HEAD
        Self { 
            rules,
            soul_storage: None,
        }
    }

    /// Add Soul KB integration
    pub fn with_soul_storage(mut self, storage: Arc<SoulStorage>) -> Self {
        self.soul_storage = Some(storage);
        self
=======
        Self { rules }
>>>>>>> origin/main
    }

    /// Add a new moral rule
    pub fn add_rule(&self, rule: MoralRule) {
        self.rules.insert(rule.name.clone(), rule);
    }

    /// Remove a moral rule
    pub fn remove_rule(&self, name: &str) -> Option<MoralRule> {
        self.rules.remove(name).map(|(_, v)| v)
    }

    /// Get all rules
    pub fn get_rules(&self) -> Vec<MoralRule> {
        self.rules.iter().map(|entry| entry.value().clone()).collect()
    }

<<<<<<< HEAD
    /// Basic evaluation against moral rules
=======
    /// Evaluate an action against all moral rules
    /// Returns a score where higher is more moral
>>>>>>> origin/main
    pub fn evaluate(&self, action: &str) -> f32 {
        let action_lower = action.to_lowercase();
        let mut score = 0.0;

        for rule in self.rules.iter() {
            // Simple keyword matching - can be enhanced with NLP
            let rule_lower = rule.description.to_lowercase();
            let keywords: Vec<&str> = rule_lower.split_whitespace().collect();
            
            let matches: usize = keywords
                .iter()
                .filter(|keyword| action_lower.contains(*keyword))
                .count();

            if matches > 0 {
                // Score based on how many keywords match and rule weight
                let match_ratio = matches as f32 / keywords.len() as f32;
                score += rule.weight * match_ratio;
            }
        }

        score
    }
<<<<<<< HEAD

    /// Evaluate action and record emotion in Soul KB if entity provided
    pub async fn evaluate_with_soul(
        &self,
        action: &str,
        entity_name: Option<&str>,
    ) -> anyhow::Result<(f32, Option<Emotion>)> {
        let score = self.evaluate(action);

        // Map score to emotion
        let emotion = if score > 8.0 {
            Some(Emotion::Joy)
        } else if score > 5.0 {
            Some(Emotion::Neutral)
        } else if score > 2.0 {
            Some(Emotion::Sadness)
        } else {
            Some(Emotion::Anger)
        };

        // Record to Soul KB if entity provided
        if let (Some(storage), Some(name), Some(em)) = 
            (&self.soul_storage, entity_name, emotion)
        {
            info!(
                entity = name,
                emotion = ?em,
                score,
                "Recording conscience evaluation emotion"
            );

            if let Some(mut entity) = storage.get_entity(name).await? {
                entity.record_emotion(em, score / 10.0);
                storage.upsert_entity(&entity).await?;
            }
        }

        Ok((score, emotion))
    }
=======
>>>>>>> origin/main
}

impl Default for ConscienceEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
<<<<<<< HEAD
    use tempfile::tempdir;
    use crate::db::init_db;
=======
>>>>>>> origin/main

    #[test]
    fn test_default_rules() {
        let engine = ConscienceEngine::new();
        let rules = engine.get_rules();
        assert_eq!(rules.len(), 2);
    }

    #[test]
    fn test_evaluate_action() {
        let engine = ConscienceEngine::new();
        let score = engine.evaluate("I will help someone in need");
        assert!(score >= 0.0);
    }
<<<<<<< HEAD

    #[tokio::test]
    async fn test_evaluate_with_soul() {
        // Initialize test database
        let db_dir = tempdir().unwrap();
        std::env::set_var("DATABASE_URL", format!("sqlite:{}/test.db", db_dir.path().display()));
        let pool = init_db().await.unwrap();

        // Create soul storage
        let storage = Arc::new(SoulStorage::new(pool));
        
        // Create conscience engine with soul integration
        let engine = ConscienceEngine::new().with_soul_storage(storage);

        // Test evaluation with entity
        let (score, emotion) = engine
            .evaluate_with_soul("I will help others", Some("test_user"))
            .await
            .unwrap();

        assert!(score > 0.0);
        assert!(emotion.is_some());
    }
}
=======
}

>>>>>>> origin/main
