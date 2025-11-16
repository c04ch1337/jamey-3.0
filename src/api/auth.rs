//! API Authentication middleware
//!
//! Provides API key authentication for protected endpoints.

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use tracing::warn;

/// API key header name
const API_KEY_HEADER: &str = "x-api-key";
const AUTHORIZATION_HEADER: &str = "authorization";

/// Authentication state
#[derive(Clone)]
pub struct AuthState {
    /// Expected API key (if None, authentication is disabled)
    pub api_key: Option<String>,
}

impl AuthState {
    pub fn new(api_key: Option<String>) -> Self {
        Self { api_key }
    }

    /// Check if authentication is required
    pub fn is_required(&self) -> bool {
        self.api_key.is_some()
    }
}

/// Middleware to authenticate requests using API key
pub async fn auth_middleware(
    State(auth_state): State<Arc<AuthState>>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // If no API key is configured, allow all requests
    if !auth_state.is_required() {
        return Ok(next.run(request).await);
    }

    // Extract API key from headers
    let headers = request.headers();
    let provided_key = headers
        .get(API_KEY_HEADER)
        .or_else(|| headers.get(AUTHORIZATION_HEADER))
        .and_then(|h| h.to_str().ok())
        .map(|s| {
            // Handle "Bearer <token>" format
            if s.starts_with("Bearer ") {
                s.strip_prefix("Bearer ").unwrap_or(s).to_string()
            } else {
                s.to_string()
            }
        });

    match provided_key {
        // Compare against the configured key as &str to avoid String/&String mismatch
        Some(key) if auth_state.api_key.as_deref() == Some(key.as_str()) => {
            // Authentication successful
            Ok(next.run(request).await)
        }
        Some(_) => {
            warn!("Invalid API key provided");
            Err(StatusCode::UNAUTHORIZED)
        }
        None => {
            warn!("Missing API key in request");
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

