//! Database Operations Module
//!
//! Provides safe database operations with transaction support,
//! error handling, and performance monitoring.

use sqlx::{SqlitePool, Row};
use anyhow::{Result, anyhow};
use tracing::{debug, warn, error};
use std::time::Instant;
use crate::db::{DbMetrics, with_transaction};

/// Safe database operations wrapper
pub struct DatabaseOperations {
    pool: SqlitePool,
    metrics: DbMetrics,
}

impl DatabaseOperations {
    /// Create new database operations instance
    pub fn new(pool: SqlitePool, metrics: DbMetrics) -> Self {
        Self { pool, metrics }
    }

    /// Get a reference to the connection pool
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Get a reference to the metrics
    pub fn metrics(&self) -> &DbMetrics {
        &self.metrics
    }

    /// Execute a query with automatic retry and error handling
    pub async fn execute_query(
        &self,
        query: &str,
        params: &[&(dyn sqlx::Encode<sqlx::Sqlite> + sqlx::Type<sqlx::Sqlite> + Sync)],
    ) -> Result<sqlx::sqlite::SqliteQueryResult> {
        let start = Instant::now();
        
        debug!("Executing query: {}", query);
        
        let result = sqlx::query(query)
            .bind_all(params)
            .execute(&self.pool)
            .await;
        
        let duration = start.elapsed();
        self.metrics.record_query(duration).await;
        
        match result {
            Ok(res) => {
                debug!("Query executed successfully in {:?}: {} rows affected", 
                       duration, res.rows_affected());
                Ok(res)
            }
            Err(e) => {
                error!("Query failed after {:?}: {} - Error: {}", duration, query, e);
                Err(anyhow!("Database query failed: {}", e))
            }
        }
    }

    /// Execute a query that returns a single value
    pub async fn query_one<T>(
        &self,
        query: &str,
        params: &[&(dyn sqlx::Encode<sqlx::Sqlite> + sqlx::Type<sqlx::Sqlite> + Sync)],
    ) -> Result<T>
    where
        T: for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Unpin,
    {
        let start = Instant::now();
        
        debug!("Executing query_one: {}", query);
        
        let result = sqlx::query_as::<_, T>(query)
            .bind_all(params)
            .fetch_one(&self.pool)
            .await;
        
        let duration = start.elapsed();
        self.metrics.record_query(duration).await;
        
        match result {
            Ok(value) => {
                debug!("Query_one executed successfully in {:?}", duration);
                Ok(value)
            }
            Err(e) => {
                error!("Query_one failed after {:?}: {} - Error: {}", duration, query, e);
                Err(anyhow!("Database query_one failed: {}", e))
            }
        }
    }

    /// Execute a query that returns multiple values
    pub async fn query_all<T>(
        &self,
        query: &str,
        params: &[&(dyn sqlx::Encode<sqlx::Sqlite> + sqlx::Type<sqlx::Sqlite> + Sync)],
    ) -> Result<Vec<T>>
    where
        T: for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Unpin,
    {
        let start = Instant::now();
        
        debug!("Executing query_all: {}", query);
        
        let result = sqlx::query_as::<_, T>(query)
            .bind_all(params)
            .fetch_all(&self.pool)
            .await;
        
        let duration = start.elapsed();
        self.metrics.record_query(duration).await;
        
        match result {
            Ok(values) => {
                debug!("Query_all executed successfully in {:?}: {} rows returned", 
                       duration, values.len());
                Ok(values)
            }
            Err(e) => {
                error!("Query_all failed after {:?}: {} - Error: {}", duration, query, e);
                Err(anyhow!("Database query_all failed: {}", e))
            }
        }
    }

