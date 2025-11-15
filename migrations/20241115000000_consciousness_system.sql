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
ALTER TABLE consciousness_metrics ADD CONSTRAINT chk_phi_value
    CHECK (phi_value >= 0.0 AND phi_value <= 1.0);
ALTER TABLE consciousness_metrics ADD CONSTRAINT chk_global_workspace_activity
    CHECK (global_workspace_activity >= 0.0 AND global_workspace_activity <= 1.0);
ALTER TABLE consciousness_metrics ADD CONSTRAINT chk_mission_alignment
    CHECK (mission_alignment >= 0.0 AND mission_alignment <= 1.0);

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
ALTER TABLE consciousness_state ADD CONSTRAINT chk_state_type
    CHECK (state_type IN ('active', 'idle', 'processing', 'learning', 'integrating'));
ALTER TABLE consciousness_state ADD CONSTRAINT chk_priority
    CHECK (priority >= 1 AND priority <= 10);

-- Add emotional tagging to memory system
ALTER TABLE memory_records ADD COLUMN emotional_tags TEXT;
ALTER TABLE memory_records ADD COLUMN context_associations TEXT;

-- Create identity matrix table
CREATE TABLE identity_matrices (
    id INTEGER PRIMARY KEY,
    role TEXT NOT NULL,
    strength REAL NOT NULL,
    last_updated DATETIME DEFAULT CURRENT_TIMESTAMP,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Add validation constraints for identity_matrices
ALTER TABLE identity_matrices ADD CONSTRAINT chk_identity_role
    CHECK (role IN ('Protector', 'Father', 'Strategist', 'Philosopher', 'Learner', 'Observer'));
ALTER TABLE identity_matrices ADD CONSTRAINT chk_strength
    CHECK (strength >= 0.0 AND strength <= 1.0);

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
ALTER TABLE mission_objectives ADD CONSTRAINT chk_objective_priority
    CHECK (priority >= 1 AND priority <= 10);
ALTER TABLE mission_objectives ADD CONSTRAINT chk_objective_status
    CHECK (status IN ('active', 'completed', 'paused', 'failed', 'pending'));

-- Create failover configuration table
CREATE TABLE failover_config (
    id INTEGER PRIMARY KEY,
    trigger_condition TEXT NOT NULL,
    action_sequence TEXT NOT NULL,
    priority INTEGER NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Add validation constraints for failover_config
ALTER TABLE failover_config ADD CONSTRAINT chk_failover_priority
    CHECK (priority >= 1 AND priority <= 10);

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
-- Remove columns from memory_records before dropping the table
ALTER TABLE memory_records DROP COLUMN IF EXISTS emotional_tags;
ALTER TABLE memory_records DROP COLUMN IF EXISTS context_associations;
DROP TABLE IF EXISTS consciousness_state;
DROP TABLE IF EXISTS consciousness_metrics;

-- Re-enable foreign key constraints
PRAGMA foreign_keys = ON;