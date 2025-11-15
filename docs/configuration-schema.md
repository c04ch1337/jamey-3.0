# Jamey 3.0 Configuration Schema

## Overview
This document defines the consolidated configuration schema for Jamey 3.0, organizing all settings into logical sections with clear documentation and validation rules.

## Schema Sections

### 1. Core Configuration
```env
# OpenRouter API Configuration
OPENROUTER_API_KEY=                # [Required] API key for OpenRouter
OPENROUTER_MODEL=                  # [Optional] Model name (default: anthropic/claude-3.5-sonnet)
OPENROUTER_API_URL=                # [Optional] Custom API endpoint

# Database Configuration
DATABASE_URL=                      # [Optional] SQLite database path (default: data/jamey.db)
DATA_DIR=                         # [Optional] Override data directory (default: ./data)
```

### 2. Security Configuration
```env
# API Authentication
API_KEY=                          # [Required in prod] API authentication key (min 32 chars)
                                 # Generate: openssl rand -hex 32

# CORS Configuration
ALLOWED_ORIGINS=                  # [Required in prod] Comma-separated list of allowed origins
                                 # Default: http://localhost:5173,http://localhost:3000

# Rate Limiting
RATE_LIMIT_RPS=                  # [Optional] Requests per second per IP (default: 50)
RATE_LIMIT_BURST=                # [Optional] Maximum burst size (default: 100)

# Input Validation
MAX_ACTION_LENGTH=               # [Optional] Maximum action text length (default: 10000)
MAX_RULE_NAME_LENGTH=           # [Optional] Maximum rule name length (default: 100)
MAX_RULE_DESCRIPTION_LENGTH=    # [Optional] Maximum rule description length (default: 500)
MIN_RULE_WEIGHT=               # [Optional] Minimum rule weight (default: 0.0)
MAX_RULE_WEIGHT=               # [Optional] Maximum rule weight (default: 100.0)
```

### 3. Soul System Configuration
```env
# Core Settings
SOUL_ENABLED=                    # [Optional] Enable/disable Soul system (default: true)
SOUL_AUTO_RECORD=               # [Optional] Auto-record emotions (default: true)

# Trust and Decay Parameters
SOUL_DEFAULT_TRUST=             # [Optional] Default trust level (default: 0.5)
SOUL_BASE_DECAY_RATE=          # [Optional] Base memory decay rate (default: 0.01)
SOUL_PRUNE_THRESHOLD=          # [Optional] Memory pruning threshold (default: 0.1)
SOUL_EMPATHY_THRESHOLD=        # [Optional] Empathy activation threshold (default: 0.7)
```

### 4. MQTT Configuration
```env
# Connection Settings
MQTT_BROKER_URL=                # [Optional] Broker URL (default: mqtt://localhost)
MQTT_PORT=                      # [Optional] Broker port (default: 8883)

# TLS Configuration
MQTT_TLS_CA_CERT=              # [Required if using TLS] Path to CA certificate
MQTT_TLS_CLIENT_CERT=          # [Optional] Path to client certificate for mTLS
MQTT_TLS_CLIENT_KEY=           # [Optional] Path to client private key for mTLS

# Authentication
MQTT_JWT_SECRET=               # [Required if enabled] JWT secret (min 32 chars)
                              # Generate: openssl rand -hex 32
MQTT_JWT_LIFETIME_SECONDS=     # [Optional] JWT token lifetime (default: 300)

# Client Settings
MQTT_CLIENT_ID=                # [Optional] Client identifier (default: jamey-{uuid})
MQTT_KEEP_ALIVE_SECONDS=       # [Optional] Keep-alive interval (default: 60)
MQTT_MAX_PACKET_SIZE=         # [Optional] Maximum packet size (default: 268435456)
MQTT_CONNECTION_TIMEOUT_SECONDS= # [Optional] Connection timeout (default: 30)

# Permissions
MQTT_PERMISSIONS=              # [Optional] Comma-separated topic permissions
                              # Default: jamey/#
```

### 5. Phoenix Backup System
```env
# Core Settings
PHOENIX_ENABLED=               # [Optional] Enable/disable backup system (default: true)
PHOENIX_BACKUP_DIR=            # [Optional] Backup directory (default: data/phoenix)

# Encryption
PHOENIX_ENCRYPTION_KEY=        # [Required if enabled] 32-byte hex encryption key
                              # Generate: openssl rand -hex 32

# Backup Configuration
PHOENIX_AUTO_BACKUP_HOURS=     # [Optional] Automatic backup interval (default: 24)
PHOENIX_MAX_BACKUPS=          # [Optional] Maximum backups to retain (default: 10)
```

### 6. Operational Settings
```env
# Server Configuration
PORT=                         # [Optional] Server port (default: 3000)
HOST=                        # [Optional] Server host (default: 0.0.0.0)

# Logging
RUST_LOG=                    # [Optional] Log level (default: info)
                            # Values: error, warn, info, debug, trace

# Development
DEV_MODE=                   # [Optional] Enable development mode (default: false)
ENABLE_TEST_FEATURES=       # [Optional] Enable test features (default: false)
```

## Validation Rules

1. **Required in Production**
   - API_KEY
   - ALLOWED_ORIGINS
   - OPENROUTER_API_KEY
   - PHOENIX_ENCRYPTION_KEY (if PHOENIX_ENABLED=true)
   - MQTT_JWT_SECRET (if MQTT broker configured)

2. **Length Constraints**
   - API_KEY: minimum 32 characters
   - PHOENIX_ENCRYPTION_KEY: exactly 64 hex characters
   - MQTT_JWT_SECRET: minimum 32 characters

3. **Numeric Ranges**
   - PORT: 1-65535
   - RATE_LIMIT_RPS: > 0
   - RATE_LIMIT_BURST: > RATE_LIMIT_RPS
   - SOUL_DEFAULT_TRUST: 0.0-1.0
   - SOUL_BASE_DECAY_RATE: 0.0-1.0
   - SOUL_PRUNE_THRESHOLD: 0.0-1.0
   - SOUL_EMPATHY_THRESHOLD: 0.0-1.0

4. **File System**
   - All paths must be valid and accessible
   - TLS certificates must exist and be readable
   - Backup directory must be writable

## Type Safety

Each configuration value should be strongly typed in the application code:

```rust
// Example type-safe config structure
pub struct Config {
    pub core: CoreConfig,
    pub security: SecurityConfig,
    pub soul: SoulConfig,
    pub mqtt: Option<MqttConfig>,
    pub phoenix: Option<PhoenixConfig>,
    pub operational: OperationalConfig,
}

// Example validation implementation
impl Config {
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Implement validation logic
    }
}
```

## Development vs Production

1. **Development Defaults**
   - Authentication optional
   - CORS allows localhost
   - Verbose logging
   - Test features available

2. **Production Requirements**
   - Authentication required
   - Strict CORS policy
   - Controlled logging
   - Test features disabled
   - Secure encryption keys
   - Regular backups enabled