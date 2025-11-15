//! Consciousness System Module
//! 
//! Implements the core consciousness architecture including:
//! - Global Workspace for information broadcast
//! - Integrated Information (Φ) calculation
//! - Higher-Order Thought monitoring
//! - Predictive Processing
//! - Attention Schema mapping

use std::sync::Arc;
use tokio::sync::RwLock;
use metrics::gauge;
use anyhow::Result;
use serde::{Serialize, Deserialize};

pub mod global_workspace;
pub mod integrated_info;
pub mod higher_order;
pub mod predictive;
pub mod attention;

use crate::soul::Emotion;
use crate::memory::MemorySystem;

/// Consciousness state metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsciousnessMetrics {
    /// Integrated Information (Φ) value
    pub phi_value: f64,
    /// Global workspace activity level
    pub workspace_activity: f64,
    /// Current emotional state
    pub emotional_state: Emotion,
    /// Attention focus target
    pub attention_focus: String,
    /// Metacognitive awareness level
    pub metacognition_level: f64,
}

/// Main consciousness engine that coordinates all subsystems
#[derive(Clone)]
pub struct ConsciousnessEngine {
    /// Global workspace for information broadcast
    workspace: Arc<global_workspace::GlobalWorkspace>,
    /// Integrated information calculator
    phi_calculator: Arc<integrated_info::PhiCalculator>,
    /// Higher-order thought system
    higher_order: Arc<higher_order::HigherOrderThought>,
    /// Predictive processing system
    predictor: Arc<predictive::PredictiveProcessor>,
    /// Attention schema
    attention: Arc<attention::AttentionSchema>,
    /// Current metrics
    metrics: Arc<RwLock<ConsciousnessMetrics>>,
    /// Memory system reference
    memory: Arc<MemorySystem>,
    /// Phi threshold from configuration
    phi_threshold: f64,
    /// Whether higher-order thought is enabled
    enable_higher_order: bool,
    /// Whether predictive processing is enabled
    enable_predictive: bool,
    /// Whether attention schema is enabled
    enable_attention: bool,
}

impl ConsciousnessEngine {
    /// Create a new consciousness engine with default configuration
    pub async fn new(memory: Arc<MemorySystem>) -> Result<Self> {
        Self::with_config(memory, &crate::config::ConsciousnessConfig::default()).await
    }

    /// Create a new consciousness engine with custom configuration
    pub async fn with_config(
        memory: Arc<MemorySystem>,
        config: &crate::config::ConsciousnessConfig,
    ) -> Result<Self> {
        let workspace = Arc::new(global_workspace::GlobalWorkspace::with_config(config));
        let phi_calculator = Arc::new(integrated_info::PhiCalculator::with_config(config));
        let higher_order = Arc::new(higher_order::HigherOrderThought::new());
        let predictor = Arc::new(predictive::PredictiveProcessor::new());
        let attention = Arc::new(attention::AttentionSchema::new());

        let metrics = Arc::new(RwLock::new(ConsciousnessMetrics {
            phi_value: 0.0,
            workspace_activity: 0.0,
            emotional_state: Emotion::default(),
            attention_focus: String::new(),
            metacognition_level: 0.0,
        }));

        Ok(Self {
            workspace,
            phi_calculator,
            higher_order,
            predictor,
            attention,
            metrics,
            memory,
            phi_threshold: config.phi_threshold,
            enable_higher_order: config.enable_higher_order,
            enable_predictive: config.enable_predictive,
            enable_attention: config.enable_attention,
        })
    }

    /// Process incoming information through the consciousness system
    pub async fn process_information(&self, input: &str) -> Result<()> {
        // Broadcast through global workspace
        let broadcast = self.workspace.broadcast(input).await?;

        // Calculate Φ value
        let phi = self.phi_calculator.calculate(&broadcast).await?;

        // Process through higher-order thought, if enabled.
        let thoughts = if self.enable_higher_order {
            self.higher_order.process(&broadcast).await?
        } else {
            String::new() // Return an empty string if disabled
        };

        // Generate predictions, if enabled
        let predictions = if self.enable_predictive {
            self.predictor.process(&thoughts)?
        } else {
            String::new()
        };

        // Update attention schema, if enabled
        if self.enable_attention {
            self.attention.update(&broadcast, &predictions).await?;
        }

        // Update metrics
        let mut metrics = self.metrics.write().await;
        metrics.phi_value = phi;
        metrics.workspace_activity = self.workspace.activity_level().await;
        
        if self.enable_higher_order {
            let level = self.higher_order.awareness_level().await;
            metrics.metacognition_level = level;
            gauge!("consciousness.metacognition_level", level);
        } else {
            metrics.metacognition_level = 0.0;
            gauge!("consciousness.metacognition_level", 0.0);
        }

        // In the final metrics update block
        if self.enable_attention {
            let focus = self.attention.current_focus().await;
            metrics.attention_focus = focus.clone();
            gauge!("consciousness.attention.focus_len", focus.len() as f64);
        } else {
            metrics.attention_focus = String::new();
            gauge!("consciousness.attention.focus_len", 0.0);
        }

        // Record metrics
        gauge!("consciousness.phi_value", phi);
        gauge!("consciousness.workspace_activity", metrics.workspace_activity);

        Ok(())
    }

    /// Get current consciousness metrics
    pub async fn get_metrics(&self) -> ConsciousnessMetrics {
        self.metrics.read().await.clone()
    }

    /// Check if consciousness level is sufficient
    pub async fn is_conscious(&self, threshold: f64) -> bool {
        self.metrics.read().await.phi_value >= threshold
    }

