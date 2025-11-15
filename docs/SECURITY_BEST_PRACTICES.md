# Security Best Practices for Configuration Management

## Table of Contents
1. [Overview](#overview)
2. [Secret Management](#secret-management)
3. [Environment Configuration](#environment-configuration)
4. [Access Control](#access-control)
5. [Network Security](#network-security)
6. [Monitoring and Auditing](#monitoring-and-auditing)
7. [Backup Security](#backup-security)
8. [Compliance Checklist](#compliance-checklist)

## Overview

This guide outlines security best practices for managing Jamey 3.0 configuration, focusing on protecting sensitive data and maintaining system integrity.

## Secret Management

### API Keys and Tokens

1. **Generation**
```bash
# Generate cryptographically secure keys
openssl rand -hex 32 > api.key
openssl rand -hex 32 > mqtt.key
openssl rand -hex 32 > phoenix.key

# Set appropriate permissions
chmod 600 api.key mqtt.key phoenix.key
```

2. **Storage**
- Never commit secrets to version control
- Use environment variables or secure vaults
- Implement secret rotation policies
- Consider using HashiCorp Vault or AWS Secrets Manager

3. **Access Control**
```bash
# Set restrictive file permissions
chmod 600 .env
chown jamey:jamey .env

# Use separate service account
useradd -r -s /bin/false jamey-service
```

4. **Rotation Schedule**
- API keys: Every 90 days
- MQTT secrets: Every 180 days
- Encryption keys: Yearly
- Document rotation procedures

### Sensitive Data Handling

1. **Classification Levels**
| Data Type | Classification | Storage |
|-----------|---------------|---------|
| API Keys | High | Secure Vault |
| Certificates | High | Protected FS |
| Configs | Medium | Protected FS |
| Logs | Low | Standard FS |

2. **Protection Methods**
```bash
# Encrypt sensitive files
gpg --encrypt --recipient admin@example.com api.key

# Secure backup storage
PHOENIX_BACKUP_DIR=/secure/encrypted/backups
```

## Environment Configuration

### Production Hardening

1. **Minimal Configuration**
```env
# Production-ready configuration
DEV_MODE=false
ENABLE_TEST_FEATURES=false
RUST_LOG=info
```

2. **Rate Limiting**
```env
# Strict rate limits
RATE_LIMIT_RPS=10
RATE_LIMIT_BURST=20
```

3. **CORS Policy**
```env
# Strict CORS configuration
ALLOWED_ORIGINS=https://example.com
```

### Security Headers

1. **Implementation**
```rust
// Example security headers
app.middleware(|response| {
    response
        .header("Strict-Transport-Security", "max-age=31536000; includeSubDomains")
        .header("X-Content-Type-Options", "nosniff")
        .header("X-Frame-Options", "DENY")
        .header("Content-Security-Policy", "default-src 'self'")
});
```

## Access Control

### File System Security

1. **Directory Permissions**
```bash
# Set secure directory permissions
chmod 750 /etc/jamey/
chmod 640 /etc/jamey/config/*
chmod 600 /etc/jamey/secrets/*

# Set ownership
chown -R jamey:jamey /etc/jamey/
```

2. **Sensitive Files**
```bash
# Protect key files
chmod 400 /etc/jamey/secrets/encryption.key
chmod 400 /etc/jamey/certs/*.key
```

### Service Account Configuration

1. **Create Service Account**
```bash
# Create service user
useradd -r -s /bin/false jamey-service

# Set up directory structure
mkdir -p /var/lib/jamey
chown jamey-service:jamey-service /var/lib/jamey
```

2. **Systemd Service**
```ini
[Service]
User=jamey-service
Group=jamey-service
PrivateTmp=true
ProtectSystem=strict
ReadWritePaths=/var/lib/jamey
NoNewPrivileges=true
```

## Network Security

### TLS Configuration

1. **Certificate Management**
```bash
# Generate CSR
openssl req -new -newkey rsa:4096 -nodes \
    -keyout private.key -out request.csr

# Install certificates
mv private.key /etc/jamey/certs/
mv certificate.crt /etc/jamey/certs/
chmod 400 /etc/jamey/certs/private.key
```

2. **MQTT TLS Settings**
```env
MQTT_TLS_CA_CERT=/etc/jamey/certs/ca.crt
MQTT_TLS_CLIENT_CERT=/etc/jamey/certs/client.crt
MQTT_TLS_CLIENT_KEY=/etc/jamey/certs/client.key
```

### Firewall Configuration

```bash
# Allow only necessary ports
ufw allow 3000/tcp  # API
ufw allow 8883/tcp  # MQTT over TLS
ufw deny 1883/tcp   # Unsecure MQTT
```

## Monitoring and Auditing

### Logging Configuration

1. **Log Settings**
```env
# Production logging
RUST_LOG=info

# Log file location
RUST_LOG_FILE=/var/log/jamey/service.log
```

2. **Log Rotation**
```conf
# /etc/logrotate.d/jamey
/var/log/jamey/*.log {
    daily
    rotate 14
    compress
    delaycompress
    notifempty
    create 640 jamey-service jamey-service
}
```

### Audit Trail

1. **Configuration Changes**
```bash
# Track configuration changes
echo "$(date -u) - $USER modified $1" >> /var/log/jamey/config-audit.log
```

2. **Access Monitoring**
```bash
# Monitor sensitive file access
auditctl -w /etc/jamey/secrets/ -p rwxa
```

## Backup Security

### Encrypted Backups

1. **Backup Encryption**
```env
# Enable encrypted backups
PHOENIX_ENABLED=true
PHOENIX_ENCRYPTION_KEY=<secure-key>
```

2. **Secure Storage**
```bash
# Set backup permissions
chmod 700 /var/lib/jamey/backups
chown jamey-service:jamey-service /var/lib/jamey/backups
```

### Key Management

1. **Key Backup**
```bash
# Export encryption keys (secure location)
gpg --export-secret-keys --armor > keys.asc
```

2. **Recovery Procedures**
- Document key recovery steps
- Store recovery codes securely
- Test recovery regularly

## Compliance Checklist

### Daily Tasks
- [ ] Check log files for security events
- [ ] Verify backup completion
- [ ] Monitor system resources

### Weekly Tasks
- [ ] Review access logs
- [ ] Check certificate expiration
- [ ] Verify file permissions

### Monthly Tasks
- [ ] Rotate API keys
- [ ] Update allowed origins
- [ ] Review security policies

### Quarterly Tasks
- [ ] Conduct security audit
- [ ] Update documentation
- [ ] Test recovery procedures

### Annual Tasks
- [ ] Rotate encryption keys
- [ ] Review security architecture
- [ ] Update compliance documentation

## Security Response Plan

### Incident Response

1. **Detection**
- Monitor logs for suspicious activity
- Set up alerts for configuration changes
- Track failed authentication attempts

2. **Response**
- Revoke compromised credentials
- Rotate affected keys
- Document incident details

3. **Recovery**
- Restore from secure backup
- Verify system integrity
- Update security measures

### Emergency Contacts

Maintain a list of emergency contacts:
- Security team lead
- System administrators
- Backup operators
- Compliance officers