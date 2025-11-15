# Environment Variables Configuration

This document describes all environment variables used by Jamey 3.0.

Create a `.env` file in the project root with these variables (copy from this example):

```bash
# ============================================
# OpenRouter API Configuration (Required for LLM features)
# ============================================
# Get your API key from: https://openrouter.ai/keys
OPENROUTER_API_KEY=your-openrouter-api-key-here
OPENROUTER_MODEL=deepseek/deepseek-chat
OPENROUTER_API_URL=https://openrouter.ai/api/v1

# ============================================
# Security Configuration
# ============================================
# API key for authentication (optional - if not set, no auth required)
# Set this in production to require API key authentication
# Clients should send this in the 'x-api-key' header or 'Authorization: Bearer <key>' header
API_KEY=

# Allowed CORS origins (comma-separated)
# For production, set to your frontend domain(s)
# Example: ALLOWED_ORIGINS=https://app.example.com,https://www.example.com
# Default: http://localhost:5173,http://localhost:3000
ALLOWED_ORIGINS=http://localhost:5173,http://localhost:3000

# Rate limiting: requests per minute per IP
# Default: 60
RATE_LIMIT_PER_MINUTE=60

# Input validation limits
MAX_ACTION_LENGTH=10000
MAX_RULE_NAME_LENGTH=100
MAX_RULE_DESCRIPTION_LENGTH=500
MIN_RULE_WEIGHT=0.0
MAX_RULE_WEIGHT=100.0

# ============================================
# Database Configuration
# ============================================
# Optional: Custom database path
# Default: data/jamey.db
DATABASE_URL=

# ============================================
# Soul Knowledge Base Configuration
# ============================================
SOUL_DEFAULT_TRUST=0.5
SOUL_BASE_DECAY_RATE=0.01
SOUL_PRUNE_THRESHOLD=0.1
SOUL_EMPATHY_THRESHOLD=0.7
SOUL_AUTO_RECORD=true

# ============================================
# MQTT Configuration (Optional)
# ============================================
# MQTT_BROKER_URL=mqtt://localhost
# MQTT_PORT=8883
# MQTT_TLS_CA_CERT=./certs/ca.crt
# MQTT_TLS_CLIENT_CERT=./certs/client.crt
# MQTT_TLS_CLIENT_KEY=./certs/client.key
# MQTT_JWT_SECRET=your-jwt-secret-minimum-32-characters
# MQTT_CLIENT_ID=jamey-instance-1
# MQTT_PERMISSIONS=jamey/#

# ============================================
# Logging Configuration
# ============================================
# Set log level: trace, debug, info, warn, error
# Example: RUST_LOG=jamey_3=info,axum=info
RUST_LOG=info
```

## Security Notes

### Production Deployment

1. **API_KEY**: **REQUIRED** in production. Generate a strong random key:
   ```bash
   openssl rand -hex 32
   ```

2. **ALLOWED_ORIGINS**: **REQUIRED** in production. Set to your actual frontend domain(s).

3. **Rate Limiting**: Adjust `RATE_LIMIT_PER_MINUTE` based on your expected traffic.

4. **Never commit `.env` files** to version control. The `.env` file is in `.gitignore`.

## Development

For local development, you can use the defaults. The API will work without authentication if `API_KEY` is not set.