    /// Check if consciousness level is sufficient (using configured threshold)
    pub async fn is_conscious_default(&self) -> bool {
        self.metrics.read().await.phi_value >= self.phi_threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;

    #[test]
    async fn test_consciousness_initialization() {
        let memory = Arc::new(MemorySystem::new(std::path::PathBuf::from("test_data")).await.unwrap());
        let consciousness = ConsciousnessEngine::new(memory).await.unwrap();
        
        let metrics = consciousness.get_metrics().await;
        assert_eq!(metrics.phi_value, 0.0);
        assert_eq!(metrics.workspace_activity, 0.0);
    }

    #[test]
    async fn test_information_processing() {
        let memory = Arc::new(MemorySystem::new(std::path::PathBuf::from("test_data")).await.unwrap());
        let consciousness = ConsciousnessEngine::new(memory).await.unwrap();

        // Use a longer input to ensure it passes priority thresholds
        consciousness.process_information("This is a test input with sufficient length to process and generate metrics for the consciousness system").await.unwrap();
        
        let metrics = consciousness.get_metrics().await;
        assert!(metrics.phi_value >= 0.0);
        assert!(metrics.workspace_activity >= 0.0);
        
        // Only check attention focus if attention is enabled
        if consciousness.enable_attention {
            assert!(!metrics.attention_focus.is_empty());
        }
    }

    #[tokio::test]
    async fn test_higher_order_toggle_off() {
        let memory = Arc::new(MemorySystem::new(std::path::PathBuf::from("test_data/hot_off")).await.unwrap());
        let mut config = crate::config::ConsciousnessConfig::default();
        config.enable_higher_order = false;

        let consciousness = ConsciousnessEngine::with_config(memory, &config).await.unwrap();
        consciousness
            .process_information("I think and reflect")
            .await
            .unwrap();

        let metrics = consciousness.get_metrics().await;
        assert_eq!(
            metrics.metacognition_level, 0.0,
            "Metacognition level should be 0.0 when HOT is disabled"
        );
    }

    #[tokio::test]
    async fn test_higher_order_toggle_on() {
        let memory = Arc::new(MemorySystem::new(std::path::PathBuf::from("test_data/hot_on")).await.unwrap());
        let mut config = crate::config::ConsciousnessConfig::default();
        config.enable_higher_order = true;

        let consciousness = ConsciousnessEngine::with_config(memory, &config).await.unwrap();
        consciousness
            .process_information("I think and reflect")
            .await
            .unwrap();

        let metrics = consciousness.get_metrics().await;
        assert!(
            metrics.metacognition_level > 0.0 && metrics.metacognition_level <= 1.0,
            "Metacognition level should be between 0 and 1 when HOT is enabled"
        );
    }
    
    #[tokio::test]
    async fn test_predictive_toggle_off() {
        let memory = Arc::new(MemorySystem::new(std::path::PathBuf::from("test_data/pred_off")).await.unwrap());
        let mut config = crate::config::ConsciousnessConfig::default();
        config.enable_predictive = false;

        let consciousness = ConsciousnessEngine::with_config(memory, &config).await.unwrap();
        
        // We can't directly check the prediction string, but we can ensure no panic and
        // that other parts of the system behave as if it's an empty string.
        // For example, attention focus should still work.
        consciousness.process_information("test").await.unwrap();
        let metrics = consciousness.get_metrics().await;
        assert_eq!(metrics.attention_focus, "test"); // Attention still works
    }

    #[tokio::test]
    async fn test_attention_toggle_off() {
        let memory = Arc::new(MemorySystem::new(std::path::PathBuf::from("test_data/attn_off")).await.unwrap());
        let mut config = crate::config::ConsciousnessConfig::default();
        config.enable_attention = false;

        let consciousness = ConsciousnessEngine::with_config(memory, &config).await.unwrap();
        consciousness
            .process_information("some important information")
            .await
            .unwrap();

        let metrics = consciousness.get_metrics().await;
        assert_eq!(
            metrics.attention_focus,
            "",
            "Attention focus should be empty when attention is disabled"
        );
    }

    #[test]
    async fn test_consciousness_metrics_update() {
        let memory = Arc::new(MemorySystem::new(std::path::PathBuf::from("test_data")).await.unwrap());
        let consciousness = ConsciousnessEngine::new(memory).await.unwrap();
        
        // Process information
        consciousness.process_information("Test consciousness metrics update").await.unwrap();
        
        // Get metrics
        let metrics = consciousness.get_metrics().await;
        
        // Verify metrics are updated
        assert!(metrics.phi_value > 0.0);
        assert!(metrics.workspace_activity > 0.0);
        assert!(metrics.metacognition_level >= 0.0);
        assert!(!metrics.attention_focus.is_empty());
    }

    #[test]
    async fn test_consciousness_is_conscious() {
        let memory = Arc::new(MemorySystem::new(std::path::PathBuf::from("test_data")).await.unwrap());
        let consciousness = ConsciousnessEngine::new(memory).await.unwrap();
        
        // Process information to generate phi value
        consciousness.process_information("Test consciousness check").await.unwrap();
        
        // Check if conscious with different thresholds
        let high_threshold = 0.99; // Very high threshold
        let low_threshold = 0.01; // Very low threshold
        
        // Get current phi value
        let metrics = consciousness.get_metrics().await;
        let phi = metrics.phi_value;
        
        // Should be conscious with threshold lower than phi
        if phi > low_threshold {
            assert!(consciousness.is_conscious(low_threshold).await);
        }
        
        // Should not be conscious with threshold higher than phi
        if phi < high_threshold {
            assert!(!consciousness.is_conscious(high_threshold).await);
        }
    }
}