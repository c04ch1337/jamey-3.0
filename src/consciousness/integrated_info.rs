//! Integrated Information (Φ) Calculator Module
//!
//! Pragmatic, bounded approximation of Integrated Information (Φ) for the
//! consciousness system. This is **not** a full IIT implementation – it is a
//! small, stable heuristic that:
//!
//! - Represents the system as a fixed, conceptual 4-node network:
//!   - Node 0: Global Workspace
//!   - Node 1: Memory Integration
//!   - Node 2: Emotional Integration
//!   - Node 3: Predictive/Model
//! - Extracts cheap, normalized text features in \[0, 1] from the
//!   [`WorkspaceContent`](super::global_workspace::WorkspaceContent).
//! - Maps those features deterministically into node activations in \[0, 1].
//! - Computes a simple Φ-like integration score based on:
//!   - Pairwise coupling between active nodes (given a fixed connectivity
//!     matrix), and
//!   - Diversity of node activations (variance).
//!
//! # Φ Approximation (high level)
//!
//! Let `a_i ∈ [0,1]` be the activation of node *i* and `w_ij ≥ 0` the
//! connectivity weight from node *i* to node *j* (off-diagonal only).
//!
//! 1. **Pairwise integration term** (how strongly nodes co-activate):
//!
//! ```text
//! raw_integration = Σ_{i<j} w_ij * min(a_i, a_j)
//! max_integration = Σ_{i<j} w_ij        (when all nodes are maximally active)
//! ```
//!
//! 2. **Diversity term** (how differentiated the node activations are):
//!
//! ```text
//! mean_a   = average(a_i)
//! variance = average( (a_i - mean_a)^2 )
//! var_norm = variance / 0.25              (max variance in [0,1] is 0.25)
//! ```
//!
//! 3. **Raw integration score**, combining coupling and diversity:
//!
//! ```text
//! raw_score = raw_integration * clamp(var_norm, 0, 1)
//! ```
//!
//! 4. **Normalized Φ** in \[0, 1]:
//!
//! ```text
//! denom = max(max_integration, epsilon)
//! phi   = clamp(raw_score / denom, 0, 1)
//! ```
//!
//! Where `epsilon` is taken from configuration and used only as a numerical
//! floor on the denominator. If all activations are zero or identical,
//! `variance == 0` so `raw_score == 0` and `phi == 0`.
//!
//! This implementation focuses on:
//! - Boundedness: Φ is guaranteed to be in \[0, 1].
//! - Stability: all divisions are guarded, NaNs are clamped and logged, and
//!   empty networks return Φ = 0.0.
//! - Observability: metrics are exported via the `metrics` crate with clear,
//!   stable names.

use ndarray::Array2;
use anyhow::Result;
use metrics::gauge;
use serde::{Serialize, Deserialize};
use tracing::warn;

use super::global_workspace::WorkspaceContent;

/// Number of conceptual nodes in the Φ network.
const NUM_NODES: usize = 4;

/// Default off-diagonal connectivity weight between conceptual nodes.
///
/// We keep the network fully connected with a uniform weight for simplicity.
/// Diagonal entries are always zero (no self-coupling).
const DEFAULT_CONNECTIVITY_WEIGHT: f64 = 0.7;

/// Calculator for Integrated Information.
///
/// This struct intentionally keeps the internal model small and cheap to
/// evaluate. All state needed for Φ calculation for a given piece of content
/// is derived on the fly from that content; there is no long-lived mutable
/// network state between calls.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhiCalculator {
    /// Fixed connectivity matrix between conceptual nodes.
    ///
    /// Shape is `(NUM_NODES, NUM_NODES)` with zero diagonal and non-zero
    /// off-diagonal entries.
    connectivity: Array2<f64>,
    /// Minimum scale / numerical floor used when normalizing the Φ score.
    epsilon: f64,
    /// Maximum content length for feature extraction (for normalization).
    feature_max_length: f64,
    /// Maximum word count for feature extraction (for normalization).
    feature_max_words: f64,
}

impl PhiCalculator {
    /// Create a new Φ calculator with default configuration.
    pub fn new() -> Self {
        Self::with_config(&crate::config::ConsciousnessConfig::default())
    }

    /// Create a new Φ calculator with custom configuration.
    pub fn with_config(config: &crate::config::ConsciousnessConfig) -> Self {
        let mut connectivity = Array2::zeros((NUM_NODES, NUM_NODES));

        // Fully connected off-diagonal matrix with a sensible constant weight.
        for i in 0..NUM_NODES {
            for j in 0..NUM_NODES {
                if i != j {
                    connectivity[[i, j]] = DEFAULT_CONNECTIVITY_WEIGHT;
                }
            }
        }

        Self {
            connectivity,
            epsilon: config.phi_epsilon,
            feature_max_length: config.feature_max_length.max(1.0),
            feature_max_words: config.feature_max_words.max(1.0),
        }
    }