    /// Execute a query that returns optional value
    pub async fn query_optional<T>(
        &self,
        query: &str,
        params: &[&(dyn sqlx::Encode<sqlx::Sqlite> + sqlx::Type<sqlx::Sqlite> + Sync)],
    ) -> Result<Option<T>>
    where
        T: for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Unpin,
    {
        let start = Instant::now();
        
        debug!("Executing query_optional: {}", query);
        
        let result = sqlx::query_as::<_, T>(query)
            .bind_all(params)
            .fetch_optional(&self.pool)
            .await;
        
        let duration = start.elapsed();
        self.metrics.record_query(duration).await;
        
        match result {
            Ok(value) => {
                debug!("Query_optional executed successfully in {:?}", duration);
                Ok(value)
            }
            Err(e) => {
                error!("Query_optional failed after {:?}: {} - Error: {}", duration, query, e);
                Err(anyhow!("Database query_optional failed: {}", e))
            }
        }
    }

    /// Execute multiple operations in a transaction
    pub async fn execute_transaction<F, R>(&self, operation: F) -> Result<R>
    where
        F: FnOnce(&mut sqlx::Transaction<sqlx::Sqlite>) -> Result<R>,
    {
        with_transaction(&self.pool, &self.metrics, operation).await
    }

    /// Insert a record and return the last insert ID
    pub async fn insert_with_return_id(
        &self,
        table: &str,
        columns: &[&str],
        values: &[&(dyn sqlx::Encode<sqlx::Sqlite> + sqlx::Type<sqlx::Sqlite> + Sync)],
    ) -> Result<i64> {
        let columns_str = columns.join(", ");
        let placeholders: Vec<String> = (0..values.len()).map(|_| "?".to_string()).collect();
        let placeholders_str = placeholders.join(", ");
        
        let query = format!(
            "INSERT INTO {} ({}) VALUES ({}) RETURNING rowid",
            table, columns_str, placeholders_str
        );
        
        let start = Instant::now();
        
        debug!("Executing insert_with_return_id: {}", query);
        
        let result = sqlx::query(&query)
            .bind_all(values)
            .fetch_one(&self.pool)
            .await;
        
        let duration = start.elapsed();
        self.metrics.record_query(duration).await;
        
        match result {
            Ok(row) => {
                let id: i64 = row.try_get("rowid")?;
                debug!("Insert executed successfully in {:?}: ID {}", duration, id);
                Ok(id)
            }
            Err(e) => {
                error!("Insert failed after {:?}: {} - Error: {}", duration, query, e);
                Err(anyhow!("Database insert failed: {}", e))
            }
        }
    }

    /// Update records and return the number of affected rows
    pub async fn update(
        &self,
        table: &str,
        set_clauses: &[&str],
        where_clause: &str,
        params: &[&(dyn sqlx::Encode<sqlx::Sqlite> + sqlx::Type<sqlx::Sqlite> + Sync)],
    ) -> Result<u64> {
        let set_str = set_clauses.join(", ");
        let query = format!("UPDATE {} SET {} WHERE {}", table, set_str, where_clause);
        
        let result = self.execute_query(&query, params).await?;
        Ok(result.rows_affected())
    }

    /// Delete records and return the number of affected rows
    pub async fn delete(
        &self,
        table: &str,
        where_clause: &str,
        params: &[&(dyn sqlx::Encode<sqlx::Sqlite> + sqlx::Type<sqlx::Sqlite> + Sync)],
    ) -> Result<u64> {
        let query = format!("DELETE FROM {} WHERE {}", table, where_clause);
        
        let result = self.execute_query(&query, params).await?;
        Ok(result.rows_affected())
    }

    /// Check if a record exists
    pub async fn exists(
        &self,
        table: &str,
        where_clause: &str,
        params: &[&(dyn sqlx::Encode<sqlx::Sqlite> + sqlx::Type<sqlx::Sqlite> + Sync)],
    ) -> Result<bool> {
        let query = format!("SELECT EXISTS(SELECT 1 FROM {} WHERE {}) as exists", table, where_clause);
        
        let result: (bool,) = self.query_one(&query, params).await?;
        Ok(result.0)
    }

