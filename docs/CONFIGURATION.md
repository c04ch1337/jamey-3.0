# Jamey 3.0 Configuration Guide

## Table of Contents
1. [Overview](#overview)
2. [Configuration Sections](#configuration-sections)
3. [Type Specifications](#type-specifications)
4. [Validation Rules](#validation-rules)
5. [Deployment Scenarios](#deployment-scenarios)

## Overview

This document provides detailed information about each configuration section in Jamey 3.0, including type specifications, validation rules, and deployment recommendations.

## Configuration Sections

### 1. Core Configuration

#### OpenRouter API Configuration
```env
OPENROUTER_API_KEY=                # [Required] Your OpenRouter API key
OPENROUTER_MODEL=                  # [Optional] Model selection
OPENROUTER_API_URL=                # [Optional] API endpoint
```

**Details:**
- `OPENROUTER_API_KEY`: Authentication key for OpenRouter API access
  - Required in all environments
  - No default value
  - Must be kept secure

- `OPENROUTER_MODEL`: Specifies the LLM model to use
  - Default: `anthropic/claude-3.5-sonnet`
  - Available models: See OpenRouter documentation
  - Consider cost/performance tradeoffs

- `OPENROUTER_API_URL`: Custom API endpoint
  - Default: `https://openrouter.ai/api/v1`
  - Use for self-hosted or enterprise deployments

#### Database Configuration
```env
DATABASE_URL=                      # [Optional] SQLite database path
DATA_DIR=                         # [Optional] Data directory path
```

**Details:**
- `DATABASE_URL`: SQLite database connection string
  - Default: `sqlite:data/jamey.db`
  - Format: `sqlite:path/to/database.db`
  - Relative or absolute paths supported

- `DATA_DIR`: Base directory for all data storage
  - Default: `./data`
  - Must be writable by application
  - Used by memory system and backups

### 2. Security Configuration

#### API Authentication
```env
API_KEY=                          # [Required in prod] API authentication key
```

**Details:**
- Required in production environments
- Minimum 32 characters
- Generate using: `openssl rand -hex 32`
- Must be transmitted securely
- Regular rotation recommended

#### CORS Configuration
```env
ALLOWED_ORIGINS=                  # [Required in prod] Allowed origins
```

**Details:**
- Comma-separated list of allowed origins
- Default: `http://localhost:5173,http://localhost:3000`
- Production: Use specific domain list
- Wildcards not recommended
- Consider subdomains carefully

#### Rate Limiting
```env
RATE_LIMIT_RPS=                  # [Optional] Requests per second
RATE_LIMIT_BURST=                # [Optional] Burst size
```

**Details:**
- `RATE_LIMIT_RPS`: Base rate limit
  - Default: 50
  - Range: 1-1000
  - Adjust based on resources

- `RATE_LIMIT_BURST`: Maximum burst
  - Default: 100
  - Must be > RATE_LIMIT_RPS
  - Handles traffic spikes

### 3. Soul System Configuration

#### Core Settings
```env
SOUL_ENABLED=                    # [Optional] Enable/disable system
SOUL_AUTO_RECORD=               # [Optional] Auto-record emotions
```

**Details:**
- `SOUL_ENABLED`: Master switch
  - Default: true
  - Boolean value
  - Affects all soul features

- `SOUL_AUTO_RECORD`: Automatic emotion recording
  - Default: true
  - Boolean value
  - Performance impact: Low

#### Trust Parameters
```env
SOUL_DEFAULT_TRUST=             # [Optional] Default trust level
SOUL_BASE_DECAY_RATE=          # [Optional] Memory decay rate
SOUL_PRUNE_THRESHOLD=          # [Optional] Pruning threshold
SOUL_EMPATHY_THRESHOLD=        # [Optional] Empathy threshold
```

**Details:**
- All values are floating-point between 0.0 and 1.0
- Affects memory system behavior
- Consider memory usage impact
- Tune based on application needs

### 4. MQTT Configuration

#### Connection Settings
```env
MQTT_BROKER_URL=                # [Optional] Broker URL
MQTT_PORT=                      # [Optional] Broker port
```

**Details:**
- `MQTT_BROKER_URL`: MQTT broker address
  - Default: `mqtt://localhost`
  - Supports TCP and SSL/TLS
  - Format: `mqtt://host` or `mqtts://host`

- `MQTT_PORT`: Broker port number
  - Default: 8883 (TLS)
  - Common ports: 1883 (TCP), 8883 (TLS)
  - Check firewall rules

#### TLS Configuration
```env
MQTT_TLS_CA_CERT=              # [Required if TLS] CA certificate
MQTT_TLS_CLIENT_CERT=          # [Optional] Client certificate
MQTT_TLS_CLIENT_KEY=           # [Optional] Client private key
```

**Details:**
- File paths must be absolute in production
- Verify file permissions
- Regular certificate rotation
- Monitor expiration dates

### 5. Phoenix Backup System

#### Core Settings
```env
PHOENIX_ENABLED=               # [Optional] Enable/disable
PHOENIX_BACKUP_DIR=            # [Optional] Backup location
```

**Details:**
- `PHOENIX_ENABLED`: Master switch
  - Default: true
  - Boolean value
  - Consider storage impact

- `PHOENIX_BACKUP_DIR`: Backup storage
  - Default: `data/phoenix`
  - Must be writable
  - Monitor disk space

#### Encryption
```env
PHOENIX_ENCRYPTION_KEY=        # [Required if enabled] Encryption key
```

**Details:**
- 32-byte hex key required
- Generate using: `openssl rand -hex 32`
- Store securely
- Backup key separately

## Type Specifications

### String Values
- API keys: Hexadecimal string, 64 characters
- URLs: Valid URL string
- File paths: Valid system path string
- Origins: Valid URL or domain string

### Numeric Values
- Ports: Integer, range 1-65535
- Rate limits: Positive integer
- Time intervals: Positive integer (seconds/hours)
- Trust levels: Float, range 0.0-1.0

### Boolean Values
- Feature flags: true/false
- System enablement: true/false

## Validation Rules

### Required Values
1. Production environment:
   - API_KEY
   - ALLOWED_ORIGINS
   - OPENROUTER_API_KEY
   - PHOENIX_ENCRYPTION_KEY (if enabled)
   - MQTT_JWT_SECRET (if MQTT used)

2. Development environment:
   - OPENROUTER_API_KEY

### Length Requirements
- API_KEY: ≥ 32 characters
- PHOENIX_ENCRYPTION_KEY: 64 characters
- MQTT_JWT_SECRET: ≥ 32 characters

### Numeric Ranges
- PORT: 1-65535
- RATE_LIMIT_RPS: > 0
- RATE_LIMIT_BURST: > RATE_LIMIT_RPS
- Trust parameters: 0.0-1.0

## Deployment Scenarios

### Local Development
```env
DEV_MODE=true
ENABLE_TEST_FEATURES=true
RUST_LOG=debug
API_KEY=                    # Optional
ALLOWED_ORIGINS=http://localhost:5173,http://localhost:3000
```

### Testing Environment
```env
DEV_MODE=true
ENABLE_TEST_FEATURES=true
RUST_LOG=info
API_KEY=<required>
ALLOWED_ORIGINS=https://test.example.com
```

### Production Environment
```env
DEV_MODE=false
ENABLE_TEST_FEATURES=false
RUST_LOG=info
API_KEY=<required>
ALLOWED_ORIGINS=https://example.com
PHOENIX_ENABLED=true
PHOENIX_ENCRYPTION_KEY=<required>
```

### High-Security Environment
```env
DEV_MODE=false
ENABLE_TEST_FEATURES=false
RUST_LOG=warn
API_KEY=<required>
ALLOWED_ORIGINS=https://secure.example.com
RATE_LIMIT_RPS=10
RATE_LIMIT_BURST=20
PHOENIX_ENABLED=true
PHOENIX_ENCRYPTION_KEY=<required>
MQTT_TLS_CA_CERT=<required>
MQTT_TLS_CLIENT_CERT=<required>
MQTT_TLS_CLIENT_KEY=<required>