use std::sync::Arc;
use std::time::Duration;

use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum OptimizationError {
    InsufficientData,
    InvalidSampleValue,
    TrendCalculationFailed,
}

impl fmt::Display for OptimizationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InsufficientData => write!(f, "Insufficient data points for analysis"),
            Self::InvalidSampleValue => write!(f, "Invalid sample value provided"),
            Self::TrendCalculationFailed => write!(f, "Failed to calculate performance trend"),
        }
    }
}

impl Error for OptimizationError {}

// Monitor for tracking cache performance metrics
#[derive(Debug)]
pub struct CacheMonitor {
    hit_count: u64,
    miss_count: u64,
    memory_pressure: f64,
    last_update: std::time::Instant,
}

// Configuration for the optimizer
#[derive(Clone)]
pub struct OptimizerConfig {
    pub adjustment_threshold: f64,
    pub growth_rate: f64,
    pub shrink_rate: f64,
    pub min_sample_size: usize,
    pub trend_window: Duration,
}

// Analyzes performance trends over time
#[derive(Debug)]
pub struct TrendAnalyzer {
    window_size: Duration,
    samples: Vec<(f64, std::time::Instant)>,
    min_samples: usize,
    max_samples: usize,
    current_trend: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub hit_rate: f64,
    pub memory_pressure: f64,
    pub response_time: f64,
    pub timestamp: std::time::Instant,
}

#[derive(Debug)]
pub enum AdjustmentDecision {
    Grow(f64),    // Growth factor
    Shrink(f64),  // Shrink factor
    NoChange,
}

#[derive(Debug)]
pub struct DecisionMaker {
    config: OptimizerConfig,
    last_adjustment: std::time::Instant,
    min_adjustment_interval: Duration,
    consecutive_adjustments: i32,
    max_consecutive_adjustments: i32,
}

impl DecisionMaker {
    pub fn new(config: OptimizerConfig) -> Self {
        Self {
            config,
            last_adjustment: std::time::Instant::now(),
            min_adjustment_interval: Duration::from_secs(60), // Minimum 1 minute between adjustments
            consecutive_adjustments: 0,
            max_consecutive_adjustments: 3, // Limit consecutive adjustments
        }
    }

    pub fn evaluate(&mut self, metrics: &PerformanceMetrics, trend: f64) -> Result<AdjustmentDecision, OptimizationError> {
        // Check if enough time has passed since last adjustment
        if self.last_adjustment.elapsed() < self.min_adjustment_interval {
            return Ok(AdjustmentDecision::NoChange);
        }

        // Reset consecutive adjustments if trend is stable
        if trend.abs() < self.config.adjustment_threshold {
            self.consecutive_adjustments = 0;
            return Ok(AdjustmentDecision::NoChange);
        }

        // Check if we've made too many consecutive adjustments
        if self.consecutive_adjustments >= self.max_consecutive_adjustments {
            return Ok(AdjustmentDecision::NoChange);
        }

        // Make decision based on performance metrics and trend
        let decision = self.calculate_adjustment(metrics, trend)?;
        
        // Update state if we're making an adjustment
        match decision {
            AdjustmentDecision::NoChange => {},
            _ => {
                self.last_adjustment = std::time::Instant::now();
                self.consecutive_adjustments += 1;
            }
        }

        Ok(decision)
    }

    fn calculate_adjustment(&self, metrics: &PerformanceMetrics, trend: f64) -> Result<AdjustmentDecision, OptimizationError> {
        // Don't adjust if memory pressure is too high
        if metrics.memory_pressure > 0.9 {
            return Ok(AdjustmentDecision::Shrink(self.config.shrink_rate));
        }

        // Calculate adjustment based on hit rate and trend
        let hit_rate_factor = if metrics.hit_rate < 0.5 {
            -1.0
        } else if metrics.hit_rate > 0.8 {
            1.0
        } else {
            0.0
        };

        // Combine hit rate factor with trend
        let combined_signal = (hit_rate_factor + trend.signum()) / 2.0;

        match combined_signal {
            x if x > 0.0 => Ok(AdjustmentDecision::Grow(self.config.growth_rate)),
            x if x < 0.0 => Ok(AdjustmentDecision::Shrink(self.config.shrink_rate)),
            _ => Ok(AdjustmentDecision::NoChange),
        }
    }

    pub fn reset_consecutive_adjustments(&mut self) {
        self.consecutive_adjustments = 0;
    }
}

// Executes cache size adjustments
pub struct ActionExecutor {
    min_size: usize,
    max_size: usize,
    current_size: usize,
}

// Controller for coordinating optimization actions
pub struct AdjustmentController {
    trend_analyzer: TrendAnalyzer,
    decision_maker: DecisionMaker,
    action_executor: ActionExecutor,
}

