//! Context Management System
//! 
//! This module provides a robust context management system for handling conversation history
//! and contextual information. It includes features such as:
//! 
//! - Token counting and window management using tiktoken-rs
//! - Conversation history tracking with configurable retention
//! - Relevance-based context selection
//! - Integration with persistent memory system
//! - Thread-safe implementation using Arc and RwLock
//! - Comprehensive metrics collection
//! 
//! # Example
//! 
//! ```rust
//! use context::{ContextManager, ContextConfig, Message};
//! use uuid::Uuid;
//! 
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let pool = sqlx::sqlite::SqlitePoolOptions::new()
//!         .connect("sqlite:memory:").await?;
//!     
//!     // Create context manager with custom config
//!     let config = ContextConfig {
//!         max_tokens: 8192,
//!         history_size: 20,
//!         relevance_threshold: 0.7,
//!         history_retention_hours: 24,
//!     };
//!     
//!     let manager = ContextManager::new(pool, Some(config)).await?;
//!     let conv_id = Uuid::new_v4();
//!     
//!     // Add messages to context
//!     let message = Message::new(
//!         "Hello, how can I help you?".to_string(),
//!         "assistant".to_string(),
//!     );
//!     manager.add_message(conv_id, message).await?;
//!     
//!     // Get relevant context
//!     let context = manager.get_relevant_context(
//!         conv_id,
//!         "help",
//!         Some(1000)
//!     ).await?;
//!     
//!     Ok(())
//! }
//! ```

use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tiktoken_rs::CoreBPE;
use metrics::{counter, gauge};
use tracing::{debug, error, info, warn};
use dashmap::DashMap;
use strsim::normalized_levenshtein;
use sqlx::SqlitePool;
use uuid::Uuid;

/// Errors that can occur in the context management system
#[derive(Error, Debug)]
pub enum ContextError {
    /// Returned when adding tokens would exceed the configured limit
    #[error("Token limit exceeded: {0}")]
    TokenLimitExceeded(usize),
    
    /// Returned when an invalid window size is configured
    #[error("Invalid window size: {0}")]
    InvalidWindowSize(usize),
    
    /// Returned when there's an error interacting with the memory system
    #[error("Memory system error: {0}")]
    MemoryError(String),
    
    /// Returned when there's an error counting tokens
    #[error("Token counting error: {0}")]
    TokenCountError(String),

    /// Returned when trying to access a conversation that doesn't exist
    #[error("No conversation found: {0}")]
    ConversationNotFound(Uuid),

    /// Returned when there's a database error
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
}

/// Configuration for the context management system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextConfig {
    /// Maximum number of tokens allowed in context window
    pub max_tokens: usize,
    
    /// Number of most recent messages to keep in conversation history
    pub history_size: usize,
    
    /// Minimum relevance score (0.0-1.0) for including context
    pub relevance_threshold: f32,
    
    /// How long to retain conversation history (in hours)
    pub history_retention_hours: u64,
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            max_tokens: 4096,
            history_size: 10,
            relevance_threshold: 0.7,
            history_retention_hours: 24,
        }
    }
}

/// Represents a message in the conversation history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// The content of the message
    pub content: String,
    
    /// The role of the sender (e.g., "user", "assistant", "system")
    pub role: String,
    
    /// When the message was created
    pub timestamp: chrono::DateTime<chrono::Utc>,
    
    /// Number of tokens in the message
    pub token_count: usize,
}

impl Message {
    /// Create a new message with the current timestamp
    pub fn new(content: String, role: String) -> Self {
        Self {
            content,
            role,
            timestamp: chrono::Utc::now(),
            token_count: 0, // Will be calculated when adding to context
        }
    }
}

/// Internal struct for scoring message relevance
#[derive(Debug, Clone)]
pub struct RelevanceScore {
    pub message: Message,
    pub score: f32,
}

/// The main context management system
///
/// Handles conversation history tracking, token counting, context selection,
/// and integration with the persistent memory system.
#[derive(Debug)]
pub struct ContextManager {
    config: Arc<RwLock<ContextConfig>>,
    history: Arc<DashMap<Uuid, Vec<Message>>>,
    tokenizer: CoreBPE,
    current_tokens: Arc<RwLock<usize>>,
    metrics: ContextMetrics,
    pool: SqlitePool,
}

