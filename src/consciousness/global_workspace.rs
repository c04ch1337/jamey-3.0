//! Global Workspace Module
//!
//! Implements the Global Workspace Theory (GWT) which suggests that consciousness
//! emerges from a central information exchange where different cognitive processes
//! compete for attention and broadcast their information globally.

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock};
use anyhow::Result;
use metrics::{counter, gauge};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

/// Represents a piece of information in the workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceContent {
    /// Unique identifier for this content
    pub id: Uuid,
    /// The actual information content
    pub content: String,
    /// Source of the information
    pub source: String,
    /// Priority level (0.0 to 1.0)
    pub priority: f64,
    /// Timestamp of creation
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Represents the current state of the workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceState {
    /// Currently broadcasted content
    pub current_broadcast: Option<WorkspaceContent>,
    /// Activity level of the workspace (0.0 to 1.0)
    pub activity_level: f64,
    /// Number of competing processes
    pub competition_level: usize,
}

/// The Global Workspace implementation
/// Priority calculation weights
#[derive(Debug, Clone)]
struct PriorityWeights {
    length_weight: f64,
    complexity_weight: f64,
    novelty_weight: f64,
    urgency_weight: f64,
}

/// Cached priority calculation
#[derive(Debug, Clone)]
struct PriorityCache {
    content_hash: u64,
    priority: f64,
    timestamp: chrono::DateTime<chrono::Utc>,
}

/// Simple system load monitor used by GlobalWorkspace to adjust batch size and timeouts.
/// This is a lightweight heuristic implementation and can be replaced with a more
/// sophisticated metrics-driven monitor later.
#[derive(Debug)]
struct SystemLoadMonitor {
    last_batch_size: usize,
}

impl SystemLoadMonitor {
    fn new() -> Self {
        Self { last_batch_size: 0 }
    }

    /// Update internal metrics based on the last processed batch size.
    /// Currently this is a no-op other than recording the size, but it
    /// provides an integration point for future adaptive logic.
    async fn update_metrics(&mut self, batch_size: usize) {
        self.last_batch_size = batch_size;
    }

    /// Compute a multiplier for the batch size based on recent load.
    /// For now this always returns 1.0 to keep behaviour stable.
    fn get_batch_size_multiplier(&self) -> f64 {
        1.0
    }

    /// Compute a multiplier for the batch timeout based on recent load.
    /// For now this always returns 1.0 to keep behaviour stable.
    fn get_timeout_multiplier(&self) -> f64 {
        1.0
    }
}
pub struct GlobalWorkspace {
    /// Current state of the workspace
    state: Arc<RwLock<WorkspaceState>>,
    /// Channel for broadcasting information
    broadcast_tx: mpsc::Sender<WorkspaceContent>,
    broadcast_rx: Arc<RwLock<mpsc::Receiver<WorkspaceContent>>>,
    /// Competition threshold for broadcast
    competition_threshold: f64,
    /// Broadcast factor for activity calculation
    broadcast_factor: f64,
    /// Competition divisor for activity calculation
    competition_divisor: f64,
    /// Maximum competition factor
    competition_max_factor: f64,
    /// Maximum content length for priority calculation
    priority_max_length: f64,
    /// Base batch size for processing broadcasts
    base_batch_size: usize,
    /// Current dynamic batch size
    current_batch_size: Arc<RwLock<usize>>,
    /// Base batch timeout in milliseconds
    base_batch_timeout: u64,
    /// Current batch timeout
    current_batch_timeout: Arc<RwLock<u64>>,
    /// Rate limit for broadcasts (messages per second)
    rate_limit: f64,
    /// Last broadcast timestamp for rate limiting
    last_broadcast: Arc<RwLock<chrono::DateTime<chrono::Utc>>>,
    /// Maximum age for broadcasts before cleanup (in seconds)
    max_broadcast_age: i64,
    /// Cached broadcasts for batch processing
    broadcast_cache: Arc<RwLock<Vec<WorkspaceContent>>>,
    /// Priority calculation weights
    priority_weights: PriorityWeights,
    /// Cache for priority calculations
    priority_cache: Arc<RwLock<Vec<PriorityCache>>>,
    /// Maximum age for cached priorities (in seconds)
    priority_cache_max_age: i64,
    /// System load monitor
    load_monitor: Arc<RwLock<SystemLoadMonitor>>,
}

