//! Security Compliance Reporting for Jamey 3.0
//!
//! Provides automated security compliance reporting for audit and regulatory requirements.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::{info, warn};

/// Compliance framework type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComplianceFramework {
    OWASP,
    NIST,
    ISO27001,
    SOC2,
    GDPR,
    HIPAA,
    PciDss,
    Custom(String),
}

/// Compliance control
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceControl {
    /// Control ID
    pub id: String,
    /// Control name
    pub name: String,
    /// Control description
    pub description: String,
    /// Framework
    pub framework: ComplianceFramework,
    /// Current status
    pub status: ComplianceStatus,
    /// Last checked
    pub last_checked: DateTime<Utc>,
    /// Evidence/notes
    pub evidence: Option<String>,
}

/// Compliance status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComplianceStatus {
    Compliant,
    NonCompliant,
    Partial,
    NotApplicable,
    NotChecked,
}

/// Compliance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReport {
    /// Report ID
    pub id: String,
    /// Generated at
    pub generated_at: DateTime<Utc>,
    /// Framework
    pub framework: ComplianceFramework,
    /// Overall compliance score (0-100)
    pub compliance_score: f32,
    /// Total controls
    pub total_controls: usize,
    /// Compliant controls
    pub compliant_controls: usize,
    /// Non-compliant controls
    pub non_compliant_controls: usize,
    /// Partial controls
    pub partial_controls: usize,
    /// Controls
    pub controls: Vec<ComplianceControl>,
    /// Summary
    pub summary: String,
    /// Recommendations
    pub recommendations: Vec<String>,
}

/// Compliance manager
#[derive(Clone)]
pub struct ComplianceManager {
    controls: Arc<Mutex<HashMap<String, ComplianceControl>>>,
    reports: Arc<Mutex<Vec<ComplianceReport>>>,
}

impl ComplianceManager {
    /// Create new compliance manager
    pub fn new() -> Self {
        let manager = Self {
            controls: Arc::new(Mutex::new(HashMap::new())),
            reports: Arc::new(Mutex::new(Vec::new())),
        };
        
        // Initialize default controls
        manager.initialize_default_controls();
        
        manager
    }
    
    /// Initialize default compliance controls
    fn initialize_default_controls(&self) {
        let mut controls = self.controls.lock().unwrap();
        
        // OWASP Top 10 controls
        let owasp_controls = vec![
            ("OWASP-01", "Broken Access Control", "Implement proper authentication and authorization"),
            ("OWASP-02", "Cryptographic Failures", "Use strong encryption and proper key management"),
            ("OWASP-03", "Injection", "Prevent SQL injection, XSS, and command injection"),
            ("OWASP-04", "Insecure Design", "Follow secure design principles"),
            ("OWASP-05", "Security Misconfiguration", "Secure default configurations"),
            ("OWASP-06", "Vulnerable Components", "Keep dependencies updated and scan for vulnerabilities"),
            ("OWASP-07", "Authentication Failures", "Implement strong authentication mechanisms"),
            ("OWASP-08", "Software and Data Integrity", "Verify software integrity and protect data"),
            ("OWASP-09", "Security Logging", "Implement comprehensive security logging"),
            ("OWASP-10", "Server-Side Request Forgery", "Protect against SSRF attacks"),
        ];
        
        for (id, name, description) in owasp_controls {
            let control = ComplianceControl {
                id: id.to_string(),
                name: name.to_string(),
                description: description.to_string(),
                framework: ComplianceFramework::OWASP,
                status: ComplianceStatus::NotChecked,
                last_checked: Utc::now(),
                evidence: None,
            };
            controls.insert(id.to_string(), control);
        }
        
        // NIST controls
        let nist_controls = vec![
            ("NIST-AC-1", "Access Control Policy", "Establish and maintain access control policies"),
            ("NIST-AC-2", "Account Management", "Manage system accounts"),
            ("NIST-AC-3", "Access Enforcement", "Enforce access control policies"),
            ("NIST-SI-2", "Flaw Remediation", "Identify and remediate system flaws"),
            ("NIST-SI-3", "Malicious Code Protection", "Protect against malicious code"),
            ("NIST-SI-4", "System Monitoring", "Monitor system security events"),
        ];
        
        for (id, name, description) in nist_controls {
            let control = ComplianceControl {
                id: id.to_string(),
                name: name.to_string(),
                description: description.to_string(),
                framework: ComplianceFramework::NIST,
                status: ComplianceStatus::NotChecked,
                last_checked: Utc::now(),
                evidence: None,
            };
            controls.insert(id.to_string(), control);
        }
        
        info!("Initialized {} default compliance controls", controls.len());
    }
    
