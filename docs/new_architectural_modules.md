# New Architectural Modules Implementation Specification

## Overview

This document provides detailed specifications for the new architectural modules that will be implemented to elevate Jamey 3.0 from B+ to A+ grade. Each module includes the complete file structure, implementation details, and integration points.

## 1. Resilience Module (`src/resilience/`)

### Purpose
Provides comprehensive error recovery and resilience patterns including circuit breakers, retry policies, bulkheads, and graceful degradation.

### File Structure
```
src/resilience/
├── mod.rs              # Main module exports and configuration
├── circuit_breaker.rs  # Circuit breaker implementation
├── retry.rs           # Retry policies with exponential backoff
├── bulkhead.rs        # Bulkhead pattern for fault isolation
├── degradation.rs     # Graceful degradation manager
└── recovery.rs        # Error recovery mechanisms
```

### Implementation Details

#### `src/resilience/mod.rs`
```rust
//! Resilience and Error Recovery Module
//! 
//! This module provides comprehensive error recovery and resilience patterns
//! for Jamey 3.0, including circuit breakers, retry policies, bulkheads,
//! and graceful degradation strategies.

pub mod circuit_breaker;
pub mod retry;
pub mod bulkhead;
pub mod degradation;
pub mod recovery;

// Re-export main types for convenience
pub use circuit_breaker::{CircuitBreaker, CircuitState};
pub use retry::{RetryPolicy, RetryStrategy};
pub use bulkhead::{Bulkhead, BulkheadConfig};
pub use degradation::{DegradationManager, DegradationLevel};
pub use recovery::{ErrorRecovery, RecoveryStrategy};

use serde::{Deserialize, Serialize};

/// Configuration for resilience patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResilienceConfig {
    /// Circuit breaker failure threshold
    pub circuit_breaker_failure_threshold: u32,
    /// Circuit breaker timeout in seconds
    pub circuit_breaker_timeout_seconds: u64,
    /// Maximum retry attempts
    pub retry_max_attempts: u32,
    /// Base delay for retries in milliseconds
    pub retry_base_delay_ms: u64,
    /// Maximum delay for retries in milliseconds
    pub retry_max_delay_ms: u64,
    /// Maximum concurrent operations for bulkhead
    pub bulkhead_max_concurrent: usize,
    /// Whether degradation is enabled
    pub degradation_enabled: bool,
}

impl Default for ResilienceConfig {
    fn default() -> Self {
        Self {
            circuit_breaker_failure_threshold: 5,
            circuit_breaker_timeout_seconds: 30,
            retry_max_attempts: 3,
            retry_base_delay_ms: 100,
            retry_max_delay_ms: 30000,
            bulkhead_max_concurrent: 10,
            degradation_enabled: true,
        }
    }
}

/// Main resilience manager that coordinates all patterns
pub struct ResilienceManager {
    config: ResilienceConfig,
    circuit_breakers: std::collections::HashMap<String, CircuitBreaker>,
    bulkheads: std::collections::HashMap<String, Bulkhead>,
    degradation_manager: DegradationManager,
}

impl ResilienceManager {
    pub fn new(config: ResilienceConfig) -> Self {
        Self {
            config,
            circuit_breakers: std::collections::HashMap::new(),
            bulkheads: std::collections::HashMap::new(),
            degradation_manager: DegradationManager::new(),
        }
    }

    pub fn get_circuit_breaker(&mut self, name: &str) -> &CircuitBreaker {
        self.circuit_breakers.entry(name.to_string())
            .or_insert_with(|| CircuitBreaker::new(
                self.config.circuit_breaker_failure_threshold,
                std::time::Duration::from_secs(self.config.circuit_breaker_timeout_seconds)
            ))
    }

    pub fn get_bulkhead(&mut self, name: &str) -> &Bulkhead {
        self.bulkheads.entry(name.to_string())
            .or_insert_with(|| Bulkhead::new(
                self.config.bulkhead_max_concurrent,
                name.to_string()
            ))
    }

    pub fn degradation_manager(&self) -> &DegradationManager {
        &self.degradation_manager
    }
}
```

