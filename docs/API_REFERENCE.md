# Jamey 3.0 API Reference

Complete API documentation for frontend developers integrating with Jamey 3.0 backend.

## Base URL

- **Development**: `http://localhost:3000`
- **Production**: Configure via `VITE_API_URL` or your backend URL

## Authentication

### Header Format

Include API key in one of these formats:

```
x-api-key: jamey_your-api-key-here
```

or

```
Authorization: Bearer jamey_your-api-key-here
```

### Authentication Status

- **Optional**: If backend doesn't require authentication, requests work without API key
- **Required**: If backend requires authentication, all requests must include valid API key

### Error Responses

**401 Unauthorized**:
```json
{
  "error": "Invalid or missing API key"
}
```

---

## Endpoints

### Health Check

#### GET /

Basic health check endpoint.

**Authentication**: Not required

**Response**:
```json
{
  "status": "ok",
  "timestamp": "2024-01-15T10:30:00Z"
}
```

**Example**:
```bash
curl http://localhost:3000/
```

---

#### GET /health

Detailed health check with dependency verification.

**Authentication**: Not required

**Response**:
```json
{
  "status": "ok",
  "timestamp": "2024-01-15T10:30:00Z",
  "database": "connected",
  "memory": "operational",
  "mqtt": "connected"
}
```

**Status Values**:
- `"ok"`: All systems operational
- `"degraded"`: Some systems unavailable but core functions work
- `"error"`: Critical systems unavailable

**Example**:
```bash
curl http://localhost:3000/health
```

---

### Action Evaluation

#### POST /evaluate

Evaluate an action's morality score using the Conscience Engine.

**Authentication**: Optional (depends on backend configuration)

**Request Body**:
```json
{
  "action": "I will help others in need",
  "entity_id": "optional-entity-name"
}
```

**Request Schema**:
- `action` (string, required): The action text to evaluate
- `entity_id` (string, optional): Entity name for soul system integration

**Response**:
```json
{
  "score": 8.5,
  "action": "I will help others in need",
  "emotion": "joy"
}
```

**Response Schema**:
- `score` (number): Moral score (0.0 to 10.0+)
- `action` (string): The evaluated action
- `emotion` (string, optional): Detected emotion (joy, sadness, anger, neutral, love)

**Status Codes**:
- `200 OK`: Evaluation successful
- `400 Bad Request`: Invalid request body
- `401 Unauthorized`: Authentication required
- `429 Too Many Requests`: Rate limit exceeded
- `500 Internal Server Error`: Server error

**Example**:
```bash
curl -X POST http://localhost:3000/evaluate \
  -H "Content-Type: application/json" \
  -H "x-api-key: jamey_your-key" \
  -d '{"action": "I will help others"}'
```

**JavaScript Example**:
```javascript
const response = await fetch('http://localhost:3000/evaluate', {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
    'x-api-key': 'jamey_your-key',
  },
  body: JSON.stringify({
    action: 'I will help others in need',
    entity_id: 'user-123', // Optional
  }),
});

const data = await response.json();
console.log('Moral Score:', data.score);
console.log('Emotion:', data.emotion);
```

**Rate Limit**: Per API key (default: 60 requests/minute)

---

### Moral Rules

#### GET /rules

Get all moral rules from the Conscience Engine.

**Authentication**: Optional

**Response**:
```json
[
  {
    "name": "no-harm",
    "description": "Do not cause physical or emotional harm",
    "weight": 10.0
  },
  {
    "name": "truth",
    "description": "Be honest and truthful",
    "weight": 8.0
  }
]
```

**Response Schema**:
Array of rule objects:
- `name` (string): Rule identifier
- `description` (string): Rule description
- `weight` (number): Rule weight (affects scoring)

**Status Codes**:
- `200 OK`: Success
- `401 Unauthorized`: Authentication required
- `500 Internal Server Error`: Server error

**Example**:
```bash
curl http://localhost:3000/rules \
  -H "x-api-key: jamey_your-key"
```

**JavaScript Example**:
```javascript
const response = await fetch('http://localhost:3000/rules', {
  headers: {
    'x-api-key': 'jamey_your-key',
  },
});

const rules = await response.json();
rules.forEach(rule => {
  console.log(`${rule.name}: ${rule.description} (weight: ${rule.weight})`);
});
```