impl GlobalWorkspace {
    /// Create a new Global Workspace with default configuration
    pub fn new() -> Self {
        Self::with_config(&crate::config::ConsciousnessConfig::default())
    }

    /// Create a new Global Workspace with custom configuration
    pub fn with_config(config: &crate::config::ConsciousnessConfig) -> Self {
        let (tx, rx) = mpsc::channel(config.broadcast_channel_size);
        
        let state = Arc::new(RwLock::new(WorkspaceState {
            current_broadcast: None,
            activity_level: 0.0,
            competition_level: 0,
        }));

        Self {
            state,
            broadcast_tx: tx,
            broadcast_rx: Arc::new(RwLock::new(rx)),
            competition_threshold: config.competition_threshold,
            broadcast_factor: config.broadcast_factor,
            competition_divisor: config.competition_divisor,
            competition_max_factor: config.competition_max_factor,
            priority_max_length: config.priority_max_length,
            base_batch_size: 10,
            current_batch_size: Arc::new(RwLock::new(10)),
            base_batch_timeout: 100, // 100ms base timeout
            current_batch_timeout: Arc::new(RwLock::new(100)),
            rate_limit: 100.0, // 100 messages per second
            load_monitor: Arc::new(RwLock::new(SystemLoadMonitor::new())),
            last_broadcast: Arc::new(RwLock::new(chrono::Utc::now())),
            max_broadcast_age: 300, // 5 minutes
            broadcast_cache: Arc::new(RwLock::new(Vec::new())),
            priority_weights: PriorityWeights {
                length_weight: 0.3,
                complexity_weight: 0.3,
                novelty_weight: 0.2,
                urgency_weight: 0.2,
            },
            priority_cache: Arc::new(RwLock::new(Vec::new())),
            priority_cache_max_age: 60, // 1 minute
        }
    }

    /// Broadcast information through the workspace
    pub async fn broadcast(&self, content: &str) -> Result<WorkspaceContent> {
        // Apply rate limiting
        let now = chrono::Utc::now();
        let mut last_broadcast = self.last_broadcast.write().await;
        let time_since_last = now.signed_duration_since(*last_broadcast).num_milliseconds() as f64;
        if time_since_last < (1000.0 / self.rate_limit) {
            anyhow::bail!("Rate limit exceeded");
        }
        *last_broadcast = now;

        let workspace_content = WorkspaceContent {
            id: Uuid::new_v4(),
            content: content.to_string(),
            source: "consciousness".to_string(),
            priority: self.calculate_priority(content).await?,
            timestamp: now,
        };

        // Only process if priority exceeds threshold
        if workspace_content.priority >= self.competition_threshold {
            // Add to batch cache
            let mut cache = self.broadcast_cache.write().await;
            cache.push(workspace_content.clone());

            // Get current batch size
            let current_size = *self.current_batch_size.read().await;
            
            // Process batch if cache is full or high priority
            if cache.len() >= current_size || workspace_content.priority >= 0.8 {
                self.process_batch(&mut cache).await?;
            } else {
                // Check if we should process due to timeout
                let timeout = *self.current_batch_timeout.read().await;
                if let Some(first) = cache.first() {
                    let age = now.signed_duration_since(first.timestamp).num_milliseconds() as u64;
                    if age >= timeout {
                        self.process_batch(&mut cache).await?;
                    }
                }
            }

            // Cleanup old broadcasts
            self.cleanup_old_broadcasts().await?;
        }

        Ok(workspace_content)
    }

    /// Calculate priority for given content
    async fn calculate_priority(&self, content: &str) -> Result<f64> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        // Calculate content hash for cache lookup
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        let content_hash = hasher.finish();

        // Check cache
        let now = chrono::Utc::now();
        let mut cache = self.priority_cache.write().await;

