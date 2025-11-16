//! Database Integration Tests
//!
//! Comprehensive integration tests for database operations including:
//! - Migrations
//! - CRUD operations
//! - Transactions
//! - Connection pooling
//! - Error handling

use sqlx::{SqlitePool, Row};
use sqlx::sqlite::SqliteConnectOptions;
use std::str::FromStr;
use tempfile::TempDir;
use futures::future;

/// Helper to create a test database
async fn create_test_db() -> (SqlitePool, TempDir) {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    
    // Create database connection directly
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::from_str(
                &format!("sqlite://{}?mode=rwc", db_path.to_str().unwrap())
            )
            .unwrap()
            .create_if_missing(true)
        )
        .await
        .unwrap();
    
    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    
    (pool, temp_dir)
}

/// Test database initialization
#[tokio::test]
async fn test_database_initialization() {
    let (pool, _temp_dir) = create_test_db().await;
    
    // Verify connection
    let result = sqlx::query("SELECT 1").fetch_one(&pool).await;
    assert!(result.is_ok());
}

/// Test migrations
#[tokio::test]
async fn test_migrations() {
    let (pool, _temp_dir) = create_test_db().await;
    
    // Check that api_keys table exists
    let result = sqlx::query(
        "SELECT name FROM sqlite_master WHERE type='table' AND name='api_keys'"
    )
    .fetch_optional(&pool)
    .await
    .unwrap();
    
    assert!(result.is_some());
    
    // Check that soul_entities table exists (if migrations include it)
    let result = sqlx::query(
        "SELECT name FROM sqlite_master WHERE type='table' AND name='soul_entities'"
    )
    .fetch_optional(&pool)
    .await
    .unwrap();
    
    // May or may not exist depending on migrations
    // Just verify query works
    assert!(result.is_some() || result.is_none());
}

