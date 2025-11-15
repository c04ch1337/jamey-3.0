-- Initial migration for Jamey 3.0
-- This creates the basic schema for the application

-- Example table (can be expanded as needed)
-- For now, we'll keep it minimal since most data is in Tantivy indices

CREATE TABLE IF NOT EXISTS app_metadata (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

-- Insert initial metadata
INSERT OR IGNORE INTO app_metadata (key, value) VALUES 
    ('version', '3.0.0'),
    ('initialized_at', datetime('now'));

