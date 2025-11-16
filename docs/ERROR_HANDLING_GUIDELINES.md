# Error Handling Guidelines - Jamey 3.0

**Date:** 2025-01-27  
**Status:** ✅ **STANDARDIZED**

---

## Overview

Jamey 3.0 uses a standardized error handling system based on `thiserror` for type-safe error handling and consistent error responses across the application.

---

## Error Handling Principles

### 1. Use `thiserror` for All Public Error Types

All public-facing error types should use `thiserror::Error`:

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MyModuleError {
    #[error("Operation failed: {0}")]
    OperationFailed(String),
    
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}
```

### 2. Use `anyhow::Result` for Internal Code

For internal functions that don't expose errors to callers, `anyhow::Result` is acceptable:

```rust
use anyhow::Result;

async fn internal_function() -> Result<()> {
    // Internal implementation
    Ok(())
}
```

### 3. Convert to `AppError` at API Boundaries

When errors cross module boundaries (especially at API endpoints), convert them to `AppError`:

```rust
use crate::error::{AppError, AppResult};

async fn api_handler() -> AppResult<Json<Response>> {
    let result = internal_function().await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(result))
}
```

---

## Standard Error Response Format

All API errors follow this format:

```json
{
  "error": "User-friendly error message",
  "code": "ERROR_CODE",
  "details": "Detailed error (only in debug builds)"
}
```

### Error Response Fields

- **error**: User-friendly message (always present)
- **code**: Machine-readable error code (always present)
- **details**: Detailed error information (only in debug builds)

---

## Error Types

### AppError

The central error type for the application:

```rust
pub enum AppError {
    // HTTP Errors
    BadRequest(String),
    Unauthorized(String),
    Forbidden(String),
    NotFound(String),
    PayloadTooLarge(String),
    Internal(String),
    ServiceUnavailable(String),
    
