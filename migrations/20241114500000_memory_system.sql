-- Memory System Migration
-- Description: Creates the memory_records table to support the consciousness system
-- This migration must run before the consciousness system migration

-- Forward migration
--------------------------------------------------------------------------------

-- Create memory_records table
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

-- Add validation constraints
-- NOTE: SQLite does not support ALTER TABLE ... ADD CONSTRAINT.
-- The allowed layer values are enforced in application logic instead of
-- as a database-level CHECK constraint.

-- Create indexes for performance
CREATE INDEX idx_memory_records_timestamp ON memory_records(timestamp);
CREATE INDEX idx_memory_records_layer ON memory_records(layer);
CREATE INDEX idx_memory_records_created_at ON memory_records(created_at);

-- Rollback migration
-- NOTE: Down migration is not implemented for SQLite in this file to avoid
-- accidentally dropping the memory_records table during forward migrations.
-- If a rollback is required, drop memory_records and related indexes manually.