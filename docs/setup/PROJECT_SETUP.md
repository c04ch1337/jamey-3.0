<<<<<<< HEAD
# Jamey 3.0 Environment Setup Guide

## Table of Contents
1. [Quick Start](#quick-start)
2. [Development Environment Setup](#development-environment)
3. [Production Environment Setup](#production-environment)
4. [Configuration Sections](#configuration-sections)
5. [Security Best Practices](#security-best-practices)
6. [Troubleshooting](#troubleshooting)

## Quick Start

1. Clone the repository and navigate to the project directory:
```bash
git clone https://github.com/your-username/jamey-3.0.git
cd jamey-3.0
```

2. Copy the environment template:
```bash
cp .env.example .env
```

3. Generate required secrets:
```bash
# API key for production
openssl rand -hex 32 > api.key

# MQTT JWT secret (if using MQTT)
openssl rand -hex 32 > mqtt.key

# Phoenix encryption key (if using backup system)
openssl rand -hex 32 > phoenix.key
```

4. Update `.env` with generated secrets and required settings.

## Development Environment

### Basic Setup
1. Install dependencies:
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Node.js (for frontend)
# Visit https://nodejs.org and install version 18+
```

2. Configure development environment:
```env
# Core - Minimal setup for development
OPENROUTER_API_KEY=your-test-key
DATABASE_URL=sqlite:data/jamey.db
DATA_DIR=./data

# Security - Relaxed for local development
API_KEY=                          # Optional in development
ALLOWED_ORIGINS=http://localhost:5173,http://localhost:3000
RATE_LIMIT_RPS=50
RATE_LIMIT_BURST=100

# Operational
DEV_MODE=true
ENABLE_TEST_FEATURES=true
RUST_LOG=debug
```

### Development Tools
- VSCode with rust-analyzer extension recommended
- SQLite browser for database inspection
- MQTT client (e.g., MQTT Explorer) for testing MQTT features

## Production Environment

### System Requirements
- Linux-based OS recommended
- Minimum 2GB RAM
- 20GB storage space
- Network access for API and MQTT connections

### Security Requirements
1. Generate secure keys:
```bash
# Generate all required keys
./scripts/generate-keys.sh
```

2. Configure production settings:
```env
# Core - Production settings
OPENROUTER_API_KEY=your-production-key
DATABASE_URL=sqlite:/var/lib/jamey/data.db
DATA_DIR=/var/lib/jamey/data

# Security - Strict requirements
API_KEY=<generated-api-key>       # Required
ALLOWED_ORIGINS=https://your-domain.com
RATE_LIMIT_RPS=10
RATE_LIMIT_BURST=20

# Operational
DEV_MODE=false
ENABLE_TEST_FEATURES=false
RUST_LOG=info
```

### SSL/TLS Configuration
1. Set up SSL certificates:
```bash
# For MQTT TLS
MQTT_TLS_CA_CERT=/etc/jamey/certs/ca.crt
MQTT_TLS_CLIENT_CERT=/etc/jamey/certs/client.crt
MQTT_TLS_CLIENT_KEY=/etc/jamey/certs/client.key
```

### Backup Configuration
1. Configure Phoenix backup system:
```env
PHOENIX_ENABLED=true
PHOENIX_BACKUP_DIR=/var/lib/jamey/backups
PHOENIX_ENCRYPTION_KEY=<generated-key>
PHOENIX_AUTO_BACKUP_HOURS=24
PHOENIX_MAX_BACKUPS=10
```

## Configuration Sections

### Core Configuration
- **OpenRouter API**: Authentication and model selection
- **Database**: SQLite configuration and data directory
- **Server**: Port and host settings

### Security Configuration
- **API Authentication**: API key management
- **CORS**: Origin restrictions
- **Rate Limiting**: Request throttling
- **Input Validation**: Content length and value constraints

### Soul System Configuration
- **Core Settings**: System enablement
- **Trust Parameters**: Trust levels and decay rates
- **Memory Settings**: Pruning and empathy thresholds

### MQTT Configuration
- **Connection**: Broker URL and port
- **TLS**: Certificate configuration
- **Authentication**: JWT settings
- **Client Settings**: Connection parameters

### Phoenix Backup System
- **Core Settings**: System enablement
- **Encryption**: Secure backup configuration
- **Backup Policy**: Scheduling and retention

## Security Best Practices

### Secret Management
1. Never commit secrets to version control
2. Use environment-specific .env files
3. Rotate secrets periodically
4. Use secure secret generation methods

### File Permissions
```bash
# Set restrictive permissions on sensitive files
chmod 600 .env
chmod 600 api.key
chmod 600 mqtt.key
chmod 600 phoenix.key
```

### Access Control
1. Run services with minimal privileges
2. Use separate service accounts
3. Implement proper file ownership

## Troubleshooting

### Common Issues

1. **Database Connection Errors**
   - Check file permissions
   - Verify directory exists
   - Ensure SQLite is installed

2. **MQTT Connection Issues**
   - Verify broker is running
   - Check certificate paths
   - Validate JWT secret

3. **API Authentication Failures**
   - Confirm API_KEY is set
   - Check key length requirements
   - Verify key is properly formatted

### Logging

Set appropriate log levels for debugging:
```env
# Detailed debugging
RUST_LOG=debug

# Production logging
RUST_LOG=info

# Error-only logging
RUST_LOG=error
```

### Health Checks

1. API Health:
```bash
curl http://localhost:3000/health
```

2. MQTT Health:
```bash
# Using mosquitto_pub
mosquitto_pub -h localhost -p 8883 -t "jamey/health" -m "ping"
```

3. Backup Health:
```bash
# Check backup status
./scripts/check-backup-health.sh
```

For additional assistance, consult the project documentation or open an issue on GitHub.

---

## Project Structure

The Jamey 3.0 source code project has the following structure:

### Backend (Rust)
- **Cargo.toml** - Rust dependencies and project configuration
- **src/main.rs** - Application entry point
- **src/lib.rs** - Library root with module exports
- **src/api/mod.rs** - Axum API routes and handlers
- **src/conscience/mod.rs** - Conscience Engine with moral rule evaluation
- **src/memory/mod.rs** - 5-Layer Memory System with Tantivy indexing
- **src/db/mod.rs** - Database layer with SQLx
- **src/soul/** - Soul Knowledge Base for entity tracking and emotions

### Frontend (React + TypeScript)
- **frontend/package.json** - Dependencies including TanStack Query, Axios, Zod
- **frontend/vite.config.ts** - Vite configuration with API proxy
- **frontend/src/main.tsx** - React entry with QueryClient setup
- **frontend/src/App.tsx** - Main application component
- **frontend/src/api/client.ts** - API client with typed functions
- **frontend/src/App.css** - Modern, responsive styling

### Configuration Files
- **.gitignore** - Git ignore patterns
- **README.md** - Project documentation
- **scripts/setup.sh** - Automated setup script
- **migrations/** - SQLx migrations directory

## Features Implemented

### Conscience Engine
- ✅ Weighted moral rule system
- ✅ Default rules: "no-harm" (10.0) and "truth" (8.0)
- ✅ Action evaluation with scoring
- ✅ Dynamic rule management (add/remove)
- ✅ Soul KB integration for emotion tracking

### Memory System
- ✅ 5-Layer Memory System:
  - Short-term: Immediate actions
  - Long-term: Persistent learnings
  - Working: Current context
  - Episodic: Event sequences
  - Semantic: Conceptual knowledge
- ✅ Tantivy indexing for fast search
- ✅ Async memory operations
- ✅ Entity linking for soul integration

### API Endpoints
- ✅ `GET /` - Health check
- ✅ `POST /evaluate` - Evaluate action morality
- ✅ `GET /rules` - Get all moral rules
- ✅ `POST /rules` - Add new moral rule

### Frontend
- ✅ React 18 with TypeScript
- ✅ TanStack Query for server state
- ✅ Axios for HTTP requests
- ✅ Zod for input validation
- ✅ Modern, responsive UI with accessibility features
- ✅ Real-time action evaluation
- ✅ Rule management interface
- ✅ Error boundaries and retry logic

## Notes

- The project follows Rust 2021 edition conventions
- Uses `tracing` for logging (set `RUST_LOG=info` for verbose output)
- CORS is configured via `ALLOWED_ORIGINS` environment variable
- Memory system uses UUID v4 for record identifiers
- All timestamps use UTC
- Frontend includes comprehensive input validation and sanitization