#### `src/resilience/circuit_breaker.rs`
```rust
//! Circuit Breaker Pattern Implementation
//! 
//! Prevents cascading failures by stopping requests to failing services
//! after a threshold of failures is reached.

use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
pub enum CircuitState {
    Closed,    // Normal operation
    Open,      // Failing, stop requests
    HalfOpen,  // Testing if service recovered
}

pub struct CircuitBreaker {
    state: Arc<RwLock<CircuitState>>,
    failure_count: Arc<RwLock<u32>>,
    failure_threshold: u32,
    timeout: Duration,
    success_threshold: u32,
    name: String,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, timeout: Duration) -> Self {
        Self {
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            failure_count: Arc::new(RwLock::new(0)),
            failure_threshold,
            timeout,
            success_threshold: 3,
            name: "unnamed".to_string(),
        }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub async fn execute<F, T, E>(&self, operation: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: FnOnce() -> Result<T, E>,
        E: std::fmt::Display,
    {
        // Check circuit state before execution
        {
            let state = self.state.read().await;
            if *state == CircuitState::Open {
                return Err(CircuitBreakerError::CircuitOpen);
            }
        }

        // Execute operation
        match operation() {
            Ok(result) => {
                self.on_success().await;
                Ok(result)
            }
            Err(error) => {
                self.on_failure().await;
                Err(CircuitBreakerError::OperationFailed(error))
            }
        }
    }

    async fn on_success(&self) {
        let mut state = self.state.write().await;
        let mut failure_count = self.failure_count.write().await;
        
        match *state {
            CircuitState::HalfOpen => {
                *failure_count = 0;
                *state = CircuitState::Closed;
                tracing::info!("Circuit breaker '{}' closed after successful test", self.name);
            }
            _ => {
                *failure_count = 0;
            }
        }
    }

    async fn on_failure(&self) {
        let mut state = self.state.write().await;
        let mut failure_count = self.failure_count.write().await;
        
        *failure_count += 1;
        
        if *failure_count >= self.failure_threshold {
            *state = CircuitState::Open;
            tracing::warn!("Circuit breaker '{}' opened after {} failures", self.name, *failure_count);
            
            // Schedule transition to half-open
            let state_clone = self.state.clone();
            let timeout = self.timeout;
            let name = self.name.clone();
            
            tokio::spawn(async move {
                tokio::time::sleep(timeout).await;
                let mut state = state_clone.write().await;
                if *state == CircuitState::Open {
                    *state = CircuitState::HalfOpen;
                    tracing::info!("Circuit breaker '{}' transitioned to half-open", name);
                }
            });
        }
    }

    pub async fn get_state(&self) -> CircuitState {
        self.state.read().await.clone()
    }

    pub async fn get_failure_count(&self) -> u32 {
        *self.failure_count.read().await
    }
}

#[derive(Debug)]
pub enum CircuitBreakerError<E> {
    CircuitOpen,
    OperationFailed(E),
}

impl<E: std::fmt::Display> std::fmt::Display for CircuitBreakerError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CircuitBreakerError::CircuitOpen => write!(f, "Circuit breaker is open"),
            CircuitBreakerError::OperationFailed(e) => write!(f, "Operation failed: {}", e),
        }
    }
}

impl<E: std::fmt::Display + std::fmt::Debug> std::error::Error for CircuitBreakerError<E> {}
```