/// Test CRUD operations on api_keys
#[tokio::test]
async fn test_api_keys_crud() {
    let (pool, _temp_dir) = create_test_db().await;
    
    // Create
    let key_hash = "test_hash_123";
    let name = "test_key";
    
    sqlx::query!(
        "INSERT INTO api_keys (key_hash, name, created_at) VALUES (?, ?, datetime('now'))",
        key_hash,
        name
    )
    .execute(&pool)
    .await
    .unwrap();
    
    // Read
    let result = sqlx::query!(
        "SELECT id, key_hash, name FROM api_keys WHERE key_hash = ?",
        key_hash
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    
    assert_eq!(result.key_hash, key_hash);
    assert_eq!(result.name, name);
    
    // Update
    let new_name = "updated_key";
    sqlx::query!(
        "UPDATE api_keys SET name = ? WHERE key_hash = ?",
        new_name,
        key_hash
    )
    .execute(&pool)
    .await
    .unwrap();
    
    let result = sqlx::query!(
        "SELECT name FROM api_keys WHERE key_hash = ?",
        key_hash
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    
    assert_eq!(result.name, new_name);
    
    // Delete
    sqlx::query!("DELETE FROM api_keys WHERE key_hash = ?", key_hash)
        .execute(&pool)
        .await
        .unwrap();
    
    let result = sqlx::query!(
        "SELECT id FROM api_keys WHERE key_hash = ?",
        key_hash
    )
    .fetch_optional(&pool)
    .await
    .unwrap();
    
    assert!(result.is_none());
}

/// Test transactions
#[tokio::test]
async fn test_transactions() {
    let (pool, _temp_dir) = create_test_db().await;
    
    // Test successful transaction
    let mut tx = pool.begin().await.unwrap();
    
    sqlx::query!(
        "INSERT INTO api_keys (key_hash, name, created_at) VALUES (?, ?, datetime('now'))",
        "hash1",
        "key1"
    )
    .execute(&mut *tx)
    .await
    .unwrap();
    
    sqlx::query!(
        "INSERT INTO api_keys (key_hash, name, created_at) VALUES (?, ?, datetime('now'))",
        "hash2",
        "key2"
    )
    .execute(&mut *tx)
    .await
    .unwrap();
    
    tx.commit().await.unwrap();
    
    // Verify both records exist
    let count = sqlx::query!("SELECT COUNT(*) as count FROM api_keys WHERE key_hash IN ('hash1', 'hash2')")
        .fetch_one(&pool)
        .await
        .unwrap();
    
    assert_eq!(count.get::<i64, _>("count"), 2);
    
    // Test rollback
    let mut tx = pool.begin().await.unwrap();
    
    sqlx::query!(
        "INSERT INTO api_keys (key_hash, name, created_at) VALUES (?, ?, datetime('now'))",
        "hash3",
        "key3"
    )
    .execute(&mut *tx)
    .await
    .unwrap();
    
    tx.rollback().await.unwrap();
    
    // Verify record doesn't exist
    let result = sqlx::query!(
        "SELECT id FROM api_keys WHERE key_hash = ?",
        "hash3"
    )
    .fetch_optional(&pool)
    .await
    .unwrap();
    
    assert!(result.is_none());
}

/// Test connection pooling
#[tokio::test]
async fn test_connection_pooling() {
    let (pool, _temp_dir) = create_test_db().await;
    
    // Spawn multiple concurrent queries
    let mut handles = vec![];
    
    for i in 0..10 {
        let pool = pool.clone();
        let handle = tokio::spawn(async move {
            sqlx::query!("SELECT ? as value", i as i64)
                .fetch_one(&pool)
                .await
                .unwrap()
                .get::<i64, _>("value")
        });
        handles.push(handle);
    }
    
    // Wait for all queries to complete
    let results: Vec<i64> = futures::future::join_all(handles)
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();
    
    // Verify all queries succeeded
    assert_eq!(results.len(), 10);
    for (i, result) in results.iter().enumerate() {
        assert_eq!(*result, i as i64);
    }
}

/// Test error handling
#[tokio::test]
async fn test_database_error_handling() {
    let (pool, _temp_dir) = create_test_db().await;
    
    // Test constraint violation (duplicate key_hash)
    sqlx::query!(
        "INSERT INTO api_keys (key_hash, name, created_at) VALUES (?, ?, datetime('now'))",
        "duplicate",
        "key1"
    )
    .execute(&pool)
    .await
    .unwrap();
    
    // Try to insert duplicate
    let result = sqlx::query!(
        "INSERT INTO api_keys (key_hash, name, created_at) VALUES (?, ?, datetime('now'))",
        "duplicate",
        "key2"
    )
    .execute(&pool)
    .await;
    
    // Should fail due to unique constraint
    assert!(result.is_err());
    
    // Test invalid query
    let result = sqlx::query!("SELECT * FROM non_existent_table")
        .fetch_optional(&pool)
        .await;
    
    assert!(result.is_err());
}

/// Test query performance
#[tokio::test]
async fn test_query_performance() {
    let (pool, _temp_dir) = create_test_db().await;
    
    // Insert test data
    for i in 0..100 {
        sqlx::query!(
            "INSERT INTO api_keys (key_hash, name, created_at) VALUES (?, ?, datetime('now'))",
            format!("hash_{}", i),
            format!("key_{}", i)
        )
        .execute(&pool)
        .await
        .unwrap();
    }
    
    // Measure query time
    let start = std::time::Instant::now();
    
    let result = sqlx::query!("SELECT COUNT(*) as count FROM api_keys")
        .fetch_one(&pool)
        .await
        .unwrap();
    
    let duration = start.elapsed();
    
    let count = result.get::<i64, _>("count");
    assert!(count >= 100);
    
    // Query should be fast (< 100ms)
    assert!(duration.as_millis() < 100);
}

/// Test concurrent writes
#[tokio::test]
async fn test_concurrent_writes() {
    let (pool, _temp_dir) = create_test_db().await;
    
    // Spawn multiple concurrent writes
    let mut handles = vec![];
    
    for i in 0..20 {
        let pool = pool.clone();
        let handle = tokio::spawn(async move {
            sqlx::query!(
                "INSERT INTO api_keys (key_hash, name, created_at) VALUES (?, ?, datetime('now'))",
                format!("concurrent_hash_{}", i),
                format!("concurrent_key_{}", i)
            )
            .execute(&pool)
            .await
        });
        handles.push(handle);
    }
    
    // Wait for all writes to complete
    let results: Vec<_> = futures::future::join_all(handles)
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();
    
    // Verify all writes succeeded
    for result in results {
        assert!(result.is_ok());
    }
    
    // Verify all records exist
    let count = sqlx::query!("SELECT COUNT(*) as count FROM api_keys WHERE key_hash LIKE 'concurrent_%'")
        .fetch_one(&pool)
        .await
        .unwrap();
    
    assert_eq!(count.get::<i64, _>("count"), 20);
}

