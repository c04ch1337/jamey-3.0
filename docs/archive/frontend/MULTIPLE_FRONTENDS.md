# Multiple Frontends Setup Guide

Guide for setting up multiple frontends (local desktop, remote web, mobile apps) with separate API keys and rate limits.

## Overview

Jamey 3.0 backend supports multiple frontends simultaneously, each with:
- Separate API keys
- Individual rate limits
- Independent monitoring
- Different CORS origins

## Use Cases

1. **Local Desktop Frontend**: Full-featured desktop application
2. **Remote Web Frontend**: Public-facing web interface
3. **Mobile App**: React Native or native mobile app
4. **Admin Dashboard**: Internal management interface

## Step 1: Create API Keys for Each Frontend

### Local Desktop Frontend

```bash
# Higher rate limit for full-featured desktop app
./scripts/create-api-key.sh local-desktop 300
```

**Output**: `jamey_abc123-def456-...`

**Configuration**:
- Rate Limit: 300 requests/minute
- Use Case: Full-featured desktop application
- Origin: `http://localhost:5173` or desktop app origin

### Remote Web Frontend

```bash
# Lower rate limit for public web interface
./scripts/create-api-key.sh remote-web 100
```

**Output**: `jamey_xyz789-uvw456-...`

**Configuration**:
- Rate Limit: 100 requests/minute
- Use Case: Public web interface
- Origin: `https://your-web-app.com`

### Mobile App

```bash
# Moderate rate limit for mobile
./scripts/create-api-key.sh mobile-app 200
```

**Output**: `jamey_mobile123-...`

**Configuration**:
- Rate Limit: 200 requests/minute
- Use Case: Mobile application
- Origin: Mobile app bundle identifier

## Step 2: Configure Backend CORS

Edit backend `.env` file:

```bash
# Allow multiple origins
ALLOWED_ORIGINS=http://localhost:5173,https://your-web-app.com,http://localhost:3000
```

Or allow all origins (development only):

```bash
ALLOWED_ORIGINS=*
```

**Note**: In production, specify exact origins for security.

## Step 3: Configure Each Frontend

### Local Desktop Frontend

**Environment** (`frontend/.env`):
```bash
VITE_API_URL=http://localhost:3000
VITE_API_KEY=jamey_abc123-def456-...
```

**Rate Limit**: 300 req/min

### Remote Web Frontend

**Environment** (separate frontend project):
```bash
VITE_API_URL=https://api.your-backend.com
VITE_API_KEY=jamey_xyz789-uvw456-...
```

**Rate Limit**: 100 req/min

### Mobile App

**Environment** (React Native `.env`):
```bash
API_URL=https://api.your-backend.com
API_KEY=jamey_mobile123-...
```

**Rate Limit**: 200 req/min

## Step 4: Verify Setup

### Test Each Frontend

**Local Desktop**:
```bash
curl -H "x-api-key: jamey_abc123-def456-..." \
  http://localhost:3000/rules
```

**Remote Web**:
```bash
curl -H "x-api-key: jamey_xyz789-uvw456-..." \
  https://api.your-backend.com/rules
```

**Mobile**:
```bash
curl -H "x-api-key: jamey_mobile123-..." \
  https://api.your-backend.com/rules
```

## Monitoring Per Frontend

### Check API Key Usage

Query backend database:

```sql
SELECT 
  name,
  rate_limit_per_minute,
  last_used_at,
  created_at
FROM api_keys
WHERE revoked_at IS NULL
ORDER BY last_used_at DESC;
```

### Monitor Rate Limits

Each API key tracks:
- `last_used_at`: Last request timestamp
- `rate_limit_per_minute`: Configured limit
- Usage can be monitored via Prometheus metrics

### Prometheus Metrics

Monitor per-key usage:
- `http_requests_total{api_key="..."}` - Request count per key
- Rate limit metrics available via `/metrics` endpoint

## Best Practices

### 1. Separate Keys Per Environment

- **Development**: `dev-frontend-key`
- **Staging**: `staging-frontend-key`
- **Production**: `prod-frontend-key`

### 2. Appropriate Rate Limits

- **Desktop Apps**: Higher limits (200-500 req/min)
- **Web Apps**: Moderate limits (100-200 req/min)
- **Mobile Apps**: Moderate limits (150-250 req/min)
- **Public APIs**: Lower limits (50-100 req/min)

### 3. Key Naming Convention

Use descriptive names:
- `local-desktop-v1`
- `remote-web-prod`
- `mobile-ios-app`
- `admin-dashboard`

### 4. Key Rotation

Rotate keys periodically:

```rust
// Revoke old key and create new one
key_manager.rotate_key("old-key", "new-key-name").await?;
```

### 5. Monitor and Adjust

- Monitor usage per key
- Adjust rate limits based on actual usage
- Revoke unused keys

## Example: Three Frontends

### Setup

```bash
# Create keys
./scripts/create-api-key.sh desktop-app 300
./scripts/create-api-key.sh web-app 100
./scripts/create-api-key.sh mobile-app 200
```

### Configuration

**Desktop App** (`desktop-app/.env`):
```bash
API_URL=http://localhost:3000
API_KEY=jamey_desktop-key-here
```

**Web App** (`web-app/.env`):
```bash
VITE_API_URL=https://api.example.com
VITE_API_KEY=jamey_web-key-here
```

**Mobile App** (`mobile-app/.env`):
```bash
API_URL=https://api.example.com
API_KEY=jamey_mobile-key-here
```

### Backend CORS

```bash
ALLOWED_ORIGINS=http://localhost:5173,https://web.example.com,*
```

### Verification

All three frontends can connect simultaneously:
- Desktop: 300 req/min limit
- Web: 100 req/min limit
- Mobile: 200 req/min limit

## Troubleshooting

### Key Not Working

**Check**:
1. Key exists in database: `SELECT * FROM api_keys WHERE name = 'key-name';`
2. Key not revoked: `revoked_at IS NULL`
3. Key not expired: `expires_at IS NULL OR expires_at > datetime('now')`

### CORS Issues

**Check**:
1. Origin in `ALLOWED_ORIGINS` list
2. Backend CORS configuration
3. Browser console for specific CORS error

**Fix**: Add origin to backend `ALLOWED_ORIGINS`:
```bash
ALLOWED_ORIGINS=http://localhost:5173,https://your-app.com
```

### Rate Limit Issues

**Check**:
1. Current rate limit: `SELECT rate_limit_per_minute FROM api_keys WHERE name = 'key-name';`
2. Usage pattern: Monitor `last_used_at`

**Fix**: Update rate limit:
```sql
UPDATE api_keys 
SET rate_limit_per_minute = 500 
WHERE name = 'key-name';
```

## Security Considerations

1. **Never Share Keys**: Each frontend should have its own key
2. **Rotate Regularly**: Change keys periodically
3. **Monitor Usage**: Watch for unusual patterns
4. **Revoke Unused**: Remove keys for deprecated frontends
5. **Use HTTPS**: Always use HTTPS in production

## Next Steps

- See [Frontend Integration Guide](FRONTEND_INTEGRATION.md) for framework-specific examples
- See [API Reference](API_REFERENCE.md) for complete API documentation
- See [Security Best Practices](../SECURITY_BEST_PRACTICES.md) for production security