#### `src/resilience/retry.rs`
```rust
//! Retry Policies with Exponential Backoff
//! 
//! Provides configurable retry strategies for handling transient failures.

use std::time::Duration;
use tokio::time::sleep;
use rand::Rng;

#[derive(Debug, Clone)]
pub enum RetryStrategy {
    Exponential {
        max_attempts: u32,
        base_delay: Duration,
        max_delay: Duration,
        multiplier: f64,
        jitter: bool,
    },
    Fixed {
        max_attempts: u32,
        delay: Duration,
    },
    Linear {
        max_attempts: u32,
        initial_delay: Duration,
        increment: Duration,
        max_delay: Duration,
    },
}

impl RetryStrategy {
    pub fn exponential(max_attempts: u32) -> Self {
        RetryStrategy::Exponential {
            max_attempts,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            multiplier: 2.0,
            jitter: true,
        }
    }

    pub fn fixed(max_attempts: u32, delay: Duration) -> Self {
        RetryStrategy::Fixed { max_attempts, delay }
    }

    pub fn linear(max_attempts: u32, initial_delay: Duration) -> Self {
        RetryStrategy::Linear {
            max_attempts,
            initial_delay,
            increment: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
        }
    }
}

pub struct RetryPolicy {
    strategy: RetryStrategy,
}

impl RetryPolicy {
    pub fn new(strategy: RetryStrategy) -> Self {
        Self { strategy }
    }

    pub async fn execute<F, T, E>(&self, mut operation: F) -> Result<T, RetryError<E>>
    where
        F: FnMut() -> Result<T, E>,
        E: std::fmt::Display + Clone,
    {
        let mut attempt = 0;
        let mut delay = match &self.strategy {
            RetryStrategy::Exponential { base_delay, .. } => *base_delay,
            RetryStrategy::Fixed { delay, .. } => *delay,
            RetryStrategy::Linear { initial_delay, .. } => *initial_delay,
        };

        let max_attempts = match &self.strategy {
            RetryStrategy::Exponential { max_attempts, .. } => *max_attempts,
            RetryStrategy::Fixed { max_attempts, .. } => *max_attempts,
            RetryStrategy::Linear { max_attempts, .. } => *max_attempts,
        };

        loop {
            match operation() {
                Ok(result) => return Ok(result),
                Err(error) => {
                    attempt += 1;
                    if attempt >= max_attempts {
                        return Err(RetryError::MaxAttemptsExceeded(error));
                    }

                    // Calculate delay based on strategy
                    let sleep_duration = match &self.strategy {
                        RetryStrategy::Exponential { 
                            multiplier, 
                            max_delay, 
                            jitter, 
                            .. 
                        } => {
                            let exponential_delay = Duration::from_millis(
                                (delay.as_millis() as f64 * multiplier) as u64
                            );
                            let capped_delay = std::cmp::min(exponential_delay, *max_delay);
                            
                            if *jitter {
                                let mut rng = rand::thread_rng();
                                let jitter_factor = rng.gen_range(0.8..1.2);
                                Duration::from_millis((capped_delay.as_millis() as f64 * jitter_factor) as u64)
                            } else {
                                capped_delay
                            }
                        }
                        RetryStrategy::Fixed { delay, .. } => *delay,
                        RetryStrategy::Linear { 
                            increment, 
                            max_delay, 
                            .. 
                        } => {
                            let linear_delay = delay + *increment;
                            let capped_delay = std::cmp::min(linear_delay, *max_delay);
                            delay = capped_delay;
                            capped_delay
                        }
                    };

                    tracing::warn!("Retry attempt {} for operation, waiting {:?}", attempt, sleep_duration);
                    sleep(sleep_duration).await;
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum RetryError<E> {
    MaxAttemptsExceeded(E),
}

impl<E: std::fmt::Display> std::fmt::Display for RetryError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RetryError::MaxAttemptsExceeded(e) => write!(f, "Max retry attempts exceeded: {}", e),
        }
    }
}

impl<E: std::fmt::Display + std::fmt::Debug> std::error::Error for RetryError<E> {}
```

