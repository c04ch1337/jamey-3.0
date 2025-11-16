use std::time::Duration;
use tokio::time::sleep;

use jamey_3_0::{
    communication::{BoundedChannel, ChannelConfig, Message, Priority},
    consciousness::global_workspace::GlobalWorkspace,
    config::ConsciousnessConfig,
};

#[tokio::test]
async fn test_global_workspace_communication_integration() {
    // Create a workspace with custom config for testing
    let mut config = ConsciousnessConfig::default();
    config.broadcast_channel_size = 10;
    config.competition_threshold = 0.3; // Lower threshold for testing
    
    let workspace = GlobalWorkspace::with_config(&config);

    // Test high-priority broadcast
    let content = "URGENT: Critical system message that requires immediate attention!";
    let result = workspace.broadcast(content).await.unwrap();
    
    // Verify broadcast was successful
    assert!(result.priority >= config.competition_threshold);
    
    // Check workspace state
    let state = workspace.get_state().await;
    assert!(state.activity_level > 0.0);
    assert!(state.competition_level > 0);
    assert_eq!(state.current_broadcast.unwrap().content, content);

    // Verify metrics
    let metrics = workspace.get_metrics().await;
    assert_eq!(metrics.messages_sent, 1);
    assert_eq!(metrics.errors, 0);

    // Test receiving broadcast
    if let Some(received) = workspace.listen().await.unwrap() {
        assert_eq!(received.content, content);
    } else {
        panic!("Expected to receive broadcast");
    }
}

#[tokio::test]
async fn test_backpressure_and_retry_integration() {
    let mut config = ConsciousnessConfig::default();
    config.broadcast_channel_size = 2;
    config.competition_threshold = 0.3;

    let workspace = GlobalWorkspace::with_config(&config);

    // Send multiple broadcasts rapidly to trigger backpressure
    let messages = vec![
        "First critical message!",
        "Second critical message!",
        "Third critical message!",
        "Fourth critical message!",
    ];

    let mut successful_broadcasts = 0;
    for msg in messages {
        match workspace.broadcast(msg).await {
            Ok(_) => successful_broadcasts += 1,
            Err(e) => println!("Expected backpressure: {}", e),
        }
        // Small delay to allow for retries
        sleep(Duration::from_millis(50)).await;
    }

    // Verify metrics
    let metrics = workspace.get_metrics().await;
    assert!(metrics.retries > 0, "Expected retries due to backpressure");
    assert!(successful_broadcasts > 0, "Expected some successful broadcasts");
    assert!(metrics.messages_sent > 0);
}

#[tokio::test]
async fn test_priority_based_processing() {
    let workspace = GlobalWorkspace::with_config(&ConsciousnessConfig::default());

    // Send messages with different priorities
    let messages = vec![
        "Low priority message.",
        "Normal priority message!",
        "High priority message!!",
        "URGENT: Critical priority message!!!",
    ];

    let mut results = Vec::new();
    for msg in messages {
        if let Ok(result) = workspace.broadcast(msg).await {
            results.push((result.priority, msg));
        }
    }

    // Verify that higher priority messages were processed
    let high_priority_processed = results.iter()
        .any(|(priority, msg)| *priority >= 0.7 && msg.contains("URGENT"));

    assert!(high_priority_processed, "Expected high priority message to be processed");

    // Check metrics for priority-based processing
    let metrics = workspace.get_metrics().await;
    assert!(metrics.messages_sent > 0);
}

#[tokio::test]
async fn test_concurrent_broadcasts() {
    let workspace = GlobalWorkspace::with_config(&ConsciousnessConfig::default());
    let workspace = std::sync::Arc::new(workspace);

    let mut handles = Vec::new();
    
    // Spawn multiple tasks to broadcast concurrently
    for i in 0..5 {
        let workspace = workspace.clone();
        let handle = tokio::spawn(async move {
            let msg = format!("Concurrent broadcast {}!", i);
            workspace.broadcast(&msg).await
        });
        handles.push(handle);
    }

    // Wait for all broadcasts to complete
    let mut successful = 0;
    for handle in handles {
        if let Ok(Ok(_)) = handle.await {
            successful += 1;
        }
    }

    // Verify metrics
    let metrics = workspace.get_metrics().await;
    assert!(metrics.messages_sent > 0);
    assert!(successful > 0, "Expected some successful concurrent broadcasts");
}

#[tokio::test]
async fn test_system_load_handling() {
    let workspace = GlobalWorkspace::with_config(&ConsciousnessConfig::default());

    // Generate sustained load
    let mut successful = 0;
    let mut total = 0;

    for i in 0..20 {
        let msg = format!("Load test message {}", i);
        match workspace.broadcast(&msg).await {
            Ok(_) => successful += 1,
            Err(_) => (),
        }
        total += 1;
        sleep(Duration::from_millis(10)).await;
    }

    // Verify system handled load
    let metrics = workspace.get_metrics().await;
    assert!(metrics.messages_sent > 0);
    assert!(successful > 0, "Expected some successful broadcasts under load");
    
    // Check if backpressure was applied
    assert!(metrics.retries > 0, "Expected retries under sustained load");
    
    println!(
        "Load test results: {}/{} messages processed, {} retries",
        successful, total, metrics.retries
    );
}