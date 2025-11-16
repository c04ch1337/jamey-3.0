# Monitoring System Documentation

This document describes the comprehensive monitoring system implemented across the application, including metrics collection, alerting, and observability features.

## Metrics System

### Core Metrics Components

1. **HTTP Metrics**
```rust
// Record request metrics
metrics::counter!(
    "http_requests_total",
    1,
    "method" => method.to_string(),
    "path" => path.to_string(),
    "status" => status.to_string(),
);

// Record request duration
metrics::histogram!(
    "http_request_duration_seconds",
    duration.as_secs_f64(),
    "method" => method.to_string(),
    "path" => path.to_string(),
);
```

2. **Memory System Metrics**
```rust
// Record memory operations
metrics::counter!(
    "memory_operations_total",
    1,
    "layer" => layer.to_string(),
    "operation" => operation.to_string(),
);

// Record memory index size
metrics::gauge!(
    "memory_index_size_bytes",
    size_bytes as f64,
    "layer" => layer.to_string(),
);
```

3. **System Metrics**
```rust
// Record system resources
metrics::gauge!("system_memory_bytes", memory_bytes as f64);
metrics::gauge!("system_disk_free_bytes", disk_free_bytes as f64);
metrics::gauge!("system_uptime_seconds", uptime_seconds as f64);
```

## Prometheus Configuration

### Scraping Configuration
```yaml
scrape_configs:
  - job_name: 'jamey-3-backend'
    static_configs:
      - targets: ['backend:3000']
    metrics_path: '/metrics'
    scheme: 'http'
    scrape_timeout: 10s
```

### Recording Rules

```yaml
recording_rules:
  groups:
    - name: aggregation
      interval: 1h
      rules:
        # Consciousness metrics
        - record: consciousness:phi_value:1h
          expr: avg_over_time(consciousness.phi_value[1h])
        - record: consciousness:workspace_activity:1h
          expr: avg_over_time(consciousness.workspace_activity[1h])
        
        # System metrics
        - record: system:cpu_usage:5m
          expr: 100 - (avg by (instance) (irate(node_cpu_seconds_total{mode="idle"}[5m])) * 100)
        - record: system:memory_usage_bytes
          expr: node_memory_MemTotal_bytes - node_memory_MemAvailable_bytes
```

## Alert Rules

### Consciousness Alerts

1. **Critical Φ Value**
```yaml
- alert: ConsciousnessPhiCritical
  expr: consciousness_phi_value < 0.3
  for: 5m
  labels:
    severity: P1
  annotations:
    summary: "Critical: Consciousness Φ value below threshold"
    description: "Φ value is {{ $value }}, below critical threshold of 0.3"
```

2. **Workspace Activity**
```yaml
- alert: WorkspaceActivityCritical
  expr: consciousness_workspace_activity < 0.2
  for: 5m
  labels:
    severity: P1
  annotations:
    summary: "Critical: Global Workspace activity critically low"
```

### System Alerts

1. **System Availability**
```yaml
- alert: SystemUnavailable
  expr: up{job="jamey-3-backend"} == 0
  for: 1m
  labels:
    severity: P1
  annotations:
    summary: "Critical: Backend system unavailable"
```

2. **Resource Usage**
```yaml
- alert: HighMemoryUsage
  expr: (node_memory_MemTotal_bytes - node_memory_MemAvailable_bytes) / node_memory_MemTotal_bytes > 0.85
  for: 5m
  labels:
    severity: P2
  annotations:
    summary: "Warning: High memory usage"
```

## Telemetry Integration

### OpenTelemetry Setup

```rust
// Initialize telemetry with OTLP exporter
let tracer = opentelemetry_otlp::new_pipeline()
    .tracing()
    .with_exporter(exporter)
    .with_trace_config(
        trace::config().with_resource(Resource::new(vec![
            KeyValue::new("service.name", "jamey-3-backend"),
        ]))
    )
    .install_batch(opentelemetry_sdk::runtime::Tokio)?;
```

### Logging Configuration

```rust
// Configure structured logging with Bunyan format
let formatting_layer = BunyanFormattingLayer::new("jamey-3".into(), std::io::stdout);
let env_filter = EnvFilter::try_from_default_env()
    .unwrap_or_else(|_| EnvFilter::new("info"));
```

## Rate Limiting

### Configuration
```rust
pub struct RateLimitConfig {
    pub requests_per_second: u32,  // Default: 10
    pub burst_size: u32,          // Default: 50
}
```

### Implementation
```rust
// Rate limiting middleware using token bucket algorithm
let quota = Quota::per_second(NonZeroU32::new(config.requests_per_second).unwrap())
    .allow_burst(NonZeroU32::new(config.burst_size).unwrap());
```

## Best Practices

### 1. Metric Naming

- Use consistent naming patterns: `<system>_<metric>_<unit>`
- Include relevant labels for filtering
- Use appropriate metric types (counter, gauge, histogram)

### 2. Alert Configuration

- Set appropriate thresholds based on system behavior
- Configure meaningful evaluation periods
- Include clear recovery actions
- Link to runbooks for detailed procedures

### 3. Resource Monitoring

- Monitor both system and application-specific resources
- Set up trending analysis for capacity planning
- Configure appropriate retention periods
- Use recording rules for frequently accessed queries

### 4. Performance Monitoring

- Track request latencies with histograms
- Monitor error rates and status codes
- Track resource utilization trends
- Set up SLO/SLI monitoring

## Example: Complete Monitoring Setup

```rust
// Initialize metrics and telemetry
async fn setup_monitoring() -> Result<()> {
    // Initialize metrics
    let handle = init_metrics().await?;
    
    // Initialize telemetry
    init_telemetry()?;
    
    // Configure rate limiting
    let rate_limit_config = RateLimitConfig {
        requests_per_second: 10,
        burst_size: 50,
    };
    
    // Set up middleware stack
    let app = Router::new()
        .layer(MetricsMiddleware::new())
        .layer(RateLimitMiddleware::new(rate_limit_config));
        
    Ok(())
}

// Record comprehensive metrics
async fn record_system_status() {
    // Record system metrics
    record_system_metrics(
        get_memory_usage(),
        get_disk_free(),
        get_uptime(),
    );
    
    // Record memory metrics
    record_memory_metrics(
        "semantic",
        "query",
        duration,
    );
    
    // Record HTTP metrics
    record_http_request(
        "GET",
        "/api/v1/status",
        200,
        duration,
    );
}
```

## Monitoring Dashboard Setup

1. **System Overview**
   - System uptime and health
   - Resource utilization
   - Request rates and latencies

2. **Consciousness Metrics**
   - Φ value trends
   - Workspace activity
   - Memory system performance

3. **Error Tracking**
   - Error rates by type
   - Status code distribution
   - Rate limiting events

4. **Performance Metrics**
   - Request latency percentiles
   - Resource utilization trends
   - Cache hit rates

## Alerting Priorities

- P1 (Critical): Immediate action required
- P2 (Warning): Investigation needed
- P3 (Info): Monitoring required

Each alert includes:
- Clear description
- Recovery actions
- Runbook links
- Appropriate thresholds