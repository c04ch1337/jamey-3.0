# Configuration Migration Guide

## Table of Contents
1. [Overview](#overview)
2. [Version Migration Steps](#version-migration-steps)
3. [Breaking Changes](#breaking-changes)
4. [Configuration Validation](#configuration-validation)
5. [Rollback Procedures](#rollback-procedures)

## Overview

This guide helps you migrate your Jamey 3.0 configuration between versions while maintaining security and functionality. Always backup your configuration before migration.

## Version Migration Steps

### Migrating to 3.0

#### Pre-Migration Checklist
1. Backup existing configuration:
```bash
cp .env .env.backup
```

2. Export current secrets:
```bash
# Save current secrets
grep -E "^(API_KEY|MQTT_JWT_SECRET|PHOENIX_ENCRYPTION_KEY)=" .env > secrets.backup
```

3. Verify backups are readable:
```bash
cat .env.backup
cat secrets.backup
```

#### Core Configuration Changes
1. Update OpenRouter settings:
```diff
- OPENAI_API_KEY=
+ OPENROUTER_API_KEY=
- OPENAI_MODEL=
+ OPENROUTER_MODEL=
- OPENAI_API_URL=
+ OPENROUTER_API_URL=
```

2. Update database configuration:
```diff
- DB_PATH=
+ DATABASE_URL=sqlite:data/jamey.db
- DATA_PATH=
+ DATA_DIR=./data
```

#### Security Configuration Updates
1. API authentication changes:
```diff
- AUTH_KEY=
+ API_KEY=                    # Must be ≥32 characters
- CORS_ORIGINS=
+ ALLOWED_ORIGINS=           # Comma-separated list
```

2. Rate limiting configuration:
```diff
- RATE_LIMIT=
+ RATE_LIMIT_RPS=           # Requests per second
+ RATE_LIMIT_BURST=         # Burst size
```

#### Soul System Migration
1. Enable new Soul system:
```env
SOUL_ENABLED=true
SOUL_AUTO_RECORD=true
```

2. Configure trust parameters:
```env
SOUL_DEFAULT_TRUST=0.5
SOUL_BASE_DECAY_RATE=0.01
SOUL_PRUNE_THRESHOLD=0.1
SOUL_EMPATHY_THRESHOLD=0.7
```

#### MQTT Configuration Updates
1. Update connection settings:
```diff
- MQTT_HOST=
- MQTT_PORT=
+ MQTT_BROKER_URL=mqtt://localhost
+ MQTT_PORT=8883
```

2. Update TLS configuration:
```diff
- MQTT_CA_FILE=
+ MQTT_TLS_CA_CERT=
- MQTT_CERT_FILE=
+ MQTT_TLS_CLIENT_CERT=
- MQTT_KEY_FILE=
+ MQTT_TLS_CLIENT_KEY=
```

#### Phoenix Backup System
1. Enable backup system:
```env
PHOENIX_ENABLED=true
PHOENIX_BACKUP_DIR=data/phoenix
```

2. Configure encryption:
```env
# Generate new key if not migrating existing backups
PHOENIX_ENCRYPTION_KEY=<generated-key>
```

### Post-Migration Steps

1. Validate new configuration:
```bash
./scripts/validate-config.sh
```

2. Test core functionality:
```bash
# Health check
curl http://localhost:3000/health

# Test API authentication
curl -H "Authorization: Bearer $API_KEY" http://localhost:3000/api/v1/test
```

3. Verify MQTT connectivity:
```bash
# Using mosquitto client
mosquitto_sub -h localhost -p 8883 -t "jamey/#" \
  --cafile ./certs/ca.crt \
  --cert ./certs/client.crt \
  --key ./certs/client.key
```

## Breaking Changes

### Version 3.0
1. OpenAI to OpenRouter migration
   - All OpenAI configuration keys renamed
   - New API authentication flow
   - Different model names

2. Database Configuration
   - New SQLite-specific connection string
   - Structured data directory

3. Security Updates
   - Stricter API key requirements
   - New CORS configuration format
   - Enhanced rate limiting

4. New Features
   - Soul System configuration
   - Phoenix backup system
   - Enhanced MQTT security

## Configuration Validation

### Automated Validation
```bash
# Run full validation suite
cargo test --package jamey -- config::tests

# Validate specific sections
cargo test --package jamey -- config::tests::test_core_config
cargo test --package jamey -- config::tests::test_security_config
```

### Manual Validation Steps
1. Core functionality:
```bash
# Test API connectivity
curl http://localhost:3000/health

# Test authentication
curl -H "Authorization: Bearer $API_KEY" http://localhost:3000/api/v1/status
```

2. MQTT connectivity:
```bash
# Test MQTT connection
./scripts/test-mqtt-connection.sh
```

3. Backup system:
```bash
# Test backup creation
./scripts/test-backup.sh
```

## Rollback Procedures

### Quick Rollback
1. Restore previous configuration:
```bash
cp .env.backup .env
```

2. Restart services:
```bash
systemctl restart jamey
```

### Clean Rollback
1. Stop services:
```bash
systemctl stop jamey
```

2. Restore configuration:
```bash
cp .env.backup .env
```

3. Restore secrets:
```bash
cat secrets.backup >> .env
```

4. Verify restoration:
```bash
diff .env.backup .env
```

5. Restart services:
```bash
systemctl start jamey
```

### Emergency Rollback
If system is unresponsive:
```bash
# Stop all services
systemctl stop jamey

# Restore known good configuration
cp .env.backup .env

# Clear any corrupted state
rm -rf /tmp/jamey-*

# Start services in safe mode
SAFE_MODE=1 systemctl start jamey
```

## Support

For migration assistance:
1. Check logs: `journalctl -u jamey`
2. Review validation errors
3. Consult documentation
4. Open GitHub issue with:
   - Previous configuration (sanitized)
   - Error messages
   - Migration steps attempted