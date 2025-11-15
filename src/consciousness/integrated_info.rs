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

use ndarray::{Array1, Array2, ArrayView1};
use anyhow::{Result, anyhow};
use metrics::{gauge, counter, histogram};
use serde::{Serialize, Deserialize};
use tracing::{warn, error, info};
use std::collections::VecDeque;
use std::ops::RangeInclusive;
use std::sync::atomic::{AtomicUsize, Ordering};
use lru::LruCache;
use std::time::Instant;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use rayon::prelude::*;

use super::global_workspace::WorkspaceContent;

#[derive(Debug)]
struct StabilityError {
    message: String,
    feature_index: usize,
    value: f64,
}

/// Feature extraction validation pipeline
#[derive(Debug)]
struct FeatureValidationPipeline {
    length_validator: NumericalBounds,
    diversity_validator: NumericalBounds,
    word_count_validator: NumericalBounds,
    validation_count: AtomicUsize,
    error_count: AtomicUsize,
}

impl FeatureValidationPipeline {
    fn new() -> Self {
        Self {
            length_validator: NumericalBounds {
                range: 0.0..=1.0,
                name: "length",
            },
            diversity_validator: NumericalBounds {
                range: 0.0..=1.0,
                name: "diversity",
            },
            word_count_validator: NumericalBounds {
                range: 0.0..=1.0,
                name: "word_count",
            },
            validation_count: AtomicUsize::new(0),
            error_count: AtomicUsize::new(0),
        }
    }

    fn validate_features(&self, features: &[f64]) -> Result<Vec<f64>> {
        self.validation_count.fetch_add(1, Ordering::Relaxed);
        let mut validated = Vec::with_capacity(features.len());

        for (i, &value) in features.iter().enumerate() {
            let validator = match i {
                0 => &self.length_validator,
                1 => &self.diversity_validator,
                2 => &self.word_count_validator,
                _ => &NumericalBounds { range: 0.0..=1.0, name: "feature" },
            };

            match validator.validate(value) {
                Ok(v) => validated.push(v),
                Err(e) => {
                    self.error_count.fetch_add(1, Ordering::Relaxed);
                    error!("Feature validation error: {}", e);
                    validated.push(validator.clamp(value));
                }
            }
        }

        // Log validation statistics periodically
        if self.validation_count.load(Ordering::Relaxed) % 100 == 0 {
            let error_rate = self.error_count.load(Ordering::Relaxed) as f64
                / self.validation_count.load(Ordering::Relaxed) as f64;
            info!(
                "Feature validation stats - Total: {}, Errors: {}, Error rate: {:.2}%",
                self.validation_count.load(Ordering::Relaxed),
                self.error_count.load(Ordering::Relaxed),
                error_rate * 100.0
            );
        }

        Ok(validated)
    }
}

/// Activation bounds enforcement
#[derive(Debug)]
struct ActivationBounds {
    bounds: NumericalBounds,
    recovery_strategy: RecoveryStrategy,
    violation_count: AtomicUsize,
}

#[derive(Debug, Clone, Copy)]
enum RecoveryStrategy {
    Clamp,
    Reset,
    Interpolate,
}

impl ActivationBounds {
    fn new() -> Self {
        Self {
            bounds: NumericalBounds {
                range: 0.0..=1.0,
                name: "activation",
            },
            recovery_strategy: RecoveryStrategy::Clamp,
            violation_count: AtomicUsize::new(0),
        }
    }

    fn enforce(&self, value: f64, prev_value: Option<f64>) -> f64 {
        match self.bounds.validate(value) {
            Ok(v) => v,
            Err(_) => {
                self.violation_count.fetch_add(1, Ordering::Relaxed);
                match self.recovery_strategy {
                    RecoveryStrategy::Clamp => self.bounds.clamp(value),
                    RecoveryStrategy::Reset => 0.0,
                    RecoveryStrategy::Interpolate => {
                        if let Some(prev) = prev_value {
                            (prev + value) / 2.0
                        } else {
                            self.bounds.clamp(value)
                        }
                    }
                }
            }
        }
    }
}

/// Numerical bounds for validation
#[derive(Debug, Clone, Copy)]
struct NumericalBounds {
    range: RangeInclusive<f64>,
    name: &'static str,
}

