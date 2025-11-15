# Jamey 3.0 Quick Start Guide

This guide helps you get Jamey 3.0 up and running quickly. For detailed information, refer to the [full documentation](CONFIGURATION_README.md).

## 5-Minute Setup

### 1. Clone & Install
```bash
# Clone repository
git clone https://github.com/your-username/jamey-3.0.git
cd jamey-3.0

# Install dependencies
cargo build
```

### 2. Basic Configuration
```bash
# Copy environment template
cp .env.example .env

# Generate API key
openssl rand -hex 32 > api.key
API_KEY=$(cat api.key)

# Update .env with minimal configuration
cat << EOF > .env
# Core Configuration
OPENROUTER_API_KEY=your-api-key-here
DATABASE_URL=sqlite:data/jamey.db
DATA_DIR=./data

# Security Configuration
API_KEY=$API_KEY
ALLOWED_ORIGINS=http://localhost:5173,http://localhost:3000

# Operational Settings
PORT=3000
HOST=0.0.0.0
RUST_LOG=info
DEV_MODE=true
EOF
```

### 3. Start the Service
```bash
# Run the service
cargo run
```

### 4. Verify Installation
```bash
# Check health endpoint
curl http://localhost:3000/health

# Test authenticated endpoint
curl -H "Authorization: Bearer $API_KEY" \
    http://localhost:3000/api/v1/status
```

## Common Configurations

### Development Environment
```env
# Development settings
DEV_MODE=true
ENABLE_TEST_FEATURES=true
RUST_LOG=debug
```

### Production Environment
```env
# Production settings
DEV_MODE=false
ENABLE_TEST_FEATURES=false
RUST_LOG=info
ALLOWED_ORIGINS=https://your-domain.com
```

### Enable MQTT (Optional)
```env
# MQTT Configuration
MQTT_BROKER_URL=mqtt://localhost
MQTT_PORT=8883
MQTT_JWT_SECRET=$(openssl rand -hex 32)
```

### Enable Backups (Optional)
```env
# Phoenix Backup System
PHOENIX_ENABLED=true
PHOENIX_BACKUP_DIR=data/phoenix
PHOENIX_ENCRYPTION_KEY=$(openssl rand -hex 32)
```

## Basic Operations

### 1. Health Check
```bash
# Check service health
curl http://localhost:3000/health
```

### 2. Test API
```bash
# Test with authentication
curl -H "Authorization: Bearer $API_KEY" \
    http://localhost:3000/api/v1/test
```

### 3. Monitor Logs
```bash
# View logs
tail -f /var/log/jamey/service.log
```

### 4. Manage Service
```bash
# Restart service
systemctl restart jamey

# Check status
systemctl status jamey
```

## Quick Troubleshooting

### 1. Connection Issues
```bash
# Check if service is running
systemctl status jamey

# Verify port is open
netstat -tulpn | grep 3000
```

### 2. Authentication Problems
```bash
# Verify API key format
echo $API_KEY | wc -c    # Should be 64

# Test authentication
curl -v -H "Authorization: Bearer $API_KEY" \
    http://localhost:3000/api/v1/status
```

### 3. Database Issues
```bash
# Check database file
ls -l data/jamey.db

# Verify permissions
sudo -u jamey-service test -w data/jamey.db && echo "Writable" || echo "Not writable"
```

## Next Steps

1. Review [Security Best Practices](SECURITY_BEST_PRACTICES.md)
2. Configure [Advanced Features](CONFIGURATION.md)
3. Set up [Monitoring](TROUBLESHOOTING.md#monitoring-and-auditing)

## Need Help?

1. Check [Troubleshooting Guide](TROUBLESHOOTING.md)
2. Review [Configuration Guide](CONFIGURATION.md)
3. Search existing issues on GitHub
4. Join our community chat

## Security Note

This quick start guide prioritizes getting started quickly. For production deployments:
1. Review [Security Best Practices](SECURITY_BEST_PRACTICES.md)
2. Configure proper authentication
3. Set up secure networking
4. Enable encrypted backups