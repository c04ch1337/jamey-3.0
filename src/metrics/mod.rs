use metrics::{Counter, Gauge, Histogram, Key, KeyName, Unit};
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder, PrometheusHandle};
use std::time::{Duration, Instant};
use tokio::sync::OnceCell;
use tracing::{info, warn};
use axum::body::Body;

/// Global metrics registry
static METRICS: OnceCell<PrometheusHandle> = OnceCell::const_new();

/// Initialize metrics system
pub async fn init_metrics() -> anyhow::Result<PrometheusHandle> {
    let handle = METRICS.get_or_init(|| async {
        PrometheusBuilder::new()
            .set_buckets_for_metric(
                Matcher::Full("http_request_duration_seconds".to_string()),
                &[0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
            )
            .unwrap()
            .install_recorder()
            .unwrap()
    }).await;

    Ok(handle.clone())
}

/// Record HTTP request metrics
pub fn record_http_request(
    method: &str,
    path: &str,
    status: u16,
    duration: std::time::Duration,
) {
    metrics::counter!(
        "http_requests_total",
        1,
        "method" => method.to_string(),
        "path" => path.to_string(),
        "status" => status.to_string(),
    );

    metrics::histogram!(
        "http_request_duration_seconds",
        duration.as_secs_f64(),
        "method" => method.to_string(),
        "path" => path.to_string(),
        "status" => status.to_string(),
    );
}

/// Record memory system metrics
pub fn record_memory_metrics(
    layer: &str,
    operation: &str,
    duration: std::time::Duration,
) {
    metrics::counter!(
        "memory_operations_total",
        1,
        "layer" => layer.to_string(),
        "operation" => operation.to_string(),
    );

    metrics::histogram!(
        "memory_operation_duration_seconds",
        duration.as_secs_f64(),
        "layer" => layer.to_string(),
        "operation" => operation.to_string(),
    );
}

/// Record MQTT metrics
pub fn record_mqtt_metrics(
    topic: &str,
    operation: &str,
    success: bool,
) {
    metrics::counter!(
        "mqtt_operations_total",
        1,
        "topic" => topic.to_string(),
        "operation" => operation.to_string(),
        "success" => success.to_string(),
    );
}

/// Record system metrics
pub fn record_system_metrics(
    memory_bytes: u64,
    disk_free_bytes: u64,
    uptime_seconds: u64,
) {
    metrics::gauge!("system_memory_bytes", memory_bytes as f64);
    metrics::gauge!("system_disk_free_bytes", disk_free_bytes as f64);
    metrics::gauge!("system_uptime_seconds", uptime_seconds as f64);
}

/// Record memory index size
pub fn record_memory_index_size(layer: &str, size_bytes: u64) {
    metrics::gauge!(
        "memory_index_size_bytes",
        size_bytes as f64,
        "layer" => layer.to_string(),
    );
}

/// Record backup operation metrics
pub fn record_backup_operation(
    status: &str,
    component: &str,
    duration: Option<Duration>,
    size_bytes: Option<u64>,
) {
    metrics::counter!(
        "backup_operations_total",
        1,
        "status" => status.to_string(),
        "component" => component.to_string(),
    );
    
    if let Some(dur) = duration {
        metrics::histogram!("backup_duration_seconds", dur.as_secs_f64());
    }
    
    if let Some(size) = size_bytes {
        metrics::gauge!("backup_size_bytes", size as f64);
    }
}

/// Middleware to collect HTTP metrics
#[derive(Clone)]
pub struct MetricsMiddleware<S> {
    inner: S,
}

impl<S> MetricsMiddleware<S> {
    pub fn new(inner: S) -> Self {
        Self { inner }
    }
}

impl<S> tower::Service<hyper::Request<Body>> for MetricsMiddleware<S>
where
    S: tower::Service<hyper::Request<Body>, Response = hyper::Response<Body>>,
    S::Future: Send + 'static,
    S::Error: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: hyper::Request<Body>) -> Self::Future {
        let start = Instant::now();
        let method = req.method().to_string();
        let path = req.uri().path().to_string();

        let inner = self.inner.call(req);

        Box::pin(async move {
            let response = inner.await?;
            let duration = start.elapsed();
            let status = response.status().as_u16();

            record_http_request(&method, &path, status, duration);

            Ok(response)
        })
    }
}

/// Rate limiting configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub requests_per_second: u32,
    pub burst_size: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_second: 10,
            burst_size: 50,
        }
    }
}

/// Rate limiting middleware using token bucket algorithm
#[derive(Clone)]
pub struct RateLimitMiddleware<S> {
    inner: S,
    limiter: governor::RateLimiter<governor::state::NotKeyed, governor::state::InMemoryState, governor::clock::DefaultClock>,
}

impl<S> RateLimitMiddleware<S> {
    pub fn new(inner: S, config: RateLimitConfig) -> Self {
        use governor::{Quota, RateLimiter};
        use std::num::NonZeroU32;

        let quota = Quota::per_second(NonZeroU32::new(config.requests_per_second).unwrap())
            .allow_burst(NonZeroU32::new(config.burst_size).unwrap());

        Self {
            inner,
            limiter: RateLimiter::direct(quota),
        }
    }
}

impl<S> tower::Service<hyper::Request<Body>> for RateLimitMiddleware<S>
where
    S: tower::Service<hyper::Request<Body>, Response = hyper::Response<Body>>,
    S::Future: Send + 'static,
    S::Error: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: hyper::Request<Body>) -> Self::Future {
        // Check rate limit
        if self.limiter.check().is_err() {
            let response = hyper::Response::builder()
                .status(429)
                .body(Body::from("Rate limit exceeded"))
                .unwrap();

            return Box::pin(async move { Ok(response) });
        }

        let future = self.inner.call(req);

        Box::pin(async move { future.await })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hyper::{body::Body, Request, Response};
    use tower::{Service, ServiceExt};
    use std::convert::Infallible;

    #[tokio::test]
    async fn test_metrics_middleware() {
        // Create test service
        let service = tower::service_fn(|_| async {
            Ok::<_, Infallible>(Response::new(Body::empty()))
        });

        let mut middleware = MetricsMiddleware::new(service);

        // Make test request
        let request = Request::builder()
            .uri("/test")
            .body(Body::empty())
            .unwrap();

        let response = middleware.ready().await.unwrap().call(request).await.unwrap();
        assert_eq!(response.status(), 200);
    }

    #[tokio::test]
    async fn test_rate_limit_middleware() {
        // Create test service
        let service = tower::service_fn(|_| async {
            Ok::<_, Infallible>(Response::new(Body::empty()))
        });

        let config = RateLimitConfig {
            requests_per_second: 1,
            burst_size: 1,
        };

        let mut middleware = RateLimitMiddleware::new(service, config);

        // First request should succeed
        let request = Request::builder()
            .uri("/test")
            .body(Body::empty())
            .unwrap();

        let response = middleware.ready().await.unwrap().call(request).await.unwrap();
        assert_eq!(response.status(), 200);

        // Second immediate request should be rate limited
        let request = Request::builder()
            .uri("/test")
            .body(Body::empty())
            .unwrap();

        let response = middleware.ready().await.unwrap().call(request).await.unwrap();
        assert_eq!(response.status(), 429);
    }
}