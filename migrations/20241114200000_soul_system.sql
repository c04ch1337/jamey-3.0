-- Soul Knowledge Base Schema
-- Creates tables for tracking entities, emotions, and trust relationships

-- Soul entities table
CREATE TABLE IF NOT EXISTS soul_entities (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    entity_name TEXT NOT NULL UNIQUE,
    trust_score REAL NOT NULL DEFAULT 0.5,
    decay_rate REAL NOT NULL DEFAULT 0.01,
    last_interaction TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Soul emotions table  
CREATE TABLE IF NOT EXISTS soul_emotions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    entity_id INTEGER NOT NULL,
    emotion TEXT NOT NULL,
    count INTEGER NOT NULL DEFAULT 1,
    FOREIGN KEY (entity_id) REFERENCES soul_entities(id) ON DELETE CASCADE,
    UNIQUE(entity_id, emotion)
);

-- Soul memory links table
CREATE TABLE IF NOT EXISTS soul_memory_links (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    entity_id INTEGER NOT NULL,
    memory_id TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (entity_id) REFERENCES soul_entities(id) ON DELETE CASCADE
);

-- Indices for performance
CREATE INDEX IF NOT EXISTS idx_soul_entities_name ON soul_entities(entity_name);
CREATE INDEX IF NOT EXISTS idx_soul_emotions_entity ON soul_emotions(entity_id);
CREATE INDEX IF NOT EXISTS idx_soul_memory_entity ON soul_memory_links(entity_id);
CREATE INDEX IF NOT EXISTS idx_soul_memory_id ON soul_memory_links(memory_id);