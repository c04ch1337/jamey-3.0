//! Advanced Threat Detection System for Jamey 3.0
//!
//! Provides behavioral analysis, anomaly detection, and threat intelligence
//! to identify and respond to security threats in real-time.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use std::net::IpAddr;
use tracing::warn;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

 /// Threat severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ThreatSeverity {
    Low,
    Medium,
    High,
    Critical,
}

 /// Threat types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ThreatType {
    /// Rapid-fire requests from same IP
    RapidFireRequests,
    /// Unusual access pattern
    UnusualAccessPattern,
    /// Privilege escalation attempt
    PrivilegeEscalation,
    /// Data exfiltration pattern
    DataExfiltration,
    /// Suspicious user agent
    SuspiciousUserAgent,
    /// Known malicious IP
    KnownMaliciousIp,
    /// Brute force attack
    BruteForce,
    /// SQL injection attempt
    SqlInjection,
    /// XSS attempt
    CrossSiteScripting,
}

 /// Threat event
 ///
 /// This type is now serialized as part of `SecurityIncident` for incident
 /// response and audit logging. We use a `DateTime<Utc>` timestamp so it
 /// can be encoded/decoded via Serde reliably across processes.
 #[derive(Debug, Clone, Serialize, Deserialize)]
 pub struct ThreatEvent {
    pub threat_type: ThreatType,
    pub severity: ThreatSeverity,
    pub ip: IpAddr,
    pub timestamp: DateTime<Utc>,
    pub details: String,
    pub confidence: f64, // 0.0 to 1.0
}

/// Behavioral pattern tracker
#[derive(Debug, Clone)]
struct BehavioralPattern {
    request_count: u32,
    unique_endpoints: std::collections::HashSet<String>,
    error_count: u32,
    last_request: Instant,
    first_seen: Instant,
}

/// Threat detection configuration
#[derive(Debug, Clone)]
pub struct ThreatDetectionConfig {
    /// Enable behavioral analysis
    pub enable_behavioral_analysis: bool,
    /// Enable anomaly detection
    pub enable_anomaly_detection: bool,
    /// Threshold for rapid-fire detection (requests per second)
    pub rapid_fire_threshold: u32,
    /// Threshold for unusual access pattern
    pub unusual_pattern_threshold: f64,
    /// Enable automatic threat response
    pub enable_auto_response: bool,
}

impl Default for ThreatDetectionConfig {
    fn default() -> Self {
        Self {
            enable_behavioral_analysis: true,
            enable_anomaly_detection: true,
            rapid_fire_threshold: 10,
            unusual_pattern_threshold: 0.7,
            enable_auto_response: true,
        }
    }
}

impl ThreatDetectionConfig {
    /// Create config from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();
        
        if let Ok(enabled) = std::env::var("THREAT_ENABLE_BEHAVIORAL") {
            config.enable_behavioral_analysis = enabled.parse().unwrap_or(true);
        }
        
        if let Ok(enabled) = std::env::var("THREAT_ENABLE_ANOMALY") {
            config.enable_anomaly_detection = enabled.parse().unwrap_or(true);
        }
        
        if let Ok(threshold) = std::env::var("THREAT_RAPID_FIRE_THRESHOLD") {
            if let Ok(parsed) = threshold.parse::<u32>() {
                config.rapid_fire_threshold = parsed;
            }
        }
        
        if let Ok(threshold) = std::env::var("THREAT_UNUSUAL_PATTERN_THRESHOLD") {
            if let Ok(parsed) = threshold.parse::<f64>() {
                config.unusual_pattern_threshold = parsed;
            }
        }
        
        if let Ok(enabled) = std::env::var("THREAT_ENABLE_AUTO_RESPONSE") {
            config.enable_auto_response = enabled.parse().unwrap_or(true);
        }
        
        config
    }
}

/// Threat detection system
#[derive(Clone)]
pub struct ThreatDetection {
    config: ThreatDetectionConfig,
    behavioral_patterns: Arc<Mutex<HashMap<IpAddr, BehavioralPattern>>>,
    threat_events: Arc<Mutex<Vec<ThreatEvent>>>,
    known_malicious_ips: Arc<Mutex<std::collections::HashSet<IpAddr>>>,
}

impl ThreatDetection {
    /// Create new threat detection system
    pub fn new(config: ThreatDetectionConfig) -> Self {
        let detection = Self {
            config,
            behavioral_patterns: Arc::new(Mutex::new(HashMap::new())),
            threat_events: Arc::new(Mutex::new(Vec::new())),
            known_malicious_ips: Arc::new(Mutex::new(std::collections::HashSet::new())),
        };
        
        // Start cleanup task
        let cleanup_detection = detection.clone();
        tokio::spawn(async move {
            cleanup_detection.cleanup_task().await;
        });
        
        detection
    }
    
