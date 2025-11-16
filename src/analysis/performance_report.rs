//! Performance Analysis Module
//! 
//! Provides tools and utilities for analyzing system performance metrics
//! and generating detailed performance reports.

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use std::time::Duration;
use metrics::{counter, gauge, histogram};

/// Represents a performance metric with its value and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetric {
    name: String,
    value: f64,
    unit: String,
    timestamp: chrono::DateTime<chrono::Utc>,
    metadata: HashMap<String, String>,
}

/// Represents a complete performance report
#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceReport {
    timestamp: chrono::DateTime<chrono::Utc>,
    system_metrics: Vec<PerformanceMetric>,
    database_metrics: Vec<PerformanceMetric>,
    cache_metrics: Vec<PerformanceMetric>,
    memory_metrics: Vec<PerformanceMetric>,
    consciousness_metrics: Vec<PerformanceMetric>,
    recommendations: Vec<String>,
}

/// Analyzes benchmark results and generates performance metrics
pub struct PerformanceAnalyzer {
    metrics: Vec<PerformanceMetric>,
    baseline_metrics: HashMap<String, f64>,
}

impl PerformanceAnalyzer {
    pub fn new() -> Self {
        Self {
            metrics: Vec::new(),
            baseline_metrics: HashMap::new(),
        }
    }

    /// Record a new performance metric
    pub fn record_metric(&mut self, name: &str, value: f64, unit: &str) {
        let metric = PerformanceMetric {
            name: name.to_string(),
            value,
            unit: unit.to_string(),
            timestamp: chrono::Utc::now(),
            metadata: HashMap::new(),
        };
        
        // Record to metrics system
        gauge!(name, value);
        self.metrics.push(metric);
    }

    /// Set baseline metrics for comparison
    pub fn set_baseline(&mut self, name: &str, value: f64) {
        self.baseline_metrics.insert(name.to_string(), value);
    }

    /// Analyze cache performance metrics
    pub fn analyze_cache_performance(&self) -> Vec<String> {
        let mut insights = Vec::new();
        
        // Analyze cache hit ratio
        if let Some(hit_ratio) = self.get_metric_value("cache_hit_ratio") {
            if hit_ratio < 0.8 {
                insights.push(format!(
                    "Cache hit ratio ({:.2}%) is below target of 80%. Consider adjusting cache size or eviction policy.",
                    hit_ratio * 100.0
                ));
            }
        }

        // Analyze cache latency
        if let Some(latency) = self.get_metric_value("cache_miss_latency") {
            if latency > 0.01 { // 10ms threshold
                insights.push(format!(
                    "Cache miss latency ({:.2}ms) is high. Consider optimizing backing store access.",
                    latency * 1000.0
                ));
            }
        }

        insights
    }

    /// Analyze memory usage patterns
    pub fn analyze_memory_usage(&self) -> Vec<String> {
        let mut insights = Vec::new();
        
        // Analyze allocation patterns
        if let Some(allocs) = self.get_metric_value("pressure_test_allocations") {
            if allocs > 10000.0 {
                insights.push(
                    "High allocation count detected. Consider using memory pools or arena allocation.".to_string()
                );
            }
        }

        // Analyze memory pressure
        if let Some(bytes) = self.get_metric_value("pressure_test_bytes") {
            if bytes > 1024.0 * 1024.0 * 100.0 { // 100MB threshold
                insights.push(
                    "High memory pressure detected. Review memory-intensive operations.".to_string()
                );
            }
        }

        insights
    }

    /// Analyze database performance
    pub fn analyze_database_performance(&self) -> Vec<String> {
        let mut insights = Vec::new();
        
        // Analyze query latency
        if let Some(latency) = self.get_metric_value("db_query_latency") {
            if latency > 0.1 { // 100ms threshold
                insights.push(format!(
                    "Database query latency ({:.2}ms) is high. Consider query optimization or indexing.",
                    latency * 1000.0
                ));
            }
        }

        insights
    }

    /// Generate a complete performance report
    pub fn generate_report(&self) -> PerformanceReport {
        let mut report = PerformanceReport {
            timestamp: chrono::Utc::now(),
            system_metrics: Vec::new(),
            database_metrics: Vec::new(),
            cache_metrics: Vec::new(),
            memory_metrics: Vec::new(),
            consciousness_metrics: Vec::new(),
            recommendations: Vec::new(),
        };

        // Analyze each subsystem
        report.recommendations.extend(self.analyze_cache_performance());
        report.recommendations.extend(self.analyze_memory_usage());
        report.recommendations.extend(self.analyze_database_performance());

        // Categorize metrics
        for metric in &self.metrics {
            match metric.name.as_str() {
                name if name.starts_with("cache_") => report.cache_metrics.push(metric.clone()),
                name if name.starts_with("db_") => report.database_metrics.push(metric.clone()),
                name if name.starts_with("memory_") => report.memory_metrics.push(metric.clone()),
                name if name.starts_with("consciousness_") => report.consciousness_metrics.push(metric.clone()),
                _ => report.system_metrics.push(metric.clone()),
            }
        }

        report
    }

    /// Get a specific metric value
    fn get_metric_value(&self, name: &str) -> Option<f64> {
        self.metrics.iter()
            .find(|m| m.name == name)
            .map(|m| m.value)
    }

    /// Compare current metrics against baseline
    pub fn compare_to_baseline(&self) -> Vec<String> {
        let mut comparisons = Vec::new();
        
        for metric in &self.metrics {
            if let Some(baseline) = self.baseline_metrics.get(&metric.name) {
                let diff_percent = ((metric.value - baseline) / baseline) * 100.0;
                
                if diff_percent.abs() > 10.0 {
                    comparisons.push(format!(
                        "{}: {:.2}% {} than baseline",
                        metric.name,
                        diff_percent.abs(),
                        if diff_percent > 0.0 { "higher" } else { "lower" }
                    ));
                }
            }
        }
        
        comparisons
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_analyzer() {
        let mut analyzer = PerformanceAnalyzer::new();
        
        // Record test metrics
        analyzer.record_metric("cache_hit_ratio", 0.75, "ratio");
        analyzer.record_metric("cache_miss_latency", 0.015, "seconds");
        
        // Test cache analysis
        let cache_insights = analyzer.analyze_cache_performance();
        assert!(!cache_insights.is_empty());
        
        // Test report generation
        let report = analyzer.generate_report();
        assert!(!report.recommendations.is_empty());
    }
}