/// Metrics tracked by the context manager
#[derive(Debug)]
struct ContextMetrics {
    total_tokens: String,
    window_utilization: String,
    cache_hits: String,
    cache_misses: String,
    error_count: String,
}

impl ContextManager {
    /// Create a new context manager with the given database pool and optional config
    pub async fn new(pool: SqlitePool, config: Option<ContextConfig>) -> Result<Self, ContextError> {
        let config = Arc::new(RwLock::new(config.unwrap_or_default()));
        
        let tokenizer = CoreBPE::new().map_err(|e| {
            ContextError::TokenCountError(e.to_string())
        })?;

        let metrics = ContextMetrics {
            total_tokens: "context_total_tokens".to_string(),
            window_utilization: "context_window_utilization".to_string(),
            cache_hits: "context_cache_hits".to_string(), 
            cache_misses: "context_cache_misses".to_string(),
            error_count: "context_errors".to_string(),
        };

        Ok(Self {
            config,
            history: Arc::new(DashMap::new()),
            tokenizer,
            current_tokens: Arc::new(RwLock::new(0)),
            metrics,
            pool,
        })
    }

    /// Update the configuration
    pub async fn update_config(&self, new_config: ContextConfig) -> Result<(), ContextError> {
        if new_config.max_tokens == 0 {
            return Err(ContextError::InvalidWindowSize(0));
        }
        
        let mut config = self.config.write().await;
        *config = new_config;
        Ok(())
    }

    /// Get the current configuration
    pub async fn get_config(&self) -> ContextConfig {
        self.config.read().await.clone()
    }

    /// Count tokens in a string using tiktoken
    pub fn count_tokens(&self, text: &str) -> Result<usize, ContextError> {
        self.tokenizer.encode_with_special_tokens(text)
            .map(|tokens| tokens.len())
            .map_err(|e| ContextError::TokenCountError(e.to_string()))
    }

    /// Check if adding tokens would exceed the window size
    async fn check_token_limit(&self, additional_tokens: usize) -> Result<(), ContextError> {
        let config = self.config.read().await;
        let current = *self.current_tokens.read().await;
        
        if current + additional_tokens > config.max_tokens {
            Err(ContextError::TokenLimitExceeded(current + additional_tokens))
        } else {
            Ok(())
        }
    }

    /// Update token count and metrics
    async fn update_token_count(&self, delta: isize) {
        let mut current = self.current_tokens.write().await;
        
        // Handle both positive and negative deltas
        if delta >= 0 {
            *current += delta as usize;
        } else {
            *current = current.saturating_sub((-delta) as usize);
        }

        // Update metrics
        gauge!(self.metrics.total_tokens, *current as f64);
        
        let config = self.config.read().await;
        let utilization = (*current as f64 / config.max_tokens as f64) * 100.0;
        gauge!(self.metrics.window_utilization, utilization);

        if utilization > 90.0 {
            warn!("Context window utilization above 90%: {:.1}%", utilization);
        }
    }

    /// Add a message to the conversation history and memory system
    ///
    /// The message will be stored both in memory and in the persistent storage.
    /// Token counts will be updated and limits enforced.
    pub async fn add_message(&self, conv_id: Uuid, mut message: Message) -> Result<(), ContextError> {
        // Count tokens in the message
        let token_count = self.count_tokens(&message.content)?;
        message.token_count = token_count;

        // Check if adding this message would exceed token limit
        self.check_token_limit(token_count).await?;

        // Add message to history
        self.history.entry(conv_id)
            .or_default()
            .push(message.clone());

        // Store in memory system
        self.store_in_memory(conv_id, &message).await?;

        // Update token count and metrics
        self.update_token_count(token_count as isize).await;

        Ok(())
    }