#### `src/resilience/bulkhead.rs`
```rust
//! Bulkhead Pattern Implementation
//! 
//! Isolates resources to prevent cascading failures by limiting
//! the number of concurrent operations.

use tokio::sync::Semaphore;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct BulkheadConfig {
    pub max_concurrent: usize,
    pub name: String,
}

impl BulkheadConfig {
    pub fn new(max_concurrent: usize, name: String) -> Self {
        Self { max_concurrent, name }
    }
}

pub struct Bulkhead {
    semaphore: Arc<Semaphore>,
    config: BulkheadConfig,
}

impl Bulkhead {
    pub fn new(max_concurrent: usize, name: String) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            config: BulkheadConfig::new(max_concurrent, name),
        }
    }

    pub fn with_config(config: BulkheadConfig) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(config.max_concurrent)),
            config,
        }
    }

    pub async fn execute<F, T>(&self, operation: F) -> Result<T, BulkheadError>
    where
        F: FnOnce() -> T,
    {
        match self.semaphore.acquire().await {
            Ok(permit) => {
                tracing::debug!("Bulkhead '{}' acquired permit", self.config.name);
                let result = operation();
                drop(permit);
                tracing::debug!("Bulkhead '{}' released permit", self.config.name);
                Ok(result)
            }
            Err(_) => {
                tracing::error!("Bulkhead '{}' semaphore closed", self.config.name);
                Err(BulkheadError::SemaphoreClosed)
            }
        }
    }

    pub async fn try_execute<F, T>(&self, operation: F) -> Result<T, BulkheadError>
    where
        F: FnOnce() -> T,
    {
        match self.semaphore.try_acquire() {
            Ok(permit) => {
                tracing::debug!("Bulkhead '{}' acquired permit (try)", self.config.name);
                let result = operation();
                drop(permit);
                tracing::debug!("Bulkhead '{}' released permit (try)", self.config.name);
                Ok(result)
            }
            Err(_) => {
                tracing::warn!("Bulkhead '{}' at capacity", self.config.name);
                Err(BulkheadError::AtCapacity)
            }
        }
    }

    pub fn available_permits(&self) -> usize {
        self.semaphore.available_permits()
    }

    pub fn total_permits(&self) -> usize {
        self.config.max_concurrent
    }

    pub fn utilization(&self) -> f64 {
        let used = self.config.max_concurrent - self.available_permits();
        used as f64 / self.config.max_concurrent as f64
    }
}

#[derive(Debug)]
pub enum BulkheadError {
    SemaphoreClosed,
    AtCapacity,
}

impl std::fmt::Display for BulkheadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BulkheadError::SemaphoreClosed => write!(f, "Bulkhead semaphore is closed"),
            BulkheadError::AtCapacity => write!(f, "Bulkhead is at capacity"),
        }
    }
}

impl std::error::Error for BulkheadError {}
```