    /// Calculate Φ value from workspace content.
    ///
    /// This function is async to match the rest of the consciousness pipeline,
    /// but the current implementation is purely CPU-bound and does not perform
    /// any `.await` internally.
    pub async fn calculate(&self, content: &WorkspaceContent) -> Result<f64> {
        // If, for any reason, the network is empty, degrade gracefully.
        if NUM_NODES == 0 {
            gauge!("consciousness.phi_value", 0.0);
            gauge!("consciousness.integration.raw_score", 0.0);
            gauge!("consciousness.integration.normalized_phi", 0.0);
            return Ok(0.0);
        }

        // 1. Extract normalized features from the content.
        let features = self.extract_features(&content.content);

        // 2. Map features into conceptual node activations in [0, 1].
        let activations = self.update_network_state(&features);

        // 3. Compute raw and normalized Φ scores.
        let (mut phi, mut raw_score) = self.compute_phi(&activations);

        // Guard against NaNs/Infs from any unexpected numeric issues.
        if !raw_score.is_finite() {
            warn!("PhiCalculator: raw_score was non-finite, clamping to 0.0");
            raw_score = 0.0;
        }
        if !phi.is_finite() {
            warn!("PhiCalculator: phi value was non-finite, clamping to 0.0");
            phi = 0.0;
        }

        // Clamp once more to the documented bounds.
        raw_score = raw_score.max(0.0);
        phi = phi.clamp(0.0, 1.0);

        // Update metrics for observability.
        gauge!("consciousness.phi_value", phi);
        gauge!("consciousness.integration.raw_score", raw_score);
        gauge!("consciousness.integration.normalized_phi", phi);

        Ok(phi)
    }

    /// Extract numerical features from content and normalize them into [0, 1].
    ///
    /// Features (in order):
    /// 1. `f_length`      – normalized content length using `feature_max_length`.
    /// 2. `f_diversity`   – ratio of unique characters to total characters.
    /// 3. `f_words`       – normalized word count using `feature_max_words`.
    /// 4. `f_uppercase`   – ratio of uppercase ASCII letters to total chars.
    /// 5. `f_punctuation` – ratio of simple punctuation characters to chars.
    fn extract_features(&self, content: &str) -> Vec<f64> {
        let mut features = Vec::with_capacity(5);

        let len = content.chars().count() as f64;

        // 1. Content length normalized.
        let f_length = if len <= 0.0 {
            0.0
        } else {
            (len.min(self.feature_max_length)) / self.feature_max_length
        };
        let f_length = f_length.clamp(0.0, 1.0);
        features.push(f_length);

        // 2. Character diversity (unique chars / total chars).
        let f_diversity = if len <= 0.0 {
            0.0
        } else {
            let unique_chars = content.chars().collect::<std::collections::HashSet<_>>();
            let diversity = (unique_chars.len() as f64) / len;
            diversity.clamp(0.0, 1.0)
        };
        features.push(f_diversity);

        // 3. Word count normalized.
        let word_count = content.split_whitespace().count() as f64;
        let f_words = if word_count <= 0.0 {
            0.0
        } else {
            (word_count.min(self.feature_max_words)) / self.feature_max_words
        };
        let f_words = f_words.clamp(0.0, 1.0);
        features.push(f_words);

        // 4. Uppercase ratio.
        let uppercase_count = content.chars().filter(|c| c.is_ascii_uppercase()).count() as f64;
        let f_uppercase = if len <= 0.0 {
            0.0
        } else {
            (uppercase_count / len).clamp(0.0, 1.0)
        };
        features.push(f_uppercase);

        // 5. Punctuation density (simple set).
        let punctuation_chars = ['.', ',', '!', '?', ';', ':'];
        let punct_count = content
            .chars()
            .filter(|c| punctuation_chars.contains(c))
            .count() as f64;
        let f_punctuation = if len <= 0.0 {
            0.0
        } else {
            (punct_count / len).clamp(0.0, 1.0)
        };
        features.push(f_punctuation);

        // Final guard against any unexpected NaNs.
        for f in &mut features {
            if !f.is_finite() || *f < 0.0 {
                *f = 0.0;
            } else if *f > 1.0 {
                *f = 1.0;
            }
        }

        features
    }

    /// Deterministically map feature vector to node activations in [0, 1].
    ///
    /// The mapping is intentionally simple and cheap:
    /// - Node 0 (Global Workspace): broad mixture of all features.
    /// - Node 1 (Memory Integration): emphasizes length and word count.
    /// - Node 2 (Emotional Integration): emphasizes punctuation/upper-case.
    /// - Node 3 (Predictive/Model): emphasizes diversity and length.
    fn update_network_state(&self, features: &[f64]) -> Vec<f64> {
        if NUM_NODES == 0 {
            return Vec::new();
        }

        let f = |idx: usize| -> f64 {
            let v = *features.get(idx).unwrap_or(&0.0);
            if v.is_finite() { v.clamp(0.0, 1.0) } else { 0.0 }
        };

        let f_length = f(0);
        let f_diversity = f(1);
        let f_words = f(2);
        let f_uppercase = f(3);
        let f_punctuation = f(4);

        let mut activations = vec![0.0; NUM_NODES];

        // Node 0: Global Workspace – broad mixture.
        activations[0] =
            0.25 * f_length +
            0.25 * f_diversity +
            0.25 * f_words +
            0.15 * f_uppercase +
            0.10 * f_punctuation;

        // Node 1: Memory Integration – length + words, slight diversity.
        activations[1] =
            0.50 * f_length +
            0.40 * f_words +
            0.10 * f_diversity;

        // Node 2: Emotional Integration – punctuation and emphasis/upper-case.
        activations[2] =
            0.50 * f_punctuation +
            0.30 * f_uppercase +
            0.20 * f_words;

        // Node 3: Predictive/Model – diversity + length + words.
        activations[3] =
            0.50 * f_diversity +
            0.30 * f_length +
            0.20 * f_words;

        // Normalize and sanitize activations to [0, 1].
        for a in &mut activations {
            if !a.is_finite() || *a < 0.0 {
                *a = 0.0;
            } else if *a > 1.0 {
                *a = 1.0;
            }
        }

        activations
    }