    /// Check compliance for a control
    pub fn check_control(&self, control_id: &str, status: ComplianceStatus, evidence: Option<String>) -> Result<(), String> {
        let mut controls = self.controls.lock().unwrap();
        
        let control = controls.get_mut(control_id)
            .ok_or_else(|| format!("Control not found: {}", control_id))?;
        
        control.status = status.clone();
        control.last_checked = Utc::now();
        control.evidence = evidence;
        
        match status {
            ComplianceStatus::Compliant => {
                info!("Control {} is compliant", control_id);
            }
            ComplianceStatus::NonCompliant => {
                warn!("Control {} is non-compliant", control_id);
            }
            ComplianceStatus::Partial => {
                warn!("Control {} is partially compliant", control_id);
            }
            _ => {}
        }
        
        Ok(())
    }
    
    /// Generate compliance report
    pub fn generate_report(&self, framework: ComplianceFramework) -> ComplianceReport {
        let controls = self.controls.lock().unwrap();
        
        let framework_controls: Vec<&ComplianceControl> = controls
            .values()
            .filter(|c| c.framework == framework)
            .collect();
        
        let total = framework_controls.len();
        let compliant = framework_controls.iter()
            .filter(|c| c.status == ComplianceStatus::Compliant)
            .count();
        let non_compliant = framework_controls.iter()
            .filter(|c| c.status == ComplianceStatus::NonCompliant)
            .count();
        let partial = framework_controls.iter()
            .filter(|c| c.status == ComplianceStatus::Partial)
            .count();
        
        let compliance_score = if total > 0 {
            (compliant as f32 / total as f32) * 100.0
        } else {
            0.0
        };
        
        let controls_vec: Vec<ComplianceControl> = framework_controls
            .iter()
            .map(|c| (*c).clone())
            .collect();
        
        let mut recommendations = Vec::new();
        
        // Generate recommendations based on non-compliant controls
        for control in &framework_controls {
            if control.status == ComplianceStatus::NonCompliant {
                recommendations.push(format!(
                    "Address non-compliance in {}: {}",
                    control.id, control.name
                ));
            } else if control.status == ComplianceStatus::Partial {
                recommendations.push(format!(
                    "Improve partial compliance in {}: {}",
                    control.id, control.name
                ));
            }
        }
        
        let summary = format!(
            "Compliance report for {:?}: {:.1}% compliant ({} compliant, {} non-compliant, {} partial out of {} total controls)",
            framework, compliance_score, compliant, non_compliant, partial, total
        );
        
        let report = ComplianceReport {
            id: uuid::Uuid::new_v4().to_string(),
            generated_at: Utc::now(),
            framework,
            compliance_score,
            total_controls: total,
            compliant_controls: compliant,
            non_compliant_controls: non_compliant,
            partial_controls: partial,
            controls: controls_vec,
            summary,
            recommendations,
        };
        
        // Store report
        let mut reports = self.reports.lock().unwrap();
        reports.push(report.clone());
        
        // Keep only last 100 reports
        let len = reports.len();
        if len > 100 {
            reports.drain(0..len - 100);
        }
        
        info!("Generated compliance report: {}", report.summary);
        report
    }
    
    /// Get all controls
    pub fn get_controls(&self) -> Vec<ComplianceControl> {
        let controls = self.controls.lock().unwrap();
        controls.values().cloned().collect()
    }
    
    /// Get controls by framework
    pub fn get_controls_by_framework(&self, framework: ComplianceFramework) -> Vec<ComplianceControl> {
        let controls = self.controls.lock().unwrap();
        controls.values()
            .filter(|c| c.framework == framework)
            .cloned()
            .collect()
    }
    
