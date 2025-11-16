//! Stress Testing Module
//! 
//! This module contains stress tests that verify system behavior under extreme conditions:
//! - High concurrency
//! - Resource exhaustion
//! - Error handling
//! - System recovery

use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::Duration;
use futures::future::join_all;
use std::collections::HashMap;

use jamey_3::{
    consciousness::{
        ConsciousnessEngine,
        cache::ConsciousnessCache,
        global_workspace::{GlobalWorkspace, WorkspaceContent},
    },
    memory::{MemorySystem, MemoryLayer},
    monitoring::MetricsCollector,
    db::operations::DatabaseOperations,
};

const STRESS_CONCURRENT_USERS: usize = 1000;
const STRESS_DURATION: Duration = Duration::from_secs(60);
const MEMORY_STRESS_SIZE_MB: usize = 1024; // 1GB

/// Test fixture for stress tests
struct StressTestFixture {
    engine: Arc<ConsciousnessEngine>,
    cache: Arc<ConsciousnessCache>,
    workspace: Arc<GlobalWorkspace>,
    memory: Arc<MemorySystem>,
    metrics: Arc<MetricsCollector>,
    db: Arc<DatabaseOperations>,
}

impl StressTestFixture {
    async fn new() -> Self {
        let metrics = Arc::new(MetricsCollector::new("stress_test"));
        let cache = Arc::new(ConsciousnessCache::new(metrics.clone()));
        let workspace = Arc::new(GlobalWorkspace::new());
        let memory = Arc::new(MemorySystem::new(metrics.clone()).await);
        let engine = Arc::new(ConsciousnessEngine::new());
        
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
async fn test_high_concurrency() {
    let fixture = StressTestFixture::new().await;
    let error_count = Arc::new(Mutex::new(0));
    
    let mut handles = Vec::new();
    
    // Spawn concurrent users
    for i in 0..STRESS_CONCURRENT_USERS {
        let cache = fixture.cache.clone();
        let workspace = fixture.workspace.clone();
        let memory = fixture.memory.clone();
        let error_count = error_count.clone();
        
        let handle = tokio::spawn(async move {
            let start = std::time::Instant::now();
            
            while start.elapsed() < STRESS_DURATION {
                // Perform mixed operations
                let key = format!("stress_key_{}", i);
                let value = format!("stress_value_{}", i);
                
                // Test cache operations
                if let Err(_) = cache.set(&key, value.clone()).await {
                    *error_count.lock().await += 1;
                }
                
                // Test workspace operations
                if let Err(_) = workspace.broadcast(&value).await {
                    *error_count.lock().await += 1;
                }
                
                // Test memory operations
                if let Err(_) = memory.store(
                    MemoryLayer::Working,
                    key.clone(),
                    value.clone(),
                ).await {
                    *error_count.lock().await += 1;
                }
                
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        });
        
        handles.push(handle);
    }
    
    // Wait for all concurrent operations to complete
    join_all(handles).await;
    
    // Verify error rate is within acceptable bounds
    let total_errors = *error_count.lock().await;
    let error_rate = total_errors as f64 / STRESS_CONCURRENT_USERS as f64;
    assert!(error_rate < 0.01, "Error rate too high: {}", error_rate);
}

#[tokio::test]
async fn test_resource_exhaustion() {
    let fixture = StressTestFixture::new().await;
    let data = Arc::new(Mutex::new(HashMap::new()));
    
    // Gradually increase memory usage
    let mut size_mb = 0;
    while size_mb < MEMORY_STRESS_SIZE_MB {
        let key = format!("large_key_{}", size_mb);
        // Create 1MB of data
        let value = "x".repeat(1024 * 1024);
        
        // Store in cache and memory
        fixture.cache.set(&key, value.clone()).await;
        fixture.memory.store(
            MemoryLayer::Working,
            key.clone(),
            value.clone(),
        ).await.unwrap();
        
        // Keep reference to prevent garbage collection
        data.lock().await.insert(key, value);
        
        size_mb += 1;
        
        // Verify system still responsive
        assert!(fixture.cache.get(&format!("test_key_{}", size_mb)).await.is_none());
    }
    
    // Verify system remains functional
    let test_key = "recovery_test";
    let test_value = "system still working";
    fixture.cache.set(test_key, test_value.to_string()).await;
    assert_eq!(fixture.cache.get(test_key).await.as_deref(), Some(test_value));
}

#[tokio::test]
async fn test_error_handling() {
    let fixture = StressTestFixture::new().await;
    let error_count = Arc::new(Mutex::new(0));
    
    // Test invalid operations
    let test_cases = vec![
        // Invalid cache operations
        async {
            if let Ok(_) = fixture.cache.get("nonexistent").await {
                *error_count.lock().await += 1;
            }
        },
        // Invalid memory operations
        async {
            if let Ok(_) = fixture.memory.retrieve(MemoryLayer::Working, "nonexistent").await {
                *error_count.lock().await += 1;
            }
        },
        // Invalid workspace operations
        async {
            if let Ok(_) = fixture.workspace.broadcast("").await {
                *error_count.lock().await += 1;
            }
        },
    ];
    
    // Run error test cases
    join_all(test_cases).await;
    
    // Verify error handling
    let total_errors = *error_count.lock().await;
    assert_eq!(total_errors, 0, "Error handling failed");
}

#[tokio::test]
async fn test_system_recovery() {
    let fixture = StressTestFixture::new().await;
    
    // Setup initial state
    let test_keys: Vec<String> = (0..100).map(|i| format!("key_{}", i)).collect();
    let test_values: Vec<String> = (0..100).map(|i| format!("value_{}", i)).collect();
    
    // Store initial data
    for (key, value) in test_keys.iter().zip(test_values.iter()) {
        fixture.cache.set(key, value.clone()).await;
        fixture.memory.store(
            MemoryLayer::Working,
            key.clone(),
            value.clone(),
        ).await.unwrap();
    }
    
    // Simulate system stress
    let mut handles = Vec::new();
    for i in 0..10 {
        let cache = fixture.cache.clone();
        let memory = fixture.memory.clone();
        let test_keys = test_keys.clone();
        let test_values = test_values.clone();
        
        let handle = tokio::spawn(async move {
            for j in 0..1000 {
                let idx = (i * 1000 + j) % test_keys.len();
                let key = &test_keys[idx];
                let value = &test_values[idx];
                
                // Perform rapid operations
                cache.set(key, value.clone()).await;
                memory.store(
                    MemoryLayer::Working,
                    key.clone(),
                    value.clone(),
                ).await.unwrap();
            }
        });
        
        handles.push(handle);
    }
    
    // Wait for stress operations to complete
    join_all(handles).await;
    
    // Verify system state after stress
    for (key, value) in test_keys.iter().zip(test_values.iter()) {
        // Verify cache recovery
        let cached = fixture.cache.get(key).await;
        assert_eq!(cached.as_deref(), Some(value));
        
        // Verify memory recovery
        let stored = fixture.memory.retrieve(MemoryLayer::Working, key).await.unwrap();
        assert_eq!(stored, *value);
    }
}