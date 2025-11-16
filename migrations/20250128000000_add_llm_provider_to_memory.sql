-- Add LLM Provider Preference to Memory Records
-- Description: Adds preferred_llm_provider field to allow documents to specify which LLM model to use

-- Forward migration
--------------------------------------------------------------------------------

-- Add preferred_llm_provider column to memory_records table
ALTER TABLE memory_records ADD COLUMN preferred_llm_provider TEXT;

-- Create index for provider lookups
CREATE INDEX IF NOT EXISTS idx_memory_records_llm_provider ON memory_records(preferred_llm_provider);

-- Add comment/documentation
-- The preferred_llm_provider field stores the OpenRouter model ID (e.g., "anthropic/claude-3-opus")
-- that should be used when processing this document. If NULL, the system will use default routing.

-- Rollback migration
-- NOTE: SQLite does not support DROP COLUMN directly. If rollback is needed:
-- 1. Create new table without the column
-- 2. Copy data (excluding preferred_llm_provider)
-- 3. Drop old table and rename new one
-- This is complex and not recommended for production systems.

