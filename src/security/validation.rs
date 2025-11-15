//! Input validation utilities for Jamey 3.0
//!
//! Provides comprehensive input validation and sanitization for API endpoints.

use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError, ValidationErrors};
use tracing::{error, warn};

/// Maximum allowed string lengths
pub mod limits {
    pub const MAX_ACTION_LENGTH: usize = 1000;
    pub const MAX_RULE_NAME_LENGTH: usize = 100;
    pub const MAX_RULE_DESCRIPTION_LENGTH: usize = 500;
    pub const MAX_CONTENT_LENGTH: usize = 10000;
    pub const MAX_USERNAME_LENGTH: usize = 50;
    pub const MAX_PASSWORD_LENGTH: usize = 128;
    pub const MIN_PASSWORD_LENGTH: usize = 8;
}

/// Common validation patterns
pub mod patterns {
    pub const ALPHANUMERIC: &str = r"^[a-zA-Z0-9]*$";
    pub const SAFE_STRING: &str = r"^[a-zA-Z0-9\s\-_.,!?@#$%^&*()]*$";
    pub const USERNAME: &str = r"^[a-zA-Z0-9_-]{3,50}$";
    pub const EMAIL: &str = r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$";
}

/// Validate and sanitize input string
pub fn validate_string(input: &str, max_length: usize, pattern: Option<&str>) -> Result<String, ValidationError> {
    // Check length
    if input.len() > max_length {
        return Err(ValidationError::new("string_too_long"));
    }

    // Check pattern if provided
    if let Some(pattern) = pattern {
        let regex = regex::Regex::new(pattern).map_err(|_| ValidationError::new("invalid_pattern"))?;
        if !regex.is_match(input) {
            return Err(ValidationError::new("pattern_mismatch"));
        }
    }

    // Basic sanitization - remove potentially dangerous characters
    let sanitized = input
        .chars()
        .filter(|c| !matches!(c, '\0' | '\x01'..='\x08' | '\x0B'..='\x0C' | '\x0E'..='\x1F' | '\x7F'))
        .collect::<String>();

    Ok(sanitized)
}

/// Validate action input for conscience evaluation
#[derive(Debug, Deserialize, Validate)]
pub struct ActionInput {
    #[validate(length(max = "limits::MAX_ACTION_LENGTH", message = "Action too long"))]
    #[validate(regex(path = "patterns::SAFE_STRING", message = "Invalid characters in action"))]
    pub action: String,
}

/// Validate rule input for conscience rules
#[derive(Debug, Deserialize, Validate)]
pub struct RuleInput {
    #[validate(length(max = "limits::MAX_RULE_NAME_LENGTH", message = "Rule name too long"))]
    #[validate(regex(path = "patterns::SAFE_STRING", message = "Invalid characters in rule name"))]
    pub name: String,

    #[validate(length(max = "limits::MAX_RULE_DESCRIPTION_LENGTH", message = "Description too long"))]
    #[validate(regex(path = "patterns::SAFE_STRING", message = "Invalid characters in description"))]
    pub description: String,

    #[validate(range(min = 0.0, max = 1.0, message = "Weight must be between 0.0 and 1.0"))]
    pub weight: f32,
}

/// Validate content input for consciousness processing
#[derive(Debug, Deserialize, Validate)]
pub struct ContentInput {
    #[validate(length(max = "limits::MAX_CONTENT_LENGTH", message = "Content too long"))]
    #[validate(regex(path = "patterns::SAFE_STRING", message = "Invalid characters in content"))]
    pub content: String,
}

/// Validate login input
#[derive(Debug, Deserialize, Validate)]
pub struct LoginInput {
    #[validate(length(max = "limits::MAX_USERNAME_LENGTH", message = "Username too long"))]
    #[validate(regex(path = "patterns::USERNAME", message = "Invalid username format"))]
    pub username: String,

    #[validate(length(min = "limits::MIN_PASSWORD_LENGTH", max = "limits::MAX_PASSWORD_LENGTH", message = "Invalid password length"))]
    pub password: String,
}

/// Validate toggle subsystem input
#[derive(Debug, Deserialize, Validate)]
pub struct ToggleSubsystemInput {
    pub enable_higher_order: Option<bool>,
    pub enable_predictive: Option<bool>,
    pub enable_attention: Option<bool>,
}

/// Generic validation function for any input
pub fn validate_input<T: Validate>(input: &T) -> Result<(), ValidationErrors> {
    input.validate()
}

/// Sanitize and validate MQTT message content
pub fn validate_mqtt_message(topic: &str, payload: &str) -> Result<(String, String), ValidationError> {
    // Validate topic format
    let topic_regex = regex::Regex::new(r"^[a-zA-Z0-9_\-/#+$]*$")
        .map_err(|_| ValidationError::new("invalid_topic_pattern"))?;
    
    if !topic_regex.is_match(topic) {
        return Err(ValidationError::new("invalid_topic_format"));
    }

    // Validate topic length
    if topic.len() > 256 {
        return Err(ValidationError::new("topic_too_long"));
    }

    // Validate payload length
    if payload.len() > 10000 {
        return Err(ValidationError::new("payload_too_long"));
    }

    // Sanitize payload
    let sanitized_payload = payload
        .chars()
        .filter(|c| !matches!(c, '\0' | '\x01'..='\x08' | '\x0B'..='\x0C' | '\x0E'..='\x1F' | '\x7F'))
        .collect::<String>();

    Ok((topic.to_string(), sanitized_payload))
}