impl NumericalBounds {
    fn validate(&self, value: f64) -> Result<f64> {
        if !value.is_finite() {
            counter!("consciousness.numerical_validation.non_finite", 1);
            return Err(anyhow!("{} value is non-finite: {}", self.name, value));
        }
        
        if !self.range.contains(&value) {
            counter!("consciousness.numerical_validation.out_of_bounds", 1);
            return Err(anyhow!(
                "{} value {} outside valid range {:?}",
                self.name,
                value,
                self.range
            ));
        }
        
        Ok(value)
    }

    fn clamp(&self, value: f64) -> f64 {
        if !value.is_finite() {
            counter!("consciousness.numerical_validation.non_finite_clamped", 1);
            return *self.range.start();
        }
        value.clamp(*self.range.start(), *self.range.end())
    }
}

#[derive(Debug)]
struct FeatureStabilityCheck {
    variance_threshold: f64,
    min_sample_size: usize,
    recovery_default: f64,
    history: VecDeque<Vec<f64>>,
}

impl FeatureStabilityCheck {
    fn new(variance_threshold: f64, min_sample_size: usize, recovery_default: f64, history_size: usize) -> Self {
        Self {
            variance_threshold,
            min_sample_size,
            recovery_default,
            history: VecDeque::with_capacity(history_size),
        }
    }

    fn validate_feature(&mut self, values: &[f64], feature_index: usize) -> Result<f64> {
        // Add current values to history
        if self.history.len() >= self.history.capacity() {
            self.history.pop_front();
        }
        self.history.push_back(values.to_vec());

        // Check if we have enough samples
        if self.history.len() < self.min_sample_size {
            return Ok(values[feature_index]);
        }

        // Calculate variance for the specific feature across history
        let mut sum = 0.0;
        let mut sum_sq = 0.0;
        let mut count = 0;

        for historical_values in &self.history {
            if let Some(&value) = historical_values.get(feature_index) {
                sum += value;
                sum_sq += value * value;
                count += 1;
            }
        }

        if count == 0 {
            return Ok(self.recovery_default);
        }

        let mean = sum / count as f64;
        let variance = (sum_sq / count as f64) - (mean * mean);

        // Check variance against threshold
        if variance > self.variance_threshold {
            error!(
                "Feature {} variance {} exceeds threshold {}",
                feature_index, variance, self.variance_threshold
            );
            return Err(anyhow!(StabilityError {
                message: format!("Feature variance {} exceeds threshold", variance),
                feature_index,
                value: values[feature_index],
            }));
        }

        Ok(values[feature_index])
    }
}

/// Number of conceptual nodes in the Φ network.
const NUM_NODES: usize = 4;

/// Default off-diagonal connectivity weight between conceptual nodes.
///
/// We keep the network fully connected with a uniform weight for simplicity.
/// Diagonal entries are always zero (no self-coupling).
const DEFAULT_CONNECTIVITY_WEIGHT: f64 = 0.7;

/// Minimum integration value for numerical stability
const MIN_INTEGRATION_VALUE: f64 = 1e-12;

/// Activation thresholds for multi-stage denominator protection
const ACTIVATION_THRESHOLD_LOW: f64 = 0.1;
const ACTIVATION_THRESHOLD_HIGH: f64 = 0.9;

/// Calculator for Integrated Information.
///
/// This struct intentionally keeps the internal model small and cheap to
/// evaluate. All state needed for Φ calculation for a given piece of content
/// is derived on the fly from that content; there is no long-lived mutable
/// network state between calls.
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Feature vector cache using LRU policy
#[derive(Debug)]
struct FeatureCache {
    cache: LruCache<u64, Vec<f64>>,
    hit_count: AtomicUsize,
    miss_count: AtomicUsize,
    last_report: Instant,
}

impl FeatureCache {
    fn new(capacity: usize) -> Self {
        Self {
            cache: LruCache::new(capacity),
            hit_count: AtomicUsize::new(0),
            miss_count: AtomicUsize::new(0),
            last_report: Instant::now(),
        }
    }

    fn compute_hash(content: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        hasher.finish()
    }

    fn get(&mut self, content: &str) -> Option<Vec<f64>> {
        let key = Self::compute_hash(content);
        match self.cache.get(&key) {
            Some(features) => {
                self.hit_count.fetch_add(1, Ordering::Relaxed);
                Some(features.clone())
            }
            None => {
                self.miss_count.fetch_add(1, Ordering::Relaxed);
                None
            }
        }
    }

