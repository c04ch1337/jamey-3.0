//! Attention Schema Module
//!
//! Maps attention focus and directs cognitive resources

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::{HashMap, VecDeque, BinaryHeap, HashSet};
use std::cmp::Ordering;
use std::time::{Duration, Instant};
use chrono::{DateTime, Utc};
use super::global_workspace::WorkspaceContent;
use std::io::Cursor;
use flate2::write::GzEncoder;
use flate2::Compression;

// [Previous implementations remain unchanged]

/// Record of a single attention focus point
#[derive(Clone, Debug)]
pub struct AttentionRecord {
    focus: String,
    state: AttentionState,
    duration: Duration,
    context: HashMap<String, f32>,
    timestamp: DateTime<Utc>,
}

/// Circular buffer implementation for attention history
pub struct AttentionHistory {
    records: Arc<RwLock<VecDeque<AttentionRecord>>>,
    max_history_size: usize,
    retention_period: Duration,
    last_cleanup: Arc<RwLock<Instant>>,
}

impl AttentionHistory {
    pub fn new(max_size: usize, retention: Duration) -> Self {
        Self {
            records: Arc::new(RwLock::new(VecDeque::with_capacity(max_size))),
            max_history_size: max_size,
            retention_period: retention,
            last_cleanup: Arc::new(RwLock::new(Instant::now())),
        }
    }

    /// Add a new attention record
    pub async fn add_record(&self, record: AttentionRecord) -> Result<()> {
        let mut records = self.records.write().await;
        
        // Add new record
        records.push_back(record);
        
        // Maintain circular buffer size
        if records.len() > self.max_history_size {
            records.pop_front();
        }
        
        // Periodic cleanup of old records
        self.cleanup_old_records().await?;
        
        Ok(())
    }

    /// Clean up records older than retention period
    async fn cleanup_old_records(&self) -> Result<()> {
        let mut last_cleanup = self.last_cleanup.write().await;
        let now = Instant::now();
        
        // Only cleanup periodically
        if now.duration_since(*last_cleanup) < Duration::from_secs(3600) {
            return Ok(());
        }
        
        let mut records = self.records.write().await;
        let cutoff = Utc::now() - chrono::Duration::from_std(self.retention_period)?;
        
        records.retain(|record| record.timestamp > cutoff);
        *last_cleanup = now;
        
        Ok(())
    }

    /// Get records within a time window
    pub async fn get_records(&self, window: Duration) -> Vec<AttentionRecord> {
        let records = self.records.read().await;
        let cutoff = Utc::now() - chrono::Duration::from_std(window).unwrap_or(chrono::Duration::zero());
        
        records.iter()
            .filter(|record| record.timestamp > cutoff)
            .cloned()
            .collect()
    }
}

/// Pattern recognition in attention sequences
#[derive(Clone, Debug)]
pub struct AttentionPattern {
    sequence: Vec<String>,
    frequency: usize,
    avg_duration: Duration,
    last_seen: DateTime<Utc>,
}

/// Temporal context analysis system
pub struct TemporalContext {
    short_term: VecDeque<AttentionRecord>,
    medium_term: Vec<AttentionPattern>,
    long_term: HashMap<String, AttentionStatistics>,
    pattern_min_length: usize,
    pattern_max_length: usize,
}

#[derive(Clone, Debug)]
pub struct AttentionStatistics {
    total_occurrences: usize,
    total_duration: Duration,
    avg_priority: f32,
    last_occurrence: DateTime<Utc>,
}

impl TemporalContext {
    pub fn new(
        short_term_size: usize,
        min_pattern_len: usize,
        max_pattern_len: usize
    ) -> Self {
        Self {
            short_term: VecDeque::with_capacity(short_term_size),
            medium_term: Vec::new(),
            long_term: HashMap::new(),
            pattern_min_length: min_pattern_len,
            pattern_max_length: max_pattern_len,
        }
    }

    /// Update context with new attention record
    pub fn update(&mut self, record: AttentionRecord) {
        // Update short-term memory
        self.short_term.push_back(record.clone());
        if self.short_term.len() > self.short_term.capacity() {
            self.short_term.pop_front();
        }

        // Update long-term statistics
        let stats = self.long_term.entry(record.focus.clone()).or_insert(AttentionStatistics {
            total_occurrences: 0,
            total_duration: Duration::from_secs(0),
            avg_priority: 0.0,
            last_occurrence: record.timestamp,
        });
        
        stats.total_occurrences += 1;
        stats.total_duration += record.duration;
        stats.last_occurrence = record.timestamp;

        // Detect patterns in short-term memory
        self.detect_patterns();
    }

