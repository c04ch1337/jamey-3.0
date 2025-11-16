//! Security Incident Response System for Jamey 3.0
//!
//! Provides automated detection, classification, and response to security incidents.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};
use std::net::IpAddr;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{error, warn, info};
use crate::security::threat_detection::{ThreatEvent, ThreatSeverity, ThreatType};

/// Incident status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IncidentStatus {
    Detected,
    Investigating,
    Contained,
    Resolved,
    FalsePositive,
}

/// Incident type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IncidentType {
    DDoS,
    BruteForce,
    UnauthorizedAccess,
    DataExfiltration,
    Malware,
    PrivilegeEscalation,
    Other(String),
}

/// Security incident
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityIncident {
    pub id: String,
    pub incident_type: IncidentType,
    pub severity: ThreatSeverity,
    pub status: IncidentStatus,
    pub ip: Option<IpAddr>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub detected_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds_option")]
    pub resolved_at: Option<DateTime<Utc>>,
    pub description: String,
    pub threat_events: Vec<ThreatEvent>,
    pub response_actions: Vec<ResponseAction>,
    pub notes: Vec<String>,
}

/// Response action taken
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseAction {
    pub action_type: ActionType,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub timestamp: DateTime<Utc>,
    pub description: String,
    pub success: bool,
}

/// Type of response action
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionType {
    IpBlocked,
    RateLimitAdjusted,
    AlertSent,
    IncidentEscalated,
    ManualIntervention,
}

/// Incident response configuration
#[derive(Debug, Clone)]
pub struct IncidentResponseConfig {
    /// Enable automatic response
    pub enable_auto_response: bool,
    /// Auto-block IPs for critical incidents
    pub auto_block_critical: bool,
    /// Auto-block IPs for high severity incidents
    pub auto_block_high: bool,
    /// Auto-block IPs for medium severity incidents
    pub auto_block_medium: bool,
    /// Escalation threshold (number of incidents)
    pub escalation_threshold: u32,
    /// Incident retention days
    pub retention_days: u32,
}

impl Default for IncidentResponseConfig {
    fn default() -> Self {
        Self {
            enable_auto_response: true,
            auto_block_critical: true,
            auto_block_high: true,
            auto_block_medium: false,
            escalation_threshold: 5,
            retention_days: 90,
        }
    }
}

impl IncidentResponseConfig {
    /// Create config from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();
        
        if let Ok(enabled) = std::env::var("INCIDENT_ENABLE_AUTO_RESPONSE") {
            config.enable_auto_response = enabled.parse().unwrap_or(true);
        }
        
        if let Ok(enabled) = std::env::var("INCIDENT_AUTO_BLOCK_CRITICAL") {
            config.auto_block_critical = enabled.parse().unwrap_or(true);
        }
        
        if let Ok(enabled) = std::env::var("INCIDENT_AUTO_BLOCK_HIGH") {
            config.auto_block_high = enabled.parse().unwrap_or(true);
        }
        
        if let Ok(threshold) = std::env::var("INCIDENT_ESCALATION_THRESHOLD") {
            if let Ok(parsed) = threshold.parse::<u32>() {
                config.escalation_threshold = parsed;
            }
        }
        
        if let Ok(days) = std::env::var("INCIDENT_RETENTION_DAYS") {
            if let Ok(parsed) = days.parse::<u32>() {
                config.retention_days = parsed;
            }
        }
        
        config
    }
}

/// Incident response system
#[derive(Clone)]
pub struct IncidentResponse {
    config: IncidentResponseConfig,
    incidents: Arc<Mutex<HashMap<String, SecurityIncident>>>,
    ip_incident_count: Arc<Mutex<HashMap<IpAddr, u32>>>,
}

impl IncidentResponse {
    /// Create new incident response system
    pub fn new(config: IncidentResponseConfig) -> Self {
        let response = Self {
            config,
            incidents: Arc::new(Mutex::new(HashMap::new())),
            ip_incident_count: Arc::new(Mutex::new(HashMap::new())),
        };
        
        // Start cleanup task
        let cleanup_response = response.clone();
        tokio::spawn(async move {
            cleanup_response.cleanup_task().await;
        });
        
        response
    }
    
