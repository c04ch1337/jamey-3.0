# Cache System Documentation

The system implements a smart caching mechanism that supports multiple cache types and configurations. This document describes the cache API, configuration options, and usage patterns.

## Cache Types

The system supports three types of caches:

```rust
CacheType::Response    // For caching API responses
CacheType::Embedding  // For caching vector embeddings
CacheType::Persistent // For caching persistent data
```

## Configuration

The cache system can be configured using `CacheConfig`:

```rust
let config = CacheConfig {
    max_capacity: 10_000,      // Maximum number of items
    time_to_live: Duration::from_secs(3600), // 1 hour TTL
    time_to_idle: Duration::from_secs(1800), // 30 minutes idle timeout
};
```

### Default Configuration
- Maximum capacity: 10,000 items
- Time to live (TTL): 1 hour
- Time to idle: 30 minutes

## API Reference

### Initialization

Create a new cache instance:

```rust
// With default configuration
let cache = SmartCache::new();

// With custom configuration
let cache = SmartCache::with_config(config);
```

### Core Operations

1. Insert Value
```rust
// Insert a response
cache.insert(
    CacheType::Response,
    "key".to_string(),
    CacheValue::Response("response data".to_string())
).await?;

// Insert an embedding
cache.insert(
    CacheType::Embedding,
    "embedding_key".to_string(),
    CacheValue::Embedding(vec![0.1, 0.2, 0.3])
).await?;
```

2. Retrieve Value
```rust
if let Some(value) = cache.get(CacheType::Response, "key").await {
    match value {
        CacheValue::Response(data) => println!("Retrieved: {}", data),
        _ => println!("Wrong value type"),
    }
}
```

3. Remove Value
```rust
cache.remove(CacheType::Response, "key").await?;
```

4. Clear Cache
```rust
cache.clear(CacheType::Response).await?;
```

5. Get Cache Size
```rust
let size = cache.get_size(CacheType::Response);
```

## Cache Value Types

The system supports different value types through the `CacheValue` enum:

```rust
CacheValue::Response(String)     // For string responses
CacheValue::Embedding(Vec<f32>)  // For vector embeddings
CacheValue::Persistent(Vec<u8>)  // For binary data
```

## Metrics and Monitoring

The cache system automatically records metrics for:
- Cache hits
- Cache misses
- Insert operations
- Remove operations
- Clear operations

These metrics are tracked per cache type and can be monitored through the metrics system.

## Best Practices

1. **Cache Type Selection**
   - Use `Response` cache for API responses and frequently accessed string data
   - Use `Embedding` cache for vector embeddings and ML model outputs
   - Use `Persistent` cache for binary data that needs to persist

2. **Configuration Guidelines**
   - Adjust `max_capacity` based on memory constraints and usage patterns
   - Set `time_to_live` based on data freshness requirements
   - Configure `time_to_idle` to clean up infrequently accessed items

3. **Error Handling**
   - Always handle the `Result` returned by insert/remove/clear operations
   - Check for None when retrieving values from cache

## Example Usage

Complete example demonstrating typical cache usage:

```rust
use std::time::Duration;

async fn example_cache_usage() -> Result<()> {
    // Create cache with custom configuration
    let config = CacheConfig {
        max_capacity: 1000,
        time_to_live: Duration::from_secs(1800),  // 30 minutes
        time_to_idle: Duration::from_secs(900),   // 15 minutes
    };
    
    let cache = SmartCache::with_config(config);
    
    // Store response
    cache.insert(
        CacheType::Response,
        "api_response_key".to_string(),
        CacheValue::Response(json_response.to_string())
    ).await?;
    
    // Retrieve response
    if let Some(value) = cache.get(CacheType::Response, "api_response_key").await {
        match value {
            CacheValue::Response(data) => process_response(data),
            _ => handle_wrong_type_error(),
        }
    }
    
    Ok(())
}
```

## Performance Considerations

- The cache uses asynchronous operations for all main functions
- Cache operations are tracked with metrics for monitoring performance
- Automatic cleanup of expired items helps manage memory usage
- The system supports concurrent access with thread-safe operations