    fn insert(&mut self, content: &str, features: Vec<f64>) {
        let key = Self::compute_hash(content);
        self.cache.put(key, features);
        
        // Report cache statistics periodically
        let now = Instant::now();
        if now.duration_since(self.last_report).as_secs() >= 60 {
            let hits = self.hit_count.load(Ordering::Relaxed);
            let misses = self.miss_count.load(Ordering::Relaxed);
            let total = hits + misses;
            
            if total > 0 {
                let hit_rate = hits as f64 / total as f64;
                info!(
                    "Feature cache stats - Hits: {}, Misses: {}, Hit rate: {:.2}%",
                    hits,
                    misses,
                    hit_rate * 100.0
                );
                histogram!("consciousness.feature_cache.hit_rate", hit_rate);
            }
            
            self.last_report = now;
        }
    }
}

/// Network state consistency validator
#[derive(Debug)]
struct NetworkStateValidator {
    /// Sum of activations should be in this range
    activation_sum_bounds: RangeInclusive<f64>,
    /// Maximum allowed variance between connected nodes
    max_connection_variance: f64,
    /// Minimum required total connectivity
    min_total_connectivity: f64,
    /// Count of validation checks performed
    check_count: AtomicUsize,
    /// Count of validation failures
    failure_count: AtomicUsize,
}

impl NetworkStateValidator {
    fn new() -> Self {
        Self {
            activation_sum_bounds: 0.1..=3.0,
            max_connection_variance: 0.5,
            min_total_connectivity: 0.1,
            check_count: AtomicUsize::new(0),
            failure_count: AtomicUsize::new(0),
        }
    }

