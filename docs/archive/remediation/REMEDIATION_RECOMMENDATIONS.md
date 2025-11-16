# Remediation Recommendations for Jamey 3.0

This document provides specific, actionable recommendations to address production readiness concerns across Performance, Security, Monitoring, Deployment, and Scaling.

## Table of Contents
1. [Performance](#performance)
2. [Security](#security)
3. [Monitoring](#monitoring)
4. [Deployment](#deployment)
5. [Scaling](#scaling)

---

## Performance

### 1. Implement Caching for Conscience Rules

**Current State**: Rules are stored in `DashMap` which is fast but accessed on every evaluation without caching.

**Recommendation**: Add LRU cache for frequently accessed rules and evaluation results.

**Implementation**:
- Add `moka` crate for high-performance caching
- Cache rule lookups and evaluation results (with TTL)
- Cache invalidation on rule updates

**Priority**: Medium  
**Effort**: 2-3 hours

**Code Changes**:
```rust
// Add to Cargo.toml
moka = "0.12"

// src/conscience/cache.rs (new file)
use moka::future::Cache;
use std::sync::Arc;
use std::time::Duration;

pub struct ConscienceCache {
    rule_cache: Arc<Cache<String, MoralRule>>,
    evaluation_cache: Arc<Cache<String, f32>>,
}

impl ConscienceCache {
    pub fn new() -> Self {
        Self {
            rule_cache: Arc::new(
                Cache::builder()
                    .max_capacity(1000)
                    .time_to_live(Duration::from_secs(300))
                    .build()
            ),
            evaluation_cache: Arc::new(
                Cache::builder()
                    .max_capacity(10000)
                    .time_to_live(Duration::from_secs(60))
                    .build()
            ),
        }
    }
}
```

### 2. Add Connection Pooling for MQTT

**Current State**: Single MQTT connection per client instance.

**Recommendation**: Implement connection pool for high-throughput scenarios.

**Implementation**:
- Create `MqttConnectionPool` struct
- Pool size configurable via environment
- Round-robin or least-connections load balancing
- Health checks for pooled connections

**Priority**: Low (unless scaling to 100+ ORCH nodes)  
**Effort**: 4-6 hours

**Code Changes**:
```rust
// src/mqtt/pool.rs (new file)
pub struct MqttConnectionPool {
    connections: Vec<Arc<MqttClient>>,
    current: Arc<AtomicUsize>,
    config: MqttConfig,
}

impl MqttConnectionPool {
    pub async fn new(config: MqttConfig, pool_size: usize) -> Result<Self, MqttError> {
        let mut connections = Vec::new();
        for i in 0..pool_size {
            let mut client_config = config.clone();
            client_config.client_id = format!("{}-{}", config.client_id, i);
            connections.push(Arc::new(MqttClient::new(client_config).await?));
        }
        Ok(Self {
            connections,
            current: Arc::new(AtomicUsize::new(0)),
            config,
        })
    }

    pub fn get_connection(&self) -> Arc<MqttClient> {
        let idx = self.current.fetch_add(1, Ordering::Relaxed) % self.connections.len();
        self.connections[idx].clone()
    }
}
```

### 3. Monitor Memory Index Sizes and Implement Pruning Strategy

**Current State**: No monitoring or pruning for Tantivy indices.

**Recommendation**: 
- Add metrics for index sizes
- Implement time-based pruning (e.g., keep only last 90 days for ShortTerm)
- Add size-based pruning (max index size per layer)
- Background task to prune old memories

**Priority**: High (prevents disk exhaustion)  
**Effort**: 6-8 hours

**Implementation**:
```rust
// Add to src/memory/mod.rs
impl MemorySystem {
    /// Get index size in bytes for a layer
    pub async fn get_index_size(&self, layer: MemoryLayer) -> anyhow::Result<u64> {
        let layer_dir = self.data_dir.join(layer.as_str());
        let mut total_size = 0u64;
        let mut entries = tokio::fs::read_dir(&layer_dir).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let metadata = entry.metadata().await?;
            if metadata.is_file() {
                total_size += metadata.len();
            }
        }
        Ok(total_size)
    }

    /// Prune memories older than specified duration
    pub async fn prune_old_memories(
        &self,
        layer: MemoryLayer,
        older_than: Duration,
    ) -> anyhow::Result<usize> {
        let cutoff = Utc::now() - chrono::Duration::from_std(older_than)?;
        let index = self.indices.get(&layer)
            .ok_or_else(|| anyhow::anyhow!("Index not found"))?;
        
        // Query for old documents and delete
        // Implementation depends on Tantivy's delete API
        // This is a placeholder for the actual implementation
        Ok(0)
    }
}

// Add to src/metrics/mod.rs
pub fn record_memory_index_size(layer: &str, size_bytes: u64) {
    metrics::gauge!("memory_index_size_bytes", size_bytes as f64, &[("layer", layer)]);
}
```

**Configuration** (add to `.env`):
```bash
# Memory pruning configuration
MEMORY_SHORT_TERM_RETENTION_DAYS=7
MEMORY_LONG_TERM_RETENTION_DAYS=365
MEMORY_WORKING_RETENTION_DAYS=30
MEMORY_EPISODIC_RETENTION_DAYS=730
MEMORY_SEMANTIC_RETENTION_DAYS=0  # Never prune semantic
MEMORY_MAX_INDEX_SIZE_MB=1024
```

---

## Security

### 4. Implement API Key Rotation Mechanism

**Current State**: Single static API key with no rotation.

**Recommendation**: 
- Support multiple API keys (active + rotating)
- Key rotation endpoint (admin-only)
- Grace period for old keys during rotation
- Store keys in database with expiration timestamps

**Priority**: High  
**Effort**: 4-6 hours

**Implementation**:
```rust
// Add migration: migrations/YYYYMMDDHHMMSS_api_keys.sql
CREATE TABLE IF NOT EXISTS api_keys (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    key_hash TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TEXT,
    revoked_at TEXT,
    last_used_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_api_keys_hash ON api_keys(key_hash);
CREATE INDEX IF NOT EXISTS idx_api_keys_active ON api_keys(expires_at, revoked_at);

// src/api/auth.rs - Update AuthState
use sha2::{Sha256, Digest};
use sqlx::SqlitePool;

pub struct ApiKeyManager {
    pool: SqlitePool,
}

impl ApiKeyManager {
    pub async fn validate_key(&self, key: &str) -> Result<bool, sqlx::Error> {
        let hash = Self::hash_key(key);
        let result = sqlx::query!(
            r#"
            SELECT id FROM api_keys
            WHERE key_hash = ? 
            AND (expires_at IS NULL OR expires_at > datetime('now'))
            AND revoked_at IS NULL
            "#,
            hash
        )
        .fetch_optional(&self.pool)
        .await?;

        if result.is_some() {
            // Update last_used_at
            sqlx::query!(
                "UPDATE api_keys SET last_used_at = datetime('now') WHERE key_hash = ?",
                hash
            )
            .execute(&self.pool)
            .await?;
        }

        Ok(result.is_some())
    }

    pub async fn rotate_key(&self, old_key: &str, new_key: &str) -> Result<(), sqlx::Error> {
        // Mark old key as revoked, add new key
        let old_hash = Self::hash_key(old_key);
        sqlx::query!(
            "UPDATE api_keys SET revoked_at = datetime('now') WHERE key_hash = ?",
            old_hash
        )
        .execute(&self.pool)
        .await?;

        let new_hash = Self::hash_key(new_key);
        sqlx::query!(
            "INSERT INTO api_keys (key_hash, name, created_at) VALUES (?, ?, datetime('now'))",
            new_hash,
            "Rotated key"
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    fn hash_key(key: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}
```

**Add to Cargo.toml**:
```toml
sha2 = "0.10"
```

### 5. Add Rate Limiting Per API Key/Client

**Current State**: Global rate limiting (50 req/s) without per-key limits.

**Recommendation**: 
- Per-API-key rate limits (configurable)
- Different limits for different key types
- Track rate limits in Redis or in-memory with TTL

**Priority**: High  
**Effort**: 3-4 hours

**Implementation**:
```rust
// Update src/metrics/mod.rs
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub struct PerKeyRateLimiter {
    limits: Arc<Mutex<HashMap<String, RateLimitEntry>>>,
    default_limit: u32,
}

struct RateLimitEntry {
    count: u32,
    reset_at: Instant,
    limit: u32,
}

impl PerKeyRateLimiter {
    pub fn new(default_limit: u32) -> Self {
        Self {
            limits: Arc::new(Mutex::new(HashMap::new())),
            default_limit,
        }
    }

    pub fn check(&self, key_id: &str) -> Result<(), StatusCode> {
        let mut limits = self.limits.lock().unwrap();
        let now = Instant::now();

        // Clean up old entries
        limits.retain(|_, entry| entry.reset_at > now);

        let entry = limits.entry(key_id.to_string())
            .or_insert_with(|| RateLimitEntry {
                count: 0,
                reset_at: now + Duration::from_secs(60),
                limit: self.default_limit,
            });

        if entry.count >= entry.limit {
            return Err(StatusCode::TOO_MANY_REQUESTS);
        }

        entry.count += 1;
        Ok(())
    }

    pub fn set_limit(&self, key_id: &str, limit: u32) {
        let mut limits = self.limits.lock().unwrap();
        if let Some(entry) = limits.get_mut(key_id) {
            entry.limit = limit;
        }
    }
}
```

### 6. Consider Adding Request Signing for Sensitive Operations

**Current State**: No request signing.

**Recommendation**: 
- HMAC-SHA256 request signing for `/rules` POST (add/remove rules)
- Signing key stored securely (env var or vault)
- Timestamp-based nonce to prevent replay attacks

**Priority**: Medium  
**Effort**: 3-4 hours

**Implementation**:
```rust
// src/api/signing.rs (new file)
use hmac::{Hmac, Mac};
use sha2::Sha256;
use axum::extract::Request;
use axum::http::HeaderValue;

type HmacSha256 = Hmac<Sha256>;

pub fn verify_request_signature(
    request: &Request,
    signing_key: &[u8],
) -> Result<(), StatusCode> {
    let signature_header = request.headers()
        .get("X-Signature")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let timestamp_header = request.headers()
        .get("X-Timestamp")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Verify timestamp is within 5 minutes
    let timestamp: i64 = timestamp_header.to_str()
        .ok()
        .and_then(|s| s.parse().ok())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let now = chrono::Utc::now().timestamp();
    if (now - timestamp).abs() > 300 {
        return Err(StatusCode::REQUEST_TIMEOUT);
    }

    // Reconstruct signed message
    let method = request.method().as_str();
    let path = request.uri().path();
    let body_hash = // Hash request body if present
        request.headers()
            .get("X-Body-Hash")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("");

    let message = format!("{}{}{}{}", method, path, timestamp, body_hash);
    
    let mut mac = HmacSha256::new_from_slice(signing_key)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    mac.update(message.as_bytes());
    let expected_signature = hex::encode(mac.finalize().into_bytes());

    let provided_signature = signature_header.to_str()
        .ok_or(StatusCode::BAD_REQUEST)?;

    if provided_signature == expected_signature {
        Ok(())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}
```

**Add to Cargo.toml**:
```toml
hmac = "0.12"
hex = "0.4"
```

---

## Monitoring

### 7. Set Up Alerting on Key Metrics

**Current State**: Metrics exported but no alerting.

**Recommendation**: 
- Integrate with Prometheus Alertmanager
- Define alert rules for:
  - High error rates (>5% 5xx)
  - High latency (p95 > 1s)
  - Memory index size exceeding threshold
  - Backup failures
  - MQTT connection failures
  - Database connection pool exhaustion

**Priority**: High  
**Effort**: 4-6 hours

**Implementation**:
```yaml
# prometheus/alerts.yml (new file)
groups:
  - name: jamey_alerts
    interval: 30s
    rules:
      - alert: HighErrorRate
        expr: rate(http_requests_total{status=~"5.."}[5m]) > 0.05
        for: 5m
        annotations:
          summary: "High error rate detected"
          description: "Error rate is {{ $value }} (threshold: 0.05)"

      - alert: HighLatency
        expr: histogram_quantile(0.95, http_request_duration_seconds) > 1.0
        for: 5m
        annotations:
          summary: "High latency detected"
          description: "P95 latency is {{ $value }}s"

      - alert: MemoryIndexSizeExceeded
        expr: memory_index_size_bytes > 1073741824  # 1GB
        for: 10m
        annotations:
          summary: "Memory index size exceeded"
          description: "Index {{ $labels.layer }} is {{ $value }} bytes"

      - alert: BackupFailure
        expr: increase(backup_operations_total{success="false"}[1h]) > 0
        annotations:
          summary: "Backup failed"
          description: "Backup operation failed"

      - alert: MQTTConnectionFailure
        expr: mqtt_connection_state == 0  # Disconnected
        for: 2m
        annotations:
          summary: "MQTT connection lost"
          description: "MQTT client is disconnected"
```

### 8. Add Tracing for Cross-Component Requests

**Current State**: Basic tracing exists but no distributed tracing.

**Recommendation**: 
- Integrate OpenTelemetry with Jaeger/Tempo
- Add trace IDs to all requests
- Propagate trace context across MQTT messages
- Add spans for: API → Conscience → Memory → MQTT

**Priority**: Medium  
**Effort**: 6-8 hours

**Implementation**:
```rust
// Add to Cargo.toml
opentelemetry = "0.21"
opentelemetry-otlp = "0.14"
opentelemetry_sdk = "0.21"
tracing-opentelemetry = "0.21"

// src/tracing/mod.rs (new file)
use opentelemetry::global;
use opentelemetry_sdk::trace::TracerProvider;
use opentelemetry_otlp::WithExportConfig;

pub fn init_tracing() -> anyhow::Result<()> {
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://localhost:4317")
        )
        .with_trace_config(
            opentelemetry_sdk::trace::config()
                .with_resource(opentelemetry_sdk::Resource::new(vec![
                    opentelemetry::KeyValue::new("service.name", "jamey-3.0"),
                ]))
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)?;

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_tracer(tracer)
        .init();

    Ok(())
}

// Update src/api/mod.rs - Add tracing spans
use tracing::{instrument, Span};

#[instrument(skip(state))]
async fn evaluate_action(
    State(state): State<AppState>,
    Json(req): Json<EvaluateRequest>,
) -> Result<Json<EvaluateResponse>, StatusCode> {
    let span = Span::current();
    span.record("action", &req.action.as_str());
    
    // ... existing code ...
}
```

### 9. Monitor Backup Success/Failure Rates

**Current State**: Backups log success/failure but no metrics.

**Recommendation**: 
- Add Prometheus metrics for backup operations
- Track: success/failure counts, duration, size
- Alert on consecutive failures

**Priority**: High  
**Effort**: 2-3 hours

**Implementation**:
```rust
// Update src/phoenix/vault.rs
use crate::metrics;

impl PhoenixVault {
    pub async fn create_backup(&self) -> Result<BackupManifest, PhoenixError> {
        let start = std::time::Instant::now();
        
        match self._create_backup_internal().await {
            Ok(manifest) => {
                let duration = start.elapsed();
                metrics::increment_counter!("backup_operations_total", &[
                    ("status", "success"),
                    ("component", "vault"),
                ]);
                metrics::histogram!("backup_duration_seconds", duration.as_secs_f64(), &[]);
                metrics::gauge!("backup_size_bytes", manifest.size_bytes as f64, &[]);
                Ok(manifest)
            }
            Err(e) => {
                metrics::increment_counter!("backup_operations_total", &[
                    ("status", "failure"),
                    ("component", "vault"),
                    ("error", &e.to_string()),
                ]);
                Err(e)
            }
        }
    }
}

// Add to src/metrics/mod.rs
pub fn record_backup_operation(
    status: &str,
    component: &str,
    duration: Option<Duration>,
    size_bytes: Option<u64>,
) {
    metrics::increment_counter!("backup_operations_total", &[
        ("status", status),
        ("component", component),
    ]);
    
    if let Some(dur) = duration {
        metrics::histogram!("backup_duration_seconds", dur.as_secs_f64(), &[]);
    }
    
    if let Some(size) = size_bytes {
        metrics::gauge!("backup_size_bytes", size as f64, &[]);
    }
}
```

---

## Deployment

### 10. Use Reverse Proxy (e.g., Nginx) in Production

**Current State**: Direct Axum server exposure.

**Recommendation**: 
- Nginx reverse proxy for TLS termination
- Load balancing (if multiple instances)
- Rate limiting at proxy level
- Static file serving

**Priority**: High  
**Effort**: 2-3 hours

**Implementation**:
```nginx
# nginx/jamey.conf (new file)
upstream jamey_backend {
    server localhost:3000;
    # Add more servers for load balancing
    # server localhost:3001;
    # server localhost:3002;
}

server {
    listen 443 ssl http2;
    server_name jamey.example.com;

    ssl_certificate /etc/ssl/certs/jamey.crt;
    ssl_certificate_key /etc/ssl/private/jamey.key;
    ssl_protocols TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;

    # Rate limiting
    limit_req_zone $binary_remote_addr zone=api_limit:10m rate=10r/s;
    limit_req zone=api_limit burst=20 nodelay;

    location / {
        proxy_pass http://jamey_backend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # Timeouts
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
    }

    # Health check endpoint (no rate limit)
    location /health {
        limit_req off;
        proxy_pass http://jamey_backend;
    }

    # Metrics endpoint (restricted)
    location /metrics {
        allow 10.0.0.0/8;  # Internal network only
        deny all;
        proxy_pass http://jamey_backend;
    }
}
```

### 11. Implement Proper TLS Termination

**Current State**: No TLS in application (assumes reverse proxy).

**Recommendation**: 
- TLS termination at Nginx (recommended)
- OR: TLS in Axum using `axum-server` with rustls
- Certificate management (Let's Encrypt)

**Priority**: High  
**Effort**: 2-4 hours (depending on approach)

**Implementation** (if doing TLS in Axum):
```rust
// Update src/main.rs
use axum_server::tls_rustls::RustlsConfig;

#[tokio::main]
async fn main() -> Result<()> {
    // ... existing setup ...

    let config = RustlsConfig::from_pem_file(
        "certs/cert.pem",
        "certs/key.pem",
    ).await?;

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8443").await?;
    
    axum_server::from_tcp_listener(listener)
        .tls(config)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
```

**Add to Cargo.toml**:
```toml
axum-server = { version = "0.6", features = ["tls-rustls"] }
```

### 12. Consider Containerization for Consistent Deployment

**Current State**: No containerization.

**Status**: ⏸️ Removed per user request - Docker configuration not included in codebase.

**Note**: If containerization is needed in the future, consider:
- Dockerfile for Jamey 3.0
- Docker Compose for local development
- Multi-stage build for smaller images
- Health checks in container

---

## Scaling

### 13. Consider Sharding Memory Indices by Date

**Current State**: Single index per layer.

**Recommendation**: 
- Shard indices by date (monthly/yearly)
- Query across shards when needed
- Archive old shards to cold storage

**Priority**: Low (unless handling millions of memories)  
**Effort**: 8-12 hours

**Implementation**:
```rust
// Update src/memory/mod.rs
pub struct MemorySystem {
    indices: std::collections::HashMap<(MemoryLayer, String), Index>,  // (layer, shard_key)
    data_dir: PathBuf,
}

impl MemorySystem {
    fn get_shard_key(timestamp: DateTime<Utc>) -> String {
        // Monthly sharding: "2024-01"
        format!("{}", timestamp.format("%Y-%m"))
    }

    async fn get_or_create_shard(
        &mut self,
        layer: MemoryLayer,
        shard_key: String,
    ) -> anyhow::Result<&Index> {
        let key = (layer, shard_key);
        if !self.indices.contains_key(&key) {
            // Create new shard index
            // ... implementation ...
        }
        Ok(&self.indices[&key])
    }

    pub async fn search(
        &self,
        layer: MemoryLayer,
        query: &str,
        limit: usize,
        date_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    ) -> anyhow::Result<Vec<MemoryRecord>> {
        // Determine which shards to query based on date_range
        // Query each shard and merge results
        // ... implementation ...
    }
}
```

### 14. Implement Read Replicas for Database if Needed

**Current State**: Single SQLite database.

**Recommendation**: 
- For SQLite: Use WAL mode (already likely enabled)
- For future PostgreSQL migration: Read replicas
- Connection pool with read/write splitting

**Priority**: Low (SQLite handles current scale)  
**Effort**: N/A (requires PostgreSQL migration first)

**Note**: SQLite with WAL mode can handle high read concurrency. Consider PostgreSQL migration only if:
- Need >100 concurrent writers
- Database size >100GB
- Need distributed deployment

### 15. Add Circuit Breakers for External Services

**Current State**: No circuit breakers for OpenRouter API or MQTT.

**Recommendation**: 
- Circuit breaker for OpenRouter API calls
- Circuit breaker for MQTT operations
- Fallback to local LLM (Ollama) when circuit open

**Priority**: Medium  
**Effort**: 4-6 hours

**Implementation**:
```rust
// Add to Cargo.toml
tower = { version = "0.4", features = ["util", "limit", "timeout"] }

// src/circuit_breaker/mod.rs (new file)
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};

pub struct CircuitBreaker {
    state: Arc<Mutex<CircuitState>>,
    failure_threshold: usize,
    timeout: Duration,
    half_open_timeout: Duration,
}

enum CircuitState {
    Closed { failure_count: usize },
    Open { opened_at: Instant },
    HalfOpen,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: usize, timeout: Duration) -> Self {
        Self {
            state: Arc::new(Mutex::new(CircuitState::Closed { failure_count: 0 })),
            failure_threshold,
            timeout,
            half_open_timeout: Duration::from_secs(60),
        }
    }

    pub async fn call<F, T, E>(&self, f: F) -> Result<T, E>
    where
        F: std::future::Future<Output = Result<T, E>>,
    {
        let mut state = self.state.lock().unwrap();
        
        match *state {
            CircuitState::Closed { failure_count } if failure_count >= self.failure_threshold => {
                *state = CircuitState::Open { opened_at: Instant::now() };
            }
            CircuitState::Open { opened_at } => {
                if opened_at.elapsed() > self.timeout {
                    *state = CircuitState::HalfOpen;
                } else {
                    return Err(/* circuit open error */);
                }
            }
            _ => {}
        }

        drop(state);
        
        match f.await {
            Ok(result) => {
                let mut state = self.state.lock().unwrap();
                *state = CircuitState::Closed { failure_count: 0 };
                Ok(result)
            }
            Err(e) => {
                let mut state = self.state.lock().unwrap();
                match *state {
                    CircuitState::HalfOpen => {
                        *state = CircuitState::Open { opened_at: Instant::now() };
                    }
                    CircuitState::Closed { ref mut failure_count } => {
                        *failure_count += 1;
                    }
                    _ => {}
                }
                Err(e)
            }
        }
    }
}
```

---

## Implementation Priority Summary

### High Priority (Implement First)
1. ✅ Monitor Memory Index Sizes and Implement Pruning Strategy
2. ✅ Implement API Key Rotation Mechanism
3. ✅ Add Rate Limiting Per API Key/Client
4. ✅ Set Up Alerting on Key Metrics
5. ✅ Monitor Backup Success/Failure Rates
6. ✅ Use Reverse Proxy (Nginx) in Production
7. ✅ Implement Proper TLS Termination
8. ⏸️ Consider Containerization for Consistent Deployment (Removed per user request)

### Medium Priority
9. ✅ Implement Caching for Conscience Rules
10. ✅ Consider Adding Request Signing for Sensitive Operations
11. ✅ Add Tracing for Cross-Component Requests
12. ✅ Add Circuit Breakers for External Services

### Low Priority (Scale-Dependent)
13. ✅ Add Connection Pooling for MQTT (if >100 ORCH nodes)
14. ✅ Consider Sharding Memory Indices by Date (if millions of memories)
15. ✅ Implement Read Replicas (if migrating to PostgreSQL)

---

## Next Steps

1. **Week 1**: Implement High Priority items 1-4 (Memory pruning, API key rotation, per-key rate limiting, alerting)
2. **Week 2**: Implement High Priority items 5-7 (Backup metrics, Nginx, TLS)
3. **Week 3**: Implement Medium Priority items 9-12 (Caching, signing, tracing, circuit breakers)
4. **Week 4**: Testing, documentation, and deployment

---

## Notes

- All code examples are illustrative and may need adjustment for your specific use case
- Test thoroughly in staging before production deployment
- Consider security implications of each change
- Monitor metrics after each implementation to validate improvements