---

#### POST /rules

Add a new moral rule to the Conscience Engine.

**Authentication**: Optional (but recommended for write operations)

**Request Body**:
```json
{
  "name": "kindness",
  "description": "Be kind and compassionate to others",
  "weight": 7.5
}
```

**Request Schema**:
- `name` (string, required): Unique rule identifier
- `description` (string, required): Rule description (used for keyword matching)
- `weight` (number, required): Rule weight (0.0 to 100.0)

**Response**: `201 Created` (no body)

**Status Codes**:
- `201 Created`: Rule added successfully
- `400 Bad Request`: Invalid request body
- `401 Unauthorized`: Authentication required
- `429 Too Many Requests`: Rate limit exceeded
- `500 Internal Server Error`: Server error

**Example**:
```bash
curl -X POST http://localhost:3000/rules \
  -H "Content-Type: application/json" \
  -H "x-api-key: jamey_your-key" \
  -d '{
    "name": "kindness",
    "description": "Be kind and compassionate",
    "weight": 7.5
  }'
```

**JavaScript Example**:
```javascript
const response = await fetch('http://localhost:3000/rules', {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
    'x-api-key': 'jamey_your-key',
  },
  body: JSON.stringify({
    name: 'kindness',
    description: 'Be kind and compassionate to others',
    weight: 7.5,
  }),
});

if (response.status === 201) {
  console.log('Rule added successfully');
}
```

---

### Metrics

#### GET /metrics

Prometheus metrics endpoint (for monitoring).

**Authentication**: Optional (but typically restricted to internal networks)

**Response**: Prometheus format metrics

**Example**:
```bash
curl http://localhost:3000/metrics
```

**Note**: This endpoint is typically restricted in production. Use for monitoring and alerting.

---

## Rate Limiting

### Per-API-Key Limits

Each API key has a configurable rate limit:
- **Default**: 60 requests per minute
- **Configurable**: Set when creating API key
- **Per-key**: Different keys can have different limits

### Rate Limit Headers

When rate limiting is active, responses include:

- `X-RateLimit-Limit`: Maximum requests per minute
- `X-RateLimit-Remaining`: Remaining requests in current window
- `Retry-After`: Seconds to wait before retrying (on 429)

### Rate Limit Exceeded

**Status**: `429 Too Many Requests`

**Response**:
```json
{
  "error": "Rate limit exceeded"
}
```

**Handling**:
```javascript
if (response.status === 429) {
  const retryAfter = response.headers.get('Retry-After') || 60;
  await new Promise(resolve => setTimeout(resolve, retryAfter * 1000));
  // Retry request
}
```

---

## Error Handling

### Standard Error Format

```json
{
  "error": "Error message description",
  "code": "ERROR_CODE"
}
```

### HTTP Status Codes

| Code | Meaning | Description |
|------|---------|-------------|
| 200 | OK | Request successful |
| 201 | Created | Resource created successfully |
| 400 | Bad Request | Invalid request format or parameters |
| 401 | Unauthorized | Invalid or missing API key |
| 429 | Too Many Requests | Rate limit exceeded |
| 500 | Internal Server Error | Server error |

### Error Handling Example

```javascript
async function makeRequest(url, options) {
  try {
    const response = await fetch(url, options);
    
    if (!response.ok) {
      const error = await response.json().catch(() => ({ error: 'Unknown error' }));
      
      switch (response.status) {
        case 400:
          throw new Error(`Bad Request: ${error.error}`);
        case 401:
          throw new Error('Authentication failed. Check your API key.');
        case 429:
          const retryAfter = response.headers.get('Retry-After');
          throw new Error(`Rate limit exceeded. Retry after ${retryAfter} seconds.`);
        case 500:
          throw new Error('Server error. Please try again later.');
        default:
          throw new Error(`HTTP ${response.status}: ${error.error}`);
      }
    }
    
    return await response.json();
  } catch (error) {
    if (error instanceof TypeError) {
      // Network error
      throw new Error('Network error. Check your connection.');
    }
    throw error;
  }
}
```

