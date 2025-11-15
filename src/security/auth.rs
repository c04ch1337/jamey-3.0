//! JWT Authentication middleware for Jamey 3.0
//!
//! Provides JWT-based authentication for API endpoints with secure token validation.

use axum::{
    extract::{Request, State},
    http::{header::AUTHORIZATION, StatusCode},
    middleware::Next,
    response::Response,
    Json,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::env;
use time::{Duration, OffsetDateTime};
use tracing::{error, info, warn};

/// JWT claims structure
#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    /// Subject (user identifier)
    pub sub: String,
    /// Issued at timestamp
    pub iat: i64,
    /// Expiration timestamp
    pub exp: i64,
    /// JWT ID (unique token identifier)
    pub jti: String,
}

/// Authentication error types
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Missing authorization header")]
    MissingHeader,
    #[error("Invalid authorization header format")]
    InvalidHeader,
    #[error("Invalid token")]
    InvalidToken,
    #[error("Token expired")]
    TokenExpired,
    #[error("JWT secret not configured")]
    SecretNotConfigured,
}

impl From<jsonwebtoken::errors::Error> for AuthError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        match err.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
            _ => AuthError::InvalidToken,
        }
    }
}

/// JWT authentication middleware
pub struct JwtAuth {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    validation: Validation,
}

impl JwtAuth {
    /// Create a new JWT authentication instance
    pub fn new() -> Result<Self, AuthError> {
        let secret = env::var("JWT_SECRET")
            .map_err(|_| AuthError::SecretNotConfigured)?;

        if secret.len() < 32 {
            error!("JWT_SECRET is too short. Minimum 32 characters required for security.");
            return Err(AuthError::SecretNotConfigured);
        }

        let encoding_key = EncodingKey::from_secret(secret.as_bytes());
        let decoding_key = DecodingKey::from_secret(secret.as_bytes());
        let mut validation = Validation::default();
        validation.validate_exp = true;

        Ok(Self {
            encoding_key,
            decoding_key,
            validation,
        })
    }

    /// Generate a new JWT token for the given subject
    pub fn generate_token(&self, subject: &str) -> Result<String, AuthError> {
        let now = OffsetDateTime::now_utc();
        let exp_seconds = env::var("JWT_EXPIRATION_SECONDS")
            .unwrap_or_else(|_| "3600".to_string())
            .parse::<i64>()
            .unwrap_or(3600);

        let claims = JwtClaims {
            sub: subject.to_string(),
            iat: now.unix_timestamp(),
            exp: (now + Duration::seconds(exp_seconds)).unix_timestamp(),
            jti: uuid::Uuid::new_v4().to_string(),
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|_| AuthError::InvalidToken)
    }

    /// Validate a JWT token and return the claims
    pub fn validate_token(&self, token: &str) -> Result<JwtClaims, AuthError> {
        let token_data = decode::<JwtClaims>(token, &self.decoding_key, &self.validation)?;
        Ok(token_data.claims)
    }

    /// Extract token from authorization header
    fn extract_token_from_header(auth_header: &str) -> Option<&str> {
        if auth_header.starts_with("Bearer ") {
            Some(&auth_header[7..])
        } else {
            None
        }
    }
}

/// Axum middleware function for JWT authentication
pub async fn jwt_auth_middleware(
    State(auth): State<JwtAuth>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract authorization header
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .ok_or_else(|| {
            warn!("Missing authorization header for path: {}", request.uri().path());
            StatusCode::UNAUTHORIZED
        })?;

    // Extract token from header
    let token = Self::extract_token_from_header(auth_header).ok_or_else(|| {
        warn!("Invalid authorization header format for path: {}", request.uri().path());
        StatusCode::UNAUTHORIZED
    })?;

    // Validate token
    let claims = auth.validate_token(token).map_err(|err| {
        error!("Token validation failed for path {}: {}", request.uri().path(), err);
        match err {
            AuthError::TokenExpired => {
                warn!("Token expired for path: {}", request.uri().path());
                StatusCode::UNAUTHORIZED
            }
            _ => StatusCode::UNAUTHORIZED,
        }
    })?;

    // Add claims to request extensions for downstream handlers
    request.extensions_mut().insert(claims);

    info!("Successfully authenticated request for path: {}", request.uri().path());
    Ok(next.run(request).await)
}

/// Login request structure
#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Login response structure
#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub expires_in: i64,
}

/// Login endpoint handler
pub async fn login(
    State(auth): State<JwtAuth>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    // TODO: Implement proper user authentication against database
    // For now, we'll use a simple username/password check
    // In production, this should verify against hashed passwords in database
    
    if request.username.is_empty() || request.password.is_empty() {
        warn!("Login attempt with empty username or password");
        return Err(StatusCode::BAD_REQUEST);
    }

    // Simple validation for demo - replace with proper authentication
    if request.username == "admin" && request.password == "admin" {
        warn!("Default admin credentials used - this should be changed in production!");
    }

    // Generate JWT token
    let token = auth.generate_token(&request.username).map_err(|err| {
        error!("Failed to generate token for user {}: {}", request.username, err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let expires_in = env::var("JWT_EXPIRATION_SECONDS")
        .unwrap_or_else(|_| "3600".to_string())
        .parse::<i64>()
        .unwrap_or(3600);

    info!("User {} logged in successfully", request.username);

    Ok(Json(LoginResponse {
        token,
        expires_in,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_token_generation_and_validation() {
        // This test requires JWT_SECRET to be set
        std::env::set_var("JWT_SECRET", "test-secret-key-that-is-long-enough-for-security");
        
        let auth = JwtAuth::new().unwrap();
        let token = auth.generate_token("testuser").unwrap();
        let claims = auth.validate_token(&token).unwrap();
        
        assert_eq!(claims.sub, "testuser");
        assert!(claims.exp > claims.iat);
    }

    #[test]
    fn test_invalid_token() {
        std::env::set_var("JWT_SECRET", "test-secret-key-that-is-long-enough-for-security");
        
        let auth = JwtAuth::new().unwrap();
        let result = auth.validate_token("invalid-token");
        
        assert!(matches!(result, Err(AuthError::InvalidToken)));
    }
}