#### `src/resilience/degradation.rs`
```rust
//! Graceful Degradation Manager
//! 
//! Manages service degradation levels to ensure partial functionality
//! during system failures or high load.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, PartialEq)]
pub enum DegradationLevel {
    Full,     // 100% functionality
    Degraded, // Limited functionality
    Minimal,  // Essential functionality only
    Offline,  // No functionality, use fallbacks
}

impl DegradationLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            DegradationLevel::Full => "full",
            DegradationLevel::Degraded => "degraded",
            DegradationLevel::Minimal => "minimal",
            DegradationLevel::Offline => "offline",
        }
    }
}

pub type FallbackHandler = Box<dyn Fn() -> String + Send + Sync>;

pub struct DegradationManager {
    services: Arc<RwLock<HashMap<String, DegradationLevel>>>,
    fallback_handlers: Arc<RwLock<HashMap<String, FallbackHandler>>>,
    metrics: Arc<RwLock<DegradationMetrics>>,
}

#[derive(Debug, Default)]
struct DegradationMetrics {
    degradation_changes: u64,
    service_failures: HashMap<String, u64>,
    recovery_times: HashMap<String, Vec<std::time::Duration>>,
}

impl DegradationManager {
    pub fn new() -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
            fallback_handlers: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(DegradationMetrics::default())),
        }
    }

    pub async fn set_degradation_level(&self, service: &str, level: DegradationLevel) {
        let mut services = self.services.write().await;
        let old_level = services.get(service).cloned().unwrap_or(DegradationLevel::Full);
        
        if old_level != level {
            services.insert(service.to_string(), level.clone());
            
            let mut metrics = self.metrics.write().await;
            metrics.degradation_changes += 1;
            
            tracing::warn!(
                "Service '{}' degradation level changed: {:?} -> {:?}",
                service, old_level, level
            );
        }
    }

    pub async fn get_degradation_level(&self, service: &str) -> DegradationLevel {
        let services = self.services.read().await;
        services.get(service).cloned().unwrap_or(DegradationLevel::Full)
    }

    pub async fn register_fallback_handler<F>(&self, service: String, handler: F)
    where
        F: Fn() -> String + Send + Sync + 'static,
    {
        let mut handlers = self.fallback_handlers.write().await;
        handlers.insert(service, Box::new(handler));
    }

    pub async fn execute_with_fallback<F, T>(&self, service: &str, operation: F) -> ExecutionResult<T>
    where
        F: FnOnce() -> T,
        T: Default,
    {
        let level = self.get_degradation_level(service).await;
        
        match level {
            DegradationLevel::Full => {
                ExecutionResult::Executed(operation())
            }
            DegradationLevel::Degraded => {
                tracing::warn!("Service '{}' operating in degraded mode", service);
                ExecutionResult::Executed(operation())
            }
            DegradationLevel::Minimal => {
                tracing::warn!("Service '{}' operating in minimal mode", service);
                ExecutionResult::Degraded(T::default())
            }
            DegradationLevel::Offline => {
                tracing::error!("Service '{}' is offline, using fallback", service);
                let handlers = self.fallback_handlers.read().await;
                if let Some(handler) = handlers.get(service) {
                    ExecutionResult::Fallback(handler())
                } else {
                    ExecutionResult::Unavailable
                }
            }
        }
    }

    pub async fn auto_degrade_on_failure(&self, service: &str, failure_count: u32, threshold: u32) {
        if failure_count >= threshold {
            let current_level = self.get_degradation_level(service).await;
            
            let new_level = match current_level {
                DegradationLevel::Full => DegradationLevel::Degraded,
                DegradationLevel::Degraded => DegradationLevel::Minimal,
                DegradationLevel::Minimal | DegradationLevel::Offline => DegradationLevel::Offline,
            };
            
            self.set_degradation_level(service, new_level).await;
            
            let mut metrics = self.metrics.write().await;
            *metrics.service_failures.entry(service.to_string()).or_insert(0) += failure_count as u64;
        }
    }

    pub async fn attempt_recovery(&self, service: &str) -> bool {
        let current_level = self.get_degradation_level(service).await;
        
        if current_level == DegradationLevel::Offline {
            // Try to recover to minimal
            self.set_degradation_level(service, DegradationLevel::Minimal).await;
            true
        } else if current_level == DegradationLevel::Minimal {
            // Try to recover to degraded
            self.set_degradation_level(service, DegradationLevel::Degraded).await;
            true
        } else if current_level == DegradationLevel::Degraded {
            // Try to recover to full
            self.set_degradation_level(service, DegradationLevel::Full).await;
            true
        } else {
            false
        }
    }

    pub async fn get_all_service_levels(&self) -> HashMap<String, DegradationLevel> {
        let services = self.services.read().await;
        services.clone()
    }

    pub async fn get_metrics(&self) -> DegradationMetricsSnapshot {
        let metrics = self.metrics.read().await;
        DegradationMetricsSnapshot {
            degradation_changes: metrics.degradation_changes,
            service_failures: metrics.service_failures.clone(),
            average_recovery_times: metrics.recovery_times.iter()
                .map(|(service, times)| {
                    let avg = times.iter().sum::<std::time::Duration>() / times.len() as u32;
                    (service.clone(), avg)
                })
                .collect(),
        }
    }
}

#[derive(Debug)]
pub enum ExecutionResult<T> {
    Executed(T),
    Degraded(T),
    Fallback(String),
    Unavailable,
}

#[derive(Debug)]
pub struct DegradationMetricsSnapshot {
    pub degradation_changes: u64,
    pub service_failures: HashMap<String, u64>,
    pub average_recovery_times: HashMap<String, std::time::Duration>,
}
```

