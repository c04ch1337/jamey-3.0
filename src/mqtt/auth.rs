use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use thiserror::Error;

/// JWT authentication errors
#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Failed to generate JWT token: {0}")]
    TokenGeneration(#[from] jsonwebtoken::errors::Error),
    
    #[error("Token has expired")]
    TokenExpired,
    
    #[error("Invalid token")]
    InvalidToken,
    
    #[error("Missing required claim: {0}")]
    MissingClaim(String),
}

/// JWT claims for MQTT authentication
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MqttClaims {
    /// Subject (client identifier)
    pub sub: String,
    
    /// Expiration time (seconds since UNIX epoch)
    pub exp: u64,
    
    /// Issued at (seconds since UNIX epoch)
    pub iat: u64,
    
    /// Allowed topic patterns for this client
    #[serde(default)]
    pub permissions: Vec<String>,
}

impl MqttClaims {
    /// Create new claims with the given client ID and lifetime
    pub fn new(client_id: String, lifetime: Duration, permissions: Vec<String>) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();
        
        Self {
            sub: client_id,
            exp: now + lifetime.as_secs(),
            iat: now,
            permissions,
        }
    }
    
    /// Check if the token has expired
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();
        
        now >= self.exp
    }
    
    /// Get time until expiration
    pub fn time_until_expiry(&self) -> Option<Duration> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();
        
        if now >= self.exp {
            None
        } else {
            Some(Duration::from_secs(self.exp - now))
        }
    }
    
    /// Check if the client has permission for a topic
    pub fn has_permission(&self, topic: &str) -> bool {
        self.permissions.iter().any(|pattern| {
            mqtt_topic_matches(pattern, topic)
        })
    }
}

/// JWT token manager for MQTT authentication
#[derive(Clone)]
pub struct JwtManager {
    secret: String,
    algorithm: Algorithm,
    token_lifetime: Duration,
}

impl JwtManager {
    /// Create a new JWT manager with the given secret and token lifetime
    pub fn new(secret: String, token_lifetime: Duration) -> Self {
        Self {
            secret,
            algorithm: Algorithm::HS256,
            token_lifetime,
        }
    }
    
    /// Generate a new JWT token for the given client ID with specified permissions
    pub fn generate_token(
        &self,
        client_id: String,
        permissions: Vec<String>,
    ) -> Result<String, AuthError> {
        let claims = MqttClaims::new(client_id, self.token_lifetime, permissions);
        
        let header = Header::new(self.algorithm);
        let key = EncodingKey::from_secret(self.secret.as_bytes());
        
        encode(&header, &claims, &key).map_err(AuthError::from)
    }
    
    /// Validate and decode a JWT token
    pub fn validate_token(&self, token: &str) -> Result<MqttClaims, AuthError> {
        let key = DecodingKey::from_secret(self.secret.as_bytes());
        let mut validation = Validation::new(self.algorithm);
        validation.validate_exp = true;
        
        let token_data = decode::<MqttClaims>(token, &key, &validation)
            .map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
                _ => AuthError::InvalidToken,
            })?;
        
        Ok(token_data.claims)
    }
    
    /// Check if a token needs refresh (within 30 seconds of expiry)
    pub fn needs_refresh(&self, claims: &MqttClaims) -> bool {
        if let Some(time_left) = claims.time_until_expiry() {
            time_left < Duration::from_secs(30)
        } else {
            true // Already expired
        }
    }
    
    /// Get the configured token lifetime
    pub fn token_lifetime(&self) -> Duration {
        self.token_lifetime
    }
}

/// Check if an MQTT topic matches a pattern (supports + and # wildcards)
fn mqtt_topic_matches(pattern: &str, topic: &str) -> bool {
    let pattern_parts: Vec<&str> = pattern.split('/').collect();
    let topic_parts: Vec<&str> = topic.split('/').collect();
    
    let mut p_idx = 0;
    let mut t_idx = 0;
    
    while p_idx < pattern_parts.len() && t_idx < topic_parts.len() {
        match pattern_parts[p_idx] {
            "#" => {
                // Multi-level wildcard - matches everything remaining
                return true;
            }
            "+" => {
                // Single-level wildcard - matches one level
                p_idx += 1;
                t_idx += 1;
            }
            part if part == topic_parts[t_idx] => {
                // Exact match
                p_idx += 1;
                t_idx += 1;
            }
            _ => {
                // No match
                return false;
            }
        }
    }
    
    // Both must be exhausted for a match (unless pattern ends with #)
    p_idx == pattern_parts.len() && t_idx == topic_parts.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_jwt_generation_and_validation() {
        let manager = JwtManager::new("test-secret".to_string(), Duration::from_secs(300));
        let permissions = vec!["jamey/+/query".to_string(), "jamey/events/#".to_string()];
        
        let token = manager
            .generate_token("test-client".to_string(), permissions.clone())
            .expect("Failed to generate token");
        
        let claims = manager
            .validate_token(&token)
            .expect("Failed to validate token");
        
        assert_eq!(claims.sub, "test-client");
        assert_eq!(claims.permissions, permissions);
        assert!(!claims.is_expired());
    }
    
    #[test]
    fn test_topic_matching() {
        assert!(mqtt_topic_matches("jamey/+/query", "jamey/memory/query"));
        assert!(mqtt_topic_matches("jamey/events/#", "jamey/events/status"));
        assert!(mqtt_topic_matches("jamey/events/#", "jamey/events/status/critical"));
        assert!(!mqtt_topic_matches("jamey/+/query", "jamey/memory/store"));
        assert!(!mqtt_topic_matches("jamey/events/status", "jamey/events/status/critical"));
    }
    
    #[test]
    fn test_permission_checking() {
        let claims = MqttClaims::new(
            "test-client".to_string(),
            Duration::from_secs(300),
            vec!["jamey/+/query".to_string(), "jamey/events/#".to_string()],
        );
        
        assert!(claims.has_permission("jamey/memory/query"));
        assert!(claims.has_permission("jamey/events/status"));
        assert!(!claims.has_permission("jamey/memory/store"));
    }
}