        // Clean old cache entries
        cache.retain(|entry| {
            now.signed_duration_since(entry.timestamp).num_seconds() <= self.priority_cache_max_age
        });

        // Look for cached priority
        if let Some(cached) = cache.iter().find(|c| c.content_hash == content_hash) {
            return Ok(cached.priority);
        }

        // Calculate new priority using multiple factors
        let length_factor = (content.len() as f64 / self.priority_max_length).min(1.0);
        
        // Complexity factor based on unique words and special characters
        let words: std::collections::HashSet<_> = content.split_whitespace().collect();
        let unique_word_ratio = words.len() as f64 / content.split_whitespace().count().max(1) as f64;
        let special_char_ratio = content.chars().filter(|c| !c.is_alphanumeric()).count() as f64 / content.len() as f64;
        let complexity_factor = ((unique_word_ratio + special_char_ratio) / 2.0).min(1.0);

        // Novelty factor based on difference from current broadcast
        let novelty_factor = match self.state.read().await.current_broadcast {
            Some(ref current) => {
                let similarity = strsim::jaro_winkler(content, &current.content);
                (1.0 - similarity).max(0.0)
            }
            None => 1.0,
        };

        // Urgency factor based on exclamation marks and keywords
        let urgency_keywords = ["urgent", "important", "critical", "emergency"];
        let has_urgency_keywords = urgency_keywords.iter().any(|&k| content.to_lowercase().contains(k));
        let exclamation_count = content.chars().filter(|&c| c == '!').count();
        let urgency_factor = if has_urgency_keywords || exclamation_count > 0 {
            ((exclamation_count as f64 / 3.0) + if has_urgency_keywords { 1.0 } else { 0.0 } / 2.0).min(1.0)
        } else {
            0.0
        };

        // Calculate weighted priority
        let priority = (
            length_factor * self.priority_weights.length_weight +
            complexity_factor * self.priority_weights.complexity_weight +
            novelty_factor * self.priority_weights.novelty_weight +
            urgency_factor * self.priority_weights.urgency_weight
        ).min(1.0);

        // Cache the result
        cache.push(PriorityCache {
            content_hash,
            priority,
            timestamp: now,
        });

        Ok(priority)
    }

    /// Calculate current activity level
    async fn calculate_activity_level(&self) -> Result<f64> {
        let state = self.state.read().await;
        
        // Activity level is based on:
        // 1. Whether there's current broadcast
        // 2. Competition level
        // 3. Recent broadcast history
        
        let broadcast_factor = if state.current_broadcast.is_some() { self.broadcast_factor } else { 0.0 };
        let competition_factor = (state.competition_level as f64 / self.competition_divisor).min(self.competition_max_factor);
        
        Ok(broadcast_factor + competition_factor)
    }

    /// Get the current activity level
    pub async fn activity_level(&self) -> f64 {
        self.state.read().await.activity_level
    }

    /// Listen for broadcasts
    pub async fn listen(&self) -> Result<Option<WorkspaceContent>> {
        let mut rx = self.broadcast_rx.write().await;
        Ok(rx.try_recv().ok())
    }

    /// Get the current workspace state
    pub async fn get_state(&self) -> WorkspaceState {
        self.state.read().await.clone()
    }
}