    /// Analyze request for threats
    pub fn analyze_request(
        &self,
        ip: IpAddr,
        endpoint: &str,
        user_agent: Option<&str>,
        is_error: bool,
    ) -> Vec<ThreatEvent> {
        let mut threats = Vec::new();
        
        if self.config.enable_behavioral_analysis {
            threats.extend(self.analyze_behavior(ip, endpoint, is_error));
        }
        
        if self.config.enable_anomaly_detection {
            threats.extend(self.detect_anomalies(ip, endpoint, user_agent));
        }
        
        // Check known malicious IPs
        if self.is_known_malicious(ip) {
            threats.push(ThreatEvent {
                threat_type: ThreatType::KnownMaliciousIp,
                severity: ThreatSeverity::High,
                ip,
                timestamp: Utc::now(),
                details: "IP is in known malicious IP database".to_string(),
                confidence: 0.9,
            });
        }
        
        // Record threats
        if !threats.is_empty() {
            let mut events = self.threat_events.lock().unwrap();
            events.extend(threats.clone());
            
            // Keep only last 1000 events
            let len = events.len();
            if len > 1000 {
                events.drain(0..len - 1000);
            }
        }
        
        threats
    }
    
    /// Analyze behavioral patterns
    fn analyze_behavior(
        &self,
        ip: IpAddr,
        endpoint: &str,
        is_error: bool,
    ) -> Vec<ThreatEvent> {
        let mut threats = Vec::new();
        let mut patterns = self.behavioral_patterns.lock().unwrap();
        let now = Instant::now();
        
        // Get or create pattern
        let pattern = patterns.entry(ip).or_insert_with(|| BehavioralPattern {
            request_count: 0,
            unique_endpoints: std::collections::HashSet::new(),
            error_count: 0,
            last_request: now,
            first_seen: now,
        });
        
        // Update pattern
        pattern.request_count += 1;
        pattern.unique_endpoints.insert(endpoint.to_string());
        if is_error {
            pattern.error_count += 1;
        }
        pattern.last_request = now;
        
        // Detect rapid-fire requests
        let time_window = now.duration_since(pattern.first_seen);
        if time_window.as_secs() > 0 {
            let requests_per_second = pattern.request_count as f64 / time_window.as_secs() as f64;
            if requests_per_second > self.config.rapid_fire_threshold as f64 {
                threats.push(ThreatEvent {
                    threat_type: ThreatType::RapidFireRequests,
                    severity: ThreatSeverity::High,
                    ip,
                    timestamp: Utc::now(),
                    details: format!(
                        "Rapid-fire requests detected: {:.2} req/s (threshold: {})",
                        requests_per_second,
                        self.config.rapid_fire_threshold
                    ),
                    confidence: 0.8,
                });
            }
        }
        
        // Detect unusual access patterns
        let endpoint_diversity = pattern.unique_endpoints.len() as f64 / pattern.request_count as f64;
        if endpoint_diversity < self.config.unusual_pattern_threshold {
            threats.push(ThreatEvent {
                threat_type: ThreatType::UnusualAccessPattern,
                severity: ThreatSeverity::Medium,
                ip,
                timestamp: Utc::now(),
                details: format!(
                    "Unusual access pattern: low endpoint diversity ({:.2})",
                    endpoint_diversity
                ),
                confidence: 0.6,
            });
        }
        
        // Detect brute force (high error rate)
        if pattern.request_count > 10 {
            let error_rate = pattern.error_count as f64 / pattern.request_count as f64;
            if error_rate > 0.5 {
                threats.push(ThreatEvent {
                    threat_type: ThreatType::BruteForce,
                    severity: ThreatSeverity::High,
                    ip,
                    timestamp: Utc::now(),
                    details: format!("High error rate: {:.2}%", error_rate * 100.0),
                    confidence: 0.7,
                });
            }
        }
        
        threats
    }
    
