//! Combined Security Middleware for Jamey 3.0
//!
//! Integrates DDoS protection, threat detection, and incident response
//! into a single middleware layer.

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{Response, IntoResponse},
};
use std::sync::Arc;
use std::net::IpAddr;
use tracing::{error, warn, info};
use crate::security::{
    DdosProtection, ThreatDetection, IncidentResponse,
};

/// Combined security middleware that integrates all security features
pub async fn security_middleware(
    request: Request,
    next: Next,
) -> Response {
    // Extract security components from extensions
    let ddos_protection = request.extensions().get::<Arc<DdosProtection>>().cloned();
    let threat_detection = request.extensions().get::<Arc<ThreatDetection>>().cloned();
    let incident_response = request.extensions().get::<Arc<IncidentResponse>>().cloned();
    
    // Get IP address
    let ip = extract_ip(&request);
    
    // Step 1: DDoS Protection Check
    if let Some(ref protection) = ddos_protection {
        if protection.is_blocked(&ip) {
            warn!("Blocked IP attempted request: {}", ip);
            return StatusCode::FORBIDDEN.into_response();
        }
        
        match protection.check_request(&ip) {
            Ok(_) => {
                // Request allowed, continue
            }
            Err(crate::security::DdosError::IpBlocked) => {
                return StatusCode::FORBIDDEN.into_response();
            }
            Err(crate::security::DdosError::RateLimitExceeded) => {
                use axum::http::HeaderValue;
                let mut response = StatusCode::TOO_MANY_REQUESTS.into_response();
                response.headers_mut().insert(
                    axum::http::header::RETRY_AFTER,
                    HeaderValue::from_static("60"),
                );
                return response;
            }
            Err(crate::security::DdosError::ConnectionLimitExceeded) => {
                return StatusCode::SERVICE_UNAVAILABLE.into_response();
            }
            Err(crate::security::DdosError::RequestSizeExceeded) => {
                return StatusCode::PAYLOAD_TOO_LARGE.into_response();
            }
        }
    }
    
    // Step 2: Threat Detection
    let mut threats = Vec::new();
    if let Some(ref detection) = threat_detection {
        let endpoint = request.uri().path().to_string();
        let user_agent = request.headers()
            .get("user-agent")
            .and_then(|h| h.to_str().ok());
        
        threats = detection.analyze_request(ip, &endpoint, user_agent, false);
        
        // Log threats
        for threat in &threats {
            use crate::security::ThreatSeverity;
            match threat.severity {
                ThreatSeverity::Critical => {
                    error!("Critical threat detected: {:?} from IP {} - {}", 
                           threat.threat_type, threat.ip, threat.details);
                }
                ThreatSeverity::High => {
                    warn!("High severity threat detected: {:?} from IP {} - {}", 
                          threat.threat_type, threat.ip, threat.details);
                }
                _ => {
                    info!("Threat detected: {:?} from IP {} - {}", 
                          threat.threat_type, threat.ip, threat.details);
                }
            }
        }
    }
    
    // Step 3: Process request
    let response = next.run(request).await;
    
    // Step 4: Incident Response (after request to capture response status)
    if !threats.is_empty() {
        if let Some(ref incident) = incident_response {
            let incident_ids = incident.process_threats(
                threats,
                ddos_protection.clone(),
                threat_detection.clone(),
            );
            
            if !incident_ids.is_empty() {
                info!("Created {} security incidents", incident_ids.len());
            }
        }
    }
    
    // Release connection in DDoS protection
    if let Some(ref protection) = ddos_protection {
        protection.release_connection(&ip);
    }
    
    response
}

/// Extract IP address from request
fn extract_ip(request: &Request) -> IpAddr {
    request
        .headers()
        .get("x-forwarded-for")
        .or_else(|| request.headers().get("x-real-ip"))
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.split(',').next())
        .and_then(|s| s.trim().parse::<IpAddr>().ok())
        .unwrap_or_else(|| {
            // Fallback to localhost if IP cannot be determined
            "127.0.0.1".parse().unwrap()
        })
}

