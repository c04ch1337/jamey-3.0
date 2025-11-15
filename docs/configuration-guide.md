# Jamey 3.0 Configuration Guide

## Overview

This guide explains how to configure Jamey 3.0 for both development and production environments, with special attention to security and sensitive data handling.

## Quick Start

1. Copy the template:
```bash
cp .env.example .env
```

2. Generate required secrets:
```bash
# API key for production
openssl rand -hex 32 > api.key

# MQTT JWT secret
openssl rand -hex 32 > mqtt.key

# Phoenix encryption key
openssl rand -hex 32 > phoenix.key
```

3. Update `.env` with generated secrets and required settings.

## Environment-Specific Configuration

### Development Environment

```env
# Core - Minimal setup for development
OPENROUTER_API_KEY=your-test-key
DATABASE_URL=sqlite:data/jamey.db

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

### Production Environment

```env
# Core - Required production settings
OPENROUTER_API_KEY=your-production-key
DATABASE_URL=sqlite:/var/lib/jamey/data.db

# Security - Strict requirements
API_KEY=<generated-api-key>       # Required in production
ALLOWED_ORIGINS=https://your-domain.com
RATE_LIMIT_RPS=10
RATE_LIMIT_BURST=20

# Operational
DEV_MODE=false
ENABLE_TEST_FEATURES=false
RUST_LOG=info
```

## Sensitive Value Handling

### Security Best Practices

1. **Secret Generation**
   - Use cryptographically secure methods for generating secrets
   - Minimum length requirements enforced for all secrets
   - Rotate secrets periodically in production

2. **Storage**
   - Never commit `.env` file to version control
   - Store production secrets in a secure vault/manager
   - Use environment-specific .env files (.env.dev, .env.prod)

3. **Access Control**
   - Restrict file permissions on .env files
   - Limit access to production secrets
   - Log access attempts to sensitive configurations

### Sensitive Values Checklist

| Configuration Value | Protection Level | Storage Recommendation |
|-------------------|------------------|----------------------|
| OPENROUTER_API_KEY | High | Secure vault |
| API_KEY | High | Secure vault |
| MQTT_JWT_SECRET | High | Secure vault |
| PHOENIX_ENCRYPTION_KEY | High | Secure vault |
| DATABASE_URL | Medium | Environment variable |
| ALLOWED_ORIGINS | Low | Configuration file |

## Validation and Security

### Required Security Measures

1. **Production Environment**
   ```rust
   // Example validation code
   if !dev_mode && api_key.is_none() {
       return Err("API_KEY is required in production");
   }
   ```

2. **TLS Configuration**
   - Verify certificate paths and permissions
   - Validate certificate expiration dates
   - Ensure secure cipher configurations

3. **Rate Limiting**
   - Adjust limits based on environment
   - Monitor and alert on threshold breaches
   - Implement graduated rate limiting

## Configuration Loading

### Secure Configuration Loading

```rust
impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        // 1. Load from environment
        dotenv().ok();
        
        // 2. Validate environment
        Self::validate_environment()?;
        
        // 3. Load sensitive values securely
        let sensitive_values = Self::load_sensitive_values()?;
        
        // 4. Create and validate configuration
        let config = Self::create_config(sensitive_values)?;
        config.validate()?;
        
        Ok(config)
    }
}
```

### Error Handling

```rust
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Missing required configuration: {0}")]
    MissingConfig(String),
    
    #[error("Invalid configuration value: {0}")]
    InvalidValue(String),
    
    #[error("Security requirement not met: {0}")]
    SecurityError(String),
}
```

## Monitoring and Maintenance

1. **Configuration Auditing**
   - Log configuration changes
   - Track sensitive value access
   - Regular security reviews

2. **Health Checks**
   - Validate configuration integrity
   - Monitor certificate expiration
   - Check backup encryption status

3. **Maintenance Tasks**
   - Rotate secrets periodically
   - Update allowed origins
   - Review rate limits

## Troubleshooting

Common configuration issues and their solutions:

1. **Missing Required Values**
   - Check environment-specific requirements
   - Verify .env file location
   - Confirm secret generation

2. **Invalid Configurations**
   - Validate format of sensitive values
   - Check file permissions
   - Verify environment variables

3. **Security Violations**
   - Review security requirements
   - Check certificate configurations
   - Verify rate limiting settings