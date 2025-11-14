use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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
    rules: Arc<DashMap<String, MoralRule>>,
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

        Self { rules }
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

    /// Evaluate an action against all moral rules
    /// Returns a score where higher is more moral
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
}

impl Default for ConscienceEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}

