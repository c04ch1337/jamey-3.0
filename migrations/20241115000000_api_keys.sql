-- API Key Management Migration
-- Supports API key rotation and per-key rate limiting

CREATE TABLE IF NOT EXISTS api_keys (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    key_hash TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TEXT,
    revoked_at TEXT,
    last_used_at TEXT,
    rate_limit_per_minute INTEGER DEFAULT 60
);

CREATE INDEX IF NOT EXISTS idx_api_keys_hash ON api_keys(key_hash);
CREATE INDEX IF NOT EXISTS idx_api_keys_active ON api_keys(expires_at, revoked_at);

-- Insert default API key if API_KEY env var is set (will be hashed by application)
-- Note: This is a placeholder - actual key creation should be done via API or CLI

