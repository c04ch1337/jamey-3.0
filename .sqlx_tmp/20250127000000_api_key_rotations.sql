-- Migration: Add API key rotation tracking table
-- Date: 2025-01-27
-- Description: Tracks API key rotations for audit and compliance

CREATE TABLE IF NOT EXISTS api_key_rotations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    key_id INTEGER NOT NULL,
    old_key_hash TEXT NOT NULL,
    new_key_name TEXT NOT NULL,
    grace_period_days INTEGER NOT NULL DEFAULT 7,
    rotated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (key_id) REFERENCES api_keys(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_api_key_rotations_key_id ON api_key_rotations(key_id);
CREATE INDEX IF NOT EXISTS idx_api_key_rotations_rotated_at ON api_key_rotations(rotated_at);

