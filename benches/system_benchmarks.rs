//! System Performance Benchmarks
//!
//! Comprehensive benchmarks for measuring and optimizing system performance
//! across all major components.

#![feature(test)]
extern crate test;

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use tokio::runtime::Runtime;
use std::sync::Arc;
use std::path::PathBuf;
use uuid::Uuid;

use jamey_3::consciousness::{ConsciousnessEngine, ConsciousnessMetrics};
use jamey_3::soul::emotion::EmotionManager;
use jamey_3::memory::{MemorySystem, MemoryLayer, holographic::HolographicMemory};
use jamey_3::monitoring::{
    metrics::MetricsCollector,
    consciousness::ConsciousnessMonitor,
};

/// Benchmark consciousness processing
fn bench_consciousness(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("consciousness");
    group.sample_size(100);
    
    // Setup test environment
    let memory = rt.block_on(async {
        Arc::new(MemorySystem::new(PathBuf::from("bench_data")).await.unwrap())
    });
    let consciousness = rt.block_on(async {
        Arc::new(ConsciousnessEngine::new(memory.clone()).await.unwrap())
    });

    // Benchmark different input sizes
    for size in [10, 100, 1000].iter() {
        let input = "x".repeat(*size);
        
        group.bench_with_input(
            BenchmarkId::new("process_information", size),
            &input,
            |b, input| {
                b.iter(|| {
                    rt.block_on(async {
                        consciousness.process_information(input).await.unwrap()
                    })
                })
            },
        );
    }

    group.finish();
}

/// Benchmark memory operations
fn bench_memory(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("memory");
    group.sample_size(100);
    
    let memory = rt.block_on(async {
        Arc::new(MemorySystem::new(PathBuf::from("bench_data")).await.unwrap())
    });

    // Benchmark storage
    for size in [10, 100, 1000].iter() {
        let content = "x".repeat(*size);
        
        group.bench_with_input(
            BenchmarkId::new("store", size),
            &content,
            |b, content| {
                b.iter(|| {
                    rt.block_on(async {
                        memory.store(MemoryLayer::Episodic, content.to_string()).await.unwrap()
                    })
                })
            },
        );
    }

    // Benchmark search
    group.bench_function("search", |b| {
        b.iter(|| {
            rt.block_on(async {
                memory.search(MemoryLayer::Episodic, "test", 10).await.unwrap()
            })
        })
    });

    group.finish();
}

/// Benchmark holographic memory
fn bench_holographic(c: &mut Criterion) {
    let mut group = c.benchmark_group("holographic");
    group.sample_size(100);
    
    let memory = HolographicMemory::new();

    // Benchmark encoding
    for size in [10, 100, 1000].iter() {
        let content = "x".repeat(*size);
        
        group.bench_with_input(
            BenchmarkId::new("encode", size),
            &content,
            |b, content| {
                b.iter(|| {
                    memory.encode(
                        content,
                        vec!["test".to_string()],
                        vec!["test".to_string()],
                    ).unwrap()
                })
            },
        );
    }

    // Benchmark pattern matching
    group.bench_function("pattern_matching", |b| {
        let trace = memory.encode(
            "test pattern",
            vec!["test".to_string()],
            vec!["test".to_string()],
        ).unwrap();

        b.iter(|| memory.find_similar(&trace, 0.5))
    });

    group.finish();
}

/// Benchmark emotional processing
fn bench_emotion(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("emotion");
    group.sample_size(100);
    
    let emotion_manager = EmotionManager::new();

    // Benchmark emotional processing
    for size in [10, 100, 1000].iter() {
        let content = "x".repeat(*size);
        
        group.bench_with_input(
            BenchmarkId::new("process_stimulus", size),
            &content,
            |b, content| {
                b.iter(|| {
                    rt.block_on(async {
                        emotion_manager.process_stimulus(
                            content,
                            Some("test".to_string()),
                        ).await.unwrap()
                    })
                })
            },
        );
    }

    // Benchmark stability calculation
    group.bench_function("calculate_stability", |b| {
        b.iter(|| {
            rt.block_on(async {
                emotion_manager.calculate_stability().await
            })
        })
    });

    group.finish();
}

/// Benchmark monitoring system
fn bench_monitoring(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("monitoring");
    group.sample_size(100);
    
    // Setup monitoring components
    let memory = rt.block_on(async {
        Arc::new(MemorySystem::new(PathBuf::from("bench_data")).await.unwrap())
    });
    let consciousness = rt.block_on(async {
        Arc::new(ConsciousnessEngine::new(memory.clone()).await.unwrap())
    });
    let emotion_manager = Arc::new(EmotionManager::new());
    
    let metrics = Arc::new(MetricsCollector::new(
        consciousness.clone(),
        emotion_manager.clone(),
        memory.clone(),
        1,
    ).unwrap());

    let monitor = ConsciousnessMonitor::new(
        consciousness.clone(),
        emotion_manager.clone(),
        metrics.clone(),
    );

    // Benchmark anomaly detection
    group.bench_function("detect_anomalies", |b| {
        b.iter(|| {
            rt.block_on(async {
                monitor.detect_anomalies().await.unwrap()
            })
        })
    });

    // Benchmark metrics collection
    group.bench_function("collect_metrics", |b| {
        b.iter(|| {
            rt.block_on(async {
                metrics.collect_metrics().await.unwrap()
            })
        })
    });

    group.finish();
}

/// Benchmark parallel processing capabilities
fn bench_parallel(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("parallel");
    group.sample_size(100);
    
    let memory = rt.block_on(async {
        Arc::new(MemorySystem::new(PathBuf::from("bench_data")).await.unwrap())
    });
    let consciousness = rt.block_on(async {
        Arc::new(ConsciousnessEngine::new(memory.clone()).await.unwrap())
    });

    // Benchmark parallel information processing
    group.bench_function("parallel_processing", |b| {
        b.iter(|| {
            rt.block_on(async {
                let futures: Vec<_> = (0..10)
                    .map(|i| {
                        let consciousness = consciousness.clone();
                        tokio::spawn(async move {
                            consciousness
                                .process_information(&format!("parallel test {}", i))
                                .await
                                .unwrap()
                        })
                    })
                    .collect();

                futures::future::join_all(futures).await
            })
        })
    });

    // Benchmark parallel memory operations
    group.bench_function("parallel_memory", |b| {
        b.iter(|| {
            rt.block_on(async {
                let futures: Vec<_> = (0..10)
                    .map(|i| {
                        let memory = memory.clone();
                        tokio::spawn(async move {
                            memory
                                .store(
                                    MemoryLayer::Episodic,
                                    format!("parallel memory test {}", i),
                                )
                                .await
                                .unwrap()
                        })
                    })
                    .collect();

                futures::future::join_all(futures).await
            })
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_consciousness,
    bench_memory,
    bench_holographic,
    bench_emotion,
    bench_monitoring,
    bench_parallel,
);
criterion_main!(benches);