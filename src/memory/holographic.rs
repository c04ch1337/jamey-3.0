//! Holographic Memory Architecture
//!
//! Implements a holographic memory system inspired by Karl Pribram's holographic
//! brain theory. This allows for distributed storage and associative recall,
//! where memories are stored across the system in a wave interference pattern,
//! making the system more resilient and enabling content-addressable memory.

use std::sync::{Arc, RwLock};
use std::collections::VecDeque;
use anyhow::Result;
use ndarray::{Array2, arr2};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use metrics::{counter, gauge, histogram};
use rayon::prelude::*;
use thiserror::Error;
use lz4_flex::{compress_prepend_size, decompress_size_prepended};
use chrono::{DateTime, Utc};

#[derive(Debug, Error)]
pub enum MemoryError {
    #[error("Buffer capacity exceeded")]
    BufferCapacityExceeded,
    #[error("Compression failed: {0}")]
    CompressionFailed(String),
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

/// Circular buffer for storing recent metrics with fixed capacity.
///
/// Invariants:
/// - The logical length is always <= `capacity`.
/// - When `capacity == 0`, the buffer never stores any items (all inserts are dropped).
///
/// "Oldest" is defined as the element that has been in the buffer the longest,
/// which is always stored at the front of the underlying `VecDeque`.
#[derive(Debug)]
pub struct CircularBuffer<T> {
    /// Internal buffer storage, ordered from oldest (front) to newest (back).
    buffer: VecDeque<T>,
    /// Maximum capacity (number of elements) this buffer will retain.
    capacity: usize,
}

impl<T> CircularBuffer<T> {
    /// Create a new CircularBuffer with specified capacity.
    ///
    /// When `capacity == 0`, the buffer behaves as a disabled sink and never
    /// stores any elements. This is useful for configurations where metrics
    /// history should be completely disabled while keeping call sites simple.
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    /// Add an item to the buffer, removing the oldest element if at capacity.
    ///
    /// This operation is O(1) per insert. The invariants are:
    /// - `self.len()` is always `<= self.capacity()`
    /// - when `capacity == 0`, the buffer remains empty and the item is dropped.
    pub fn push(&mut self, item: T) -> Result<(), MemoryError> {
        // When capacity is zero, metrics history is effectively disabled.
        if self.capacity == 0 {
            gauge!("memory.circular_buffer.size", 0.0);
            gauge!("memory.circular_buffer.capacity", 0.0);
            return Ok(());
        }

        if self.buffer.len() == self.capacity {
            // Remove oldest item to make room for the new one.
            self.buffer.pop_front();
        }
        self.buffer.push_back(item);
        
        // Update metrics
        gauge!("memory.circular_buffer.size", self.buffer.len() as f64);
        gauge!("memory.circular_buffer.capacity", self.capacity as f64);
        
        Ok(())
    }

    /// Get the current number of items in the buffer.
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Maximum number of items the buffer will retain.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Check if the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Get an iterator over the buffer contents from oldest to newest.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.buffer.iter()
    }
}

/// Compressed storage for archiving metrics data
#[derive(Debug)]
pub struct CompressedStore<T> where T: Serialize + for<'de> Deserialize<'de> {
    /// Compressed data blocks
    blocks: Vec<CompressedBlock<T>>,
    /// Maximum size in bytes
    max_size: usize,
    /// Current size in bytes
    current_size: usize,
}

/// A compressed block of data with metadata
#[derive(Debug, Serialize, Deserialize)]
struct CompressedBlock<T> where T: Serialize + for<'de> Deserialize<'de> {
    /// Compressed data bytes
    data: Vec<u8>,
    /// Original data size
    original_size: usize,
    /// Compression timestamp
    timestamp: DateTime<Utc>,
    /// Data type marker
    _marker: std::marker::PhantomData<T>,
}

impl<T> CompressedStore<T> where T: Serialize + for<'de> Deserialize<'de> {
    /// Create a new CompressedStore with specified maximum size
    pub fn new(max_size: usize) -> Self {
        Self {
            blocks: Vec::new(),
            max_size,
            current_size: 0,
        }
    }