// Main optimizer that coordinates all components
pub struct CacheOptimizer {
    monitor: Arc<CacheMonitor>,
    config: OptimizerConfig,
    adjustment_controller: Arc<AdjustmentController>,
}

impl TrendAnalyzer {
    pub fn new(window_size: Duration, min_samples: usize, max_samples: usize) -> Self {
        Self {
            window_size,
            samples: Vec::with_capacity(max_samples),
            min_samples,
            max_samples,
            current_trend: None,
        }
    }

    pub fn add_sample(&mut self, metrics: PerformanceMetrics) -> Result<(), OptimizationError> {
        if metrics.hit_rate < 0.0 || metrics.hit_rate > 1.0 {
            return Err(OptimizationError::InvalidSampleValue);
        }

        let now = std::time::Instant::now();
        let composite_score = self.calculate_composite_score(&metrics);
        
        self.samples.push((composite_score, now));
        
        // Maintain window and capacity constraints
        let cutoff = now - self.window_size;
        self.samples.retain(|(_, timestamp)| *timestamp >= cutoff);
        
        if self.samples.len() > self.max_samples {
            self.samples.remove(0);
        }

        // Update current trend if we have enough samples
        if self.samples.len() >= self.min_samples {
            self.current_trend = self.calculate_trend()?;
        }

        Ok(())
    }

    fn calculate_composite_score(&self, metrics: &PerformanceMetrics) -> f64 {
        // Weighted combination of performance metrics
        const HIT_RATE_WEIGHT: f64 = 0.5;
        const MEMORY_WEIGHT: f64 = 0.3;
        const RESPONSE_WEIGHT: f64 = 0.2;

        let normalized_response_time = 1.0 / (1.0 + metrics.response_time); // Convert to [0,1] range
        
        (metrics.hit_rate * HIT_RATE_WEIGHT) +
        ((1.0 - metrics.memory_pressure) * MEMORY_WEIGHT) +
        (normalized_response_time * RESPONSE_WEIGHT)
    }

    pub fn calculate_trend(&self) -> Result<f64, OptimizationError> {
        if self.samples.len() < self.min_samples {
            return Err(OptimizationError::InsufficientData);
        }

        // Calculate linear regression slope with error handling
        let n = self.samples.len() as f64;
        let sum_x: f64 = self.samples.iter()
            .map(|(_, t)| t.elapsed().as_secs_f64())
            .sum();
        let sum_y: f64 = self.samples.iter()
            .map(|(v, _)| *v)
            .sum();
        let sum_xy: f64 = self.samples.iter()
            .map(|(v, t)| v * t.elapsed().as_secs_f64())
            .sum();
        let sum_xx: f64 = self.samples.iter()
            .map(|(_, t)| t.elapsed().as_secs_f64().powi(2))
            .sum();

        let denominator = n * sum_xx - sum_x * sum_x;
        if denominator == 0.0 {
            return Err(OptimizationError::TrendCalculationFailed);
        }

        let slope = (n * sum_xy - sum_x * sum_y) / denominator;
        Ok(slope)
    }

    pub fn get_current_trend(&self) -> Option<f64> {
        self.current_trend
    }

    pub fn clear_samples(&mut self) {
        self.samples.clear();
        self.current_trend = None;
    }
}

impl DecisionMaker {
    pub fn new(config: OptimizerConfig) -> Self {
        Self {
            config,
            last_adjustment: std::time::Instant::now(),
        }
    }

    pub fn should_adjust(&self, trend: f64) -> bool {
        trend.abs() > self.config.adjustment_threshold
    }
}

impl ActionExecutor {
    pub fn new(min_size: usize, max_size: usize) -> Self {
        Self {
            min_size,
            max_size,
            current_size: min_size,
        }
    }

    pub fn adjust_size(&mut self, growth_factor: f64) -> usize {
        let new_size = ((self.current_size as f64) * growth_factor) as usize;
        self.current_size = new_size.clamp(self.min_size, self.max_size);
        self.current_size
    }
}

impl AdjustmentController {
    pub fn new(config: OptimizerConfig, min_size: usize, max_size: usize) -> Self {
        Self {
            trend_analyzer: TrendAnalyzer::new(config.trend_window),
            decision_maker: DecisionMaker::new(config.clone()),
            action_executor: ActionExecutor::new(min_size, max_size),
        }
    }
}

impl CacheOptimizer {
    pub fn new(monitor: Arc<CacheMonitor>, config: OptimizerConfig, min_size: usize, max_size: usize) -> Self {
        Self {
            monitor,
            config: config.clone(),
            adjustment_controller: Arc::new(AdjustmentController::new(config, min_size, max_size)),
        }
    }
}