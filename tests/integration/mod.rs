//! Integration Testing Module
//! 
//! This module contains integration tests that verify the correct interaction
//! between different system components.

use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::Duration;

use jamey_3::{
    consciousness::{
        ConsciousnessEngine,
        cache::ConsciousnessCache,
        global_workspace::GlobalWorkspace,
    },
    memory::{MemorySystem, MemoryLayer},
    monitoring::MetricsCollector,
    db::operations::DatabaseOperations,
};

/// Test fixture for integration tests
struct TestFixture {
    engine: Arc<ConsciousnessEngine>,
    cache: Arc<ConsciousnessCache>,
    workspace: Arc<GlobalWorkspace>,
    memory: Arc<MemorySystem>,
    metrics: Arc<MetricsCollector>,
    db: Arc<DatabaseOperations>,
}

impl TestFixture {
    async fn new() -> Self {
        let metrics = Arc::new(MetricsCollector::new("integration_test"));
        let cache = Arc::new(ConsciousnessCache::new(metrics.clone()));
        let workspace = Arc::new(GlobalWorkspace::new());
        let memory = Arc::new(MemorySystem::new(metrics.clone()).await);
        let engine = Arc::new(ConsciousnessEngine::new());
        
        // Setup test database
        let (db_ops, _temp_dir) = super::common::setup_test_db().await;
        let db = Arc::new(db_ops);

        Self {
            engine,
            cache,
            workspace,
            memory,
            metrics,
            db,
        }
    }
}

#[tokio::test]
async fn test_cache_memory_integration() {
    let fixture = TestFixture::new().await;
    
    // Test data
    let key = "test_memory_key";
    let value = "test_memory_value";
    
    // Store in cache
    fixture.cache.set(key, value.to_string()).await;
    
    // Verify cache retrieval
    let cached_value = fixture.cache.get(key).await;
    assert_eq!(cached_value.as_deref(), Some(value));
    
    // Store in memory system
    fixture.memory.store(
        MemoryLayer::Working,
        key.to_string(),
        value.to_string(),
    ).await.unwrap();
    
    // Verify memory retrieval
    let memory_value = fixture.memory.retrieve(
        MemoryLayer::Working,
        key
    ).await.unwrap();
    assert_eq!(memory_value, value);
    
    // Verify cache and memory consistency
    assert_eq!(cached_value.as_deref(), Some(&memory_value));
}

#[tokio::test]
async fn test_async_communication_integration() {
    let fixture = TestFixture::new().await;
    let received_messages = Arc::new(Mutex::new(Vec::new()));
    
    // Setup message receiver
    let rx_messages = received_messages.clone();
    let workspace = fixture.workspace.clone();
    
    let _subscription = workspace.subscribe(move |msg| {
        let rx_messages = rx_messages.clone();
        Box::pin(async move {
            rx_messages.lock().await.push(msg.to_string());
            Ok(())
        })
    }).await;
    
    // Test messages
    let messages = vec![
        "Message 1",
        "Message 2",
        "Message 3",
    ];
    
    // Broadcast messages
    for msg in &messages {
        fixture.workspace.broadcast(msg).await.unwrap();
    }
    
    // Allow time for message processing
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Verify received messages
    let received = received_messages.lock().await;
    assert_eq!(received.len(), messages.len());
    for (sent, received) in messages.iter().zip(received.iter()) {
        assert_eq!(sent, received);
    }
}

#[tokio::test]
async fn test_context_management_integration() {
    let fixture = TestFixture::new().await;
    
    // Test contexts
    let contexts = vec![
        "context_1",
        "context_2",
        "context_3",
    ];
    
    // Switch contexts and verify state
    for context in &contexts {
        // Switch context
        fixture.engine.switch_context(context).await;
        
        // Verify context state
        let current = fixture.engine.current_context().await;
        assert_eq!(&current, context);
        
        // Store context-specific data
        let key = format!("{}_key", context);
        let value = format!("{}_value", context);
        
        fixture.cache.set(&key, value.clone()).await;
        
        // Verify context-specific data
        let cached = fixture.cache.get(&key).await;
        assert_eq!(cached.as_deref(), Some(&value));
    }
    
    // Verify context isolation
    for context in &contexts {
        fixture.engine.switch_context(context).await;
        
        let key = format!("{}_key", context);
        let value = format!("{}_value", context);
        
        let cached = fixture.cache.get(&key).await;
        assert_eq!(cached.as_deref(), Some(&value));
    }
}

#[tokio::test]
async fn test_monitoring_integration() {
    let fixture = TestFixture::new().await;
    
    // Track operations across components
    let operations = vec![
        ("cache_op", Duration::from_millis(5)),
        ("memory_op", Duration::from_millis(10)),
        ("workspace_op", Duration::from_millis(3)),
    ];
    
    // Record operations
    for (op, duration) in operations {
        fixture.metrics.record_operation(op, duration);
    }
    
    // Verify metrics
    for (op, duration) in operations {
        let stats = fixture.metrics.get_statistics(op);
        assert!(stats.avg_duration <= duration);
        assert_eq!(stats.count, 1);
        assert_eq!(stats.error_count, 0);
    }
    
    // Test error tracking
    fixture.metrics.record_error("test_error");
    let error_stats = fixture.metrics.get_statistics("test_error");
    assert_eq!(error_stats.error_count, 1);
}