    /// Add data to compressed storage
    pub fn store(&mut self, data: &[T]) -> Result<(), MemoryError> {
        // Serialize and compress data
        let serialized = serde_json::to_vec(data)
            .map_err(|e| MemoryError::CompressionFailed(e.to_string()))?;
        
        let compressed = compress_prepend_size(&serialized);
        let block = CompressedBlock {
            data: compressed,
            original_size: serialized.len(),
            timestamp: Utc::now(),
            _marker: std::marker::PhantomData,
        };

        // Check if adding this block would exceed max size
        if self.current_size + block.data.len() > self.max_size {
            return Err(MemoryError::BufferCapacityExceeded);
        }

        // Update metrics
        histogram!("memory.compressed_store.compression_ratio",
                  block.original_size as f64 / block.data.len() as f64);
        gauge!("memory.compressed_store.total_size", self.current_size as f64);
        
        // Add block and update size
        self.current_size += block.data.len();
        self.blocks.push(block);
        
        Ok(())
    }

    /// Retrieve and decompress data from a specific block
    pub fn retrieve(&self, block_index: usize) -> Result<Vec<T>, MemoryError> {
        let block = self.blocks.get(block_index)
            .ok_or_else(|| MemoryError::InvalidConfig("Block index out of bounds".into()))?;

        // Decompress data
        let decompressed = decompress_size_prepended(&block.data)
            .map_err(|e| MemoryError::CompressionFailed(e.to_string()))?;

        // Deserialize
        serde_json::from_slice(&decompressed)
            .map_err(|e| MemoryError::CompressionFailed(e.to_string()))
    }

    /// Get the number of compressed blocks
    pub fn block_count(&self) -> usize {
        self.blocks.len()
    }

    /// Get the total size of compressed data
    pub fn total_size(&self) -> usize {
        self.current_size
    }

    /// Calculate the overall compression ratio
    pub fn compression_ratio(&self) -> f64 {
        let total_original: usize = self.blocks.iter()
            .map(|block| block.original_size)
            .sum();
        if self.current_size == 0 {
            1.0
        } else {
            total_original as f64 / self.current_size as f64
        }
    }
}

/// Configuration for consciousness metrics storage.
///
/// For Phase 1 we only enforce an explicit bound on the in-memory
/// recent-metrics buffer via `max_recent_size`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MetricsStorageConfig {
    /// Maximum size of recent metrics buffer.
    ///
    /// When set to 0, metrics history is disabled and no entries are stored.
    pub max_recent_size: usize,
}

impl Default for MetricsStorageConfig {
    fn default() -> Self {
        Self {
            max_recent_size: 1024,
        }
    }
}

impl MetricsStorageConfig {
    pub fn new(max_recent_size: usize) -> Self {
        Self { max_recent_size }
    }
}

/// Bounded in-memory store for recent consciousness metrics.
///
/// This is intentionally minimal for Phase 1: metrics are kept only in a
/// fixed-size circular buffer, and no archival/compression is performed yet.
#[derive(Debug)]
pub struct ConsciousnessMetricsStore {
    /// Circular buffer for recent metrics snapshots (oldest -> newest).
    recent_metrics: CircularBuffer<crate::consciousness::ConsciousnessMetrics>,
    /// Storage configuration.
    config: MetricsStorageConfig,
}

impl ConsciousnessMetricsStore {
    /// Create a new metrics store with the given configuration.
    pub fn new(config: MetricsStorageConfig) -> Self {
        Self {
            recent_metrics: CircularBuffer::new(config.max_recent_size),
            config,
        }
    }

    /// Record a new snapshot of consciousness metrics.
    ///
    /// Oldest entries are dropped first when the buffer is at capacity.
    /// Invariant: `self.len() <= self.capacity()` at all times.
    pub fn record(
        &mut self,
        metrics: crate::consciousness::ConsciousnessMetrics,
    ) -> Result<(), MemoryError> {
        self.recent_metrics.push(metrics)
    }

