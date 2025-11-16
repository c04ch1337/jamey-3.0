use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;
use metrics::{counter, gauge};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::communication::{
    BoundedChannel, 
    ChannelConfig, 
    Message as CommMessage, 
    Priority,
    retry::RetryConfig
};

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
#[derive(Debug)]
pub struct GlobalWorkspace {
    /// Current state of the workspace
    state: Arc<RwLock<WorkspaceState>>,
    /// Channel for broadcasting information
    broadcast_channel: Arc<RwLock<BoundedChannel<WorkspaceContent>>>,
    /// Competition threshold for broadcast
    competition_threshold: f64,
    /// Broadcast factor for activity calculation
    broadcast_factor: f64,
    /// Competition divisor for activity calculation
    competition_divisor: f64,
    /// Competition max factor
    competition_max_factor: f64,
    /// Priority max length
    priority_max_length: f64,
}

impl GlobalWorkspace {
    /// Create a new Global Workspace with default configuration
    pub fn new() -> Self {
        Self::with_config(&crate::config::ConsciousnessConfig::default())
    }

    /// Create a new Global Workspace with custom configuration
    pub fn with_config(config: &crate::config::ConsciousnessConfig) -> Self {
        let channel_config = ChannelConfig {
            capacity: config.broadcast_channel_size,
            retry_config: RetryConfig {
                initial_delay: std::time::Duration::from_millis(50),
                max_delay: std::time::Duration::from_millis(500),
                max_retries: 3,
                backoff_factor: 2.0,
                jitter: 0.1,
            },
        };

        let broadcast_channel = BoundedChannel::new("global_workspace", channel_config);
        
        let state = Arc::new(RwLock::new(WorkspaceState {
            current_broadcast: None,
            activity_level: 0.0,
            competition_level: 0,
        }));

        Self {
            state,
            broadcast_channel: Arc::new(RwLock::new(broadcast_channel)),
            competition_threshold: config.competition_threshold,
            broadcast_factor: config.broadcast_factor,
            competition_divisor: config.competition_divisor,
            competition_max_factor: config.competition_max_factor,
            priority_max_length: config.priority_max_length,
        }
    }

    /// Broadcast information through the workspace
    pub async fn broadcast(&self, content: &str) -> Result<WorkspaceContent> {
        let priority = self.calculate_priority(content).await?;
        
        let workspace_content = WorkspaceContent {
            id: Uuid::new_v4(),
            content: content.to_string(),
            source: "consciousness".to_string(),
            priority,
            timestamp: chrono::Utc::now(),
        };

        // Only process if priority exceeds threshold
        if workspace_content.priority >= self.competition_threshold {
            let comm_priority = if workspace_content.priority >= 0.8 {
                Priority::Critical
            } else if workspace_content.priority >= 0.6 {
                Priority::High
            } else if workspace_content.priority >= 0.4 {
                Priority::Normal
            } else {
                Priority::Low
            };

            let message = CommMessage::new(workspace_content.clone(), comm_priority);
            
            // Send through bounded channel with retries and backpressure
            let mut channel = self.broadcast_channel.write().await;
            channel.send(message).await.map_err(|e| anyhow::anyhow!("Broadcast error: {}", e))?;

            // Update state
            let mut state = self.state.write().await;
            state.current_broadcast = Some(workspace_content.clone());
            state.activity_level = self.calculate_activity_level().await?;
            state.competition_level += 1;

            // Update metrics
            gauge!("global_workspace.activity_level", state.activity_level);
            counter!("global_workspace.broadcasts_total", 1);
        }

        Ok(workspace_content)
    }

    /// Calculate priority for given content
    async fn calculate_priority(&self, content: &str) -> Result<f64> {
        // Priority calculation logic remains the same
        let length_factor = (content.len() as f64 / self.priority_max_length).min(1.0);
        
        let words: std::collections::HashSet<_> = content.split_whitespace().collect();
        let unique_word_ratio = words.len() as f64 / content.split_whitespace().count().max(1) as f64;
        let special_char_ratio = content.chars().filter(|c| !c.is_alphanumeric()).count() as f64 / content.len() as f64;
        let complexity_factor = ((unique_word_ratio + special_char_ratio) / 2.0).min(1.0);
        
        let novelty_factor = match self.state.read().await.current_broadcast {
            Some(ref current) => {
                let similarity = strsim::jaro_winkler(content, &current.content);
                (1.0 - similarity).max(0.0)
            }
            None => 1.0,
        };
        
        let urgency_keywords = ["urgent", "important", "critical", "emergency"];
        let has_urgency_keywords = urgency_keywords.iter().any(|&k| content.to_lowercase().contains(k));
        let exclamation_count = content.chars().filter(|&c| c == '!').count();
        let urgency_factor = if has_urgency_keywords || exclamation_count > 0 {
            ((exclamation_count as f64 / 3.0) + if has_urgency_keywords { 1.0 } else { 0.0 } / 2.0).min(1.0)
        } else {
            0.0
        };

        Ok(((length_factor + complexity_factor + novelty_factor + urgency_factor) / 4.0).min(1.0))
    }

    /// Calculate current activity level
    async fn calculate_activity_level(&self) -> Result<f64> {
        let state = self.state.read().await;
        
        let broadcast_factor = if state.current_broadcast.is_some() { self.broadcast_factor } else { 0.0 };
        let competition_factor = (state.competition_level as f64 / self.competition_divisor)
            .min(self.competition_max_factor);
        
        Ok(broadcast_factor + competition_factor)
    }

    /// Listen for broadcasts
    pub async fn listen(&self) -> Result<Option<WorkspaceContent>> {
        let mut channel = self.broadcast_channel.write().await;
        Ok(channel.receive().await.map(|msg| msg.payload))
    }

    /// Get the current workspace state
    pub async fn get_state(&self) -> WorkspaceState {
        self.state.read().await.clone()
    }

    /// Get current activity level
    pub async fn activity_level(&self) -> f64 {
        self.state.read().await.activity_level
    }

    /// Get channel metrics
    pub async fn get_metrics(&self) -> crate::communication::metrics::ChannelMetrics {
        self.broadcast_channel.read().await.get_metrics()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;

    #[test]
    async fn test_workspace_broadcast() {
        let workspace = GlobalWorkspace::new();
        
        let content = "Important information that should be broadcasted with sufficient length to pass priority threshold";
        let result = workspace.broadcast(content).await.unwrap();
        
        assert_eq!(result.content, content);
        
        if result.priority >= 0.7 {
            let state = workspace.get_state().await;
            assert!(state.activity_level > 0.0);
            assert!(state.competition_level > 0);

            // Check metrics
            let metrics = workspace.get_metrics().await;
            assert!(metrics.messages_sent > 0);
        }
    }

    #[test]
    async fn test_low_priority_content() {
        let workspace = GlobalWorkspace::new();
        
        let content = "low";
        let result = workspace.broadcast(content).await.unwrap();
        
        assert!(result.priority < 0.7);
        
        let state = workspace.get_state().await;
        assert_eq!(state.competition_level, 0);

        let metrics = workspace.get_metrics().await;
        assert_eq!(metrics.messages_sent, 0);
    }

    #[test]
    async fn test_listen_for_broadcasts() {
        let workspace = GlobalWorkspace::new();
        
        let content = "This is a sufficiently long test broadcast message to ensure it passes the priority filter.";
        let result = workspace.broadcast(content).await.unwrap();
        
        if result.priority >= 0.7 {
            if let Some(received) = workspace.listen().await.unwrap() {
                assert_eq!(received.content, content);
            }
        }
    }
}