    /// Detect anomalies
    fn detect_anomalies(
        &self,
        ip: IpAddr,
        endpoint: &str,
        user_agent: Option<&str>,
    ) -> Vec<ThreatEvent> {
        let mut threats = Vec::new();
        
        // Check for suspicious user agent
        if let Some(ua) = user_agent {
            if self.is_suspicious_user_agent(ua) {
                threats.push(ThreatEvent {
                    threat_type: ThreatType::SuspiciousUserAgent,
                    severity: ThreatSeverity::Low,
                    ip,
                    timestamp: Utc::now(),
                    details: format!("Suspicious user agent: {}", ua),
                    confidence: 0.5,
                });
            }
        }
        
        // Check for SQL injection patterns
        if self.contains_sql_injection(endpoint) {
            threats.push(ThreatEvent {
                threat_type: ThreatType::SqlInjection,
                severity: ThreatSeverity::Critical,
                ip,
                timestamp: Utc::now(),
                details: "Potential SQL injection detected in endpoint".to_string(),
                confidence: 0.8,
            });
        }
        
        // Check for XSS patterns
        if self.contains_xss(endpoint) {
            threats.push(ThreatEvent {
                threat_type: ThreatType::CrossSiteScripting,
                severity: ThreatSeverity::High,
                ip,
                timestamp: Utc::now(),
                details: "Potential XSS detected in endpoint".to_string(),
                confidence: 0.7,
            });
        }
        
        threats
    }
    
    /// Check if user agent is suspicious
    fn is_suspicious_user_agent(&self, user_agent: &str) -> bool {
        let suspicious_patterns = [
            "sqlmap", "nikto", "nmap", "masscan", "zap", "burp",
            "scanner", "bot", "crawler", "spider",
        ];
        
        let ua_lower = user_agent.to_lowercase();
        suspicious_patterns.iter().any(|pattern| ua_lower.contains(pattern))
    }
    
    /// Check for SQL injection patterns
    fn contains_sql_injection(&self, input: &str) -> bool {
        let sql_patterns = [
            "' OR '1'='1",
            "'; DROP TABLE",
            "UNION SELECT",
            "1=1",
            "1' OR '1'='1",
        ];
        
        let input_lower = input.to_lowercase();
        sql_patterns.iter().any(|pattern| input_lower.contains(pattern))
    }
    
    /// Check for XSS patterns
    fn contains_xss(&self, input: &str) -> bool {
        let xss_patterns = [
            "<script>",
            "javascript:",
            "onerror=",
            "onload=",
            "onclick=",
        ];
        
        let input_lower = input.to_lowercase();
        xss_patterns.iter().any(|pattern| input_lower.contains(pattern))
    }
    
    /// Check if IP is known malicious
    pub fn is_known_malicious(&self, ip: IpAddr) -> bool {
        let malicious = self.known_malicious_ips.lock().unwrap();
        malicious.contains(&ip)
    }
    
    /// Add IP to known malicious list
    pub fn add_malicious_ip(&self, ip: IpAddr) {
        let mut malicious = self.known_malicious_ips.lock().unwrap();
        malicious.insert(ip);
        warn!("Added IP to malicious list: {}", ip);
    }
    
    /// Get recent threat events
    pub fn get_recent_threats(&self, limit: usize) -> Vec<ThreatEvent> {
        let events = self.threat_events.lock().unwrap();
        events.iter().rev().take(limit).cloned().collect()
    }
    
    /// Cleanup task
    async fn cleanup_task(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(300)); // 5 minutes
        
        loop {
            interval.tick().await;
            
            let now = Instant::now();
            
            // Cleanup old behavioral patterns (older than 1 hour)
            {
                let mut patterns = self.behavioral_patterns.lock().unwrap();
                patterns.retain(|_, pattern| {
                    now.duration_since(pattern.first_seen) < Duration::from_secs(3600)
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;
    
    #[test]
    fn test_threat_detection_config() {
        let config = ThreatDetectionConfig::default();
        assert!(config.enable_behavioral_analysis);
        assert!(config.enable_anomaly_detection);
    }
    
    #[test]
    fn test_suspicious_user_agent() {
        let detection = ThreatDetection::new(ThreatDetectionConfig::default());
        assert!(detection.is_suspicious_user_agent("sqlmap"));
        assert!(detection.is_suspicious_user_agent("Mozilla/5.0") == false);
    }
    
    #[test]
    fn test_sql_injection_detection() {
        let detection = ThreatDetection::new(ThreatDetectionConfig::default());
        assert!(detection.contains_sql_injection("' OR '1'='1"));
        assert!(detection.contains_sql_injection("normal endpoint") == false);
    }
    
    #[test]
    fn test_xss_detection() {
        let detection = ThreatDetection::new(ThreatDetectionConfig::default());
        assert!(detection.contains_xss("<script>alert('xss')</script>"));
        assert!(detection.contains_xss("normal endpoint") == false);
    }
}

