//! Security middleware and utilities for Jamey 3.0
//!
//! This module provides security features including:
//! - JWT authentication middleware
//! - Security headers middleware
//! - Rate limiting middleware
//! - Input validation utilities
//! - Security event logging

pub mod auth;
pub mod headers;
pub mod rate_limit;
pub mod validation;

pub use auth::{JwtAuth, JwtClaims, AuthError};
pub use headers::SecurityHeadersLayer;
pub use rate_limit::RateLimitLayer;
pub use validation::validate_input;