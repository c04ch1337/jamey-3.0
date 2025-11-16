//! CSRF Protection Module
//!
//! Implements Cross-Site Request Forgery (CSRF) protection using double-submit cookie pattern.
//! This is suitable for stateless APIs and works with both cookie-based and token-based authentication.

use axum::{
    extract::{Request, State},
    http::{HeaderMap, HeaderName, StatusCode},
    middleware::Next,
    response::Response,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{error, warn};
use uuid::Uuid;

/// CSRF token stored in memory (for stateless validation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsrfToken {
    pub token: String,
    pub expires_at: u64,
}

/// CSRF protection configuration
#[derive(Debug, Clone)]
pub struct CsrfConfig {
    /// Token expiration time in seconds (default: 3600 = 1 hour)
    pub token_expiration: Duration,
    /// Cookie name for CSRF token (default: "csrf-token")
    pub cookie_name: String,
    /// Header name for CSRF token (default: "x-csrf-token")
    pub header_name: String,
    /// Whether CSRF protection is enabled
    pub enabled: bool,
    /// Secret key for token signing (optional, for signed tokens)
    pub secret: Option<String>,
}

impl Default for CsrfConfig {
    fn default() -> Self {
        Self {
            token_expiration: Duration::from_secs(3600),
            cookie_name: "csrf-token".to_string(),
            header_name: "x-csrf-token".to_string(),
            enabled: true,
            secret: None,
        }
    }
}

impl CsrfConfig {
    /// Create config from environment variables
    pub fn from_env() -> Self {
        let enabled = std::env::var("CSRF_ENABLED")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);

        let token_expiration = std::env::var("CSRF_TOKEN_EXPIRATION")
            .unwrap_or_else(|_| "3600".to_string())
            .parse::<u64>()
            .unwrap_or(3600);

        let cookie_name = std::env::var("CSRF_COOKIE_NAME")
            .unwrap_or_else(|_| "csrf-token".to_string());

        let header_name = std::env::var("CSRF_HEADER_NAME")
            .unwrap_or_else(|_| "x-csrf-token".to_string());

        let secret = std::env::var("CSRF_SECRET").ok();

        Self {
            token_expiration: Duration::from_secs(token_expiration),
            cookie_name,
            header_name,
            enabled,
            secret,
        }
    }
}

/// CSRF token store (in-memory for stateless validation)
/// In production, consider using Redis or database for distributed systems
#[derive(Debug, Clone)]
pub struct CsrfProtection {
    config: CsrfConfig,
    // In-memory token store (for single-instance deployments)
    // For distributed systems, use Redis or database
    tokens: Arc<RwLock<std::collections::HashMap<String, CsrfToken>>>,
}

impl CsrfProtection {
    /// Create new CSRF protection instance
    pub fn new(config: CsrfConfig) -> Self {
        Self {
            config,
            tokens: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Generate a new CSRF token
    pub fn generate_token(&self) -> String {
        // Generate a secure random token
        let token = format!("csrf_{}", Uuid::new_v4().to_string().replace('-', ""));
        token
    }

    /// Store a CSRF token
    pub async fn store_token(&self, token: String) -> Result<(), String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| format!("Time error: {}", e))?;

        let expires_at = now.as_secs() + self.config.token_expiration.as_secs();

        let csrf_token = CsrfToken {
            token: token.clone(),
            expires_at,
        };

        let mut tokens = self.tokens.write().await;
        tokens.insert(token, csrf_token);

        // Clean up expired tokens periodically
        if tokens.len() > 10000 {
            self.cleanup_expired_tokens().await;
        }

        Ok(())
    }

    /// Validate a CSRF token
    pub async fn validate_token(&self, token: &str) -> Result<(), String> {
        let tokens = self.tokens.read().await;

        let csrf_token = tokens
            .get(token)
            .ok_or_else(|| "CSRF token not found".to_string())?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| format!("Time error: {}", e))?;

