//! Centralized error handling for Jamey 3.0
//!
//! Provides a unified error handling system with:
//! - Standardized error types using `thiserror`
//! - Consistent error response format
//! - Safe error message sanitization
//! - Conversion utilities between error types

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

/// Application-wide error type
#[derive(Debug, Error)]
pub enum AppError {
    // API Errors
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

    // Database Errors
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    // Configuration Errors
    #[error("Configuration error: {0}")]
    Config(String),

    // Validation Errors
    #[error("Validation error: {0}")]
    Validation(String),

    // Security Errors
    #[error("Security error: {0}")]
    Security(String),

    // Memory System Errors
    #[error("Memory system error: {0}")]
    Memory(String),

    // MQTT Errors
    #[error("MQTT error: {0}")]
    Mqtt(String),

    // Soul System Errors
    #[error("Soul system error: {0}")]
    Soul(String),

    // Conscience Errors
    #[error("Conscience error: {0}")]
    Conscience(String),

    // Backup/Restore Errors
    #[error("Backup error: {0}")]
    Backup(String),

    // IO Errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    // Serialization Errors
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    // Generic anyhow errors (for compatibility)
    #[error("Error: {0}")]
    Anyhow(#[from] anyhow::Error),
}

impl AppError {
    /// Get the HTTP status code for this error
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AppError::Forbidden(_) => StatusCode::FORBIDDEN,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::PayloadTooLarge(_) => StatusCode::PAYLOAD_TOO_LARGE,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::ServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            AppError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Config(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Validation(_) => StatusCode::BAD_REQUEST,
            AppError::Security(_) => StatusCode::FORBIDDEN,
            AppError::Memory(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Mqtt(_) => StatusCode::SERVICE_UNAVAILABLE,
            AppError::Soul(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Conscience(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Backup(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Serialization(_) => StatusCode::BAD_REQUEST,
            AppError::Anyhow(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Get the error code string
    pub fn error_code(&self) -> &'static str {
        match self {
            AppError::BadRequest(_) => "BAD_REQUEST",
            AppError::Unauthorized(_) => "UNAUTHORIZED",
            AppError::Forbidden(_) => "FORBIDDEN",
            AppError::NotFound(_) => "NOT_FOUND",
            AppError::PayloadTooLarge(_) => "PAYLOAD_TOO_LARGE",
            AppError::Internal(_) => "INTERNAL_ERROR",
            AppError::ServiceUnavailable(_) => "SERVICE_UNAVAILABLE",
            AppError::Database(_) => "DATABASE_ERROR",
            AppError::Config(_) => "CONFIG_ERROR",
            AppError::Validation(_) => "VALIDATION_ERROR",
            AppError::Security(_) => "SECURITY_ERROR",
            AppError::Memory(_) => "MEMORY_ERROR",
            AppError::Mqtt(_) => "MQTT_ERROR",
            AppError::Soul(_) => "SOUL_ERROR",
            AppError::Conscience(_) => "CONSCIENCE_ERROR",
            AppError::Backup(_) => "BACKUP_ERROR",
            AppError::Io(_) => "IO_ERROR",
            AppError::Serialization(_) => "SERIALIZATION_ERROR",
            AppError::Anyhow(_) => "INTERNAL_ERROR",
        }
    }

    /// Get sanitized error message (no internal details in production)
    pub fn sanitized_message(&self) -> String {
        match self {
            AppError::BadRequest(msg) => msg.clone(),
            AppError::Unauthorized(_) => "Authentication required".to_string(),
            AppError::Forbidden(_) => "Access denied".to_string(),
            AppError::NotFound(msg) => msg.clone(),
            AppError::PayloadTooLarge(_) => "Request payload too large".to_string(),
            AppError::Internal(_) => "An internal error occurred".to_string(),
            AppError::ServiceUnavailable(_) => "Service temporarily unavailable".to_string(),
            AppError::Database(_) => "Database operation failed".to_string(),
            AppError::Config(_) => "Configuration error".to_string(),
            AppError::Validation(msg) => msg.clone(),
            AppError::Security(_) => "Security violation detected".to_string(),
            AppError::Memory(_) => "Memory operation failed".to_string(),
            AppError::Mqtt(_) => "MQTT operation failed".to_string(),
            AppError::Soul(_) => "Soul system operation failed".to_string(),
            AppError::Conscience(_) => "Conscience evaluation failed".to_string(),
            AppError::Backup(_) => "Backup operation failed".to_string(),
            AppError::Io(_) => "IO operation failed".to_string(),
            AppError::Serialization(_) => "Invalid data format".to_string(),
            AppError::Anyhow(_) => "An internal error occurred".to_string(),
        }
    }

    /// Get detailed error message (for logging, only in debug builds)
    pub fn detailed_message(&self) -> String {
        #[cfg(debug_assertions)]
        {
            self.to_string()
        }
        #[cfg(not(debug_assertions))]
        {
            self.sanitized_message()
        }
    }
}

/// Standard error response format
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

    /// Create from AppError
    pub fn from_app_error(err: &AppError) -> Self {
        Self::with_details(
            err.sanitized_message(),
            err.error_code().to_string(),
            err.detailed_message(),
        )
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let error_response = ErrorResponse::from_app_error(&self);

        // Log error details (always log full details internally)
        tracing::error!(
            error = %self,
            code = %self.error_code(),
            status = %status.as_u16(),
            "Request failed"
        );

        (status, Json(error_response)).into_response()
    }
}

/// Result type alias for convenience
pub type AppResult<T> = Result<T, AppError>;

/// Conversion utilities
impl From<&str> for AppError {
    fn from(msg: &str) -> Self {
        AppError::Internal(msg.to_string())
    }
}

impl From<String> for AppError {
    fn from(msg: String) -> Self {
        AppError::Internal(msg)
    }
}

/// Helper macro for creating errors
#[macro_export]
macro_rules! app_error {
    ($variant:ident, $msg:expr) => {
        $crate::error::AppError::$variant($msg.to_string())
    };
    ($variant:ident, $fmt:expr, $($arg:tt)*) => {
        $crate::error::AppError::$variant(format!($fmt, $($arg)*))
    };
}

/// Helper macro for converting anyhow errors
#[macro_export]
macro_rules! handle_anyhow {
    ($result:expr) => {
        $result.map_err(|e| $crate::error::AppError::Anyhow(e))
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        let err = AppError::BadRequest("test".to_string());
        assert_eq!(err.error_code(), "BAD_REQUEST");
        assert_eq!(err.status_code(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_sanitized_message() {
        let err = AppError::Internal("sensitive details".to_string());
        let msg = err.sanitized_message();
        assert_eq!(msg, "An internal error occurred");
        assert!(!msg.contains("sensitive"));
    }

    #[test]
    fn test_error_response() {
        let err = AppError::BadRequest("invalid input".to_string());
        let response = ErrorResponse::from_app_error(&err);
        assert_eq!(response.error, "invalid input");
        assert_eq!(response.code, "BAD_REQUEST");
    }
}

