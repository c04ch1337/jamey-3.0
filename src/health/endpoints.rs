use axum::{
    extract::State,
    routing::get,
    Router,
    Json,
};
use std::sync::Arc;
use serde::Serialize;
use super::{HealthManager, HealthStatus, HealthCheckResult};

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    components: Vec<ComponentHealth>,
    timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize)]
struct ComponentHealth {
    name: String,
    status: String,
    details: Option<String>,
    response_time_ms: u64,
    last_check: chrono::DateTime<chrono::Utc>,
}

pub fn health_routes(health_manager: Arc<HealthManager>) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/health/details", get(detailed_health_check))
        .with_state(health_manager)
}

async fn health_check(
    State(health_manager): State<Arc<HealthManager>>,
) -> Json<HealthResponse> {
    let system_health = health_manager.get_system_health().await;
    
    let status = match system_health {
        HealthStatus::Healthy => "healthy".to_string(),
        HealthStatus::Degraded { reason: _ } => "degraded".to_string(),
        HealthStatus::Unhealthy { reason: _ } => "unhealthy".to_string(),
    };

    Json(HealthResponse {
        status,
        components: vec![], // Basic check doesn't include component details
        timestamp: chrono::Utc::now(),
    })
}

async fn detailed_health_check(
    State(health_manager): State<Arc<HealthManager>>,
) -> Json<HealthResponse> {
    let component_results = health_manager.get_health_status().await;
    let system_health = health_manager.get_system_health().await;
    
    let status = match system_health {
        HealthStatus::Healthy => "healthy".to_string(),
        HealthStatus::Degraded { reason: _ } => "degraded".to_string(),
        HealthStatus::Unhealthy { reason: _ } => "unhealthy".to_string(),
    };

    let components = component_results
        .into_iter()
        .map(|(name, result)| ComponentHealth {
            name,
            status: format_health_status(&result.status),
            details: get_health_details(&result.status),
            response_time_ms: result.response_time.as_millis() as u64,
            last_check: chrono::DateTime::from(result.last_check),
        })
        .collect();

    Json(HealthResponse {
        status,
        components,
        timestamp: chrono::Utc::now(),
    })
}

fn format_health_status(status: &HealthStatus) -> String {
    match status {
        HealthStatus::Healthy => "healthy".to_string(),
        HealthStatus::Degraded { .. } => "degraded".to_string(),
        HealthStatus::Unhealthy { .. } => "unhealthy".to_string(),
    }
}

fn get_health_details(status: &HealthStatus) -> Option<String> {
    match status {
        HealthStatus::Healthy => None,
        HealthStatus::Degraded { reason } => Some(reason.clone()),
        HealthStatus::Unhealthy { reason } => Some(reason.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;
    use std::time::Duration;

    #[tokio::test]
    async fn test_health_endpoints() {
        let health_manager = Arc::new(HealthManager::new(Duration::from_secs(60)));
        let app = health_routes(health_manager);

        // Test basic health endpoint
        let response = app
            .clone()
            .oneshot(Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Test detailed health endpoint
        let response = app
            .oneshot(Request::builder()
                .uri("/health/details")
                .body(Body::empty())
                .unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}