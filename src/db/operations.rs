//! Database Operations Module
//!
//! This module previously contained a rich set of helpers around sqlx. As of the
//! SQLite + Tantivy refactor it is only used as a thin wrapper around the
//! connection pool and database metrics. Higher-level helpers can be reintroduced
//! incrementally as needed.

use sqlx::SqlitePool;

use crate::db::DbMetrics;

 /// Safe database operations wrapper.
 ///
 /// The original implementation exposed many generic helper methods that bound
 /// arbitrary parameter lists, performed timing/metrics, and provided table-level
 /// helpers. Those helpers drifted out of sync with `sqlx` 0.7 and the current
 /// schema, which caused compile-time errors.
 ///
 /// To keep the public API surface coherent while avoiding unused complexity,
 /// this struct is now a lightweight wrapper. Callers are expected to use raw
 /// `sqlx` queries together with [`DbMetrics`](src/db/mod.rs:15) (via the
 /// function-level helpers in [`crate::db`](src/db/mod.rs:1)).
 ///
 /// TODO: if you need richer helpers again, add them back here with targeted,
 /// well-tested implementations built on `sqlx` 0.7.
#[derive(Clone)]
pub struct DatabaseOperations {
    pool: SqlitePool,
    metrics: DbMetrics,
}

impl DatabaseOperations {
    /// Create new database operations instance.
    pub fn new(pool: SqlitePool, metrics: DbMetrics) -> Self {
        Self { pool, metrics }
    }

    /// Get a reference to the connection pool.
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Get a reference to the metrics handle.
    pub fn metrics(&self) -> &DbMetrics {
        &self.metrics
    }
}

// No tests here at the moment. Behaviour is covered indirectly by
// higher-level components that exercise database initialisation in
// [`crate::db`](src/db/mod.rs:1).