# Troubleshooting Guide

This guide provides solutions for common issues you might encounter while running Jamey 3.0, organized by system component.

## Table of Contents
1. [General System Issues](#general-system-issues)
2. [Configuration Problems](#configuration-problems)
3. [Memory System Issues](#memory-system-issues)
4. [Cache System Problems](#cache-system-problems)
5. [Communication Issues](#communication-issues)
6. [Monitoring Alerts](#monitoring-alerts)
7. [Performance Problems](#performance-problems)

## General System Issues

### System Won't Start

1. **Check Configuration**
```bash
# Verify environment file exists
ls -l .env

# Validate required variables
grep -E "OPENROUTER_API_KEY|API_KEY" .env
```

2. **Database Issues**
```bash
# Check database file permissions
ls -l data/jamey.db

# Verify SQLite integrity
sqlite3 data/jamey.db "PRAGMA integrity_check;"
```

3. **Port Conflicts**
```bash
# Check if port is already in use
netstat -tuln | grep 3000
```

### Common Error Messages

| Error | Possible Cause | Solution |
|-------|---------------|----------|
| "Failed to initialize metrics" | Port 9090 in use | Change metrics port or stop conflicting service |
| "Database connection failed" | SQLite file permissions | Check file and directory permissions |
| "API key not configured" | Missing environment variable | Set OPENROUTER_API_KEY in .env |

## Configuration Problems

### Environment Variables

1. **Missing Variables**
```bash
# Check for required variables
source .env
echo $OPENROUTER_API_KEY
echo $API_KEY
```

2. **Invalid Values**
```bash
# Validate key lengths
[ ${#API_KEY} -ge 32 ] && echo "API key OK" || echo "API key too short"
```

### File Permissions

```bash
# Fix data directory permissions
chmod 755 data/
chmod 644 data/jamey.db

# Fix TLS certificate permissions
chmod 644 certs/*.crt
chmod 600 certs/*.key
```

## Memory System Issues

### Storage Problems

1. **Index Corruption**
```bash
# Check index size and permissions
ls -lh data/*/index

# Rebuild index if necessary
rm -rf data/*/index/*
systemctl restart jamey
```

2. **Out of Space**
```bash
# Check disk usage
df -h data/

# Clean old indices
find data/ -name "*.idx" -mtime +30 -delete
```

### Common Memory Errors

| Error | Cause | Solution |
|-------|-------|----------|
| "Index not found" | Missing or corrupt index | Rebuild affected index |
| "Memory limit exceeded" | Too many records | Increase limits or prune old data |
| "Failed to store record" | Disk space or permissions | Check space and permissions |

## Cache System Problems

### Cache Performance Issues

1. **High Miss Rate**
```rust
// Check cache configuration
let config = CacheConfig {
    max_capacity: 20_000,  // Increase if miss rate is high
    time_to_live: Duration::from_secs(7200),
    time_to_idle: Duration::from_secs(3600),
};
```

2. **Memory Usage**
```bash
# Monitor cache memory usage
watch -n1 'ps -o pid,rss,command -p $(pgrep jamey)'
```

### Cache Errors

| Error | Cause | Solution |
|-------|-------|----------|
| "Cache overflow" | Exceeded capacity | Increase max_capacity or reduce TTL |
| "Invalid cache key" | Malformed key | Check key generation logic |
| "Cache write failed" | Memory pressure | Monitor system resources |

## Communication Issues

### MQTT Problems

1. **Connection Failures**
```bash
# Check MQTT broker status
systemctl status mosquitto

# Verify TLS certificates
openssl verify -CAfile certs/ta-ca.crt certs/client.crt
```

2. **Authentication Issues**
```bash
# Check JWT token generation
curl -X POST http://localhost:3000/api/mqtt/token
```

### Network Issues

```bash
# Check network connectivity
ping openrouter.ai

# Verify DNS resolution
nslookup api.openrouter.ai

# Test TLS connection
openssl s_client -connect api.openrouter.ai:443
```

## Monitoring Alerts

### Critical Alerts

1. **ConsciousnessPhiCritical**
```bash
# Check recent consciousness metrics
curl -s localhost:9090/api/v1/query?query=consciousness_phi_value

# Review consciousness logs
journalctl -u jamey -n 100 | grep consciousness
```

2. **SystemUnavailable**
```bash
# Check system status
systemctl status jamey

# View recent errors
journalctl -u jamey -p err
```

### Performance Alerts

1. **HighMemoryUsage**
```bash
# Check memory usage
free -h

# Find memory-intensive operations
ps aux --sort=-%mem | head
```

2. **HighCPUUsage**
```bash
# Monitor CPU usage
top -b -n 1

# Check for resource-intensive processes
pidstat -u 1 10
```

## Performance Problems

### High Latency

1. **Identify Bottlenecks**
```bash
# Monitor request latencies
curl -s localhost:9090/api/v1/query?query=http_request_duration_seconds

# Check database performance
sqlite3 data/jamey.db "PRAGMA wal_checkpoint;"
```

2. **Resource Constraints**
```bash
# Monitor system resources
vmstat 1 10

# Check I/O wait
iostat -x 1 10
```

### Memory Leaks

```bash
# Monitor memory growth
watch -n10 'ps -o pid,rss,vsz $(pgrep jamey)'

# Check for file descriptor leaks
lsof -p $(pgrep jamey) | wc -l
```

## Best Practices

1. **Regular Maintenance**
   - Monitor log files
   - Check disk usage
   - Verify backup integrity
   - Review performance metrics

2. **Preventive Measures**
   - Set up monitoring alerts
   - Configure log rotation
   - Implement resource limits
   - Regular security updates

3. **Debugging Steps**
   - Check logs first
   - Verify configuration
   - Test connectivity
   - Monitor resources
   - Review metrics

4. **Recovery Procedures**
   - Backup before changes
   - Document all actions
   - Test after fixes
   - Monitor for recurrence

## Getting Help

1. **Gathering Information**
   - System logs
   - Error messages
   - Configuration files
   - Metrics data
   - Recent changes

2. **Reporting Issues**
   - Include error details
   - Provide configuration
   - Share relevant logs
   - Describe steps to reproduce
   - List attempted solutions