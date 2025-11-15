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
ALTER TABLE memory_records ADD CONSTRAINT chk_memory_layer 
    CHECK (layer IN ('short_term', 'long_term', 'working', 'episodic', 'semantic'));

-- Create indexes for performance
CREATE INDEX idx_memory_records_timestamp ON memory_records(timestamp);
CREATE INDEX idx_memory_records_layer ON memory_records(layer);
CREATE INDEX idx_memory_records_created_at ON memory_records(created_at);

-- Rollback migration
--------------------------------------------------------------------------------
DROP TABLE IF EXISTS memory_records;