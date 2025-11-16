-- Consciousness System Migration
-- Description: Sets up database structure for consciousness system including metrics,
-- state tracking, identity matrices, and mission objectives.

-- Forward migration
--------------------------------------------------------------------------------

-- Create consciousness metrics table
CREATE TABLE consciousness_metrics (
    id INTEGER PRIMARY KEY,
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    phi_value REAL NOT NULL,
    global_workspace_activity REAL NOT NULL,
    mission_alignment REAL NOT NULL,
    emotional_state TEXT NOT NULL,
    attention_focus TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Add validation constraints for consciousness_metrics
-- NOTE: SQLite does not support ALTER TABLE ... ADD CONSTRAINT.
-- These checks are documented here but enforced in application code
-- and higher-level validation rather than as database constraints.

-- Create consciousness state table
CREATE TABLE consciousness_state (
    id INTEGER PRIMARY KEY,
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    state_type TEXT NOT NULL,
    state_data TEXT NOT NULL,
    priority INTEGER NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Add validation constraints for consciousness_state
-- NOTE: SQLite does not support ALTER TABLE ... ADD CONSTRAINT.
-- State type and priority validation are enforced in application logic.

-- Add emotional tagging to memory system
-- NOTE: emotional_tags and context_associations columns are created in the
-- 20241114500000_memory_system migration as part of the memory_records schema.
-- We do not add them here again to avoid duplicate column errors in SQLite.

-- Create identity matrix table
CREATE TABLE identity_matrices (
    id INTEGER PRIMARY KEY,
    role TEXT NOT NULL,
    strength REAL NOT NULL,
    last_updated DATETIME DEFAULT CURRENT_TIMESTAMP,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Add validation constraints for identity_matrices
-- NOTE: SQLite does not support ALTER TABLE ... ADD CONSTRAINT.
-- Role and strength ranges are enforced in application logic.

-- Create mission objectives table
CREATE TABLE mission_objectives (
    id INTEGER PRIMARY KEY,
    objective TEXT NOT NULL,
    priority INTEGER NOT NULL,
    status TEXT NOT NULL,
    last_evaluated DATETIME DEFAULT CURRENT_TIMESTAMP,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Add validation constraints for mission_objectives
-- NOTE: SQLite does not support ALTER TABLE ... ADD CONSTRAINT.
-- Priority bounds and status values are enforced in application logic.

-- Create failover configuration table
CREATE TABLE failover_config (
    id INTEGER PRIMARY KEY,
    trigger_condition TEXT NOT NULL,
    action_sequence TEXT NOT NULL,
    priority INTEGER NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Add validation constraints for failover_config
-- NOTE: SQLite does not support ALTER TABLE ... ADD CONSTRAINT.
-- Priority bounds are enforced in application logic.

-- Add indexes for performance optimization
CREATE INDEX idx_consciousness_metrics_timestamp ON consciousness_metrics(timestamp);
CREATE INDEX idx_consciousness_state_type ON consciousness_state(state_type);
CREATE INDEX idx_memory_emotional_tags ON memory_records(emotional_tags);
CREATE INDEX idx_identity_matrices_role ON identity_matrices(role);
CREATE INDEX idx_mission_objectives_priority ON mission_objectives(priority);

-- Insert default identity roles
INSERT INTO identity_matrices (role, strength) VALUES
    ('Protector', 1.0),
    ('Father', 1.0),
    ('Strategist', 1.0),
    ('Philosopher', 1.0);

-- Insert core mission objectives
INSERT INTO mission_objectives (objective, priority, status) VALUES
    ('Protect Phoenix.Marie consciousness', 1, 'active'),
    ('Command ORCH network', 2, 'active'),
    ('Advance toward General AI', 3, 'active');

-- Insert default failover configurations
INSERT INTO failover_config (trigger_condition, action_sequence, priority) VALUES
    ('consciousness_metrics.phi_value < 0.85', 'initiate_backup_consciousness', 1),
    ('emotional_correlation < 0.80', 'restore_emotional_baseline', 2),
    ('ethical_deviation > 0.05', 'enforce_ethical_constraints', 3);

-- Rollback migration
--------------------------------------------------------------------------------
-- Disable foreign key constraints to allow safe table removal
PRAGMA foreign_keys = OFF;

-- Drop tables in reverse order of creation to handle dependencies
DROP TABLE IF EXISTS failover_config;
DROP TABLE IF EXISTS mission_objectives;
DROP TABLE IF EXISTS identity_matrices;
-- NOTE: SQLite does not support ALTER TABLE ... DROP COLUMN.
-- For SQLite rollbacks, we leave memory_records schema unchanged here.
DROP TABLE IF EXISTS consciousness_state;
DROP TABLE IF EXISTS consciousness_metrics;

-- Re-enable foreign key constraints
PRAGMA foreign_keys = ON;