    /// Detect attention patterns in short-term memory
    fn detect_patterns(&mut self) {
        let sequences: Vec<String> = self.short_term.iter()
            .map(|record| record.focus.clone())
            .collect();

        // Look for patterns of different lengths
        for len in self.pattern_min_length..=self.pattern_max_length {
            if len > sequences.len() {
                continue;
            }

            for window in sequences.windows(len) {
                let pattern = window.to_vec();
                if let Some(existing) = self.medium_term.iter_mut()
                    .find(|p| p.sequence == pattern)
                {
                    existing.frequency += 1;
                    existing.last_seen = Utc::now();
                } else {
                    self.medium_term.push(AttentionPattern {
                        sequence: pattern,
                        frequency: 1,
                        avg_duration: Duration::from_secs(0),
                        last_seen: Utc::now(),
                    });
                }
            }
        }
    }

    /// Get most frequent patterns
    pub fn get_frequent_patterns(&self, min_frequency: usize) -> Vec<AttentionPattern> {
        self.medium_term.iter()
            .filter(|p| p.frequency >= min_frequency)
            .cloned()
            .collect()
    }

    /// Predict next likely focus based on current sequence
    pub fn predict_next_focus(&self, current_sequence: &[String]) -> Option<String> {
        self.medium_term.iter()
            .filter(|pattern| {
                pattern.sequence.len() > current_sequence.len() &&
                pattern.sequence.starts_with(current_sequence)
            })
            .max_by_key(|pattern| pattern.frequency)
            .map(|pattern| pattern.sequence[current_sequence.len()].clone())
    }
}

// Update AttentionSchema to use history system
pub struct AttentionSchema {
    current_focus: Arc<RwLock<String>>,
    semantic_analyzer: SemanticAnalyzer,
    relevance_scorer: RelevanceScore,
    weights: AttentionWeights,
    state_machine: AttentionStateMachine,
    queue: AttentionQueue,
    conflict_resolver: ConflictResolver,
    history: AttentionHistory,
    temporal_context: Arc<RwLock<TemporalContext>>,
}

impl AttentionSchema {
    pub fn new() -> Self {
        Self {
            current_focus: Arc::new(RwLock::new(String::new())),
            semantic_analyzer: SemanticAnalyzer::new(5),
            relevance_scorer: RelevanceScore::new(0.0, 0.0, 0.0, 1.0),
            weights: AttentionWeights::new(),
            state_machine: AttentionStateMachine::new(3, Duration::from_millis(100)),
            queue: AttentionQueue::new(20, 0.5),
            conflict_resolver: ConflictResolver::new(0.7, Duration::from_millis(200), 3),
            history: AttentionHistory::new(1000, Duration::from_secs(3600 * 24)), // 24 hour retention
            temporal_context: Arc::new(RwLock::new(TemporalContext::new(20, 2, 5))),
        }
    }

    // [Previous methods remain unchanged]

