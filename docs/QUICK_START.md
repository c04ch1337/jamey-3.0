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
cp ENV_EXAMPLE.md .env

# Edit .env and add your OpenRouter API key
# OPENROUTER_API_KEY=your-api-key-here
```

### 3. Start the Service
```bash
# Run the service
cargo run
```

The backend will:
- Start on `http://localhost:3000`
- Initialize database automatically
- Set up API endpoints
- Connect to MQTT (if configured)

**Expected output:**
```
Starting Jamey 3.0 - General & Guardian
Server listening on http://0.0.0.0:3000
```

### 4. Verify Installation
```bash
# Check health endpoint
curl http://localhost:3000/health
```

Should return:
```json
{
  "status": "ok",
  "service": "Jamey 3.0",
  "version": "3.0.0"
}
```

## Using the CLI

### Interactive Chat
```bash
# Start CLI chat interface
cargo run --bin jamey-cli chat
```

### Connect to Running Backend
```bash
# Connect to local backend
cargo run --bin jamey-cli connect

# Connect to remote backend
cargo run --bin jamey-cli connect --url http://remote-server:3000
```

See [CLI Usage Guide](CLI_USAGE.md) for complete documentation.

## Common Configurations

### Development Environment
```env
# Development settings
RUST_LOG=debug
SERVER_PORT=3000
```

### Production Environment
```env
# Production settings
RUST_LOG=info
SERVER_PORT=3000
```

### Enable MQTT (Optional)
```env
# MQTT Configuration
MQTT_BROKER_URL=mqtt://localhost
MQTT_PORT=8883
```

### Enable Backups (Optional)
```env
# Phoenix Backup System
BACKUP_ENABLED=true
BACKUP_DIR=data/backups
```

## Basic Operations

### 1. Health Check
```bash
# Check service health
curl http://localhost:3000/health
```

### 2. Test API
```bash
# Evaluate an action
curl -X POST http://localhost:3000/evaluate \
  -H "Content-Type: application/json" \
  -d '{"action": "help someone in need"}'
```

### 3. View Rules
```bash
# Get all moral rules
curl http://localhost:3000/rules
```

## Quick Troubleshooting

### Backend Won't Start

**Port 3000 already in use:**
```bash
# Linux/Mac
lsof -ti:3000 | xargs kill -9

# Windows PowerShell
Get-NetTCPConnection -LocalPort 3000 | ForEach-Object { Stop-Process -Id $_.OwningProcess -Force }
```

**Database errors:**
```bash
# Delete and recreate database
rm -rf data/jamey.db
cargo run  # Will recreate database automatically
```

### Connection Issues
```bash
# Check if service is running
curl http://localhost:3000/health

# Verify port is open
# Linux/Mac: netstat -tulpn | grep 3000
# Windows: Get-NetTCPConnection -LocalPort 3000
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