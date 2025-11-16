# Context Management System

The Context Management System provides a robust solution for handling conversation history and contextual information. This document describes the system's architecture, features, and usage patterns.

## Core Features

- Token counting and window management using tiktoken-rs
- Conversation history tracking with configurable retention
- Relevance-based context selection
- Integration with persistent memory system
- Thread-safe implementation using Arc and RwLock
- Comprehensive metrics collection

## Configuration

```rust
struct ContextConfig {
    max_tokens: usize,         // Maximum tokens in context window (default: 4096)
    history_size: usize,       // Number of recent messages to keep (default: 10)
    relevance_threshold: f32,  // Minimum relevance score (default: 0.7)
    history_retention_hours: u64, // History retention period (default: 24)
}
```

## Basic Usage

```rust
use context::{ContextManager, ContextConfig, Message};
use uuid::Uuid;

// Create context manager with custom config
let config = ContextConfig {
    max_tokens: 8192,
    history_size: 20,
    relevance_threshold: 0.7,
    history_retention_hours: 24,
};

let manager = ContextManager::new(pool, Some(config)).await?;
let conv_id = Uuid::new_v4();

// Add message to context
let message = Message::new(
    "Hello, how can I help you?".to_string(),
    "assistant".to_string(),
);
manager.add_message(conv_id, message).await?;

// Get relevant context
let context = manager.get_relevant_context(
    conv_id,
    "help",
    Some(1000)
).await?;
```

## Message Structure

```rust
struct Message {
    content: String,           // Message content
    role: String,             // Sender role (user/assistant/system)
    timestamp: DateTime<Utc>,  // Creation timestamp
    token_count: usize,       // Number of tokens in message
}
```

## Core Operations

### Adding Messages

```rust
// Add a new message to conversation
async fn add_message(&self, conv_id: Uuid, message: Message) -> Result<(), ContextError> {
    // Message will be:
    // 1. Token counted
    // 2. Added to in-memory history
    // 3. Stored in persistent memory
    // 4. Token limits enforced
    manager.add_message(conv_id, message).await?;
}
```

### Retrieving Context

```rust
// Get relevant context for a query
async fn get_relevant_context(
    &self,
    conv_id: Uuid,
    query: &str,
    max_tokens: Option<usize>
) -> Result<Vec<Message>, ContextError> {
    let context = manager.get_relevant_context(
        conv_id,
        query,
        Some(1000)  // Limit to 1000 tokens
    ).await?;
}
```

### Managing History

```rust
// Prune old messages
async fn prune_history(&self, conv_id: Uuid) -> Result<(), ContextError> {
    manager.prune_history(conv_id).await?;
}

// Get full conversation history
fn get_conversation_history(&self, conv_id: Uuid) -> Option<Vec<Message>> {
    manager.get_conversation_history(conv_id)
}
```

## Token Management

The system automatically handles token counting and window management:

```rust
// Token counting
let token_count = manager.count_tokens("Hello world")?;

// Token limit checking
manager.check_token_limit(additional_tokens).await?;

// Token count updates
manager.update_token_count(delta).await;
```

## Relevance Scoring

The system uses normalized Levenshtein distance for relevance scoring:

```rust
// Calculate relevance between two pieces of text
fn calculate_relevance(&self, text1: &str, text2: &str) -> f32 {
    normalized_levenshtein(text1, text2)
}
```

## Memory Integration

The context system integrates with a persistent memory store:

```rust
// Store message in memory system
async fn store_in_memory(&self, conv_id: Uuid, message: &Message) -> Result<(), ContextError> {
    sqlx::query(
        "INSERT INTO memory_records (id, content, timestamp, layer, emotional_tags, context_associations) VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(Uuid::new_v4().to_string())
    .bind(&message.content)
    .bind(message.timestamp)
    .bind("context")
    .bind("[]")
    .bind(format!("{{\"conversation_id\": \"{}\"}}", conv_id))
    .execute(&self.pool)
    .await?;
}
```

## Error Handling

The system provides detailed error types:

```rust
enum ContextError {
    TokenLimitExceeded(usize),
    InvalidWindowSize(usize),
    MemoryError(String),
    TokenCountError(String),
    ConversationNotFound(Uuid),
    DatabaseError(sqlx::Error),
}
```

## Metrics and Monitoring

The system exports various metrics:

- Token counts and window utilization
- Cache hits/misses
- Error counts
- Operation durations

```rust
// Metrics are automatically recorded
gauge!("context_total_tokens", current_tokens);
gauge!("context_window_utilization", utilization);
counter!("context_cache_hits", 1);
```

## Best Practices

1. **Configuration**
   - Set appropriate token limits based on your model
   - Configure history size based on memory constraints
   - Adjust relevance threshold based on application needs
   - Set retention period based on privacy requirements

2. **Token Management**
   - Monitor window utilization metrics
   - Handle token limit errors gracefully
   - Prune history regularly
   - Consider compression for long-term storage

3. **Memory Integration**
   - Use appropriate indices for efficient queries
   - Monitor database performance
   - Implement backup strategies
   - Consider caching frequently accessed data

4. **Error Handling**
   - Handle all error types appropriately
   - Implement fallback strategies
   - Log errors with context
   - Monitor error rates

## Example: Complete Context Management Flow

```rust
use context::{ContextManager, ContextConfig, Message};
use uuid::Uuid;

async fn manage_conversation() -> Result<(), ContextError> {
    // Initialize manager
    let config = ContextConfig {
        max_tokens: 4096,
        history_size: 10,
        relevance_threshold: 0.7,
        history_retention_hours: 24,
    };
    let manager = ContextManager::new(pool, Some(config)).await?;
    let conv_id = Uuid::new_v4();

    // Add user message
    let user_msg = Message::new(
        "What's the weather like?".to_string(),
        "user".to_string(),
    );
    manager.add_message(conv_id, user_msg).await?;

    // Get relevant context
    let context = manager.get_relevant_context(
        conv_id,
        "weather",
        Some(1000)
    ).await?;

    // Add assistant response
    let assistant_msg = Message::new(
        "The weather is sunny.".to_string(),
        "assistant".to_string(),
    );
    manager.add_message(conv_id, assistant_msg).await?;

    // Periodically prune old messages
    manager.prune_history(conv_id).await?;

    Ok(())
}