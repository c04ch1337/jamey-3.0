//! Higher-Order Thought Module
//!
//! Implements metacognitive monitoring and self-awareness
//!
//! This module provides a pragmatic, lightweight implementation of a
//! Higher-Order Thought (HOT) layer. It estimates how "introspective"
//! the current workspace content is using cheap lexical heuristics and
//! maintains a smoothed awareness score in [0.0, 1.0].

use anyhow::Result;
use metrics::gauge;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::global_workspace::WorkspaceContent;

/// Higher-Order Thought system for metacognitive monitoring.
///
/// This struct implements a simple, observable Higher-Order Thought (HOT) system.
/// It tracks an internal awareness state based on introspective and cognitive cues
/// found in the content it processes.
pub struct HigherOrderThought {
    /// Holds an incrementally-updated mean of recent awareness scores.
    awareness_avg: Arc<RwLock<f64>>,
    /// The number of processed items, used for the incremental mean calculation.
    count: Arc<RwLock<u64>>,
}

impl HigherOrderThought {
    /// Create a new Higher-Order Thought system.
    pub fn new() -> Self {
        Self {
            awareness_avg: Arc::new(RwLock::new(0.0)),
            count: Arc::new(RwLock::new(0)),
        }
    }

    /// Process workspace content to update and report metacognitive awareness.
    ///
    /// This function implements the core awareness heuristic:
    /// 1.  It tokenizes the input content, filtering for introspection-related words.
    /// 2.  It computes a per-call `score` based on the frequency of these keywords.
    /// 3.  It updates a simple incremental mean (`awareness_avg`) to track awareness over time.
    /// 4.  An instantaneous metric is emitted for real-time monitoring.
    ///
    /// Returns a short summary string of the immediate score.
    pub async fn process(&self, content: &WorkspaceContent) -> Result<String> {
        // Define keyword sets for introspection and cognitive processing.
        const INTROSPECTIVE_KEYWORDS: &[&str] =
            &["i", "me", "my", "think", "feel", "aware", "should", "decide"];
        const COGNITIVE_VERBS: &[&str] =
            &["plan", "consider", "reflect", "remember", "decide", "analyze"];

        let lower_content = content.content.to_lowercase();
        let tokens: Vec<_> = lower_content.split_whitespace().collect();
        let total_words = tokens.len();

        let introspective_hits = tokens
            .iter()
            .filter(|&t| INTROSPECTIVE_KEYWORDS.contains(t))
            .count();
        let cognitive_hits = tokens
            .iter()
            .filter(|&t| COGNITIVE_VERBS.contains(t))
            .count();

        // Calculate a score, scaled up slightly to improve dynamic range and clamped.
        let base = (introspective_hits + cognitive_hits) as f64 / (total_words.max(1) as f64);
        let score = (base * 1.5).clamp(0.0, 1.0);

        // Update the incremental moving average.
        {
            let mut avg = self.awareness_avg.write().await;
            let mut cnt = self.count.write().await;
            *cnt += 1;
            let n = *cnt as f64;
            *avg += (score - *avg) / n.max(1.0);
        }

        // Emit a metric for the instantaneous awareness score.
        gauge!("consciousness.higher_order.awareness_instant", score);

        Ok(format!("meta_awareness:{:.2}", score))
    }

    /// Get current rolling awareness level.
    ///
    /// Returns the smoothed awareness score, clamped to `[0.0, 1.0]`.
    /// If the average is not a finite number (e.g., NaN, infinity) or if no
    /// content has been processed, it returns a neutral default of 0.5.
    pub async fn awareness_level(&self) -> f64 {
        let avg = *self.awareness_avg.read().await;
        if avg.is_finite() && avg >= 0.0 {
            avg.min(1.0)
        } else {
            0.0 // Default to 0 if not yet computed or NaN
        }
    }
}

impl Default for HigherOrderThought {
    fn default() -> Self {
        Self::new()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::consciousness::global_workspace::WorkspaceContent;
    use uuid::Uuid;

    fn make_content(text: &str) -> WorkspaceContent {
        WorkspaceContent {
            id: Uuid::new_v4(),
            content: text.to_string(),
            source: "test".to_string(),
            priority: 1.0,
            timestamp: chrono::Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_awareness_level_range() {
        let hot = HigherOrderThought::new();

        let neutral_content = make_content("This is a simple statement.");
        let introspective_content =
            make_content("I think I should reflect on my decisions and what I should plan.");

        // Process neutral content
        hot.process(&neutral_content).await.unwrap();
        let level1 = hot.awareness_level().await;
        assert!((0.0..=1.0).contains(&level1), "Level should be in [0, 1]");

        // Process introspective content
        hot.process(&introspective_content).await.unwrap();
        let level2 = hot.awareness_level().await;
        assert!((0.0..=1.0).contains(&level2), "Level should be in [0, 1]");

        // The awareness level should be higher after processing introspective content.
        assert!(
            level2 > level1,
            "Introspective content did not increase awareness level"
        );
    }
    
    #[tokio::test]
    async fn awareness_level_is_bounded_and_sensitive_to_introspection() {
        let hot = HigherOrderThought::new();

        let introspective = make_content("I think I should reflect on what I decide next");
        let neutral = make_content("System log entry without any self reference or planning");

        hot.process(&neutral).await.unwrap();
        let level_neutral = hot.awareness_level().await;
        
        hot.process(&introspective).await.unwrap();
        let level_introspective = hot.awareness_level().await;
        

        assert!((0.0..=1.0).contains(&level_introspective));
        assert!((0.0..=1.0).contains(&level_neutral));
        assert!(level_introspective > level_neutral);
    }

    #[tokio::test]
    async fn default_awareness_is_neutral_before_processing() {
        let hot = HigherOrderThought::new();
        let level = hot.awareness_level().await;
        assert!((0.0..=1.0).contains(&level));
        // Before processing, the awareness level should be exactly 0.0.
        assert!((level - 0.0).abs() < f64::EPSILON, "Initial awareness should be 0.0");
    }
}