    /// Get an iterator over recent metrics, ordered from oldest to newest.
    pub fn iter(
        &self,
    ) -> impl Iterator<Item = &crate::consciousness::ConsciousnessMetrics> {
        self.recent_metrics.iter()
    }

    /// Current number of metrics snapshots stored.
    pub fn len(&self) -> usize {
        self.recent_metrics.len()
    }

    pub fn is_empty(&self) -> bool {
        self.recent_metrics.is_empty()
    }

    /// Maximum number of metrics entries to retain.
    pub fn capacity(&self) -> usize {
        self.config.max_recent_size
    }
}


/// Represents a holographic memory trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HolographicTrace {
    /// Unique identifier
    pub id: Uuid,
    /// Interference pattern matrix
    pub pattern: Array2<f64>,
    /// Original content hash
    pub content_hash: String,
    /// Creation timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Associated emotional tags
    pub emotional_tags: Vec<String>,
    /// Context associations
    pub context_associations: Vec<String>,
}

/// Holographic memory encoder/decoder
pub struct HolographicMemory {
    /// Fourier transform size
    transform_size: usize,
    /// Phase encoding resolution
    phase_resolution: usize,
    /// Minimum pattern strength
    min_strength: f64,
}

impl HolographicMemory {
    /// Create new holographic memory system
    pub fn new() -> Self {
        Self {
            transform_size: 1024,
            phase_resolution: 256,
            min_strength: 0.1,
        }
    }

    /// Encode content into holographic trace
    pub fn encode(
        &self,
        content: &str,
        emotional_tags: Vec<String>,
        context_associations: Vec<String>,
    ) -> Result<HolographicTrace> {
        // Convert content to frequency domain
        let frequencies = self.content_to_frequencies(content);
        
        // Generate reference wave
        let reference = self.generate_reference_wave();
        
        // Create interference pattern
        let pattern = self.create_interference_pattern(&frequencies, &reference);

        let trace = HolographicTrace {
            id: Uuid::new_v4(),
            pattern,
            content_hash: self.hash_content(content),
            timestamp: chrono::Utc::now(),
            emotional_tags,
            context_associations,
        };

        // Update metrics
        counter!("memory.holographic.encodings_total", 1);
        gauge!("memory.holographic.pattern_strength", self.calculate_pattern_strength(&trace));

        Ok(trace)
    }

    /// Decode content from holographic trace
    pub fn decode(&self, trace: &HolographicTrace) -> Result<String> {
        // Generate reference wave
        let reference = self.generate_reference_wave();
        
        // Extract frequencies from interference pattern
        let frequencies = self.extract_frequencies(&trace.pattern, &reference);
        
        // Convert frequencies back to content
        let content = self.frequencies_to_content(&frequencies)?;

        // Update metrics
        counter!("memory.holographic.decodings_total", 1);

        Ok(content)
    }

    /// Convert content to frequency domain
    fn content_to_frequencies(&self, content: &str) -> Array2<f64> {
        // Convert content to numerical representation
        let mut numerical: Vec<f64> = content.bytes()
            .map(|b| b as f64 / 255.0)
            .collect();

        // Pad to transform size
        numerical.resize(self.transform_size, 0.0);

        // Apply FFT
        let mut frequencies = Array2::zeros((self.transform_size, 2));
        for i in 0..self.transform_size {
            let phase = 2.0 * std::f64::consts::PI * i as f64 / self.transform_size as f64;
            frequencies[[i, 0]] = numerical[i] * phase.cos(); // Real
            frequencies[[i, 1]] = numerical[i] * phase.sin(); // Imaginary
        }

        frequencies
    }

    /// Generate reference wave for encoding/decoding
    fn generate_reference_wave(&self) -> Array2<f64> {
        let mut reference = Array2::zeros((self.transform_size, 2));
        
        for i in 0..self.transform_size {
            let phase = 2.0 * std::f64::consts::PI * i as f64 / self.phase_resolution as f64;
            reference[[i, 0]] = phase.cos();
            reference[[i, 1]] = phase.sin();
        }

        reference
    }