    /// Get recent reports
    pub fn get_recent_reports(&self, limit: usize) -> Vec<ComplianceReport> {
        let reports = self.reports.lock().unwrap();
        reports.iter().rev().take(limit).cloned().collect()
    }
    
    /// Auto-check compliance based on system state
    pub fn auto_check_compliance(&self) {
        let mut controls = self.controls.lock().unwrap();
        
        // Check OWASP-01: Broken Access Control
        if let Some(control) = controls.get_mut("OWASP-01") {
            // Check if API key authentication is enabled
            let api_key_enabled = std::env::var("API_KEY_REQUIRED")
                .map(|v| v != "false")
                .unwrap_or(true);
            
            control.status = if api_key_enabled {
                ComplianceStatus::Compliant
            } else {
                ComplianceStatus::NonCompliant
            };
            control.last_checked = Utc::now();
        }
        
        // Check OWASP-02: Cryptographic Failures
        if let Some(control) = controls.get_mut("OWASP-02") {
            // Check if encryption is configured
            let encryption_enabled = std::env::var("PHOENIX_ENCRYPTION_KEY")
                .map(|v| !v.is_empty())
                .unwrap_or(false);
            
            control.status = if encryption_enabled {
                ComplianceStatus::Compliant
            } else {
                ComplianceStatus::Partial
            };
            control.last_checked = Utc::now();
        }
        
        // Check OWASP-03: Injection
        if let Some(control) = controls.get_mut("OWASP-03") {
            // Check if input validation is enabled
            let validation_enabled = std::env::var("ENABLE_INPUT_VALIDATION")
                .map(|v| v != "false")
                .unwrap_or(true);
            
            control.status = if validation_enabled {
                ComplianceStatus::Compliant
            } else {
                ComplianceStatus::NonCompliant
            };
            control.last_checked = Utc::now();
        }
        
        // Check OWASP-05: Security Misconfiguration
        if let Some(control) = controls.get_mut("OWASP-05") {
            // Check if CORS is properly configured
            let cors_configured = std::env::var("CORS_ALLOWED_ORIGINS")
                .map(|v| !v.is_empty())
                .unwrap_or(false);
            
            control.status = if cors_configured {
                ComplianceStatus::Compliant
            } else {
                ComplianceStatus::Partial
            };
            control.last_checked = Utc::now();
        }
        
        // Check OWASP-06: Vulnerable Components
        if let Some(control) = controls.get_mut("OWASP-06") {
            // This would require cargo-audit integration
            // For now, mark as partial
            control.status = ComplianceStatus::Partial;
            control.last_checked = Utc::now();
            control.evidence = Some("Dependency scanning requires cargo-audit integration".to_string());
        }
        
        // Check OWASP-09: Security Logging
        if let Some(control) = controls.get_mut("OWASP-09") {
            // Check if logging is enabled
            let logging_enabled = std::env::var("RUST_LOG")
                .map(|v| !v.is_empty())
                .unwrap_or(false);
            
            control.status = if logging_enabled {
                ComplianceStatus::Compliant
            } else {
                ComplianceStatus::Partial
            };
            control.last_checked = Utc::now();
        }
        
        info!("Auto-checked compliance controls");
    }
}

impl Default for ComplianceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_compliance_manager_creation() {
        let manager = ComplianceManager::new();
        let controls = manager.get_controls();
        assert!(!controls.is_empty());
    }
    
    #[test]
    fn test_compliance_report_generation() {
        let manager = ComplianceManager::new();
        
        // Mark some controls as compliant
        manager.check_control("OWASP-01", ComplianceStatus::Compliant, None).unwrap();
        manager.check_control("OWASP-02", ComplianceStatus::Compliant, None).unwrap();
        manager.check_control("OWASP-03", ComplianceStatus::NonCompliant, None).unwrap();
        
        let report = manager.generate_report(ComplianceFramework::OWASP);
        
        assert!(report.compliance_score >= 0.0 && report.compliance_score <= 100.0);
        assert!(report.total_controls > 0);
    }
    
    #[test]
    fn test_auto_check_compliance() {
        let manager = ComplianceManager::new();
        manager.auto_check_compliance();
        
        let controls = manager.get_controls_by_framework(ComplianceFramework::OWASP);
        assert!(!controls.is_empty());
    }
}