---

## CORS Configuration

### Development

Backend allows all origins by default in development.

### Production

Configure allowed origins in backend `.env`:

```bash
ALLOWED_ORIGINS=https://your-frontend.com,https://www.your-frontend.com
```

### CORS Headers

Backend sends:
- `Access-Control-Allow-Origin`: Allowed origin(s)
- `Access-Control-Allow-Methods`: GET, POST, OPTIONS
- `Access-Control-Allow-Headers`: Content-Type, x-api-key, Authorization

---

## Request/Response Examples

### Complete Example: Evaluate Action

**Request**:
```http
POST /evaluate HTTP/1.1
Host: localhost:3000
Content-Type: application/json
x-api-key: jamey_abc123-def456-ghi789

{
  "action": "I will help a friend in need",
  "entity_id": "friend-alice"
}
```

**Response**:
```http
HTTP/1.1 200 OK
Content-Type: application/json
X-RateLimit-Limit: 60
X-RateLimit-Remaining: 59

{
  "score": 8.5,
  "action": "I will help a friend in need",
  "emotion": "joy"
}
```

### Complete Example: Get Rules

**Request**:
```http
GET /rules HTTP/1.1
Host: localhost:3000
x-api-key: jamey_abc123-def456-ghi789
```

**Response**:
```http
HTTP/1.1 200 OK
Content-Type: application/json

[
  {
    "name": "no-harm",
    "description": "Do not cause physical or emotional harm",
    "weight": 10.0
  },
  {
    "name": "truth",
    "description": "Be honest and truthful",
    "weight": 8.0
  }
]
```

---

## SDK Examples

### React Hook

```typescript
import { useQuery, useMutation } from '@tanstack/react-query';
import { apiClient } from './api-client';

export const useEvaluateAction = () => {
  return useMutation({
    mutationFn: async (action: string) => {
      const response = await apiClient.post('/evaluate', { action });
      return response.data;
    },
  });
};
```

### Vue Composable

```typescript
import { ref } from 'vue';
import { apiClient } from './api-client';

export const useEvaluateAction = () => {
  const loading = ref(false);
  const error = ref(null);

  const evaluate = async (action: string) => {
    loading.value = true;
    error.value = null;
    try {
      const response = await apiClient.post('/evaluate', { action });
      return response.data;
    } catch (e) {
      error.value = e;
      throw e;
    } finally {
      loading.value = false;
    }
  };

  return { evaluate, loading, error };
};
```

### Vanilla JavaScript

```javascript
class JameyClient {
  constructor(baseURL, apiKey) {
    this.baseURL = baseURL;
    this.apiKey = apiKey;
  }

  async evaluate(action) {
    const response = await fetch(`${this.baseURL}/evaluate`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'x-api-key': this.apiKey,
      },
      body: JSON.stringify({ action }),
    });
    
    if (!response.ok) {
      throw new Error(`HTTP ${response.status}`);
    }
    
    return await response.json();
  }

  async getRules() {
    const response = await fetch(`${this.baseURL}/rules`, {
      headers: {
        'x-api-key': this.apiKey,
      },
    });
    
    if (!response.ok) {
      throw new Error(`HTTP ${response.status}`);
    }
    
    return await response.json();
  }
}

// Usage
const client = new JameyClient('http://localhost:3000', 'jamey_your-key');
const result = await client.evaluate('I will help others');
console.log('Score:', result.score);
```

---

## Best Practices

1. **Store API Keys Securely**: Never commit API keys to version control
2. **Use Environment Variables**: Store keys in `.env` files (not committed)
3. **Handle Errors Gracefully**: Implement proper error handling and user feedback
4. **Respect Rate Limits**: Implement retry logic with exponential backoff
5. **Monitor Usage**: Track API key usage and adjust rate limits as needed
6. **Use HTTPS in Production**: Always use HTTPS for API communication in production

---

## Support

- **Documentation**: See [Frontend Integration Guide](FRONTEND_INTEGRATION.md)
- **Issues**: Report issues on GitHub
- **Questions**: Check [Troubleshooting Guide](TROUBLESHOOTING.md)

