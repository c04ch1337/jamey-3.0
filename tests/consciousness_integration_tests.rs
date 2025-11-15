//! Integration tests for the Consciousness System
//!
//! These tests verify the end-to-end functionality of the consciousness system,
//! including the interaction between its various components.

use std::sync::Arc;
use std::path::PathBuf;
use anyhow::Result;

use jamey_3::config::ConsciousnessConfig;
use jamey_3::consciousness::{ConsciousnessEngine, ConsciousnessMetrics};
use jamey_3::memory::MemorySystem;
use jamey_3::api::consciousness::{evaluate_action, get_rules, add_rule, AppState};
use axum::extract::State;
use axum::{Router, Json};
use axum::routing::{get, post};
use serde_json::{json, Value};
use hyper::{Body, Request, StatusCode};
use tower::ServiceExt;
use sqlx::PgPool;

/// Test helper to create a test consciousness engine
async fn create_test_engine() -> Result<Arc<ConsciousnessEngine>> {
    // Create a temporary memory system for testing
    let temp_dir = std::env::temp_dir().join("jamey_test_memory");
    std::fs::create_dir_all(&temp_dir)?;
    let memory = Arc::new(MemorySystem::new(temp_dir).await?);
    
    // Create a consciousness engine with default config
    let engine = ConsciousnessEngine::new(memory).await?;
    Ok(Arc::new(engine))
}

/// Test helper to create a test consciousness engine with custom config
async fn create_test_engine_with_config(config: &ConsciousnessConfig) -> Result<Arc<ConsciousnessEngine>> {
    // Create a temporary memory system for testing
    let temp_dir = std::env::temp_dir().join("jamey_test_memory");
    std::fs::create_dir_all(&temp_dir)?;
    let memory = Arc::new(MemorySystem::new(temp_dir).await?);
    
    // Create a consciousness engine with custom config
    let engine = ConsciousnessEngine::with_config(memory, config).await?;
    Ok(Arc::new(engine))
}

#[tokio::test]
async fn test_consciousness_flow() -> Result<()> {
    // Create a test engine
    let engine = create_test_engine().await?;
    
    // Process some information
    let input = "I am thinking about the nature of consciousness and self-awareness.";
    engine.process_information(input).await?;
    
    // Get metrics
    let metrics = engine.get_metrics().await;
    
    // Verify metrics
    assert!(metrics.phi_value > 0.0, "Phi value should be positive");
    assert!(metrics.phi_value <= 1.0, "Phi value should be <= 1.0");
    assert!(metrics.workspace_activity > 0.0, "Workspace activity should be positive");
    assert!(metrics.metacognition_level > 0.0, "Metacognition level should be positive");
    assert!(!metrics.attention_focus.is_empty(), "Attention focus should not be empty");
    
    Ok(())
}

#[tokio::test]
async fn test_subsystem_toggles() -> Result<()> {
    // Create a config with higher-order thought disabled
    let mut config = ConsciousnessConfig::default();
    config.enable_higher_order = false;
    
    // Create a test engine with this config
    let engine = create_test_engine_with_config(&config).await?;
    
    // Process some information
    let input = "I am thinking about the nature of consciousness and self-awareness.";
    engine.process_information(input).await?;
    
    // Get metrics
    let metrics = engine.get_metrics().await;
    
    // Verify higher-order thought is disabled
    assert_eq!(metrics.metacognition_level, 0.0, "Metacognition level should be 0.0 when higher-order thought is disabled");
    
    // Create a config with predictive processing disabled
    let mut config = ConsciousnessConfig::default();
    config.enable_predictive = false;
    
    // Create a test engine with this config
    let engine = create_test_engine_with_config(&config).await?;
    
    // Process some information
    engine.process_information(input).await?;
    
    // Verify processing still works
    let metrics = engine.get_metrics().await;
    assert!(metrics.phi_value > 0.0, "Phi value should still be positive");
    
    Ok(())
}

#[tokio::test]
async fn test_phi_calculation_with_different_inputs() -> Result<()> {
    // Create a test engine
    let engine = create_test_engine().await?;
    
    // Process different types of input
    let inputs = [
        "",  // Empty
        "Hello",  // Short
        "This is a longer sentence with more content to process.",  // Medium
        "This is a complex, introspective thought about the nature of consciousness and how it relates to self-awareness and metacognition. It contains many words and concepts that should trigger higher phi values.", // Complex
    ];
    
    let mut phi_values = Vec::new();
    
    for input in inputs.iter() {
        engine.process_information(input).await?;
        let metrics = engine.get_metrics().await;
        phi_values.push(metrics.phi_value);
        
        // Verify bounds
        assert!(metrics.phi_value >= 0.0, "Phi value should be non-negative");
        assert!(metrics.phi_value <= 1.0, "Phi value should be <= 1.0");
    }
    
    // Verify that more complex inputs generally yield higher phi values
    // Empty should be lowest
    assert!(phi_values[0] <= phi_values[1], "Empty input should have lower phi than short input");
    // Complex should be highest
    assert!(phi_values[2] <= phi_values[3], "Medium input should have lower phi than complex input");
    
    Ok(())
}

#[tokio::test]
async fn test_attention_focus_changes() -> Result<()> {
    // Create a test engine
    let engine = create_test_engine().await?;
    
    // Process different inputs with distinct focus areas
    let inputs = [
        "Phoenix is the primary focus of this thought.",
        "Protection and security are the main concerns here.",
        "Philosophical questions about existence are interesting.",
    ];
    
    let mut focus_values = Vec::new();
    
    for input in inputs.iter() {
        engine.process_information(input).await?;
        let metrics = engine.get_metrics().await;
        focus_values.push(metrics.attention_focus.clone());
    }
    
    // Verify that different inputs yield different focus values
    assert_ne!(focus_values[0], focus_values[1], "Different inputs should yield different focus values");
    assert_ne!(focus_values[1], focus_values[2], "Different inputs should yield different focus values");
    
    Ok(())
}