    // System Errors
    Database(#[from] sqlx::Error),
    Config(String),
    Validation(String),
    Security(String),
    Memory(String),
    Mqtt(String),
    Soul(String),
    Conscience(String),
    Backup(String),
    Io(#[from] std::io::Error),
    Serialization(#[from] serde_json::Error),
    Anyhow(#[from] anyhow::Error),
}
```

### Module-Specific Errors

Modules can define their own error types, but should implement `From` conversions to `AppError`:

```rust
#[derive(Debug, Error)]
pub enum MyModuleError {
    #[error("Module error: {0}")]
    Error(String),
}

impl From<MyModuleError> for AppError {
    fn from(err: MyModuleError) -> Self {
        AppError::Internal(err.to_string())
    }
}
```

---

## Error Message Sanitization

### Production Safety

Error messages are automatically sanitized in production builds:

- **Debug builds**: Full error details included
- **Release builds**: Only sanitized messages (no internal details)

### Sanitization Rules

1. **Internal errors**: Always sanitized to generic message
2. **Database errors**: Sanitized to "Database operation failed"
3. **Security errors**: Sanitized to "Security violation detected"
4. **Validation errors**: User-friendly messages (not sanitized)
5. **Bad request errors**: User-friendly messages (not sanitized)

---

## Usage Examples

### API Endpoint Handler

```rust
use crate::error::{AppError, AppResult};
use axum::{Json, response::IntoResponse};

async fn my_endpoint() -> AppResult<Json<Response>> {
    // Validate input
    let input = validate_input()
        .map_err(|e| AppError::Validation(e.to_string()))?;
    
    // Process request
    let result = process_request(input).await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    
    Ok(Json(result))
}
```

### Converting anyhow Errors

```rust
use crate::error::{AppError, AppResult};
use crate::handle_anyhow;

async fn function_with_anyhow() -> AppResult<()> {
    let result: anyhow::Result<()> = some_function().await;
    handle_anyhow!(result)?;
    Ok(())
}
```

### Using Error Macros

```rust
use crate::app_error;

// Simple error
let err = app_error!(BadRequest, "Invalid input");

// Formatted error
let err = app_error!(Internal, "Operation failed: {}", operation_name);
```

---

## Error Logging

### Automatic Logging

All `AppError` instances are automatically logged when converted to HTTP responses:

```rust
// This automatically logs:
// - Error message
// - Error code
// - HTTP status code
// - Full error details (for debugging)
```

### Manual Logging

For errors that don't go through HTTP responses:

```rust
use tracing::error;

match result {
    Ok(value) => value,
    Err(e) => {
        error!(error = %e, "Operation failed");
        return Err(AppError::Internal(e.to_string()));
    }
}
```

---

## Best Practices

### ✅ DO

1. **Use `AppError` at API boundaries**
2. **Use `thiserror` for module-specific errors**
3. **Convert errors at module boundaries**
4. **Log errors before returning them**
5. **Use sanitized messages in production**
6. **Provide user-friendly error messages**

### ❌ DON'T

1. **Don't expose internal error details in production**
2. **Don't use `unwrap()` or `expect()` in production code**
3. **Don't return raw database errors to clients**
4. **Don't leak sensitive information in error messages**
5. **Don't use `anyhow::Result` in public APIs**

---

## Migration Guide

### Converting Existing Code

1. **Identify error types**: Find all `anyhow::Result` in public APIs
2. **Create error types**: Define `thiserror` error types for modules
3. **Add conversions**: Implement `From` conversions to `AppError`
4. **Update handlers**: Convert errors at API boundaries
5. **Test**: Verify error responses are properly formatted

### Example Migration

**Before:**
```rust
async fn handler() -> anyhow::Result<Json<Response>> {
    let result = process().await?;
    Ok(Json(result))
}
```

**After:**
```rust
use crate::error::{AppError, AppResult};

async fn handler() -> AppResult<Json<Response>> {
    let result = process().await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(result))
}
```

---

## Error Codes Reference

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `BAD_REQUEST` | 400 | Invalid request parameters |
| `UNAUTHORIZED` | 401 | Authentication required |
| `FORBIDDEN` | 403 | Access denied |
| `NOT_FOUND` | 404 | Resource not found |
| `PAYLOAD_TOO_LARGE` | 413 | Request too large |
| `INTERNAL_ERROR` | 500 | Internal server error |
| `SERVICE_UNAVAILABLE` | 503 | Service temporarily unavailable |
| `DATABASE_ERROR` | 500 | Database operation failed |
| `CONFIG_ERROR` | 500 | Configuration error |
| `VALIDATION_ERROR` | 400 | Input validation failed |
| `SECURITY_ERROR` | 403 | Security violation |
| `MEMORY_ERROR` | 500 | Memory system error |
| `MQTT_ERROR` | 503 | MQTT operation failed |
| `SOUL_ERROR` | 500 | Soul system error |
| `CONSCIENCE_ERROR` | 500 | Conscience evaluation failed |
| `BACKUP_ERROR` | 500 | Backup operation failed |
| `IO_ERROR` | 500 | IO operation failed |
| `SERIALIZATION_ERROR` | 400 | Invalid data format |

---

## Testing Error Handling

### Unit Tests

```rust
#[test]
fn test_error_conversion() {
    let err = AppError::BadRequest("test".to_string());
    assert_eq!(err.error_code(), "BAD_REQUEST");
    assert_eq!(err.status_code(), StatusCode::BAD_REQUEST);
}

#[test]
fn test_error_sanitization() {
    let err = AppError::Internal("sensitive".to_string());
    let msg = err.sanitized_message();
    assert!(!msg.contains("sensitive"));
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_error_response() {
    let app = create_test_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/invalid-endpoint")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let error: ErrorResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(error.code, "NOT_FOUND");
}
```

---

## Summary

- ✅ **Standardized**: All errors use `thiserror` and `AppError`
- ✅ **Safe**: Error messages sanitized in production
- ✅ **Consistent**: Uniform error response format
- ✅ **Type-safe**: Compile-time error checking
- ✅ **Logged**: Automatic error logging
- ✅ **Documented**: Clear error codes and messages

---

**Last Updated:** 2025-01-27

