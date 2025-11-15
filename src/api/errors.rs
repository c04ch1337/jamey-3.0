//! API error handling
//!
//! Provides sanitized error responses to prevent information leakage.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::Serialize;
use thiserror::Error;

/// API error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl ErrorResponse {
    /// Create a new error response
    pub fn new(error: String, code: String) -> Self {
        Self {
            error,
            code,
            details: None,
        }
    }

    /// Create an error response with details (only in debug mode)
    pub fn with_details(error: String, code: String, details: String) -> Self {
        #[cfg(debug_assertions)]
        {
            Self {
                error,
                code,
                details: Some(details),
            }
        }
        #[cfg(not(debug_assertions))]
        {
            Self {
                error,
                code,
                details: None,
            }
        }
    }
}

/// API errors
#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Payload too large: {0}")]
    PayloadTooLarge(String),

    #[error("Internal server error")]
    Internal(String),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
}

impl ApiError {
    /// Get the HTTP status code for this error
    pub fn status_code(&self) -> StatusCode {
        match self {
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            ApiError::Forbidden(_) => StatusCode::FORBIDDEN,
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            ApiError::PayloadTooLarge(_) => StatusCode::PAYLOAD_TOO_LARGE,
            ApiError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::ServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
        }
    }

    /// Get the error code string
    pub fn error_code(&self) -> &'static str {
        match self {
            ApiError::BadRequest(_) => "BAD_REQUEST",
            ApiError::Unauthorized(_) => "UNAUTHORIZED",
            ApiError::Forbidden(_) => "FORBIDDEN",
            ApiError::NotFound(_) => "NOT_FOUND",
            ApiError::PayloadTooLarge(_) => "PAYLOAD_TOO_LARGE",
            ApiError::Internal(_) => "INTERNAL_ERROR",
            ApiError::ServiceUnavailable(_) => "SERVICE_UNAVAILABLE",
        }
    }

    /// Get sanitized error message (no internal details in production)
    pub fn sanitized_message(&self) -> String {
        match self {
            ApiError::BadRequest(msg) => msg.clone(),
            ApiError::Unauthorized(_) => "Authentication required".to_string(),
            ApiError::Forbidden(_) => "Access denied".to_string(),
            ApiError::NotFound(msg) => msg.clone(),
            ApiError::PayloadTooLarge(_) => "Request payload too large".to_string(),
            ApiError::Internal(_) => "An internal error occurred".to_string(),
            ApiError::ServiceUnavailable(_) => "Service temporarily unavailable".to_string(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let status = self.status_code();
        let error_response = ErrorResponse::with_details(
            self.sanitized_message(),
            self.error_code().to_string(),
            self.to_string(), // Full error only in debug builds
        );

        (status, Json(error_response)).into_response()
    }
}

/// Convert validation errors to API errors
impl From<(StatusCode, &'static str)> for ApiError {
    fn from((status, msg): (StatusCode, &'static str)) -> Self {
        match status {
            StatusCode::BAD_REQUEST => ApiError::BadRequest(msg.to_string()),
            StatusCode::UNAUTHORIZED => ApiError::Unauthorized(msg.to_string()),
            StatusCode::FORBIDDEN => ApiError::Forbidden(msg.to_string()),
            StatusCode::NOT_FOUND => ApiError::NotFound(msg.to_string()),
            StatusCode::PAYLOAD_TOO_LARGE => ApiError::PayloadTooLarge(msg.to_string()),
            StatusCode::SERVICE_UNAVAILABLE => ApiError::ServiceUnavailable(msg.to_string()),
            _ => ApiError::Internal(msg.to_string()),
        }
    }
}