    /// Process threat events and create incidents
    pub fn process_threats(
        &self,
        threats: Vec<ThreatEvent>,
        ddos_protection: Option<Arc<crate::security::DdosProtection>>,
        threat_detection: Option<Arc<crate::security::ThreatDetection>>,
    ) -> Vec<String> {
        let mut incident_ids = Vec::new();
        
        // Group threats by IP and type
        let mut threat_groups: HashMap<(IpAddr, ThreatType), Vec<ThreatEvent>> = HashMap::new();
        
        for threat in threats {
            if let Some(ip) = Some(threat.ip) {
                let key = (ip, threat.threat_type.clone());
                threat_groups.entry(key).or_insert_with(Vec::new).push(threat);
            }
        }
        
        // Create incidents from threat groups
        for ((ip, threat_type), events) in threat_groups {
            // Determine incident type and severity
            let (incident_type, severity) = self.classify_incident(&threat_type, &events);
            
            // Check if we should create an incident
            if severity >= ThreatSeverity::Medium {
                let incident_id = self.create_incident(
                    incident_type,
                    severity,
                    Some(ip),
                    events.clone(),
                    ddos_protection.clone(),
                    threat_detection.clone(),
                );
                incident_ids.push(incident_id);
            }
        }
        
        incident_ids
    }
    
    /// Classify incident based on threat type and events
    fn classify_incident(
        &self,
        threat_type: &ThreatType,
        events: &[ThreatEvent],
    ) -> (IncidentType, ThreatSeverity) {
        let max_severity = events.iter()
            .map(|e| e.severity)
            .max()
            .unwrap_or(ThreatSeverity::Low);
        
        let incident_type = match threat_type {
            ThreatType::RapidFireRequests => IncidentType::DDoS,
            ThreatType::BruteForce => IncidentType::BruteForce,
            ThreatType::PrivilegeEscalation => IncidentType::PrivilegeEscalation,
            ThreatType::DataExfiltration => IncidentType::DataExfiltration,
            ThreatType::KnownMaliciousIp => IncidentType::UnauthorizedAccess,
            _ => IncidentType::Other(format!("{:?}", threat_type)),
        };
        
        (incident_type, max_severity)
    }
    
    /// Create a new security incident
    fn create_incident(
        &self,
        incident_type: IncidentType,
        severity: ThreatSeverity,
        ip: Option<IpAddr>,
        threat_events: Vec<ThreatEvent>,
        ddos_protection: Option<Arc<crate::security::DdosProtection>>,
        threat_detection: Option<Arc<crate::security::ThreatDetection>>,
    ) -> String {
        let incident_id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();
        
        let description = format!(
            "{:?} incident detected with {:?} severity. {} threat events.",
            incident_type,
            severity,
            threat_events.len()
        );
        
        let mut response_actions = Vec::new();
        
        // Automatic response actions
        if self.config.enable_auto_response {
            if let Some(ip_addr) = ip {
                // Auto-block based on severity
                let should_block = match severity {
                    ThreatSeverity::Critical => self.config.auto_block_critical,
                    ThreatSeverity::High => self.config.auto_block_high,
                    _ => false,
                };
                
                if should_block {
                    if let Some(ref ddos) = ddos_protection {
                        ddos.block_ip(&ip_addr, format!("Security incident: {:?}", incident_type));
                        response_actions.push(ResponseAction {
                            action_type: ActionType::IpBlocked,
                            timestamp: now,
                            description: format!("IP {} automatically blocked", ip_addr),
                            success: true,
                        });
                        info!("Auto-blocked IP {} due to security incident", ip_addr);
                    }
                    
                    // Add to malicious IP list
                    if let Some(ref threat) = threat_detection {
                        threat.add_malicious_ip(ip_addr);
                    }
                }
                
                // Track incident count per IP
                {
                    let mut counts = self.ip_incident_count.lock().unwrap();
                    *counts.entry(ip_addr).or_insert(0) += 1;
                    
                    // Escalate if threshold exceeded
                    if counts.get(&ip_addr).unwrap_or(&0) >= &self.config.escalation_threshold {
                        response_actions.push(ResponseAction {
                            action_type: ActionType::IncidentEscalated,
                            timestamp: now,
                            description: format!(
                                "Incident escalated: {} incidents from IP {}",
                                counts.get(&ip_addr).unwrap_or(&0),
                                ip_addr
                            ),
                            success: true,
                        });
                        warn!("Incident escalation: {} incidents from IP {}", 
                              counts.get(&ip_addr).unwrap_or(&0), ip_addr);
                    }
                }
            }
        }
        
        let incident = SecurityIncident {
            id: incident_id.clone(),
            incident_type,
            severity,
            status: IncidentStatus::Detected,
            ip,
            detected_at: now,
            resolved_at: None,
            description,
            threat_events,
            response_actions,
            notes: Vec::new(),
        };
        
        // Store incident (extract values before moving)
        let incident_type_clone = incident_type.clone();
        let severity_clone = severity;
        {
            let mut incidents = self.incidents.lock().unwrap();
            incidents.insert(incident_id.clone(), incident);
        }
        
        error!("Security incident created: {} - {:?} - {:?}", 
               incident_id, incident_type_clone, severity_clone);
        
        incident_id
    }
    