/// Security validation result
#[derive(Debug, Serialize)]
pub struct SecurityValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl SecurityValidationResult {
    pub fn success() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn failure(errors: Vec<String>) -> Self {
        Self {
            is_valid: false,
            errors,
            warnings: Vec::new(),
        }
    }

    pub fn with_warnings(mut self, warnings: Vec<String>) -> Self {
        self.warnings = warnings;
        self
    }
}

/// Comprehensive security validation for API requests
pub fn validate_request_security(
    user_agent: Option<&str>,
    ip_address: Option<&str>,
    content_type: Option<&str>,
) -> SecurityValidationResult {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // Check user agent
    if let Some(user_agent) = user_agent {
        if user_agent.is_empty() {
            warnings.push("Empty User-Agent header".to_string());
        }
        
        // Check for suspicious user agents
        let suspicious_patterns = [
            "sqlmap", "nikto", "nmap", "masscan", "zap", "burp", "scanner", "bot", "crawler"
        ];
        
        let user_agent_lower = user_agent.to_lowercase();
        for pattern in &suspicious_patterns {
            if user_agent_lower.contains(pattern) {
                warnings.push(format!("Suspicious User-Agent detected: {}", pattern));
                break;
            }
        }
    } else {
        warnings.push("Missing User-Agent header".to_string());
    }

    // Check content type for POST/PUT requests
    if let Some(content_type) = content_type {
        if !content_type.starts_with("application/json") && 
           !content_type.starts_with("text/plain") &&
           !content_type.starts_with("multipart/form-data") {
            warnings.push(format!("Unusual Content-Type: {}", content_type));
        }
    }

    // Log security validation results
    if !errors.is_empty() {
        error!("Security validation failed: {:?}", errors);
    }
    if !warnings.is_empty() {
        warn!("Security validation warnings: {:?}", warnings);
    }

    if errors.is_empty() {
        SecurityValidationResult::success().with_warnings(warnings)
    } else {
        SecurityValidationResult::failure(errors)
    }
}

/// Middleware function to validate request security
pub async fn validate_request_security_middleware(
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, axum::http::StatusCode> {
    let user_agent = request.headers().get("user-agent")
        .and_then(|h| h.to_str().ok());
    
    let content_type = request.headers().get("content-type")
        .and_then(|h| h.to_str().ok());

    // Note: IP address would need to be extracted from connection info
    // This is a simplified version
    let validation_result = validate_request_security(user_agent, None, content_type);

    if !validation_result.is_valid {
        error!("Request security validation failed: {:?}", validation_result.errors);
        return Err(axum::http::StatusCode::BAD_REQUEST);
    }

    if !validation_result.warnings.is_empty() {
        warn!("Security validation warnings: {:?}", validation_result.warnings);
    }

    Ok(next.run(request).await)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_string_success() {
        let result = validate_string("hello world", 100, None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "hello world");
    }

    #[test]
    fn test_validate_string_too_long() {
        let long_string = "a".repeat(101);
        let result = validate_string(&long_string, 100, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_string_pattern() {
        let result = validate_string("hello123", 100, Some(patterns::ALPHANUMERIC));
        assert!(result.is_ok());

        let result = validate_string("hello@123", 100, Some(patterns::ALPHANUMERIC));
        assert!(result.is_err());
    }

    #[test]
    fn test_action_input_validation() {
        let valid_input = ActionInput {
            action: "Help someone in need".to_string(),
        };
        assert!(validate_input(&valid_input).is_ok());

        let invalid_input = ActionInput {
            action: "a".repeat(1001),
        };
        assert!(validate_input(&invalid_input).is_err());
    }

    #[test]
    fn test_login_input_validation() {
        let valid_input = LoginInput {
            username: "testuser".to_string(),
            password: "password123".to_string(),
        };
        assert!(validate_input(&valid_input).is_ok());

        let invalid_input = LoginInput {
            username: "test@user".to_string(),
            password: "123".to_string(),
        };
        assert!(validate_input(&invalid_input).is_err());
    }

    #[test]
    fn test_mqtt_message_validation() {
        let result = validate_mqtt_message("jamey/test", "Hello world");
        assert!(result.is_ok());

        let result = validate_mqtt_message("jamey\x00/test", "Hello world");
        assert!(result.is_err());

        let result = validate_mqtt_message("jamey/test", &"a".repeat(10001));
        assert!(result.is_err());
    }

    #[test]
    fn test_request_security_validation() {
        let result = validate_request_security(
            Some("Mozilla/5.0"),
            None,
            Some("application/json")
        );
        assert!(result.is_valid);

        let result = validate_request_security(
            Some("sqlmap/1.0"),
            None,
            Some("application/json")
        );
        assert!(result.is_valid); // Should be valid but with warnings
        assert!(!result.warnings.is_empty());
    }
}