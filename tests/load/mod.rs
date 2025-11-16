//! Load Testing Module
//! 
//! This module contains load tests for various system components to verify
//! performance and stability under high load conditions.

use tokio::runtime::Runtime;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;
use futures::future::join_all;

use jamey_3::consciousness::{
    ConsciousnessEngine,
    global_workspace::GlobalWorkspace,
    cache::ConsciousnessCache,
};
use jamey_3::memory::MemorySystem;
use jamey_3::monitoring::MetricsCollector;

/// Test configuration for load tests
struct LoadTestConfig {
    concurrent_users: usize,
    test_duration: Duration,
    request_interval: Duration,
}

impl Default for LoadTestConfig {
    fn default() -> Self {
        Self {
            concurrent_users: 100,
            test_duration: Duration::from_secs(30),
            request_interval: Duration::from_millis(50),
        }
    }
}

/// Sets up test environment with required components
async fn setup_test_environment() -> (ConsciousnessCache, Arc<MetricsCollector>) {
    let metrics = Arc::new(MetricsCollector::new("load_test"));
    let cache = ConsciousnessCache::new(metrics.clone());
    (cache, metrics)
}

#[tokio::test]
async fn test_cache_performance_under_load() {
    let config = LoadTestConfig::default();
    let (cache, metrics) = setup_test_environment().await;
    let cache = Arc::new(cache);
    
    let mut handles = Vec::new();
    
    // Spawn concurrent users
    for i in 0..config.concurrent_users {
        let cache = cache.clone();
        let metrics = metrics.clone();
        
        let handle = tokio::spawn(async move {
            let start = std::time::Instant::now();
            
            while start.elapsed() < config.test_duration {
                // Perform cache operations
                let key = format!("test_key_{}", i);
                let value = format!("test_value_{}", i);
                
                cache.set(&key, value.clone()).await;
                let _retrieved = cache.get(&key).await;
                
                metrics.record_operation("cache_operation", start.elapsed());
                time::sleep(config.request_interval).await;
            }
        });
        
        handles.push(handle);
    }
    
    // Wait for all test users to complete
    join_all(handles).await;
    
    // Verify metrics
    let stats = metrics.get_statistics("cache_operation");
    assert!(stats.avg_duration < Duration::from_millis(10), 
            "Average cache operation took too long: {:?}", stats.avg_duration);
    assert!(stats.error_rate < 0.01, 
            "Cache error rate too high: {}", stats.error_rate);
}

#[tokio::test]
async fn test_async_communication_under_load() {
    let config = LoadTestConfig::default();
    let workspace = Arc::new(GlobalWorkspace::new());
    let metrics = Arc::new(MetricsCollector::new("async_comm_test"));
    
    let mut handles = Vec::new();
    
    // Spawn concurrent message publishers
    for i in 0..config.concurrent_users {
        let workspace = workspace.clone();
        let metrics = metrics.clone();
        
        let handle = tokio::spawn(async move {
            let start = std::time::Instant::now();
            
            while start.elapsed() < config.test_duration {
                let message = format!("Test message {} at {:?}", i, start.elapsed());
                
                let timing = std::time::Instant::now();
                workspace.broadcast(&message).await.unwrap();
                metrics.record_operation("message_broadcast", timing.elapsed());
                
                time::sleep(config.request_interval).await;
            }
        });
        
        handles.push(handle);
    }
    
    // Wait for all publishers to complete
    join_all(handles).await;
    
    // Verify communication performance
    let stats = metrics.get_statistics("message_broadcast");
    assert!(stats.avg_duration < Duration::from_millis(5),
            "Average broadcast took too long: {:?}", stats.avg_duration);
    assert!(stats.error_rate < 0.001,
            "Communication error rate too high: {}", stats.error_rate);
}

#[tokio::test]
async fn test_context_management_under_load() {
    let config = LoadTestConfig::default();
    let engine = Arc::new(ConsciousnessEngine::new());
    let metrics = Arc::new(MetricsCollector::new("context_test"));
    
    let mut handles = Vec::new();
    
    // Spawn concurrent context operations
    for i in 0..config.concurrent_users {
        let engine = engine.clone();
        let metrics = metrics.clone();
        
        let handle = tokio::spawn(async move {
            let start = std::time::Instant::now();
            
            while start.elapsed() < config.test_duration {
                let context_id = format!("context_{}", i);
                let timing = std::time::Instant::now();
                
                engine.switch_context(&context_id).await;
                metrics.record_operation("context_switch", timing.elapsed());
                
                time::sleep(config.request_interval).await;
            }
        });
        
        handles.push(handle);
    }
    
    // Wait for all context operations to complete
    join_all(handles).await;
    
    // Verify context management performance
    let stats = metrics.get_statistics("context_switch");
    assert!(stats.avg_duration < Duration::from_millis(15),
            "Average context switch took too long: {:?}", stats.avg_duration);
    assert!(stats.error_rate < 0.005,
            "Context management error rate too high: {}", stats.error_rate);
}

#[tokio::test]
async fn test_monitoring_system_under_load() {
    let config = LoadTestConfig::default();
    let metrics = Arc::new(MetricsCollector::new("monitoring_test"));
    
    let mut handles = Vec::new();
    
    // Spawn concurrent metric reporters
    for i in 0..config.concurrent_users {
        let metrics = metrics.clone();
        
        let handle = tokio::spawn(async move {
            let start = std::time::Instant::now();
            
            while start.elapsed() < config.test_duration {
                let timing = std::time::Instant::now();
                
                // Record various metrics types
                metrics.record_gauge("test_gauge", i as f64);
                metrics.record_counter("test_counter", 1);
                metrics.record_histogram("test_hist", timing.elapsed().as_secs_f64());
                
                metrics.record_operation("metric_recording", timing.elapsed());
                time::sleep(config.request_interval).await;
            }
        });
        
        handles.push(handle);
    }
    
    // Wait for all metric operations to complete
    join_all(handles).await;
    
    // Verify monitoring system performance
    let stats = metrics.get_statistics("metric_recording");
    assert!(stats.avg_duration < Duration::from_millis(1),
            "Average metric recording took too long: {:?}", stats.avg_duration);
    assert!(stats.error_rate < 0.001,
            "Monitoring system error rate too high: {}", stats.error_rate);
}