    fn validate_network_state(
        &self,
        activations: &[f64],
        connectivity: &Array2<f64>
    ) -> Result<()> {
        self.check_count.fetch_add(1, Ordering::Relaxed);
        
        // Check activation sum constraints
        let activation_sum: f64 = activations.iter().sum();
        if !self.activation_sum_bounds.contains(&activation_sum) {
            self.failure_count.fetch_add(1, Ordering::Relaxed);
            return Err(anyhow!(
                "Activation sum {} outside valid range {:?}",
                activation_sum,
                self.activation_sum_bounds
            ));
        }

        // Check connectivity matrix properties
        let total_connectivity: f64 = connectivity.sum();
        if total_connectivity < self.min_total_connectivity {
            self.failure_count.fetch_add(1, Ordering::Relaxed);
            return Err(anyhow!(
                "Total connectivity {} below minimum {}",
                total_connectivity,
                self.min_total_connectivity
            ));
        }

        // Verify activation differences between connected nodes
        let n = activations.len();
        for i in 0..n {
            for j in 0..n {
                if connectivity[[i, j]] > 0.0 {
                    let variance = (activations[i] - activations[j]).abs();
                    if variance > self.max_connection_variance {
                        self.failure_count.fetch_add(1, Ordering::Relaxed);
                        return Err(anyhow!(
                            "Connected nodes {},{} have excessive variance {}",
                            i, j, variance
                        ));
                    }
                }
            }
        }

        // Log validation statistics periodically
        if self.check_count.load(Ordering::Relaxed) % 100 == 0 {
            let failure_rate = self.failure_count.load(Ordering::Relaxed) as f64
                / self.check_count.load(Ordering::Relaxed) as f64;
            info!(
                "Network state validation stats - Total: {}, Failures: {}, Failure rate: {:.2}%",
                self.check_count.load(Ordering::Relaxed),
                self.failure_count.load(Ordering::Relaxed),
                failure_rate * 100.0
            );
        }

        Ok(())
    }
}

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
    /// Feature stability checker
    #[serde(skip)]
    stability_check: Option<FeatureStabilityCheck>,
    /// Feature validation pipeline
    #[serde(skip)]
    validation_pipeline: FeatureValidationPipeline,
    /// Activation bounds enforcement
    #[serde(skip)]
    activation_bounds: ActivationBounds,
    /// Network state validator
    #[serde(skip)]
    network_validator: NetworkStateValidator,
    /// Feature vector cache
    #[serde(skip)]
    feature_cache: FeatureCache,
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

        let calculator = Self {
            connectivity,
            epsilon: config.phi_epsilon,
            feature_max_length: config.feature_max_length.max(1.0),
            feature_max_words: config.feature_max_words.max(1.0),
            stability_check: Some(FeatureStabilityCheck::new(
                0.25, // variance threshold
                3,    // min sample size
                0.0,  // recovery default
                10,   // history size
            )),
            validation_pipeline: FeatureValidationPipeline::new(),
            activation_bounds: ActivationBounds::new(),
            network_validator: NetworkStateValidator::new(),
            feature_cache: FeatureCache::new(1000), // Cache up to 1000 feature vectors
        };

        // Validate initial network state
        if let Err(e) = calculator.network_validator.validate_network_state(
            &vec![0.0; NUM_NODES],
            &calculator.connectivity
        ) {
            warn!("Initial network state validation failed: {}", e);
        }

        calculator
    }

    /// Calculate Φ value from workspace content.
    ///
    /// This function is async to match the rest of the consciousness pipeline,
    /// but the current implementation is purely CPU-bound and does not perform
    /// any `.await` internally.
    pub async fn calculate(&mut self, content: &WorkspaceContent) -> Result<f64> {
        let start_time = Instant::now();

        // If, for any reason, the network is empty, degrade gracefully.
        if NUM_NODES == 0 {
            gauge!("consciousness.phi_value", 0.0);
            gauge!("consciousness.integration.raw_score", 0.0);
            gauge!("consciousness.integration.normalized_phi", 0.0);
            return Ok(0.0);
        }

        // 1. Extract normalized features from the content.
        let feature_start = Instant::now();
        let features = self.extract_features(&content.content);
        let feature_duration = feature_start.elapsed();
        histogram!("consciousness.feature_extraction.total_duration_ms", feature_duration.as_secs_f64() * 1000.0);

        // 2. Map features into conceptual node activations in [0, 1].
        let activation_start = Instant::now();
        let activations = self.update_network_state(&features);
        let activation_duration = activation_start.elapsed();
        histogram!("consciousness.activation_calculation.duration_ms", activation_duration.as_secs_f64() * 1000.0);

        // 3. Compute raw and normalized Φ scores.
        let phi_start = Instant::now();
        let (mut phi, mut raw_score) = self.compute_phi(&activations);
        let phi_duration = phi_start.elapsed();
        histogram!("consciousness.phi_calculation.duration_ms", phi_duration.as_secs_f64() * 1000.0);

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

        // Record total calculation time
        let total_duration = start_time.elapsed();
        histogram!("consciousness.total_calculation.duration_ms", total_duration.as_secs_f64() * 1000.0);

        // Record memory usage
        gauge!("consciousness.memory.feature_cache_size", self.feature_cache.cache.len() as f64);
        gauge!("consciousness.memory.feature_cache_capacity", self.feature_cache.cache.cap() as f64);

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
    fn extract_features(&mut self, content: &str) -> Vec<f64> {
        // Try to get features from cache first
        if let Some(cached_features) = self.feature_cache.get(content) {
            return cached_features;
        }

        let start_time = Instant::now();
        let len = content.chars().count() as f64;

        // Extract features in parallel
        let features: Vec<f64> = vec![
            self.extract_length_feature(content, len),
            self.extract_diversity_feature(content, len),
            self.extract_word_count_feature(content),
            self.extract_uppercase_feature(content, len),
            self.extract_punctuation_feature(content, len),
        ].into_par_iter()
         .map(|f| f.clamp(0.0, 1.0))
         .collect();

        let mut validated_features = Vec::with_capacity(5);

        // Run features through validation pipeline
        let validated_features = self.validation_pipeline.validate_features(&features)?;

        // Then validate for stability
        if let Some(stability_check) = &mut self.stability_check {
            let mut stable_features = Vec::with_capacity(validated_features.len());
            for (idx, &value) in validated_features.iter().enumerate() {
                match stability_check.validate_feature(&validated_features, idx) {
                    Ok(validated) => stable_features.push(validated),
                    Err(e) => {
                        error!("Feature stability error: {}", e);
                        stable_features.push(self.recover_unstable_feature(value, idx));
                    }
                }
            }
            stable_features
        } else {
            validated_features
        };

        // Cache the computed features before returning
        let features_clone = features.clone();
        self.feature_cache.insert(content, features_clone);

        // Record feature extraction time
        let duration = start_time.elapsed();
        histogram!("consciousness.feature_extraction.duration_ms", duration.as_secs_f64() * 1000.0);

        features
    }

    /// Recover from unstable feature values
    fn recover_unstable_feature(&self, value: f64, feature_index: usize) -> f64 {
        // Simple recovery strategy: clamp to valid range
        value.clamp(0.0, 1.0)
    }

    /// Weight matrices for each node's feature combination
    const NODE_WEIGHTS: [[f64; 5]; NUM_NODES] = [
        // Global Workspace
        [0.25, 0.25, 0.25, 0.15, 0.10],
        // Memory Integration
        [0.50, 0.10, 0.40, 0.00, 0.00],
        // Emotional Integration
        [0.00, 0.00, 0.20, 0.30, 0.50],
        // Predictive/Model
        [0.30, 0.50, 0.20, 0.00, 0.00],
    ];

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

        // Convert features to Array1 and ensure valid range
        let features = Array1::from_vec(features.iter().map(|&v| {
            if v.is_finite() { v.clamp(0.0, 1.0) } else { 0.0 }
        }).collect());

        // Create weight matrix
        let weights = Array2::from_shape_vec(
            (NUM_NODES, features.len()),
            Self::NODE_WEIGHTS.iter().flat_map(|w| w.iter().copied()).collect()
        ).unwrap();

        // Compute activations using matrix multiplication
        let activations = weights.dot(&features);

        // Enforce activation bounds with history-based recovery
        let mut bounded_activations = Vec::with_capacity(NUM_NODES);
        for (i, &a) in activations.iter().enumerate() {
            let prev_value = if i > 0 { Some(bounded_activations[i-1]) } else { None };
            bounded_activations.push(self.activation_bounds.enforce(a, prev_value));
        }

        // Validate network state consistency
        if let Err(e) = self.network_validator.validate_network_state(
            &bounded_activations,
            &self.connectivity
        ) {
            warn!("Network state validation failed: {}", e);
            // Fall back to previous valid state or default
            return vec![0.0; NUM_NODES];
        }

        bounded_activations
    }

    /// Compute adaptive epsilon based on network properties
    fn compute_adaptive_epsilon(&self, network_size: usize, activations: &[f64]) -> f64 {
        let base_epsilon = self.epsilon;
        
        // Calculate connectivity factor
        let connectivity_factor = self.connectivity.sum() / (network_size * network_size) as f64;
        
        // Calculate size factor using log scale
        let size_factor = (network_size as f64).log2().max(1.0);
        
        // Calculate activation spread factor
        let max_activation = activations.iter().copied().fold(0.0, f64::max);
        let min_activation = activations.iter().copied().fold(1.0, f64::min);
        let activation_spread = (max_activation - min_activation).abs();
        let spread_factor = activation_spread.max(0.1);

        base_epsilon * connectivity_factor * size_factor * spread_factor
    }

    /// Apply multi-stage denominator protection
    fn apply_denominator_protection(&self, raw_integration: f64, max_integration: f64, epsilon: f64) -> f64 {
        if max_integration <= epsilon {
            MIN_INTEGRATION_VALUE
        } else if max_integration <= epsilon * 10.0 {
            epsilon
        } else {
            max_integration
        }
    }

    /// Compute the Φ approximation given node activations.
    ///
    /// Returns `(phi, raw_score)` where:
    /// - `raw_score` is the unnormalized integration score
    ///   (`raw_integration * var_norm`), and
    /// - `phi` is the normalized Φ ∈ [0, 1].
    // Helper methods for parallel feature extraction
    fn extract_length_feature(&self, content: &str, len: f64) -> f64 {
        if len <= 0.0 {
            0.0
        } else {
            (len.min(self.feature_max_length)) / self.feature_max_length
        }
    }

    fn extract_diversity_feature(&self, content: &str, len: f64) -> f64 {
        if len <= 0.0 {
            0.0
        } else {
            let unique_chars = content.par_chars()
                .collect::<std::collections::HashSet<_>>();
            (unique_chars.len() as f64) / len
        }
    }

    fn extract_word_count_feature(&self, content: &str) -> f64 {
        let word_count = content.par_split_whitespace().count() as f64;
        if word_count <= 0.0 {
            0.0
        } else {
            (word_count.min(self.feature_max_words)) / self.feature_max_words
        }
    }

    fn extract_uppercase_feature(&self, content: &str, len: f64) -> f64 {
        if len <= 0.0 {
            0.0
        } else {
            let uppercase_count = content.par_chars()
                .filter(|c| c.is_ascii_uppercase())
                .count() as f64;
            uppercase_count / len
        }
    }

    fn extract_punctuation_feature(&self, content: &str, len: f64) -> f64 {
        if len <= 0.0 {
            0.0
        } else {
            let punctuation_chars = ['.', ',', '!', '?', ';', ':'];
            let punct_count = content.par_chars()
                .filter(|c| punctuation_chars.contains(c))
                .count() as f64;
            punct_count / len
        }
    }

    fn compute_phi(&self, activations: &[f64]) -> (f64, f64) {
        let start_time = Instant::now();
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

        // Calculate adaptive epsilon based on current network state
        let adaptive_epsilon = self.compute_adaptive_epsilon(n, activations);
        
        // Apply multi-stage denominator protection
        let protected_denom = self.apply_denominator_protection(raw_integration, max_integration, adaptive_epsilon);
        
        // Normalize to [0, 1] with protected denominator
        let phi = (raw_score / protected_denom).clamp(0.0, 1.0);

        // Record computation breakdown
        let duration = start_time.elapsed();
        histogram!("consciousness.phi_calculation.breakdown.duration_ms", duration.as_secs_f64() * 1000.0);
        gauge!("consciousness.phi_calculation.raw_integration", raw_integration);
        gauge!("consciousness.phi_calculation.max_integration", max_integration);
        gauge!("consciousness.phi_calculation.variance", variance);
        gauge!("consciousness.phi_calculation.var_norm", var_norm);
        gauge!("consciousness.phi_calculation.adaptive_epsilon", adaptive_epsilon);

        (phi, raw_score.max(0.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use tokio::test;
    use uuid::Uuid;
    use std::f64::INFINITY;
    use std::f64::NAN;

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
    
        #[test]
        async fn test_numerical_stability() {
            let calculator = PhiCalculator::new();
            
            // Test adaptive epsilon scaling
            let small_network = vec![0.1, 0.2];
            let large_network = vec![0.1; 100];
            
            let small_epsilon = calculator.compute_adaptive_epsilon(small_network.len(), &small_network);
            let large_epsilon = calculator.compute_adaptive_epsilon(large_network.len(), &large_network);
            
            assert!(large_epsilon > small_epsilon, "Epsilon should scale with network size");
    
            // Test denominator protection
            let test_cases = vec![
                (0.0, 0.0, MIN_INTEGRATION_VALUE),
                (1e-13, 1e-13, calculator.epsilon),
                (1.0, 1.0, 1.0),
            ];
    
            for (raw, max, expected) in test_cases {
                let protected = calculator.apply_denominator_protection(raw, max, calculator.epsilon);
                assert!((protected - expected).abs() < 1e-10);
            }
        }
    
        #[test]
        async fn test_feature_stability() {
            let mut calculator = PhiCalculator::new();
            
            // Test stable features
            let stable_content = "This is a stable test content.";
            let features1 = calculator.extract_features(stable_content);
            let features2 = calculator.extract_features(stable_content);
            
            assert_eq!(features1, features2, "Same content should produce same features");
            
            // Test feature bounds
            let extreme_content = "A".repeat(10000) + &"!".repeat(1000);
            let features = calculator.extract_features(&extreme_content);
            
            for f in features {
                assert!(f >= 0.0 && f <= 1.0, "Features should be bounded in [0,1]");
            }
        }
    
        #[test]
        async fn test_numerical_edge_cases() {
            let mut calculator = PhiCalculator::new();
            
            // Test NaN handling
            let nan_content = make_content(&"x".repeat(usize::MAX >> 20)); // Should cause numeric overflow
            let phi = calculator.calculate(&nan_content).await.unwrap();
            assert!(phi >= 0.0 && phi <= 1.0, "Phi should be bounded even with NaN inputs");
    
            // Test infinity handling
            let mut config = crate::config::ConsciousnessConfig::default();
            config.phi_epsilon = INFINITY;
            let inf_calculator = PhiCalculator::with_config(&config);
            let content = make_content("Test infinity handling");
            let phi = inf_calculator.calculate(&content).await.unwrap();
            assert!(phi >= 0.0 && phi <= 1.0, "Phi should be bounded even with infinite epsilon");
        }
    
        #[test]
        async fn test_variance_based_stability() {
            let mut calculator = PhiCalculator::new();
            
            // Test variance threshold enforcement
            let unstable_sequence = vec![
                "Short text",
                "Much longer text with more content!!!",
                "A",
                "Very very very long text with lots of variation!!!!!!!!",
            ];
    
            let mut features_history = Vec::new();
            for text in unstable_sequence {
                let features = calculator.extract_features(text);
                features_history.push(features.clone());
            }
    
            // Check if variance is being controlled
            let max_variance = features_history.windows(2)
                .map(|w| {
                    w[0].iter().zip(w[1].iter())
                        .map(|(&a, &b)| (a - b).powi(2))
                        .sum::<f64>()
                })
                .fold(0.0, f64::max);
    
            assert!(max_variance <= 0.25, "Feature variance should be bounded");
        }
    
        #[test]
        async fn test_network_state_validation() {
            let mut calculator = PhiCalculator::new();
            
            // Test activation bounds enforcement
            let test_cases = vec![
                ("Normal text", true),  // Should pass validation
                ("", false),           // Should fail (sum too low)
                ("!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!", false),  // Should fail (too much punctuation)
            ];
    
            for (text, should_pass) in test_cases {
                let content = make_content(text);
                let result = calculator.calculate(&content).await;
                
                match (result, should_pass) {
                    (Ok(_), true) => (), // Expected pass
                    (Ok(_), false) => panic!("Expected validation failure for: {}", text),
                    (Err(_), true) => panic!("Unexpected validation failure for: {}", text),
                    (Err(_), false) => (), // Expected failure
                }
            }
        }
    
        #[test]
        async fn test_activation_bounds() {
            let mut calculator = PhiCalculator::new();
            
            // Test activation bounds with different recovery strategies
            let test_activations = vec![
                vec![-0.5, 0.3, 1.5, 0.7],  // Out of bounds values
                vec![0.2, 0.4, 0.6, 0.8],   // Valid values
                vec![f64::INFINITY, 0.5, f64::NEG_INFINITY, 0.9], // Extreme values
            ];
    
            for activations in test_activations {
                let bounded = activations.iter().enumerate().map(|(i, &a)| {
                    let prev = if i > 0 { Some(activations[i-1]) } else { None };
                    calculator.activation_bounds.enforce(a, prev)
                }).collect::<Vec<_>>();
    
                // Verify bounds are enforced
                assert!(bounded.iter().all(|&x| x >= 0.0 && x <= 1.0),
                       "All activations should be in [0,1]");
            }
        }
    
        #[test]
        async fn test_state_recovery() {
            let mut calculator = PhiCalculator::new();
            
            // Test recovery from invalid states
            let invalid_states = vec![
                "".to_string(),  // Empty content
                "A".repeat(10000), // Extremely long content
                "!".repeat(1000),  // Excessive punctuation
                "\n".repeat(100),  // Many newlines
            ];
    
            for content in invalid_states {
                let result = calculator.calculate(&make_content(&content)).await.unwrap();
                
                // Verify recovery produces valid phi value
                assert!(result >= 0.0 && result <= 1.0,
                       "Recovery should produce valid phi value");
            }
        }
    
        #[test]
        async fn test_feature_validation_pipeline() {
            let mut calculator = PhiCalculator::new();
            
            // Test validation pipeline stages
            let test_cases = vec![
                ("Normal text", true),
                ("", true),  // Empty is valid but produces zero features
                ("A".repeat(10000), true),  // Long but valid
                ("\u{FFFF}".repeat(100), false),  // Invalid characters
            ];
    
            for (text, should_validate) in test_cases {
                let features = calculator.extract_features(text);
                
                // Check feature validation results
                assert!(features.iter().all(|&f| f >= 0.0 && f <= 1.0),
                       "Features should always be in valid range");
                
                if should_validate {
                    assert!(!features.is_empty(), "Valid input should produce features");
                }
            }
        }
    }

    #[test]
    async fn test_feature_extraction_in_range() {
        let calculator = PhiCalculator::new();
        let features = calculator.extract_features("Test content with VARIETY and punctuation!!");

        assert!(!features.is_empty());
        assert!(features.iter().all(|&f| f >= 0.0 && f <= 1.0));
    }
    
    #[test]
    async fn test_phi_calculation_with_custom_config() {
        // Create a custom config with different epsilon
        let mut config = crate::config::ConsciousnessConfig::default();
        config.phi_epsilon = 1e-3; // Different from default
        
        let calculator = PhiCalculator::with_config(&config);
        let content = make_content("Test content for custom config");
        
        let phi = calculator.calculate(&content).await.unwrap();
        
        // Verify phi is in valid range
        assert!(phi >= 0.0 && phi <= 1.0);
    }
    
    #[test]
    async fn test_network_state_update() {
        let calculator = PhiCalculator::new();
        
        // Test with empty features
        let empty_features: Vec<f64> = vec![];
        let empty_activations = calculator.update_network_state(&empty_features);
        assert!(empty_activations.is_empty());
        
        // Test with valid features
        let features = calculator.extract_features("Test content for network state");
        let activations = calculator.update_network_state(&features);
        
        // Verify activations are in valid range
        assert_eq!(activations.len(), NUM_NODES);
        assert!(activations.iter().all(|&a| a >= 0.0 && a <= 1.0));
        
        // Verify different nodes have different activations (diversity)
        // We'll check if there are at least two different values (with some tolerance)
        let mut has_different_values = false;
        for i in 0..activations.len() {
            for j in (i+1)..activations.len() {
                // Check if values are different with some tolerance
                if (activations[i] - activations[j]).abs() > 0.001 {
                    has_different_values = true;
                    break;
                }
            
                #[test]
                async fn test_feature_cache() {
                    let mut calculator = PhiCalculator::new();
                    
                    // Test cache hit/miss behavior
                    let test_content = "Test content for caching";
                    
                    // First call should be a cache miss
                    let features1 = calculator.extract_features(test_content);
                    
                    // Second call should be a cache hit
                    let features2 = calculator.extract_features(test_content);
                    
                    // Results should be identical
                    assert_eq!(features1, features2, "Cached features should match original");
                    
                    // Test cache capacity
                    for i in 0..2000 {  // More than cache capacity
                        let unique_content = format!("Unique content {}", i);
                        calculator.extract_features(&unique_content);
                    }
                    
                    // Verify cache size doesn't exceed capacity
                    assert!(calculator.feature_cache.cache.len() <= 1000,
                            "Cache size should not exceed capacity");
                }
            
                #[test]
                async fn test_parallel_processing() {
                    let mut calculator = PhiCalculator::new();
                    
                    // Test parallel feature extraction with large input
                    let large_content = "A".repeat(10000) + &"B".repeat(10000) + &"C".repeat(10000);
                    
                    let start = Instant::now();
                    let features = calculator.extract_features(&large_content);
                    let duration = start.elapsed();
                    
                    // Basic validation of parallel processing results
                    assert_eq!(features.len(), 5, "Should extract all features");
                    assert!(features.iter().all(|&f| f >= 0.0 && f <= 1.0),
                           "All features should be normalized");
                    
                    // Test vectorized activation calculations
                    let start = Instant::now();
                    let activations = calculator.update_network_state(&features);
                    let duration = start.elapsed();
                    
                    assert_eq!(activations.len(), NUM_NODES,
                              "Should compute activations for all nodes");
                }
            
                #[test]
                async fn test_performance_monitoring() {
                    let mut calculator = PhiCalculator::new();
                    let content = make_content("Test performance monitoring");
                    
                    // Calculate phi and verify metrics are recorded
                    let _ = calculator.calculate(&content).await.unwrap();
                    
                    // Note: We can't directly verify metric values in tests
                    // as they are typically collected by external monitoring systems.
                    // Instead, we verify the code executes without errors when
                    // recording metrics.
                }
            
                #[test]
                async fn test_error_handling_and_recovery() {
                    let mut calculator = PhiCalculator::new();
                    
                    // Test error handling for various edge cases
                    let edge_cases = vec![
                        "",                     // Empty content
                        "A".repeat(1_000_000), // Very large content
                        "\0",                  // Null character
                        "🦀",                  // Unicode emoji
                        "\n\t\r",             // Control characters
                    ];
            
                    for content in edge_cases {
                        let result = calculator.calculate(&make_content(&content)).await;
                        assert!(result.is_ok(), "Should handle edge case gracefully: {:?}", content);
                        
                        let phi = result.unwrap();
                        assert!(phi >= 0.0 && phi <= 1.0,
                               "Should produce valid phi value even for edge cases");
                    }
                }
            }
            if has_different_values {
                break;
            }
        }
        assert!(has_different_values, "Nodes should have different activation values");
    }
}