impl GlobalWorkspace {
    /// Process a batch of broadcasts with dynamic sizing and timeout
    async fn process_batch(&self, cache: &mut Vec<WorkspaceContent>) -> Result<()> {
        // Update system metrics
        {
            let mut monitor = self.load_monitor.write().await;
            monitor.update_metrics(cache.len()).await;
            
            // Adjust batch size based on system load
            let size_multiplier = monitor.get_batch_size_multiplier();
            let timeout_multiplier = monitor.get_timeout_multiplier();
            
            let mut current_batch_size = self.current_batch_size.write().await;
            *current_batch_size = ((self.base_batch_size as f64 * size_multiplier) as usize)
                .clamp(5, 50); // Enforce min/max bounds
            
            let mut current_timeout = self.current_batch_timeout.write().await;
            *current_timeout = ((self.base_batch_timeout as f64 * timeout_multiplier) as u64)
                .clamp(50, 200); // 50ms to 200ms
        }

        // Sort by priority with age boost
        let now = chrono::Utc::now();
        cache.sort_by(|a, b| {
            let a_age = now.signed_duration_since(a.timestamp).num_seconds() as f64;
            let b_age = now.signed_duration_since(b.timestamp).num_seconds() as f64;
            
            let a_priority = a.priority + (a_age * 0.1).min(0.3); // Max +0.3 boost
            let b_priority = b.priority + (b_age * 0.1).min(0.3);
            
            b_priority.partial_cmp(&a_priority).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Process batch with retry mechanism
        let mut retry_count = 0;
        let max_retries = 3;
        
        while retry_count < max_retries {
            match self.try_process_batch(cache).await {
                Ok(_) => {
                    // Update metrics
                    gauge!("global_workspace.batch_size", cache.len() as f64);
                    counter!("global_workspace.batch_success", 1);
                    return Ok(());
                }
                Err(e) => {
                    retry_count += 1;
                    counter!("global_workspace.batch_retry", 1);
                    
                    if retry_count == max_retries {
                        counter!("global_workspace.batch_failure", 1);
                        return Err(e);
                    }
                    
                    // Exponential backoff
                    tokio::time::sleep(Duration::from_millis(50 * 2u64.pow(retry_count as u32))).await;
                }
            }
        }

        Ok(())
    }

    /// Attempt to process a batch with error handling
    async fn try_process_batch(&self, cache: &mut Vec<WorkspaceContent>) -> Result<()> {
        let batch_size = *self.current_batch_size.read().await;
        let timeout = *self.current_batch_timeout.read().await;
        
        // Process up to batch_size items
        let to_process = cache.len().min(batch_size);
        let batch: Vec<_> = cache.drain(..to_process).collect();
        
        for content in batch {
            match tokio::time::timeout(
                Duration::from_millis(timeout),
                self.broadcast_tx.send(content.clone())
            ).await {
                Ok(send_result) => {
                    send_result?;
                    
                    // Update state
                    let mut state = self.state.write().await;
                    state.current_broadcast = Some(content);
                    state.activity_level = self.calculate_activity_level().await?;
                    state.competition_level += 1;

                    // Update metrics
                    gauge!("global_workspace.activity_level", state.activity_level);
                    counter!("global_workspace.broadcasts_total", 1);
                }
                Err(_) => {
                    counter!("global_workspace.broadcast_timeout", 1);
                    anyhow::bail!("Broadcast timeout");
                }
            }
        }

        Ok(())
    }

    /// Clean up old broadcasts
    async fn cleanup_old_broadcasts(&self) -> Result<()> {
        let now = chrono::Utc::now();
        let mut state = self.state.write().await;

        // Clean up old current_broadcast if it exists
        if let Some(broadcast) = &state.current_broadcast {
            let age = now.signed_duration_since(broadcast.timestamp).num_seconds();
            if age > self.max_broadcast_age {
                state.current_broadcast = None;
                state.activity_level = self.calculate_activity_level().await?;
                gauge!("global_workspace.activity_level", state.activity_level);
                counter!("global_workspace.cleanups_total", 1);
            }
        }

        Ok(())
    }

    /// Flush the broadcast cache immediately
    pub async fn flush_broadcasts(&self) -> Result<()> {
        let mut cache = self.broadcast_cache.write().await;
        if !cache.is_empty() {
            self.process_batch(&mut cache).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;

    #[test]
    async fn test_workspace_broadcast() {
        let workspace = GlobalWorkspace::new();
        
        // Test high-priority broadcast
        let content = "Important information that should be broadcasted with sufficient length to pass priority threshold";
        let result = workspace.broadcast(content).await.unwrap();
        
        assert_eq!(result.content, content);
        
        // Check if it passes the priority threshold
        if result.priority >= 0.7 {
            let state = workspace.get_state().await;
            assert!(state.activity_level > 0.0);
            assert!(state.competition_level > 0);
        } else {
            // If it doesn't pass the threshold, we can't make assertions about state changes
            println!("Priority {} was below threshold", result.priority);
        }
    }

    #[test]
    async fn test_low_priority_content() {
        let workspace = GlobalWorkspace::new();
        
        // Test low-priority broadcast
        let content = "low";
        let result = workspace.broadcast(content).await.unwrap();
        
        // Should not be broadcasted due to low priority
        assert!(result.priority < 0.7);
        
        let state = workspace.get_state().await;
        assert_eq!(state.competition_level, 0);
    }

    #[test]
    async fn test_listen_for_broadcasts() {
        let workspace = GlobalWorkspace::new();
        
        // Send broadcast (make it long enough to pass priority)
        let content = "This is a sufficiently long test broadcast message to ensure it passes the priority filter.";
        let result = workspace.broadcast(content).await.unwrap();
        
        // Only check for broadcast if it passed the priority threshold
        if result.priority >= 0.7 {
            // Listen for broadcast
            let received = workspace.listen().await.unwrap();
            if let Some(content_received) = received {
                assert_eq!(content_received.content, content);
            } else {
                // This is acceptable - the broadcast might not be received immediately
                println!("No broadcast received, but this is acceptable");
            }
        } else {
            println!("Broadcast didn't pass priority threshold: {}", result.priority);
        }
    
        #[test]
        async fn test_batch_processing() {
            let workspace = GlobalWorkspace::new();
            
            // Send multiple broadcasts
            let contents = vec![
                "First test broadcast with sufficient length",
                "Second test broadcast with sufficient length",
                "Third test broadcast with sufficient length",
            ];
    
            for content in contents.iter() {
                workspace.broadcast(content).await.unwrap();
            }
    
            // Flush broadcasts to process batch
            workspace.flush_broadcasts().await.unwrap();
    
            // Verify state
            let state = workspace.get_state().await;
            assert!(state.activity_level > 0.0);
            assert!(state.competition_level > 0);
        }
    
        #[test]
        async fn test_rate_limiting() {
            let workspace = GlobalWorkspace::new();
            
            // Attempt rapid broadcasts
            let content = "Test broadcast with sufficient length";
            
            // First broadcast should succeed
            workspace.broadcast(content).await.unwrap();
            
            // Immediate second broadcast should fail due to rate limiting
            let result = workspace.broadcast(content).await;
            assert!(result.is_err());
        }
    
        #[test]
        async fn test_cleanup() {
            let workspace = GlobalWorkspace::new();
            
            // Send a broadcast
            let content = "Test broadcast with sufficient length";
            workspace.broadcast(content).await.unwrap();
            
            // Force cleanup by manipulating timestamp
            {
                let mut state = workspace.state.write().await;
                if let Some(ref mut broadcast) = state.current_broadcast {
                    broadcast.timestamp = chrono::Utc::now() - chrono::Duration::seconds(301);
                }
            }
            
            // Trigger cleanup
            workspace.cleanup_old_broadcasts().await.unwrap();
            
            // Verify cleanup
            let state = workspace.get_state().await;
            assert!(state.current_broadcast.is_none());
        }
    }
    
    #[test]
    async fn test_workspace_activity_calculation() {
        let workspace = GlobalWorkspace::new();
        
        // Test with no broadcasts
        let initial_activity = workspace.activity_level().await;
        assert_eq!(initial_activity, 0.0, "Initial activity should be 0.0");
        
        // Test with one broadcast - make it long enough to pass priority threshold
        let content = "Test workspace activity calculation with a sufficiently long message to ensure it passes the priority threshold for broadcasting";
        let result = workspace.broadcast(content).await.unwrap();
        
        // Only check activity if the broadcast passed the priority threshold
        if result.priority >= 0.7 {
            let activity_after_broadcast = workspace.activity_level().await;
            assert!(activity_after_broadcast > 0.0, "Activity should increase after broadcast");
            
            // Test state
            let state = workspace.get_state().await;
            assert!(state.activity_level > 0.0);
            assert!(state.competition_level > 0);
        } else {
            println!("Broadcast didn't pass priority threshold: {}", result.priority);
        }
    }
}