    /// Compute the Φ approximation given node activations.
    ///
    /// Returns `(phi, raw_score)` where:
    /// - `raw_score` is the unnormalized integration score
    ///   (`raw_integration * var_norm`), and
    /// - `phi` is the normalized Φ ∈ [0, 1].
    fn compute_phi(&self, activations: &[f64]) -> (f64, f64) {
        let n = activations.len();
        if n == 0 {
            return (0.0, 0.0);
        }

        // If all activations are identical (including all zero), variance will
        // be 0 and Φ will be 0 by construction.
        let mean = activations.iter().copied().sum::<f64>() / (n as f64);
        let mut variance = 0.0;
        for &a in activations {
            let diff = a - mean;
            variance += diff * diff;
        }
        variance /= n as f64;

        // Normalize variance to [0, 1] based on the maximal variance for
        // values constrained to [0, 1], which is 0.25.
        let var_norm = (variance / 0.25).clamp(0.0, 1.0);

        // Pairwise integration term (using min(a_i, a_j) weighted by connectivity).
        let mut raw_integration = 0.0;
        let mut max_integration = 0.0;

        let shape = self.connectivity.dim();
        let rows = shape.0.min(n);
        let cols = shape.1.min(n);

        for i in 0..rows {
            for j in (i + 1)..cols {
                let w = self.connectivity[[i, j]].max(0.0); // ignore negative weights
                if w <= 0.0 {
                    continue;
                }

                let a_i = activations[i].clamp(0.0, 1.0);
                let a_j = activations[j].clamp(0.0, 1.0);
                let pair_min = a_i.min(a_j);

                raw_integration += w * pair_min;
                max_integration += w * 1.0;
            }
        }

        // Combine integration with diversity.
        let raw_score = raw_integration * var_norm;

        if max_integration <= 0.0 {
            // No meaningful connectivity – treat as fully partitioned system.
            return (0.0, 0.0);
        }

        // Normalize to [0, 1] with an epsilon floor to avoid pathological tiny denominators.
        let denom = max_integration.max(self.epsilon.max(1e-12));
        let phi = (raw_score / denom).clamp(0.0, 1.0);

        (phi, raw_score.max(0.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use tokio::test;
    use uuid::Uuid;

    fn make_content(text: &str) -> WorkspaceContent {
        WorkspaceContent {
            id: Uuid::new_v4(),
            content: text.to_string(),
            source: "test".to_string(),
            priority: 0.8,
            timestamp: Utc::now(),
        }
    }

    #[test]
    async fn test_phi_calculation() {
        let calculator = PhiCalculator::new();

        let simple_content = make_content("short");
        let rich_content = make_content(
            "This is a much richer, more complex piece of content! \
             It mentions phoenix, protection, and multiple nuanced ideas.",
        );

        let phi_simple = calculator.calculate(&simple_content).await.unwrap();
        let phi_rich = calculator.calculate(&rich_content).await.unwrap();

        assert!(phi_simple >= 0.0);
        assert!(phi_rich >= 0.0);
        // Expect richer content to have at least as much or more integration.
        assert!(phi_rich >= phi_simple);
    }

    #[test]
    async fn test_phi_zero_for_empty_content() {
        let calculator = PhiCalculator::new();
        let empty_content = make_content("");

        let phi = calculator.calculate(&empty_content).await.unwrap();
        assert_eq!(phi, 0.0);
    }

    #[test]
    async fn test_phi_stable_range() {
        let calculator = PhiCalculator::new();

        let inputs = [
            "",
            "hi",
            "Regular sentence with some punctuation!",
            "ALL CAPS WITH STRONG EMPHASIS!!!",
            "Diverse content: Phoenix rising, protect the future, adapt & evolve...",
        ];

        for text in &inputs {
            let content = make_content(text);
            let phi = calculator.calculate(&content).await.unwrap();
            assert!(
                (0.0..=1.0).contains(&phi),
                "phi out of range for input {:?}: {}",
                text,
                phi
            );
        }
    }

    #[test]
    async fn test_feature_extraction_in_range() {
        let calculator = PhiCalculator::new();
        let features = calculator.extract_features("Test content with VARIETY and punctuation!!");

        assert!(!features.is_empty());
        assert!(features.iter().all(|&f| f >= 0.0 && f <= 1.0));
    }
}