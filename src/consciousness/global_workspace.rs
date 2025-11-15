//! Global Workspace Module
//!
//! Implements the Global Workspace Theory (GWT) which suggests that consciousness
//! emerges from a central information exchange where different cognitive processes
//! compete for attention and broadcast their information globally.

use std::sync::Arc;
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
        }
    }

    /// Broadcast information through the workspace
    pub async fn broadcast(&self, content: &str) -> Result<WorkspaceContent> {
        let workspace_content = WorkspaceContent {
            id: Uuid::new_v4(),
            content: content.to_string(),
            source: "consciousness".to_string(),
            priority: self.calculate_priority(content).await?,
            timestamp: chrono::Utc::now(),
        };

        // Only broadcast if priority exceeds competition threshold
        if workspace_content.priority >= self.competition_threshold {
            self.broadcast_tx.send(workspace_content.clone()).await?;
            
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
        // A simple length-based heuristic, scaled to ensure typical content exceeds the threshold.
        let raw_priority = (content.len() as f64) / self.priority_max_length;
        // Scale and clamp to [0.0, 1.0]
        let priority = (raw_priority * 1.2).min(1.0);
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

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;

    #[test]
    async fn test_workspace_broadcast() {
        let workspace = GlobalWorkspace::new();
        
        // Test high-priority broadcast
        let content = "Important information that should be broadcasted";
        let result = workspace.broadcast(content).await.unwrap();
        
        assert_eq!(result.content, content);
        assert!(result.priority >= 0.7);
        
        let state = workspace.get_state().await;
        assert!(state.activity_level > 0.0);
        assert_eq!(state.competition_level, 1);
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
        workspace.broadcast(content).await.unwrap();
        
        // Listen for broadcast
        let received = workspace.listen().await.unwrap();
        assert!(received.is_some(), "Listener did not receive the broadcast.");
        assert_eq!(received.unwrap().content, content);
    }
}