    /// Create interference pattern between content and reference
    fn create_interference_pattern(
        &self,
        frequencies: &Array2<f64>,
        reference: &Array2<f64>,
    ) -> Array2<f64> {
        let mut pattern = Array2::zeros((self.transform_size, self.transform_size));

        // Parallel computation of interference pattern
        pattern.axis_iter_mut(ndarray::Axis(0))
            .into_par_iter()
            .enumerate()
            .for_each(|(i, mut row)| {
                for j in 0..self.transform_size {
                    // Complex multiplication
                    let real = frequencies[[i, 0]] * reference[[j, 0]] -
                             frequencies[[i, 1]] * reference[[j, 1]];
                    let imag = frequencies[[i, 0]] * reference[[j, 1]] +
                             frequencies[[i, 1]] * reference[[j, 0]];
                    
                    // Store magnitude
                    row[j] = (real * real + imag * imag).sqrt();
                }
            });

        pattern
    }

    /// Extract frequencies from interference pattern
    fn extract_frequencies(
        &self,
        pattern: &Array2<f64>,
        reference: &Array2<f64>,
    ) -> Array2<f64> {
        let mut frequencies = Array2::zeros((self.transform_size, 2));

        // Parallel frequency extraction
        frequencies.axis_iter_mut(ndarray::Axis(0))
            .into_par_iter()
            .enumerate()
            .for_each(|(i, mut row)| {
                let mut real_sum = 0.0;
                let mut imag_sum = 0.0;

                for j in 0..self.transform_size {
                    real_sum += pattern[[i, j]] * reference[[j, 0]];
                    imag_sum += pattern[[i, j]] * reference[[j, 1]];
                }

                row[0] = real_sum / self.transform_size as f64;
                row[1] = imag_sum / self.transform_size as f64;
            });

        frequencies
    }

    /// Convert frequencies back to content
    fn frequencies_to_content(&self, frequencies: &Array2<f64>) -> Result<String> {
        let mut content = Vec::with_capacity(self.transform_size);

        for i in 0..self.transform_size {
            let magnitude = (frequencies[[i, 0]].powi(2) + frequencies[[i, 1]].powi(2)).sqrt();
            if magnitude > self.min_strength {
                let byte = (magnitude * 255.0).round() as u8;
                content.push(byte);
            }
        }

        String::from_utf8(content)
            .map_err(|e| anyhow::anyhow!("Failed to decode content: {}", e))
    }

