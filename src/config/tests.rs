use super::*;
use std::env;

fn setup() {
    env::remove_var("OPENROUTER_API_KEY");
    env::remove_var("DATABASE_URL");
    env::remove_var("MQTT_BROKER_URL");
    env::remove_var("PHOENIX_ENABLED");
    env::remove_var("SOUL_ENABLED");
    env::remove_var("API_KEY");
    env::remove_var("PORT");
    env::remove_var("HOST");
}

#[test]
fn test_core_config_required_fields() {
    setup();
    
    env::set_var("OPENROUTER_API_KEY", "test-key");
    let config = Config::from_env().unwrap().unwrap();
    
    assert_eq!(config.core.openrouter_api_key, "test-key");
    assert_eq!(config.core.openrouter_model, "anthropic/claude-3.5-sonnet"); // Default model
    assert_eq!(config.core.openrouter_api_url, "https://openrouter.ai/api/v1"); // Default URL
    assert_eq!(config.core.database_url, None); // Optional field
    assert_eq!(config.core.data_dir, None); // Optional field
}

#[test]
fn test_core_config_optional_fields() {
    setup();
    
    env::set_var("OPENROUTER_API_KEY", "test-key");
    env::set_var("OPENROUTER_MODEL", "test-model");
    env::set_var("OPENROUTER_API_URL", "https://test.api/v1");
    env::set_var("DATABASE_URL", "test.db");
    env::set_var("DATA_DIR", "/test/data");

    let config = Config::from_env().unwrap().unwrap();
    
    assert_eq!(config.core.openrouter_api_key, "test-key");
    assert_eq!(config.core.openrouter_model, "test-model");
    assert_eq!(config.core.openrouter_api_url, "https://test.api/v1");
    assert_eq!(config.core.database_url, Some("test.db".to_string()));
    assert_eq!(config.core.data_dir, Some("/test/data".to_string()));
}

#[test]
fn test_core_config_missing_required() {
    setup();
    
    // Don't set OPENROUTER_API_KEY
    let config = Config::from_env().unwrap();
    assert!(config.is_none(), "Config should be None when OPENROUTER_API_KEY is missing");
}

#[test]
fn test_core_config_empty_api_key() {
    setup();
    
    env::set_var("OPENROUTER_API_KEY", "");
    let config = Config::from_env().unwrap();
    assert!(config.is_none(), "Config should be None when OPENROUTER_API_KEY is empty");
}

#[test]
fn test_operational_config() {
    setup();
    
    env::set_var("PORT", "8080");
    env::set_var("HOST", "127.0.0.1");
    env::set_var("DEV_MODE", "true");
    env::set_var("ENABLE_TEST_FEATURES", "true");
    env::set_var("RUST_LOG", "debug");

    let config = Config::from_env().unwrap().unwrap();
    
    assert_eq!(config.operational.port, 8080);
    assert_eq!(config.operational.host, "127.0.0.1");
    assert!(config.operational.dev_mode);
    assert!(config.operational.enable_test_features);
    assert_eq!(config.operational.log_level, "debug");
}

#[test]
fn test_soul_config_defaults() {
    setup();
    let config = Config::from_env().unwrap().unwrap();
    let soul = config.soul;
    
    // Check default values
    assert_eq!(soul.default_trust, 0.5);
    assert_eq!(soul.base_decay_rate, 0.01);
    assert_eq!(soul.prune_threshold, 0.1);
    assert_eq!(soul.empathy_threshold, 0.7);
    assert!(soul.auto_record_emotions);
}

#[test]
fn test_soul_config_custom_values() {
    setup();
    
    env::set_var("SOUL_DEFAULT_TRUST", "0.7");
    env::set_var("SOUL_BASE_DECAY_RATE", "0.02");
    env::set_var("SOUL_PRUNE_THRESHOLD", "0.2");
    env::set_var("SOUL_EMPATHY_THRESHOLD", "0.8");
    env::set_var("SOUL_AUTO_RECORD", "false");

    let config = Config::from_env().unwrap().unwrap();
    let soul = config.soul;
    
    assert_eq!(soul.default_trust, 0.7);
    assert_eq!(soul.base_decay_rate, 0.02);
    assert_eq!(soul.prune_threshold, 0.2);
    assert_eq!(soul.empathy_threshold, 0.8);
    assert!(!soul.auto_record_emotions);
}

#[test]
fn test_soul_config_invalid_values() {
    setup();
    
    // Test values outside valid ranges
    env::set_var("SOUL_DEFAULT_TRUST", "1.5"); // > 1.0
    env::set_var("SOUL_BASE_DECAY_RATE", "-0.1"); // < 0.0
    env::set_var("SOUL_PRUNE_THRESHOLD", "2.0"); // > 1.0
    env::set_var("SOUL_EMPATHY_THRESHOLD", "-0.5"); // < 0.0

    let config = Config::from_env().unwrap().unwrap();
    let validation_result = config.validate();
    assert!(validation_result.is_err(), "Validation should fail with invalid soul parameters");
}

#[test]
fn test_soul_config_edge_cases() {
    setup();
    
    // Test boundary values
    env::set_var("SOUL_DEFAULT_TRUST", "1.0"); // Maximum
    env::set_var("SOUL_BASE_DECAY_RATE", "0.0"); // Minimum
    env::set_var("SOUL_PRUNE_THRESHOLD", "1.0"); // Maximum
    env::set_var("SOUL_EMPATHY_THRESHOLD", "0.0"); // Minimum

    let config = Config::from_env().unwrap().unwrap();
    let soul = config.soul;
    
    assert_eq!(soul.default_trust, 1.0);
    assert_eq!(soul.base_decay_rate, 0.0);
    assert_eq!(soul.prune_threshold, 1.0);
    assert_eq!(soul.empathy_threshold, 0.0);
}