    /// Store a message in the memory system
    async fn store_in_memory(&self, conv_id: Uuid, message: &Message) -> Result<(), ContextError> {
        sqlx::query(
            r#"
            INSERT INTO memory_records (
                id, content, timestamp, layer, emotional_tags, context_associations
            )
            VALUES (?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(Uuid::new_v4().to_string())
        .bind(&message.content)
        .bind(message.timestamp)
        .bind("context")
        .bind("[]") // Empty emotional tags
        .bind(format!("{{\"conversation_id\": \"{}\"}}", conv_id))
        .execute(&self.pool)
        .await
        .map_err(|e| ContextError::MemoryError(e.to_string()))?;

        Ok(())
    }

    /// Calculate relevance score between two pieces of text
    fn calculate_relevance(&self, text1: &str, text2: &str) -> f32 {
        normalized_levenshtein(text1, text2)
    }

    /// Get relevant context messages for a given query
    ///
    /// This method combines messages from both the in-memory history and
    /// the persistent storage, scores them for relevance against the query,
    /// and returns the most relevant messages within the token limit.
    pub async fn get_relevant_context(&self, conv_id: Uuid, query: &str, max_tokens: Option<usize>) -> Result<Vec<Message>, ContextError> {
        let mut relevant_messages = Vec::new();

        // Get messages from conversation history
        if let Some(history) = self.history.get(&conv_id) {
            for msg in history.iter() {
                let score = self.calculate_relevance(&msg.content, query);
                relevant_messages.push(RelevanceScore {
                    message: msg.clone(),
                    score,
                });
            }
        }

        // Get messages from memory system
        let memory_messages = self.get_from_memory(conv_id, query).await?;
        for msg in memory_messages {
            let score = self.calculate_relevance(&msg.content, query);
            relevant_messages.push(RelevanceScore {
                message: msg,
                score,
            });
        }

        // Sort by relevance score (highest first)
        relevant_messages.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        let config = self.config.read().await;
        let max_tokens = max_tokens.unwrap_or(config.max_tokens);
        let threshold = config.relevance_threshold;

        // Filter by threshold and collect messages up to token limit
        let mut selected_messages = Vec::new();
        let mut total_tokens = 0;

        for scored in relevant_messages {
            if scored.score < threshold {
                continue;
            }

            if total_tokens + scored.message.token_count > max_tokens {
                break;
            }

            total_tokens += scored.message.token_count;
            selected_messages.push(scored.message);
        }

        // Sort by timestamp to maintain conversation flow
        selected_messages.sort_by_key(|msg| msg.timestamp);

        Ok(selected_messages)
    }

    /// Retrieve messages from the memory system
    async fn get_from_memory(&self, conv_id: Uuid, query: &str) -> Result<Vec<Message>, ContextError> {
        let rows = sqlx::query(
            r#"
            SELECT content, timestamp
            FROM memory_records
            WHERE layer = 'context'
            AND context_associations LIKE ?
            "#
        )
        .bind(format!("%{}%", conv_id))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ContextError::MemoryError(e.to_string()))?;

        let mut messages = Vec::new();
        for row in rows {
            let content: String = row.get("content");
            let timestamp: chrono::DateTime<chrono::Utc> = row.get("timestamp");
            
            let token_count = self.count_tokens(&content)?;
            
            messages.push(Message {
                content,
                role: "memory".to_string(),
                timestamp,
                token_count,
            });
        }

        Ok(messages)
    }

    /// Remove old messages that exceed the retention period
    ///
    /// This method prunes both the in-memory history and persistent storage
    /// based on the configured retention period.
    pub async fn prune_history(&self, conv_id: Uuid) -> Result<(), ContextError> {
        let config = self.config.read().await;
        let retention_duration = chrono::Duration::hours(config.history_retention_hours as i64);
        let cutoff = chrono::Utc::now() - retention_duration;

        // Prune conversation history
        if let Some(mut history) = self.history.get_mut(&conv_id) {
            let initial_len = history.len();
            let initial_tokens: usize = history.iter().map(|m| m.token_count).sum();

            history.retain(|msg| msg.timestamp > cutoff);

            let final_tokens: usize = history.iter().map(|m| m.token_count).sum();
            let removed_tokens = initial_tokens.saturating_sub(final_tokens);

            if removed_tokens > 0 {
                self.update_token_count(-(removed_tokens as isize)).await;
                info!("Pruned {} old messages, freed {} tokens", initial_len - history.len(), removed_tokens);
            }
        }

        // Prune memory system
        sqlx::query(
            r#"
            DELETE FROM memory_records 
            WHERE layer = 'context'
            AND context_associations LIKE ?
            AND timestamp < ?
            "#
        )
        .bind(format!("%{}%", conv_id))
        .bind(cutoff)
        .execute(&self.pool)
        .await
        .map_err(|e| ContextError::MemoryError(e.to_string()))?;

        Ok(())
    }

    /// Get all messages for a conversation
    pub fn get_conversation_history(&self, conv_id: Uuid) -> Option<Vec<Message>> {
        self.history.get(&conv_id).map(|h| h.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect("sqlite::memory:")
            .await
            .unwrap();

        sqlx::query(
            r#"
            CREATE TABLE memory_records (
                id TEXT PRIMARY KEY,
                content TEXT NOT NULL,
                timestamp DATETIME NOT NULL,
                layer TEXT NOT NULL,
                emotional_tags TEXT,
                context_associations TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            "#
        )
        .execute(&pool)
        .await
        .unwrap();

        pool
    }

    #[tokio::test]
    async fn test_context_manager_creation() {
        let pool = setup_test_db().await;
        let manager = ContextManager::new(pool, None).await;
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_config_update() {
        let pool = setup_test_db().await;
        let manager = ContextManager::new(pool, None).await.unwrap();
        
        let new_config = ContextConfig {
            max_tokens: 8192,
            history_size: 20,
            relevance_threshold: 0.8,
            history_retention_hours: 48,
        };

        let result = manager.update_config(new_config.clone()).await;
        assert!(result.is_ok());

        let updated_config = manager.get_config().await;
        assert_eq!(updated_config.max_tokens, 8192);
        assert_eq!(updated_config.history_size, 20);
    }

    #[tokio::test]
    async fn test_token_counting() {
        let pool = setup_test_db().await;
        let manager = ContextManager::new(pool, None).await.unwrap();
        
        let text = "Hello, world!";
        let count = manager.count_tokens(text);
        assert!(count.is_ok());
        assert!(count.unwrap() > 0);
    }

    #[tokio::test]
    async fn test_message_addition() {
        let pool = setup_test_db().await;
        let manager = ContextManager::new(pool, None).await.unwrap();
        let conv_id = Uuid::new_v4();
        
        let message = Message::new(
            "Test message".to_string(),
            "user".to_string(),
        );

        let result = manager.add_message(conv_id, message).await;
        assert!(result.is_ok());

        let history = manager.history.get(&conv_id);
        assert!(history.is_some());
        assert_eq!(history.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_token_limit() {
        let pool = setup_test_db().await;
        let config = ContextConfig {
            max_tokens: 5,
            ..Default::default()
        };
        let manager = ContextManager::new(pool, Some(config)).await.unwrap();
        let conv_id = Uuid::new_v4();

        let message = Message::new(
            "This is a long message that should exceed the token limit".to_string(),
            "user".to_string(),
        );

        let result = manager.add_message(conv_id, message).await;
        assert!(matches!(result, Err(ContextError::TokenLimitExceeded(_))));
    }

    #[tokio::test]
    async fn test_memory_integration() {
        let pool = setup_test_db().await;
        let manager = ContextManager::new(pool, None).await.unwrap();
        let conv_id = Uuid::new_v4();

        // Add a message
        let message = Message::new(
            "Test memory integration".to_string(),
            "user".to_string(),
        );
        manager.add_message(conv_id, message).await.unwrap();

        // Retrieve from memory
        let context = manager.get_relevant_context(conv_id, "memory", None).await.unwrap();
        assert!(!context.is_empty());
        assert!(context[0].content.contains("memory"));
    }

    #[tokio::test]
    async fn test_relevance_scoring() {
        let pool = setup_test_db().await;
        let manager = ContextManager::new(pool, None).await.unwrap();
        let conv_id = Uuid::new_v4();

        // Add some test messages
        let messages = vec![
            ("Tell me about Rust programming", "user"),
            ("Rust is a systems programming language", "assistant"),
            ("What's the weather like today?", "user"),
            ("The weather is sunny", "assistant"),
        ];

        for (content, role) in messages {
            let message = Message::new(content.to_string(), role.to_string());
            manager.add_message(conv_id, message).await.unwrap();
        }

        // Test relevance-based retrieval
        let query = "Tell me about Rust";
        let relevant = manager.get_relevant_context(conv_id, query, None).await.unwrap();
        
        assert!(!relevant.is_empty());
        
        // The Rust-related messages should be included and have higher relevance
        let has_rust_content = relevant.iter()
            .any(|msg| msg.content.contains("Rust"));
        assert!(has_rust_content);
    }
}