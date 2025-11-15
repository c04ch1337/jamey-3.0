//! Security middleware and utilities for Jamey 3.0
//!
//! This module provides security features including:
//! - JWT authentication middleware
//! - Security headers middleware
//! - Rate limiting middleware
//! - DDoS protection
//! - Input validation utilities
//! - Security event logging

pub mod auth;
pub mod headers;
pub mod rate_limit;
pub mod validation;
pub mod ddos_protection;
pub mod threat_detection;
pub mod incident_response;
pub mod security_middleware;
pub mod secret_rotation;
pub mod compliance;
pub mod csrf;

pub use auth::{JwtAuth, JwtClaims, AuthError};
pub use headers::SecurityHeadersLayer;
// Re-export tower's RateLimitLayer for external use.
pub use tower::limit::RateLimitLayer;
pub use validation::validate_input;
pub use ddos_protection::{DdosProtection, DdosProtectionConfig, ddos_protection_middleware, DdosError};
pub use threat_detection::{ThreatDetection, ThreatDetectionConfig, ThreatEvent, ThreatType, ThreatSeverity};
pub use incident_response::{IncidentResponse, IncidentResponseConfig, SecurityIncident, IncidentType, IncidentStatus};
pub use security_middleware::security_middleware;
pub use secret_rotation::{SecretRotationManager, SecretRotationConfig, SecretType, RotationPolicy};
pub use compliance::{ComplianceManager, ComplianceFramework, ComplianceReport, ComplianceStatus};
pub use csrf::{CsrfProtection, CsrfConfig, csrf_middleware, get_csrf_token};