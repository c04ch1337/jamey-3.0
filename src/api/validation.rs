//! Input validation utilities
//!
//! Provides validation functions for API request data.

use axum::http::StatusCode;
use std::char;

/// Sanitize a string by removing control characters
pub fn sanitize_string(input: &str) -> String {
    input
        .chars()
        .filter(|c| !c.is_control() || c.is_whitespace())
        .collect()
}

/// Validate and sanitize action string
pub fn validate_action(
    action: &str,
    max_length: usize,
) -> Result<String, (StatusCode, &'static str)> {
    if action.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Action cannot be empty"));
    }

    if action.len() > max_length {
        return Err((
            StatusCode::PAYLOAD_TOO_LARGE,
            "Action exceeds maximum length",
        ));
    }

    let sanitized = sanitize_string(action);
    if sanitized.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Action contains only control characters"));
    }

    Ok(sanitized)
}

/// Validate rule name
pub fn validate_rule_name(
    name: &str,
    max_length: usize,
) -> Result<String, (StatusCode, &'static str)> {
    if name.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Rule name cannot be empty"));
    }

    if name.len() > max_length {
        return Err((
            StatusCode::PAYLOAD_TOO_LARGE,
            "Rule name exceeds maximum length",
        ));
    }

    // Rule names should be alphanumeric with hyphens and underscores
    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err((
            StatusCode::BAD_REQUEST,
            "Rule name must contain only alphanumeric characters, hyphens, and underscores",
        ));
    }

    Ok(sanitize_string(name))
}

/// Validate rule description
pub fn validate_rule_description(
    description: &str,
    max_length: usize,
) -> Result<String, (StatusCode, &'static str)> {
    if description.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Rule description cannot be empty"));
    }

    if description.len() > max_length {
        return Err((
            StatusCode::PAYLOAD_TOO_LARGE,
            "Rule description exceeds maximum length",
        ));
    }

    Ok(sanitize_string(description))
}

/// Validate rule weight
pub fn validate_rule_weight(
    weight: f32,
    min: f32,
    max: f32,
) -> Result<f32, (StatusCode, &'static str)> {
    if weight.is_nan() || weight.is_infinite() {
        return Err((StatusCode::BAD_REQUEST, "Rule weight must be a valid number"));
    }

    if weight < min || weight > max {
        return Err((
            StatusCode::BAD_REQUEST,
            "Rule weight is outside allowed range",
        ));
    }

    Ok(weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_string() {
        let input = "Hello\x00World\n";
        let result = sanitize_string(input);
        assert_eq!(result, "HelloWorld\n");
    }

    #[test]
    fn test_validate_action() {
        // Valid action
        assert!(validate_action("Help someone", 100).is_ok());

        // Empty action
        assert!(validate_action("", 100).is_err());

        // Too long
        assert!(validate_action(&"a".repeat(10001), 10000).is_err());
    }

    #[test]
    fn test_validate_rule_name() {
        // Valid names
        assert!(validate_rule_name("no-harm", 100).is_ok());
        assert!(validate_rule_name("rule_123", 100).is_ok());

        // Invalid names
        assert!(validate_rule_name("rule with spaces", 100).is_err());
        assert!(validate_rule_name("", 100).is_err());
    }
}