#### `src/resilience/recovery.rs`
```rust
//! Error Recovery Mechanisms
//! 
//! Provides automated error recovery strategies for different types of failures.

use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Clone)]
pub enum RecoveryStrategy {
    Immediate,
    Delayed(Duration),
    ExponentialBackoff {
        initial_delay: Duration,
        max_delay: Duration,
        multiplier: f64,
    },
    Custom(Box<dyn Fn() -> Duration + Send + Sync>),
}

#[derive(Debug, Clone)]
pub enum FailureType {
    Transient,    // Temporary failures that will resolve
    Permanent,    // Permanent failures requiring intervention
    Degraded,     // Service is degraded but partially functional
    Timeout,      // Operation timed out
    RateLimited,  // Service is rate limiting requests
}

pub struct ErrorRecovery {
    strategies: HashMap<FailureType, RecoveryStrategy>,
    max_recovery_attempts: u32,
}

impl ErrorRecovery {
    pub fn new() -> Self {
        let mut strategies = HashMap::new();
        
        // Default strategies for different failure types
        strategies.insert(FailureType::Transient, RecoveryStrategy::ExponentialBackoff {
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            multiplier: 2.0,
        });
        
        strategies.insert(FailureType::Timeout, RecoveryStrategy::Delayed(Duration::from_secs(5)));
        strategies.insert(FailureType::RateLimited, RecoveryStrategy::Delayed(Duration::from_secs(60)));
        strategies.insert(FailureType::Degraded, RecoveryStrategy::Immediate);
        strategies.insert(FailureType::Permanent, RecoveryStrategy::Delayed(Duration::from_secs(300)));

        Self {
            strategies,
            max_recovery_attempts: 5,
        }
    }

    pub fn with_strategy(mut self, failure_type: FailureType, strategy: RecoveryStrategy) -> Self {
        self.strategies.insert(failure_type, strategy);
        self
    }

    pub fn with_max_attempts(mut self, max_attempts: u32) -> Self {
        self.max_recovery_attempts = max_attempts;
        self
    }

    pub async fn attempt_recovery<F, T, E>(&self, failure_type: FailureType, mut operation: F) -> Result<T, RecoveryError<E>>
    where
        F: FnMut() -> Result<T, E>,
        E: std::fmt::Display,
    {
        let strategy = self.strategies.get(&failure_type)
            .cloned()
            .unwrap_or(RecoveryStrategy::Immediate);

        let mut attempt = 0;
        let mut delay = match strategy {
            RecoveryStrategy::ExponentialBackoff { initial_delay, .. } => initial_delay,
            RecoveryStrategy::Delayed(d) => d,
            _ => Duration::ZERO,
        };

        loop {
            match operation() {
                Ok(result) => return Ok(result),
                Err(error) => {
                    attempt += 1;
                    if attempt >= self.max_recovery_attempts {
                        return Err(RecoveryError::MaxAttemptsExceeded(error));
                    }

                    let sleep_duration = match &strategy {
                        RecoveryStrategy::Immediate => Duration::ZERO,
                        RecoveryStrategy::Delayed(d) => *d,
                        RecoveryStrategy::ExponentialBackoff { 
                            multiplier, 
                            max_delay, 
                            .. 
                        } => {
                            let exponential_delay = Duration::from_millis(
                                (delay.as_millis() as f64 * multiplier) as u64
                            );
                            delay = std::cmp::min(exponential_delay, *max_delay);
                            delay
                        }
                        RecoveryStrategy::Custom(f) => f(),
                    };

                    tracing::info!(
                        "Recovery attempt {} for {:?} failure, waiting {:?}",
                        attempt, failure_type, sleep_duration
                    );
                    
                    sleep(sleep_duration).await;
                }
            }
        }
    }

    pub fn classify_error<E: std::fmt::Display>(&self, error: &E) -> FailureType {
        let error_str = error.to_string().to_lowercase();
        
        if error_str.contains("timeout") || error_str.contains("timed out") {
            FailureType::Timeout
        } else if error_str.contains("rate limit") || error_str.contains("too many requests") {
            FailureType::RateLimited
        } else if error_str.contains("connection refused") || error_str.contains("network") {
            FailureType::Transient
        } else if error_str.contains("not found") || error_str.contains("invalid") {
            FailureType::Permanent
        } else {
            FailureType::Transient // Default to transient for unknown errors
        }
    }
}

#[derive(Debug)]
pub enum RecoveryError<E> {
    MaxAttemptsExceeded(E),
}

impl<E: std::fmt::Display> std::fmt::Display for RecoveryError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecoveryError::MaxAttemptsExceeded(e) => write!(f, "Max recovery attempts exceeded: {}", e),
        }
    }
}

impl<E: std::fmt::Display + std::fmt::Debug> std::error::Error for RecoveryError<E> {}
```

## 2. Observability Module (`src/observability/`)

### Purpose
Provides comprehensive monitoring, distributed tracing, health checks, and metrics collection for system visibility.

