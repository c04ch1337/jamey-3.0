# Configuration Troubleshooting Guide

## Table of Contents
1. [Common Issues](#common-issues)
2. [Diagnostic Tools](#diagnostic-tools)
3. [Configuration Validation](#configuration-validation)
4. [Environment-Specific Issues](#environment-specific-issues)
5. [Recovery Procedures](#recovery-procedures)

## Common Issues

### API Authentication Issues

1. **Invalid API Key**
```
Error: Authentication failed: Invalid API key
```

**Solution:**
1. Verify API key length:
```bash
# Should be 64 characters (32 bytes hex)
echo $API_KEY | wc -c
```

2. Check key format:
```bash
# Should be hexadecimal
if [[ $API_KEY =~ ^[0-9a-f]{64}$ ]]; then
    echo "Valid format"
else
    echo "Invalid format"
fi
```

3. Regenerate if needed:
```bash
openssl rand -hex 32 > api.key
```

### Database Connection Errors

1. **SQLite Path Issues**
```
Error: Unable to open database file
```

**Solution:**
1. Check directory permissions:
```bash
ls -la data/
ls -la data/jamey.db
```

2. Verify path in configuration:
```bash
# Should match DATABASE_URL in .env
sqlite3 "$(grep DATABASE_URL .env | cut -d= -f2)" ".tables"
```

3. Create directory if missing:
```bash
mkdir -p data
chmod 750 data
```

### MQTT Connection Problems

1. **TLS Certificate Issues**
```
Error: SSL/TLS handshake failed
```

**Solution:**
1. Verify certificate paths:
```bash
# Check certificate files
ls -l /etc/jamey/certs/
```

2. Validate certificate chain:
```bash
openssl verify -CAfile /etc/jamey/certs/ca.crt \
    /etc/jamey/certs/client.crt
```

3. Check certificate expiration:
```bash
openssl x509 -in /etc/jamey/certs/client.crt -noout -dates
```

### Phoenix Backup Errors

1. **Encryption Key Issues**
```
Error: Invalid encryption key format
```

**Solution:**
1. Verify key format:
```bash
# Should be 64 hex characters
echo $PHOENIX_ENCRYPTION_KEY | wc -c
```

2. Check backup directory permissions:
```bash
ls -la $PHOENIX_BACKUP_DIR
```

3. Test backup creation:
```bash
./scripts/test-backup.sh
```

## Diagnostic Tools

### Configuration Validator

```bash
# Run full validation
./scripts/validate-config.sh

# Check specific section
./scripts/validate-config.sh --section security
```

### Log Analysis

1. **View Recent Errors**
```bash
# Last 50 error messages
grep -i error /var/log/jamey/service.log | tail -n 50

# Configuration-related errors
grep -i "config" /var/log/jamey/service.log | grep -i error
```

2. **Monitor Real-time Logs**
```bash
# Follow log output
tail -f /var/log/jamey/service.log | grep --line-buffered "config"
```

### Network Diagnostics

1. **MQTT Connectivity**
```bash
# Test MQTT connection
mosquitto_sub -h localhost -p 8883 \
    --cafile /etc/jamey/certs/ca.crt \
    --cert /etc/jamey/certs/client.crt \
    --key /etc/jamey/certs/client.key \
    -t "jamey/test" -v
```

2. **API Endpoint Test**
```bash
# Test API health
curl -v http://localhost:3000/health

# Test authenticated endpoint
curl -v -H "Authorization: Bearer $API_KEY" \
    http://localhost:3000/api/v1/status
```

## Configuration Validation

### Environment Variables

1. **Required Variables**
```bash
# Check required variables
./scripts/check-env.sh

# Validate formats
./scripts/validate-env.sh
```

2. **Value Constraints**
```bash
# Check numeric ranges
./scripts/validate-ranges.sh

# Verify string lengths
./scripts/validate-lengths.sh
```

### File System Checks

1. **Directory Structure**
```bash
# Verify required directories
./scripts/check-dirs.sh

# Check permissions
./scripts/verify-permissions.sh
```

2. **File Access**
```bash
# Test file access
sudo -u jamey-service test -r /etc/jamey/config/app.conf && echo "Readable" || echo "Not readable"
```

## Environment-Specific Issues

### Development Environment

1. **CORS Issues**
```
Error: CORS policy violation
```

**Solution:**
```env
# Development CORS settings
ALLOWED_ORIGINS=http://localhost:5173,http://localhost:3000
```

2. **Hot Reload Problems**
```
Error: Unable to watch files
```

**Solution:**
```bash
# Increase inotify limits
echo fs.inotify.max_user_watches=524288 | sudo tee -a /etc/sysctl.conf
sudo sysctl -p
```

### Production Environment

1. **Permission Denied**
```
Error: Permission denied writing to /var/lib/jamey
```

**Solution:**
```bash
# Fix ownership
sudo chown -R jamey-service:jamey-service /var/lib/jamey

# Set correct permissions
sudo chmod 750 /var/lib/jamey
```

2. **Resource Limits**
```
Error: Too many open files
```

**Solution:**
```bash
# Update systemd service limits
sudo systemctl edit jamey.service

# Add:
[Service]
LimitNOFILE=65535
```

## Recovery Procedures

### Configuration Recovery

1. **Backup Current State**
```bash
# Backup configuration
cp .env .env.backup-$(date +%Y%m%d)

# Export current secrets
./scripts/export-secrets.sh
```

2. **Restore from Backup**
```bash
# Restore configuration
cp .env.backup-20251115 .env

# Verify restoration
diff .env.backup-20251115 .env
```

### Emergency Recovery

1. **Safe Mode Start**
```bash
# Start in safe mode
SAFE_MODE=1 systemctl start jamey

# Check status
systemctl status jamey
```

2. **Configuration Reset**
```bash
# Reset to defaults
cp .env.example .env

# Generate new secrets
./scripts/generate-secrets.sh
```

### Verification Steps

1. **System Health**
```bash
# Check service status
systemctl status jamey

# Verify logs
journalctl -u jamey --since "5 minutes ago"
```

2. **Configuration Test**
```bash
# Test configuration
./scripts/test-config.sh

# Verify core functionality
./scripts/verify-core.sh
```

## Support Resources

### Getting Help

1. **Log Collection**
```bash
# Collect logs
./scripts/collect-logs.sh

# Generate support bundle
./scripts/create-support-bundle.sh
```

2. **Issue Reporting**
- Include configuration (sanitized)
- Attach relevant logs
- Describe steps to reproduce
- Include environment details

### Documentation
- Configuration Guide: [CONFIGURATION.md](./CONFIGURATION.md)
- Security Guide: [SECURITY_BEST_PRACTICES.md](./SECURITY_BEST_PRACTICES.md)
- Migration Guide: [CONFIGURATION_MIGRATION.md](./CONFIGURATION_MIGRATION.md)