        if now.as_secs() > csrf_token.expires_at {
            return Err("CSRF token expired".to_string());
        }

        Ok(())
    }

    /// Remove a CSRF token (after use)
    pub async fn remove_token(&self, token: &str) {
        let mut tokens = self.tokens.write().await;
        tokens.remove(token);
    }

    /// Clean up expired tokens
    async fn cleanup_expired_tokens(&self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .ok()
            .and_then(|d| Some(d.as_secs()))
            .unwrap_or(0);

        let mut tokens = self.tokens.write().await;
        tokens.retain(|_, token| token.expires_at > now);
    }

    /// Get token from cookie
    fn get_token_from_cookie(&self, headers: &HeaderMap) -> Option<String> {
        headers
            .get("cookie")
            .and_then(|cookie| cookie.to_str().ok())
            .and_then(|cookie_str| {
                cookie_str
                    .split(';')
                    .find_map(|part| {
                        let part = part.trim();
                        if part.starts_with(&format!("{}=", self.config.cookie_name)) {
                            Some(
                                part
                                    .strip_prefix(&format!("{}=", self.config.cookie_name))
                                    .unwrap_or("")
                                    .to_string(),
                            )
                        } else {
                            None
                        }
                    })
            })
    }

    /// Get token from header
    fn get_token_from_header(&self, headers: &HeaderMap) -> Option<String> {
        headers
            .get(&self.config.header_name)
            .and_then(|header| header.to_str().ok())
            .map(|s| s.to_string())
    }
}

/// CSRF middleware - validates CSRF tokens for state-changing operations
pub async fn csrf_middleware(
    State(csrf): State<Arc<CsrfProtection>>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Skip CSRF check if disabled
    if !csrf.config.enabled {
        return Ok(next.run(request).await);
    }

    // Only check CSRF for state-changing methods
    let method = request.method();
    let is_state_changing = matches!(
        method.as_str(),
        "POST" | "PUT" | "PATCH" | "DELETE" | "PATCH"
    );

    if !is_state_changing {
        // For GET/HEAD/OPTIONS, allow through without CSRF check
        return Ok(next.run(request).await);
    }

    // Extract CSRF token from header
    let header_token = csrf.get_token_from_header(request.headers());

    // Extract CSRF token from cookie
    let cookie_token = csrf.get_token_from_cookie(request.headers());

    // Validate: token must be present in both header and cookie, and they must match
    match (header_token, cookie_token) {
        (Some(header), Some(cookie)) if header == cookie => {
            // Tokens match - validate
            match csrf.validate_token(&header).await {
                Ok(_) => {
                    // Token is valid - remove it (one-time use)
                    csrf.remove_token(&header).await;
                    Ok(next.run(request).await)
                }
                Err(e) => {
                    warn!("CSRF validation failed: {}", e);
                    error!("CSRF token validation failed for {} {}", method, request.uri().path());
                    Err(StatusCode::FORBIDDEN)
                }
            }
        }
        (Some(_), Some(_)) => {
            warn!("CSRF tokens in header and cookie do not match");
            error!("CSRF token mismatch for {} {}", method, request.uri().path());
            Err(StatusCode::FORBIDDEN)
        }
        _ => {
            warn!("CSRF token missing in header or cookie");
            error!("CSRF token missing for {} {}", method, request.uri().path());
            Err(StatusCode::FORBIDDEN)
        }
    }
}

/// Endpoint to get a CSRF token
pub async fn get_csrf_token(
    State(csrf): State<Arc<CsrfProtection>>,
) -> Result<axum::response::Json<serde_json::Value>, StatusCode> {
    if !csrf.config.enabled {
        return Ok(axum::response::Json(serde_json::json!({
            "token": "",
            "enabled": false
        })));
    }

    let token = csrf.generate_token();
    csrf.store_token(token.clone())
        .await
        .map_err(|e| {
            error!("Failed to store CSRF token: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(axum::response::Json(serde_json::json!({
        "token": token,
        "enabled": true
    })))
}