    /// Get incident by ID
    pub fn get_incident(&self, id: &str) -> Option<SecurityIncident> {
        let incidents = self.incidents.lock().unwrap();
        incidents.get(id).cloned()
    }
    
    /// Get all incidents
    pub fn get_all_incidents(&self) -> Vec<SecurityIncident> {
        let incidents = self.incidents.lock().unwrap();
        incidents.values().cloned().collect()
    }
    
    /// Get recent incidents
    pub fn get_recent_incidents(&self, limit: usize) -> Vec<SecurityIncident> {
        let incidents = self.incidents.lock().unwrap();
        let mut sorted: Vec<_> = incidents.values().cloned().collect();
        sorted.sort_by(|a, b| b.detected_at.cmp(&a.detected_at));
        sorted.into_iter().take(limit).collect()
    }
    
    /// Resolve an incident
    pub fn resolve_incident(&self, id: &str, notes: Option<String>) -> bool {
        let mut incidents = self.incidents.lock().unwrap();
        if let Some(incident) = incidents.get_mut(id) {
            incident.status = IncidentStatus::Resolved;
            incident.resolved_at = Some(Utc::now());
            if let Some(note) = notes {
                incident.notes.push(note);
            }
            info!("Incident resolved: {}", id);
            true
        } else {
            false
        }
    }
    
    /// Mark incident as false positive
    pub fn mark_false_positive(&self, id: &str, notes: Option<String>) -> bool {
        let mut incidents = self.incidents.lock().unwrap();
        if let Some(incident) = incidents.get_mut(id) {
            incident.status = IncidentStatus::FalsePositive;
            incident.resolved_at = Some(Utc::now());
            if let Some(note) = notes {
                incident.notes.push(note);
            }
            info!("Incident marked as false positive: {}", id);
            true
        } else {
            false
        }
    }
    
    /// Cleanup task
    async fn cleanup_task(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(3600)); // 1 hour
        
        loop {
            interval.tick().await;
            
            let cutoff = Utc::now() - chrono::Duration::seconds(
                self.config.retention_days as i64 * 24 * 3600
            );
            
            // Remove old incidents
            {
                let mut incidents = self.incidents.lock().unwrap();
                incidents.retain(|_, incident| incident.detected_at > cutoff);
            }
            
            // Cleanup IP incident counts (older than 24 hours)
            {
                let mut counts = self.ip_incident_count.lock().unwrap();
                // Reset counts periodically (simplified - in production, track timestamps)
                if counts.len() > 1000 {
                    counts.clear();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;
    use crate::security::threat_detection::ThreatEvent;
    
    #[test]
    fn test_incident_creation() {
        let config = IncidentResponseConfig::default();
        let response = IncidentResponse::new(config);
        
        let ip: IpAddr = Ipv4Addr::new(127, 0, 0, 1).into();
        let threat = ThreatEvent {
            threat_type: ThreatType::BruteForce,
            severity: ThreatSeverity::High,
            ip,
            timestamp: Utc::now(),
            details: "Test threat".to_string(),
            confidence: 0.8,
        };
        
        let incident_id = response.process_threats(
            vec![threat],
            None,
            None,
        );
        
        assert!(!incident_id.is_empty());
    }
    
    #[test]
    fn test_incident_resolution() {
        let config = IncidentResponseConfig::default();
        let response = IncidentResponse::new(config);
        
        let ip: IpAddr = Ipv4Addr::new(127, 0, 0, 1).into();
        let threat = ThreatEvent {
            threat_type: ThreatType::BruteForce,
            severity: ThreatSeverity::High,
            ip,
            timestamp: Utc::now(),
            details: "Test threat".to_string(),
            confidence: 0.8,
        };
        
        let incident_ids = response.process_threats(
            vec![threat],
            None,
            None,
        );
        
        assert!(!incident_ids.is_empty());
        
        let resolved = response.resolve_incident(&incident_ids[0], Some("Test resolution".to_string()));
        assert!(resolved);
        
        let incident = response.get_incident(&incident_ids[0]);
        assert!(incident.is_some());
        assert_eq!(incident.unwrap().status, IncidentStatus::Resolved);
    }
}

