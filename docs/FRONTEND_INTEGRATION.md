# Frontend Integration Guide - Universal

This guide enables **ANY frontend framework** (React, Vue, Angular, vanilla JavaScript, desktop apps, mobile apps) to connect to the Jamey 3.0 backend API.

## Table of Contents

1. [Getting Started](#getting-started)
2. [Authentication](#authentication)
3. [Framework Examples](#framework-examples)
4. [API Endpoints](#api-endpoints)
5. [Advanced Topics](#advanced-topics)
6. [Multiple Frontends](#multiple-frontends)
7. [Troubleshooting](#troubleshooting)

---

## Getting Started

### Prerequisites

1. **Backend Running**: Jamey 3.0 backend must be running and accessible
   - Default: `http://localhost:3000`
   - Check health: `curl http://localhost:3000/health`

2. **API Key** (Optional): Create an API key if backend requires authentication
   - See [API Key Creation](#api-key-creation) below

### Base Configuration

**Base URL**: `http://localhost:3000` (or your backend URL)

**API Version**: All endpoints are at the root level (no version prefix)

**Content-Type**: `application/json` for all requests

### API Key Creation

**Option 1: Using Script** (Linux/Mac)
```bash
./scripts/create-api-key.sh frontend-name 60
```

**Option 2: Programmatically** (Rust)
```rust
use jamey_3::api::key_manager::ApiKeyManager;
use std::sync::Arc;

let pool = // your database pool
let key_manager = Arc::new(ApiKeyManager::new(Arc::new(pool)));
let key = key_manager.create_key("frontend-name", None, Some(60)).await?;
println!("API Key: {}", key);
```

**Option 3: Direct SQL** (Not Recommended)
```sql
-- First hash your key with SHA-256, then:
INSERT INTO api_keys (key_hash, name, created_at, rate_limit_per_minute)
VALUES ('<sha256-hash>', 'frontend-name', datetime('now'), 60);
```

---

## Authentication

### Header Formats

The backend accepts API keys in two formats:

**Format 1: x-api-key header** (Recommended)
```
x-api-key: jamey_your-api-key-here
```

**Format 2: Authorization Bearer**
```
Authorization: Bearer jamey_your-api-key-here
```

### Optional vs Required

- **Development**: Authentication is optional if backend allows unauthenticated requests
- **Production**: Authentication is typically required

### Error Handling

**401 Unauthorized**: Invalid or missing API key
```javascript
if (response.status === 401) {
  console.error('Authentication failed. Check your API key.');
}
```

**429 Too Many Requests**: Rate limit exceeded
```javascript
if (response.status === 429) {
  console.error('Rate limit exceeded. Slow down requests.');
}
```

---

## Framework Examples

### React (with Hooks + TanStack Query)

**Installation**:
```bash
npm install axios @tanstack/react-query
```

**API Client** (`src/api/client.ts`):
```typescript
import axios from 'axios';

const API_BASE_URL = process.env.REACT_APP_API_URL || 'http://localhost:3000';
const API_KEY = process.env.REACT_APP_API_KEY;

export const apiClient = axios.create({
  baseURL: API_BASE_URL,
  headers: {
    'Content-Type': 'application/json',
    ...(API_KEY && { 'x-api-key': API_KEY }),
  },
});

// Types
export interface EvaluateResponse {
  score: number;
  action: string;
  emotion?: string;
}

export interface MoralRule {
  name: string;
  description: string;
  weight: number;
}

// API functions
export const evaluateAction = async (action: string): Promise<EvaluateResponse> => {
  const response = await apiClient.post<EvaluateResponse>('/evaluate', { action });
  return response.data;
};

export const getRules = async (): Promise<MoralRule[]> => {
  const response = await apiClient.get<MoralRule[]>('/rules');
  return response.data;
};

export const addRule = async (rule: Omit<MoralRule, 'name'> & { name: string }): Promise<void> => {
  await apiClient.post('/rules', rule);
};
```

**Custom Hook** (`src/hooks/useJamey.ts`):
```typescript
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { evaluateAction, getRules, addRule, type MoralRule } from '../api/client';

export const useRules = () => {
  return useQuery({
    queryKey: ['rules'],
    queryFn: getRules,
  });
};

export const useEvaluateAction = () => {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: evaluateAction,
    onSuccess: () => {
      // Optionally invalidate queries or update cache
    },
  });
};

export const useAddRule = () => {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: addRule,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['rules'] });
    },
  });
};
```

**Component Usage**:
```typescript
import { useRules, useEvaluateAction } from './hooks/useJamey';

function MyComponent() {
  const { data: rules, isLoading } = useRules();
  const evaluateMutation = useEvaluateAction();
  
  const handleEvaluate = (action: string) => {
    evaluateMutation.mutate(action);
  };
  
  return (
    <div>
      {isLoading ? 'Loading...' : rules?.map(rule => (
        <div key={rule.name}>{rule.name}</div>
      ))}
    </div>
  );
}
```

### Vue 3 (Composition API + axios)

**Installation**:
```bash
npm install axios
```

**Composable** (`src/composables/useJamey.ts`):
```typescript
import { ref, computed } from 'vue';
import axios from 'axios';

const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:3000';
const API_KEY = import.meta.env.VITE_API_KEY;

const apiClient = axios.create({
  baseURL: API_BASE_URL,
  headers: {
    'Content-Type': 'application/json',
    ...(API_KEY && { 'x-api-key': API_KEY }),
  },
});

export interface EvaluateResponse {
  score: number;
  action: string;
  emotion?: string;
}

export const useJamey = () => {
  const rules = ref([]);
  const loading = ref(false);
  const error = ref(null);

  const fetchRules = async () => {
    loading.value = true;
    error.value = null;
    try {
      const response = await apiClient.get('/rules');
      rules.value = response.data;
    } catch (e) {
      error.value = e;
    } finally {
      loading.value = false;
    }
  };

  const evaluateAction = async (action: string) => {
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

  return {
    rules: computed(() => rules.value),
    loading: computed(() => loading.value),
    error: computed(() => error.value),
    fetchRules,
    evaluateAction,
  };
};
```

**Component Usage**:
```vue
<template>
  <div>
    <button @click="fetchRules">Load Rules</button>
    <div v-if="loading">Loading...</div>
    <div v-for="rule in rules" :key="rule.name">
      {{ rule.name }}: {{ rule.description }}
    </div>
  </div>
</template>

<script setup>
import { useJamey } from '@/composables/useJamey';

const { rules, loading, fetchRules, evaluateAction } = useJamey();

fetchRules();
</script>
```

### Angular (Services + HttpClient)

**Service** (`src/app/services/jamey.service.ts`):
```typescript
import { Injectable } from '@angular/core';
import { HttpClient, HttpHeaders } from '@angular/common/http';
import { Observable } from 'rxjs';
import { environment } from '../environments/environment';

export interface EvaluateResponse {
  score: number;
  action: string;
  emotion?: string;
}

export interface MoralRule {
  name: string;
  description: string;
  weight: number;
}

@Injectable({
  providedIn: 'root'
})
export class JameyService {
  private apiUrl = environment.apiUrl || 'http://localhost:3000';
  private apiKey = environment.apiKey;

  private getHeaders(): HttpHeaders {
    let headers = new HttpHeaders({
      'Content-Type': 'application/json'
    });
    
    if (this.apiKey) {
      headers = headers.set('x-api-key', this.apiKey);
    }
    
    return headers;
  }

  constructor(private http: HttpClient) {}

  evaluateAction(action: string): Observable<EvaluateResponse> {
    return this.http.post<EvaluateResponse>(
      `${this.apiUrl}/evaluate`,
      { action },
      { headers: this.getHeaders() }
    );
  }

  getRules(): Observable<MoralRule[]> {
    return this.http.get<MoralRule[]>(
      `${this.apiUrl}/rules`,
      { headers: this.getHeaders() }
    );
  }

  addRule(rule: MoralRule): Observable<void> {
    return this.http.post<void>(
      `${this.apiUrl}/rules`,
      rule,
      { headers: this.getHeaders() }
    );
  }
}
```

**Component Usage**:
```typescript
import { Component, OnInit } from '@angular/core';
import { JameyService, MoralRule } from './services/jamey.service';

@Component({
  selector: 'app-jamey',
  template: `
    <div *ngFor="let rule of rules">
      {{ rule.name }}: {{ rule.description }}
    </div>
  `
})
export class JameyComponent implements OnInit {
  rules: MoralRule[] = [];

  constructor(private jameyService: JameyService) {}

  ngOnInit() {
    this.jameyService.getRules().subscribe(rules => {
      this.rules = rules;
    });
  }
}
```

### Vanilla JavaScript (Fetch API)

**API Client** (`api-client.js`):
```javascript
const API_BASE_URL = process.env.API_URL || 'http://localhost:3000';
const API_KEY = process.env.API_KEY;

function getHeaders() {
  const headers = {
    'Content-Type': 'application/json',
  };
  
  if (API_KEY) {
    headers['x-api-key'] = API_KEY;
  }
  
  return headers;
}

async function evaluateAction(action) {
  const response = await fetch(`${API_BASE_URL}/evaluate`, {
    method: 'POST',
    headers: getHeaders(),
    body: JSON.stringify({ action }),
  });
  
  if (!response.ok) {
    if (response.status === 401) {
      throw new Error('Authentication failed');
    }
    if (response.status === 429) {
      throw new Error('Rate limit exceeded');
    }
    throw new Error(`HTTP error! status: ${response.status}`);
  }
  
  return await response.json();
}

async function getRules() {
  const response = await fetch(`${API_BASE_URL}/rules`, {
    method: 'GET',
    headers: getHeaders(),
  });
  
  if (!response.ok) {
    throw new Error(`HTTP error! status: ${response.status}`);
  }
  
  return await response.json();
}

async function addRule(rule) {
  const response = await fetch(`${API_BASE_URL}/rules`, {
    method: 'POST',
    headers: getHeaders(),
    body: JSON.stringify(rule),
  });
  
  if (!response.ok) {
    throw new Error(`HTTP error! status: ${response.status}`);
  }
}

// Usage
(async () => {
  try {
    const rules = await getRules();
    console.log('Rules:', rules);
    
    const result = await evaluateAction('I will help others');
    console.log('Score:', result.score);
  } catch (error) {
    console.error('Error:', error);
  }
})();
```

### Desktop Apps (Electron)

**Main Process** (`main.js`):
```javascript
const { app, BrowserWindow, ipcMain } = require('electron');
const axios = require('axios');

const API_BASE_URL = process.env.API_URL || 'http://localhost:3000';
const API_KEY = process.env.API_KEY;

const apiClient = axios.create({
  baseURL: API_BASE_URL,
  headers: {
    'Content-Type': 'application/json',
    ...(API_KEY && { 'x-api-key': API_KEY }),
  },
});

// IPC handlers
ipcMain.handle('evaluate-action', async (event, action) => {
  try {
    const response = await apiClient.post('/evaluate', { action });
    return { success: true, data: response.data };
  } catch (error) {
    return { success: false, error: error.message };
  }
});

ipcMain.handle('get-rules', async () => {
  try {
    const response = await apiClient.get('/rules');
    return { success: true, data: response.data };
  } catch (error) {
    return { success: false, error: error.message };
  }
});
```

**Renderer Process** (`renderer.js`):
```javascript
const { ipcRenderer } = require('electron');

async function evaluateAction(action) {
  const result = await ipcRenderer.invoke('evaluate-action', action);
  if (result.success) {
    console.log('Score:', result.data.score);
  } else {
    console.error('Error:', result.error);
  }
}

async function loadRules() {
  const result = await ipcRenderer.invoke('get-rules');
  if (result.success) {
    console.log('Rules:', result.data);
  }
}
```

### Mobile Apps (React Native)

**API Client** (`src/api/client.ts`):
```typescript
import axios from 'axios';

const API_BASE_URL = process.env.API_URL || 'http://localhost:3000';
const API_KEY = process.env.API_KEY;

export const apiClient = axios.create({
  baseURL: API_BASE_URL,
  headers: {
    'Content-Type': 'application/json',
    ...(API_KEY && { 'x-api-key': API_KEY }),
  },
  timeout: 10000, // 10 second timeout for mobile
});

// Add retry logic for mobile networks
apiClient.interceptors.response.use(
  (response) => response,
  async (error) => {
    if (error.response?.status >= 500 && error.config && !error.config.__isRetryRequest) {
      error.config.__isRetryRequest = true;
      await new Promise(resolve => setTimeout(resolve, 1000));
      return apiClient.request(error.config);
    }
    return Promise.reject(error);
  }
);

export const evaluateAction = async (action: string) => {
  const response = await apiClient.post('/evaluate', { action });
  return response.data;
};
```

---

## API Endpoints

### POST /evaluate

Evaluate an action's morality score.

**Request**:
```json
{
  "action": "I will help others in need",
  "entity_id": "optional-entity-name"
}
```

**Response**:
```json
{
  "score": 8.5,
  "action": "I will help others in need",
  "emotion": "joy"
}
```

**Example**:
```javascript
const result = await fetch('http://localhost:3000/evaluate', {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
    'x-api-key': 'your-api-key',
  },
  body: JSON.stringify({ action: 'I will help others' }),
});
const data = await result.json();
console.log('Score:', data.score);
```

### GET /rules

Get all moral rules.

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

**Example**:
```javascript
const response = await fetch('http://localhost:3000/rules', {
  headers: {
    'x-api-key': 'your-api-key',
  },
});
const rules = await response.json();
```

### POST /rules

Add a new moral rule.

**Request**:
```json
{
  "name": "kindness",
  "description": "Be kind to others",
  "weight": 7.5
}
```

**Response**: `201 Created` (no body)

**Example**:
```javascript
await fetch('http://localhost:3000/rules', {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
    'x-api-key': 'your-api-key',
  },
  body: JSON.stringify({
    name: 'kindness',
    description: 'Be kind to others',
    weight: 7.5,
  }),
});
```

### GET /health

Health check endpoint.

**Response**:
```json
{
  "status": "ok",
  "timestamp": "2024-01-15T10:30:00Z"
}
```

---

## Advanced Topics

### Rate Limiting

Each API key has a configurable rate limit (default: 60 requests/minute).

**Headers returned on rate limit**:
- `X-RateLimit-Limit`: Maximum requests per minute
- `X-RateLimit-Remaining`: Remaining requests in current window
- `Retry-After`: Seconds to wait before retrying

**Handling Rate Limits**:
```javascript
async function evaluateWithRetry(action, maxRetries = 3) {
  for (let i = 0; i < maxRetries; i++) {
    try {
      return await evaluateAction(action);
    } catch (error) {
      if (error.response?.status === 429) {
        const retryAfter = error.response.headers['retry-after'] || 60;
        await new Promise(resolve => setTimeout(resolve, retryAfter * 1000));
        continue;
      }
      throw error;
    }
  }
}
```

### Error Handling

**Standard Error Response**:
```json
{
  "error": "Error message",
  "code": "ERROR_CODE"
}
```

**Common Status Codes**:
- `200 OK`: Success
- `201 Created`: Resource created
- `400 Bad Request`: Invalid request
- `401 Unauthorized`: Invalid/missing API key
- `429 Too Many Requests`: Rate limit exceeded
- `500 Internal Server Error`: Server error

**Error Handling Pattern**:
```javascript
try {
  const result = await evaluateAction(action);
  // Handle success
} catch (error) {
  if (error.response) {
    // Server responded with error
    switch (error.response.status) {
      case 401:
        // Handle authentication error
        break;
      case 429:
        // Handle rate limit
        break;
      default:
        // Handle other errors
    }
  } else if (error.request) {
    // Request made but no response
    console.error('Network error');
  } else {
    // Something else happened
    console.error('Error:', error.message);
  }
}
```

### Request/Response Interceptors

**Axios Example**:
```javascript
import axios from 'axios';

const apiClient = axios.create({
  baseURL: 'http://localhost:3000',
});

// Request interceptor
apiClient.interceptors.request.use(
  (config) => {
    // Add API key to every request
    if (API_KEY) {
      config.headers['x-api-key'] = API_KEY;
    }
    return config;
  },
  (error) => Promise.reject(error)
);

// Response interceptor
apiClient.interceptors.response.use(
  (response) => response,
  (error) => {
    // Handle common errors
    if (error.response?.status === 401) {
      // Redirect to login or show error
    }
    return Promise.reject(error);
  }
);
```

### Retry Logic

**Exponential Backoff**:
```javascript
async function fetchWithRetry(url, options, maxRetries = 3) {
  for (let i = 0; i < maxRetries; i++) {
    try {
      const response = await fetch(url, options);
      if (response.ok) return response;
      
      if (response.status === 429 || response.status >= 500) {
        const delay = Math.pow(2, i) * 1000; // Exponential backoff
        await new Promise(resolve => setTimeout(resolve, delay));
        continue;
      }
      
      throw new Error(`HTTP ${response.status}`);
    } catch (error) {
      if (i === maxRetries - 1) throw error;
      const delay = Math.pow(2, i) * 1000;
      await new Promise(resolve => setTimeout(resolve, delay));
    }
  }
}
```

---

## Multiple Frontends

### Creating Separate API Keys

Create different API keys for different frontends:

```bash
# Local desktop frontend
./scripts/create-api-key.sh local-desktop 300

# Remote web frontend
./scripts/create-api-key.sh remote-web 100

# Mobile app
./scripts/create-api-key.sh mobile-app 200
```

### Different Rate Limits

Each frontend can have different rate limits:

- **Local Desktop**: 300 req/min (higher limit for full-featured app)
- **Remote Web**: 100 req/min (lower limit for public-facing)
- **Mobile App**: 200 req/min (moderate limit)

### CORS Configuration

Configure backend to allow multiple origins:

**Backend `.env`**:
```bash
ALLOWED_ORIGINS=http://localhost:5173,https://your-remote-app.com,http://localhost:3000
```

### Monitoring Per Frontend

Track usage per frontend by API key name:

```sql
SELECT name, last_used_at, rate_limit_per_minute 
FROM api_keys 
WHERE revoked_at IS NULL;
```

---

## Troubleshooting

### Connection Issues

**Problem**: Cannot connect to backend

**Solutions**:
1. Verify backend is running: `curl http://localhost:3000/health`
2. Check backend URL matches your configuration
3. Verify network/firewall allows connection
4. Check browser console for CORS errors

### Authentication Errors

**Problem**: 401 Unauthorized

**Solutions**:
1. Verify API key is correct
2. Check API key is not revoked: `SELECT * FROM api_keys WHERE key_hash = '<hash>'`
3. Ensure API key hasn't expired
4. Verify header format: `x-api-key` or `Authorization: Bearer <key>`

### CORS Errors

**Problem**: CORS policy blocking requests

**Solutions**:
1. Backend allows all origins in development (should work)
2. For production, add your origin to `ALLOWED_ORIGINS` in backend `.env`
3. Use proxy in development (Vite/Webpack proxy)
4. Check browser console for specific CORS error message

### Rate Limit Exceeded

**Problem**: 429 Too Many Requests

**Solutions**:
1. Implement exponential backoff retry logic
2. Reduce request frequency
3. Request higher rate limit for your API key
4. Check `X-RateLimit-Remaining` header to monitor usage

### Network Timeouts

**Problem**: Requests timeout

**Solutions**:
1. Increase timeout in your HTTP client
2. Implement retry logic
3. Check backend is responsive: `curl http://localhost:3000/health`
4. Verify network connectivity

---

## Next Steps

- See [API Reference](API_REFERENCE.md) for complete endpoint documentation
- See [Multiple Frontends Guide](MULTIPLE_FRONTENDS.md) for advanced setup
- See [Quick Start](FRONTEND_QUICK_START.md) for React frontend setup