#[test]
fn test_phoenix_config() {
    setup();
    
    env::set_var("PHOENIX_ENABLED", "true");
    env::set_var("PHOENIX_BACKUP_DIR", "/test/backups");
    env::set_var("PHOENIX_ENCRYPTION_KEY", "test-encryption-key");
    env::set_var("PHOENIX_AUTO_BACKUP_HOURS", "12");
    env::set_var("PHOENIX_MAX_BACKUPS", "5");

    let config = Config::from_env().unwrap().unwrap();
    let phoenix = config.phoenix.unwrap();
    
    assert!(phoenix.enabled);
    assert_eq!(phoenix.backup_dir, "/test/backups");
    assert_eq!(phoenix.encryption_key, "test-encryption-key");
    assert_eq!(phoenix.auto_backup_hours, 12);
    assert_eq!(phoenix.max_backups, 5);
}

#[test]
fn test_security_config_defaults() {
    setup();
    let config = Config::from_env().unwrap().unwrap();
    
    // Check default values
    assert_eq!(config.security.api_key, None);
    assert_eq!(config.security.allowed_origins, vec!["http://localhost:5173", "http://localhost:3000"]);
    assert_eq!(config.security.rate_limit_per_minute, 60);
    assert_eq!(config.security.max_action_length, 10_000);
    assert_eq!(config.security.max_rule_name_length, 100);
    assert_eq!(config.security.max_rule_description_length, 500);
    assert_eq!(config.security.min_rule_weight, 0.0);
    assert_eq!(config.security.max_rule_weight, 100.0);
}

#[test]
fn test_security_config_custom_values() {
    setup();
    
    env::set_var("API_KEY", "test-api-key");
    env::set_var("ALLOWED_ORIGINS", "https://test.com,https://api.test.com");
    env::set_var("RATE_LIMIT_PER_MINUTE", "100");
    env::set_var("MAX_ACTION_LENGTH", "5000");
    env::set_var("MAX_RULE_NAME_LENGTH", "50");
    env::set_var("MAX_RULE_DESCRIPTION_LENGTH", "250");
    env::set_var("MIN_RULE_WEIGHT", "1.0");
    env::set_var("MAX_RULE_WEIGHT", "50.0");

    let config = Config::from_env().unwrap().unwrap();
    
    assert_eq!(config.security.api_key, Some("test-api-key".to_string()));
    assert_eq!(config.security.allowed_origins, vec!["https://test.com", "https://api.test.com"]);
    assert_eq!(config.security.rate_limit_per_minute, 100);
    assert_eq!(config.security.max_action_length, 5000);
    assert_eq!(config.security.max_rule_name_length, 50);
    assert_eq!(config.security.max_rule_description_length, 250);
    assert_eq!(config.security.min_rule_weight, 1.0);
    assert_eq!(config.security.max_rule_weight, 50.0);
}

#[test]
fn test_security_config_production_validation() {
    setup();
    
    // Set dev_mode to false to test production requirements
    env::set_var("DEV_MODE", "false");
    
    let config = Config::from_env().unwrap().unwrap();
    let validation_result = config.validate();
    assert!(validation_result.is_err(), "Validation should fail in production mode without required security settings");
    
    // Add required security settings
    env::set_var("API_KEY", "test-api-key");
    env::set_var("ALLOWED_ORIGINS", "https://prod.com");
    
    let config = Config::from_env().unwrap().unwrap();
    let validation_result = config.validate();
    assert!(validation_result.is_ok(), "Validation should pass with required security settings");
}

#[test]
fn test_security_config_allowed_origins_empty() {
    setup();
    
    env::set_var("ALLOWED_ORIGINS", "");
    let config = Config::from_env().unwrap().unwrap();
    
    // In dev mode, should fall back to default localhost origins
    assert_eq!(config.security.allowed_origins, vec!["http://localhost:5173", "http://localhost:3000"]);
}

#[test]
fn test_validation() {
    setup();
    
    // Test invalid values
    env::set_var("SOUL_DEFAULT_TRUST", "2.0"); // Invalid: > 1.0
    env::set_var("PHOENIX_MAX_BACKUPS", "0"); // Invalid: == 0
    env::set_var("PORT", "999999"); // Invalid: > 65535

    let config = Config::from_env().unwrap().unwrap();
    assert!(config.validate().is_err());

    setup();
    
    // Test valid values
    env::set_var("SOUL_DEFAULT_TRUST", "0.5");
    env::set_var("PHOENIX_MAX_BACKUPS", "10");
    env::set_var("PORT", "8080");

    let config = Config::from_env().unwrap().unwrap();
    assert!(config.validate().is_ok());
}

#[test]
fn test_backward_compatibility() {
    setup();
    
    // Test old environment variables still work
    env::set_var("OPENROUTER_API_KEY", "test-key");
    env::set_var("MQTT_BROKER_URL", "mqtt://localhost");
    env::set_var("SOUL_DEFAULT_TRUST", "0.5");

    let config = Config::from_env().unwrap().unwrap();
    
    assert_eq!(config.core.openrouter_api_key, "test-key");
    assert!(config.mqtt.is_some());
    assert!(config.soul.is_some());
}