    /// Count records
    pub async fn count(
        &self,
        table: &str,
        where_clause: Option<&str>,
        params: &[&(dyn sqlx::Encode<sqlx::Sqlite> + sqlx::Type<sqlx::Sqlite> + Sync)],
    ) -> Result<i64> {
        let query = if let Some(where_clause) = where_clause {
            format!("SELECT COUNT(*) as count FROM {} WHERE {}", table, where_clause)
        } else {
            format!("SELECT COUNT(*) as count FROM {}", table)
        };
        
        let result: (i64,) = self.query_one(&query, params).await?;
        Ok(result.0)
    }

    /// Get table statistics
    pub async fn get_table_stats(&self, table: &str) -> Result<TableStats> {
        let count_query = format!("SELECT COUNT(*) as count FROM {}", table);
        let size_query = format!(
            "SELECT SUM(pgsize) as size FROM dbstat WHERE name = '{}'",
            table
        );
        
        let count: (i64,) = self.query_one(&count_query, &[]).await?;
        
        let size: (Option<i64>,) = self.query_optional(&size_query, &[]).await?;
        let size_bytes = size.0.unwrap_or(0);
        
        Ok(TableStats {
            table_name: table.to_string(),
            row_count: count.0,
            size_bytes,
        })
    }
}

/// Table statistics
#[derive(Debug, Clone)]
pub struct TableStats {
    pub table_name: String,
    pub row_count: i64,
    pub size_bytes: i64,
}

/// Memory record operations
impl DatabaseOperations {
    /// Insert a memory record
    pub async fn insert_memory_record(
        &self,
        id: &str,
        content: &str,
        timestamp: &chrono::DateTime<chrono::Utc>,
        layer: &str,
        emotional_tags: Option<&str>,
        context_associations: Option<&str>,
    ) -> Result<i64> {
        let query = r#"
            INSERT INTO memory_records (id, content, timestamp, layer, emotional_tags, context_associations)
            VALUES (?, ?, ?, ?, ?, ?)
        "#;
        
        self.execute_simple_query(query).await?;
        
        // Get the last insert ID
        let result: (i64,) = sqlx::query_as("SELECT last_insert_rowid()")
            .fetch_one(&self.pool)
            .await?;
        
        Ok(result.0)
    }

    /// Get memory records by layer
    pub async fn get_memory_records_by_layer(
        &self,
        layer: &str,
        limit: Option<i64>,
    ) -> Result<Vec<MemoryRecord>> {
        let query = if let Some(limit) = limit {
            format!(
                "SELECT id, content, timestamp, layer, emotional_tags, context_associations 
                 FROM memory_records 
                 WHERE layer = ? 
                 ORDER BY timestamp DESC 
                 LIMIT ?"
            )
        } else {
            format!(
                "SELECT id, content, timestamp, layer, emotional_tags, context_associations 
                 FROM memory_records 
                 WHERE layer = ? 
                 ORDER BY timestamp DESC"
            )
        };
        
        self.query_all(&query).await
    }

    /// Update memory record emotional tags
    pub async fn update_memory_emotional_tags(
        &self,
        id: &str,
        emotional_tags: &str,
    ) -> Result<u64> {
        let query = format!(
            "UPDATE memory_records SET emotional_tags = '{}', updated_at = CURRENT_TIMESTAMP WHERE id = '{}'",
            emotional_tags, id
        );
        
        self.update(&query).await
    }
}

/// Memory record model
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MemoryRecord {
    pub id: String,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub layer: String,
    pub emotional_tags: Option<String>,
    pub context_associations: Option<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{init_db_with_config, DbMetrics, DatabaseConfig};
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_database_operations() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        
        let config = DatabaseConfig {
            max_connections: 1,
            ..Default::default()
        };
        
        let pool = init_db_with_config(config).await.unwrap();
        let metrics = DbMetrics::new();
        let ops = DatabaseOperations::new(pool, metrics);
        
        // Test basic operations
        let result: (i64,) = ops.query_one("SELECT 1 as test").await.unwrap();
        assert_eq!(result.0, 1);
        
        // Test exists
        let exists = ops.exists("SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table') as exists").await.unwrap();
        assert!(exists);
        
        // Test count
        let count = ops.count("SELECT COUNT(*) as count FROM sqlite_master").await.unwrap();
        assert!(count > 0);
    }
}