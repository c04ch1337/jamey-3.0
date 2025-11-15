//! Attention Schema Module
//!
//! Maps attention focus and directs cognitive resources

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use super::global_workspace::WorkspaceContent;

/// Attention Schema system.
///
/// This module tracks the current focus of attention based on incoming information.
/// It is stateful and uses heuristics to determine what is most salient.
pub struct AttentionSchema {
    current_focus: Arc<RwLock<String>>,
}

impl AttentionSchema {
    /// Create a new Attention Schema with an empty initial focus.
    pub fn new() -> Self {
        Self {
            current_focus: Arc::new(RwLock::new(String::new())),
        }
    }

    /// Safely retrieves the current attentional focus.
    pub async fn current_focus(&self) -> String {
        self.current_focus.read().await.clone()
    }

    /// Update attention schema based on broadcast content.
    ///
    /// # Arguments
    /// * `broadcast` - The content broadcast from the global workspace.
    /// * `_predictions` - Predictions from the predictive processor (currently unused).
    ///
    /// # Heuristic
    /// The new focus is determined by finding the longest non-stopword in the
    /// broadcast content. This is a simple heuristic to model salience.
    pub async fn update(&self, broadcast: &WorkspaceContent, _predictions: &str) -> Result<()> {
        // 1. Determine New Focus: Find the longest non-stopword from the broadcast content.
        let stopwords = ["a", "an", "the", "in", "on", "of", "for", "to"];
        let new_focus = broadcast
            .content
            .split_whitespace()
            // Sanitize words by removing non-alphabetic characters
            .map(|word| word.chars().filter(|c| c.is_alphabetic()).collect::<String>())
            .filter(|word| !stopwords.contains(&word.to_lowercase().as_str()))
            .max_by_key(|word| word.len())
            .unwrap_or_else(|| "none".to_string());

        // 2. Update State and Metrics
        let mut focus_lock = self.current_focus.write().await;
        if *focus_lock != new_focus {
            metrics::counter!("consciousness.attention.shifts_total", 1);
            *focus_lock = new_focus;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consciousness::global_workspace::WorkspaceContent;
    use uuid::Uuid;

    fn create_broadcast(content: &str) -> WorkspaceContent {
        WorkspaceContent {
            id: Uuid::new_v4(),
            content: content.to_string(),
            source: "test".to_string(),
            priority: 1.0,
            timestamp: chrono::Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_attention_focus_update() {
        let schema = AttentionSchema::new();
        assert_eq!(schema.current_focus().await, "");

        // First update
        let broadcast1 = create_broadcast("A test of the emergency broadcast system.");
        schema.update(&broadcast1, "").await.unwrap();
        assert_eq!(schema.current_focus().await, "emergency");

        // Second update with different content
        let broadcast2 = create_broadcast("Focus on this important keyword.");
        schema.update(&broadcast2, "").await.unwrap();
        assert_eq!(schema.current_focus().await, "important");
    }

    #[tokio::test]
    async fn test_focus_remains_stable() {
        let schema = AttentionSchema::new();
        assert_eq!(schema.current_focus().await, "");

        // First update
        let broadcast1 = create_broadcast("The most salient word here is salient.");
        schema.update(&broadcast1, "").await.unwrap();
        assert_eq!(schema.current_focus().await, "salient.");

        // Second update with similar content that shouldn't change the focus
        let broadcast2 = create_broadcast("Another post about the salient features.");
        schema.update(&broadcast2, "").await.unwrap();
        assert_eq!(schema.current_focus().await, "salient.");
    }
    
    #[tokio::test]
    async fn test_empty_content_yields_none_focus() {
        let schema = AttentionSchema::new();
        let broadcast = create_broadcast("");
        schema.update(&broadcast, "").await.unwrap();
        assert_eq!(schema.current_focus().await, "none");
    }
}