    /// Calculate hash of content for verification
    fn hash_content(&self, content: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Calculate strength of interference pattern
    fn calculate_pattern_strength(&self, trace: &HolographicTrace) -> f64 {
        trace.pattern.iter().sum::<f64>() / (self.transform_size * self.transform_size) as f64
    }

    /// Find similar traces using pattern matching
    pub fn find_similar(&self, trace: &HolographicTrace, threshold: f64) -> Vec<f64> {
        trace.pattern.axis_iter(ndarray::Axis(0))
            .into_par_iter()
            .map(|row| {
                let similarity = row.iter()
                    .zip(trace.pattern.iter())
                    .map(|(a, b)| (a - b).powi(2))
                    .sum::<f64>()
                    .sqrt();
                
                if similarity < threshold {
                    similarity
                } else {
                    f64::INFINITY
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::soul::Emotion;

    #[test]
    fn test_encode_decode() {
        let memory = HolographicMemory::new();
        let content = "Test memory content";
        
        let trace = memory.encode(
            content,
            vec!["calm".to_string()],
            vec!["test".to_string()],
        ).unwrap();
        
        let decoded = memory.decode(&trace).unwrap();
        assert!(decoded.contains(content));
    }

    #[test]
    fn test_pattern_strength() {
        let memory = HolographicMemory::new();
        let trace = memory.encode(
            "Strong pattern test",
            vec![],
            vec![],
        ).unwrap();
        
        let strength = memory.calculate_pattern_strength(&trace);
        assert!(strength > 0.0);
        assert!(strength <= 1.0);
    }

    #[test]
    fn test_similar_patterns() {
        let memory = HolographicMemory::new();
        let trace1 = memory.encode(
            "Similar content A",
            vec![],
            vec![],
        ).unwrap();
        
        let trace2 = memory.encode(
            "Similar content B",
            vec![],
            vec![],
        ).unwrap();
        
        let similarities = memory.find_similar(&trace1, 0.5);
        assert!(!similarities.is_empty());
    }

    #[test]
    fn circular_buffer_enforces_bound_and_drops_oldest() {
        let mut buffer = CircularBuffer::new(3);
        buffer.push(1).unwrap();
        buffer.push(2).unwrap();
        buffer.push(3).unwrap();
        buffer.push(4).unwrap();
        buffer.push(5).unwrap();

        let items: Vec<_> = buffer.iter().cloned().collect();
        assert_eq!(buffer.len(), 3);
        assert_eq!(buffer.capacity(), 3);
        assert_eq!(items, vec![3, 4, 5]);
    }

    #[test]
    fn circular_buffer_zero_capacity_never_stores() {
        let mut buffer: CircularBuffer<u32> = CircularBuffer::new(0);
        buffer.push(1).unwrap();
        buffer.push(2).unwrap();
        buffer.push(3).unwrap();

        assert_eq!(buffer.len(), 0);
        assert_eq!(buffer.capacity(), 0);
        assert!(buffer.iter().next().is_none());
    }

    #[test]
    fn circular_buffer_one_capacity_keeps_most_recent_only() {
        let mut buffer = CircularBuffer::new(1);
        buffer.push("first").unwrap();
        buffer.push("second").unwrap();
        buffer.push("third").unwrap();

        let items: Vec<_> = buffer.iter().cloned().collect();
        assert_eq!(buffer.len(), 1);
        assert_eq!(buffer.capacity(), 1);
        assert_eq!(items, vec!["third"]);
    }

    #[test]
    fn metrics_store_bound_enforced_and_drops_oldest() {
        let config = MetricsStorageConfig::new(2);
        let mut store = ConsciousnessMetricsStore::new(config);

        let m1 = crate::consciousness::ConsciousnessMetrics {
            phi_value: 1.0,
            workspace_activity: 0.1,
            emotional_state: Emotion::default(),
            attention_focus: "first".to_string(),
            metacognition_level: 0.0,
        };
        let m2 = crate::consciousness::ConsciousnessMetrics {
            phi_value: 2.0,
            workspace_activity: 0.2,
            emotional_state: Emotion::default(),
            attention_focus: "second".to_string(),
            metacognition_level: 0.0,
        };
        let m3 = crate::consciousness::ConsciousnessMetrics {
            phi_value: 3.0,
            workspace_activity: 0.3,
            emotional_state: Emotion::default(),
            attention_focus: "third".to_string(),
            metacognition_level: 0.0,
        };

        store.record(m1).unwrap();
        store.record(m2).unwrap();
        store.record(m3).unwrap();

        let phi_values: Vec<f64> = store.iter().map(|m| m.phi_value).collect();
        assert_eq!(store.len(), 2);
        assert_eq!(store.capacity(), 2);
        assert_eq!(phi_values, vec![2.0, 3.0]);
    }

    #[test]
    fn metrics_store_zero_capacity_disables_storage() {
        let config = MetricsStorageConfig::new(0);
        let mut store = ConsciousnessMetricsStore::new(config);

        let m = crate::consciousness::ConsciousnessMetrics {
            phi_value: 1.0,
            workspace_activity: 0.1,
            emotional_state: Emotion::default(),
            attention_focus: "zero-capacity".to_string(),
            metacognition_level: 0.0,
        };

        store.record(m).unwrap();
        assert_eq!(store.len(), 0);
        assert_eq!(store.capacity(), 0);
        assert!(store.iter().next().is_none());
    }
}