    /// Process attention queue with history tracking
    pub async fn process_queue(&self) -> Result<()> {
        while let Some(item) = self.queue.dequeue().await {
            let current_state = self.state_machine.current_state().await;
            
            match self.conflict_resolver.resolve_conflict(&current_state, &item).await {
                Resolution::Allow => {
                    if self.conflict_resolver.acquire_lock(item.content.clone()).await? {
                        if let Ok(transitioned) = self.state_machine.transition(item.content.clone()).await {
                            if transitioned {
                                let start_time = Instant::now();
                                let mut focus = self.current_focus.write().await;
                                *focus = item.content.clone();
                                
                                // Create and store attention record
                                let record = AttentionRecord {
                                    focus: item.content.clone(),
                                    state: current_state.clone(),
                                    duration: start_time.elapsed(),
                                    context: HashMap::new(), // Could be populated with relevant context
                                    timestamp: Utc::now(),
                                };
                                
                                self.history.add_record(record.clone()).await?;
                                self.temporal_context.write().await.update(record);
                                
                                metrics::counter!("consciousness.attention.shifts_total").increment(1);
                            }
                        }
                        self.conflict_resolver.release_lock(&item.content).await;
                    }
                }
                // [Rest of match arms remain unchanged]
                _ => {}
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // [Previous tests remain unchanged]

    #[tokio::test]
    async fn test_attention_history() {
        let history = AttentionHistory::new(5, Duration::from_secs(3600));
        
        // Add some records
        let record = AttentionRecord {
            focus: "test".to_string(),
            state: AttentionState::Focused("test".to_string()),
            duration: Duration::from_secs(1),
            context: HashMap::new(),
            timestamp: Utc::now(),
        };
        
        history.add_record(record.clone()).await.unwrap();
        
        // Check retrieval
        let records = history.get_records(Duration::from_secs(3600)).await;
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].focus, "test");
    }

    #[tokio::test]
    async fn test_temporal_context() {
        let mut context = TemporalContext::new(5, 2, 3);
        
        // Add sequence of records
        let record1 = AttentionRecord {
            focus: "A".to_string(),
            state: AttentionState::Focused("A".to_string()),
            duration: Duration::from_secs(1),
            context: HashMap::new(),
            timestamp: Utc::now(),
        };
        
        let record2 = AttentionRecord {
            focus: "B".to_string(),
            state: AttentionState::Focused("B".to_string()),
            duration: Duration::from_secs(1),
            context: HashMap::new(),
            timestamp: Utc::now(),
        };
        
        context.update(record1);
        context.update(record2.clone());
        context.update(record1.clone());
        context.update(record2);
        
        // Check pattern detection
        let patterns = context.get_frequent_patterns(2);
        assert!(!patterns.is_empty(), "Should detect repeated patterns");
    }

    #[tokio::test]
    async fn test_pattern_prediction() {
        let mut context = TemporalContext::new(5, 2, 3);
        
        // Create repeating pattern A -> B -> C
        for _ in 0..3 {
            for focus in ["A", "B", "C"].iter() {
                let record = AttentionRecord {
                    focus: focus.to_string(),
                    state: AttentionState::Focused(focus.to_string()),
                    duration: Duration::from_secs(1),
                    context: HashMap::new(),
                    timestamp: Utc::now(),
                };
                context.update(record);
            }
        }
        
        // Test prediction
        let sequence = vec!["A".to_string(), "B".to_string()];
        if let Some(prediction) = context.predict_next_focus(&sequence) {
            assert_eq!(prediction, "C");
        } else {
            panic!("Should predict next focus in pattern");
        }
    }

    #[test]
    fn test_retention_policy() {
        let mut policy = RetentionPolicy::new(
            Duration::from_secs(3600),
            Duration::from_secs(86400),
            0.1,
            0.7,
        );

        let mut records = VecDeque::new();
        let now = Utc::now();

        // Add records of varying ages
        for i in 0..10 {
            let record = AttentionRecord {
                focus: format!("focus_{}", i),
                state: AttentionState::Focused(format!("focus_{}", i)),
                duration: Duration::from_secs(i * 100),
                context: HashMap::new(),
                timestamp: now - chrono::Duration::hours(i),
            };
            records.push_back(record);
        }

        // Apply policy
        policy.apply_policy(&mut records).unwrap();

        // Verify retention
        assert!(records.len() < 10, "Some records should be filtered out");
    }

    #[tokio::test]
    async fn test_record_compression() {
        let history = AttentionHistory::new(5, Duration::from_secs(3600));
        
        // Add some records
        for i in 0..10 {
            let record = AttentionRecord {
                focus: format!("focus_{}", i),
                state: AttentionState::Focused(format!("focus_{}", i)),
                duration: Duration::from_secs(i),
                context: HashMap::new(),
                timestamp: Utc::now() - chrono::Duration::hours(i),
            };
            history.add_record(record).await.unwrap();
        }

        // Archive old records
        history.archive_old_records(Duration::from_secs(3600)).await.unwrap();
        
        // Check compression
        let compressed = history.compressed_records.read().await;
        assert!(!compressed.is_empty(), "Should have compressed records");
    }

    #[tokio::test]
    async fn test_indexed_search() {
        let history = AttentionHistory::new(5, Duration::from_secs(3600));
        
        // Add records with specific focus
        let record = AttentionRecord {
            focus: "test_focus".to_string(),
            state: AttentionState::Focused("test_focus".to_string()),
            duration: Duration::from_secs(1),
            context: HashMap::new(),
            timestamp: Utc::now(),
        };
        history.add_record(record).await.unwrap();
        
        // Search by focus
        let results = history.search_by_focus("test_focus").await;
        assert_eq!(results.len(), 1, "Should find record by focus");
        assert_eq!(results[0].focus, "test_focus");
    }
}