### File Structure
```
src/observability/
├── mod.rs              # Main module exports
├── tracing.rs          # Distributed tracing implementation
├── health.rs           # Health check architecture
├── metrics.rs          # Metrics collection system
├── logging.rs          # Centralized logging
└── alerting.rs         # Alerting system
```

### Implementation Details

#### `src/observability/mod.rs`
```rust
//! Observability Module
//! 
//! Provides comprehensive monitoring, tracing, health checks, and metrics
//! for Jamey 3.0 to ensure system visibility and debuggability.

pub mod tracing;
pub mod health;
pub mod metrics;
pub mod logging;
pub mod alerting;

// Re-export main types
pub use tracing::{DistributedTracer, TraceContext, Span};
pub use health::{HealthRegistry, HealthCheck, HealthStatus};
pub use metrics::{MetricsCollector, MetricType};
pub use logging::{StructuredLogger, LogContext};
pub use alerting::{AlertManager, Alert, AlertSeverity};

use serde::{Deserialize, Serialize};

/// Configuration for observability systems
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityConfig {
    /// Whether distributed tracing is enabled
    pub tracing_enabled: bool,
    /// Jaeger endpoint for distributed tracing
    pub jaeger_endpoint: Option<String>,
    /// Health check interval in seconds
    pub health_check_interval_seconds: u64,
    /// Metrics collection interval in seconds
    pub metrics_interval_seconds: u64,
    /// Prometheus metrics endpoint
    pub prometheus_endpoint: Option<String>,
    /// Log level
    pub log_level: String,
    /// Whether structured logging is enabled
    pub structured_logging: bool,
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            tracing_enabled: true,
            jaeger_endpoint: Some("http://localhost:14268/api/traces".to_string()),
            health_check_interval_seconds: 30,
            metrics_interval_seconds: 15,
            prometheus_endpoint: Some("http://localhost:9090".to_string()),
            log_level: "info".to_string(),
            structured_logging: true,
        }
    }
}

/// Main observability manager
pub struct ObservabilityManager {
    config: ObservabilityConfig,
    tracer: DistributedTracer,
    health_registry: HealthRegistry,
    metrics_collector: MetricsCollector,
    alert_manager: AlertManager,
}

impl ObservabilityManager {
    pub fn new(config: ObservabilityConfig) -> Self {
        let tracer = DistributedTracer::new("jamey-3.0".to_string());
        let health_registry = HealthRegistry::new();
        let metrics_collector = MetricsCollector::new("jamey-3.0".to_string());
        let alert_manager = AlertManager::new();

        Self {
            config,
            tracer,
            health_registry,
            metrics_collector,
            alert_manager,
        }
    }

    pub fn tracer(&self) -> &DistributedTracer {
        &self.tracer
    }

    pub fn health_registry(&self) -> &HealthRegistry {
        &self.health_registry
    }

    pub fn metrics_collector(&self) -> &MetricsCollector {
        &self.metrics_collector
    }

    pub fn alert_manager(&self) -> &AlertManager {
        &self.alert_manager
    }

    pub async fn start_background_tasks(&self) {
        // Start health check monitoring
        let health_registry = self.health_registry.clone();
        let alert_manager = self.alert_manager.clone();
        let interval = Duration::from_secs(self.config.health_check_interval_seconds);
        
        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            loop {
                interval_timer.tick().await;
                
                let results = health_registry.run_all_checks().await;
                for (name, check) in results {
                    if matches!(check.status, HealthStatus::Unhealthy) {
                        let alert = Alert {
                            id: uuid::Uuid::new_v4(),
                            service: name.clone(),
                            message: format!("Health check failed: {}", check.message),
                            severity: AlertSeverity::Critical,
                            timestamp: chrono::Utc::now(),
                        };
                        
                        alert_manager.send_alert(alert).await;
                    }
                }
            }
        });

        // Start metrics collection
        let metrics_collector = self.metrics_collector.clone();
        let interval = Duration::from_secs(self.config.metrics_interval_seconds);
        
        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            loop {
                interval_timer.tick().await;
                // Collect and export metrics
                metrics_collector.export_metrics().await;
            }
        });
    }
}
```

## 3. Security Module (`src/security/`)

### Purpose
Implements zero-trust security patterns, comprehensive audit logging, and secrets management.

### File Structure
```
src/security/
├── mod.rs              # Main module exports
├── zero_trust.rs       # Zero-trust security patterns
├── audit.rs            # Comprehensive audit logging
├── secrets.rs          # Secrets management
├── auth.rs             # Authentication and authorization
└── encryption.rs       # Encryption utilities
```

## 4. Scaling Module (`src/scaling/`)

### Purpose
Provides horizontal scaling, load balancing, caching, and resource management capabilities.

### File Structure
```
src/scaling/
├── mod.rs              # Main module exports
├── scaler.rs           # Horizontal scaling manager
├── load_balancer.rs    # Load balancing strategies
├── cache.rs            # Multi-level caching
├── pool.rs             # Resource pooling
└── auto_scaling.rs     # Auto-scaling policies
```

## Integration with Existing Architecture

### Configuration Updates
Add to `src/config/mod.rs`:
```rust
/// Resilience configuration
pub resilience: ResilienceConfig,

/// Observability configuration
pub observability: ObservabilityConfig,

/// Security configuration
pub security: SecurityConfig,

/// Scaling configuration
pub scaling: ScalingConfig,
```

### Main Application Integration
Update `src/main.rs` to initialize new modules:
```rust
// Initialize resilience manager
let resilience_config = ResilienceConfig::from_env();
let resilience_manager = ResilienceManager::new(resilience_config);

// Initialize observability manager
let observability_config = ObservabilityConfig::from_env();
let observability_manager = ObservabilityManager::new(observability_config);
observability_manager.start_background_tasks().await;

// Initialize security manager
let security_config = SecurityConfig::from_env();
let security_manager = SecurityManager::new(security_config);

// Initialize scaling manager
let scaling_config = ScalingConfig::from_env();
let scaling_manager = ScalingManager::new(scaling_config);
```

### API Integration
Update `src/api/mod.rs` to include new middleware:
```rust
// Add resilience middleware
app.layer(ResilienceLayer::new(resilience_manager.clone()));

// Add observability middleware
app.layer(TracingLayer::new(observability_manager.tracer().clone()));
app.layer(MetricsLayer::new(observability_manager.metrics_collector().clone()));

// Add security middleware
app.layer(AuthLayer::new(security_manager.clone()));
app.layer(AuditLayer::new(security_manager.audit_logger().clone()));
```

## Dependencies to Add to Cargo.toml

```toml
[dependencies]
# For resilience patterns
tokio = { version = "1", features = ["full"] }
rand = "0.8"

# For observability
opentelemetry = { version = "0.21", features = ["rt-tokio"] }
opentelemetry-jaeger = { version = "0.20", features = ["rt-tokio"] }
prometheus = "0.13"
metrics = "0.21"
metrics-exporter-prometheus = "0.12"

# For security
jsonwebtoken = "9.3"
argon2 = "0.5"
ring = "0.17"

# For scaling
lru = "0.12"
moka = { version = "0.12", features = ["future"] }

# Additional utilities
uuid = { version = "1.0", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
```

## Testing Strategy

Each module will include comprehensive tests:
- Unit tests for individual components
- Integration tests for module interactions
- Performance tests for resilience patterns
- Security tests for authentication and authorization
- Load tests for scaling components

## Success Metrics

### Resilience Module
- Circuit breaker effectiveness: > 99%
- Retry success rate: > 80%
- System availability: > 99.9%
- Mean time to recovery: < 5 minutes

### Observability Module
- Trace collection rate: 100%
- Health check coverage: 100%
- Metrics accuracy: > 99%
- Alert response time: < 1 minute

### Security Module
- Authentication success rate: > 99.9%
- Audit log completeness: 100%
- Zero-trust compliance: 100%
- Security incident detection: < 5 minutes

### Scaling Module
- Auto-scaling response time: < 2 minutes
- Cache hit ratio: > 80%
- Load balancing efficiency: > 95%
- Resource utilization: 70-80%

This comprehensive architectural module design provides the foundation for elevating Jamey 3.0 to A+ grade architecture while maintaining the innovative consciousness system features.