//! System Performance Benchmarks
//!
//! Comprehensive benchmarks for measuring and optimizing system performance
//! across all major components.

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId, blackbox};
use tokio::runtime::Runtime;
use std::sync::Arc;
use std::path::PathBuf;
use uuid::Uuid;

// Jamey 3.0 components
use jamey_3::consciousness::{
    ConsciousnessEngine,
    global_workspace::{GlobalWorkspace, WorkspaceContent},
    higher_order::HigherOrderThought,
    integrated_info::PhiCalculator,
    predictive::PredictiveProcessor,
};
use jamey_3::memory::{MemorySystem, MemoryLayer, holographic::HolographicMemory};
use jamey_3::soul::emotion::EmotionManager;
use jamey_3::db::{
    self,
    operations::{DatabaseOperations, MemoryRecord},
};

/// Sets up a Tokio runtime for benchmarks.
fn setup_runtime() -> Runtime {
    Runtime::new().unwrap()
}

/// Creates a temporary in-memory database for isolated testing.
async fn setup_test_db() -> (DatabaseOperations, tempfile::TempDir) {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test_bench.db");
    
    // Use an in-memory database by specifying ":memory:" or a temp file
    let pool = db::init_db_with_config(db::DatabaseConfig {
        database_url: format!("sqlite://{}?mode=rwc", db_path.to_str().unwrap()),
        max_connections: 1,
        ..Default::default()
    }).await.unwrap();
    
    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    let metrics = db::DbMetrics::new();
    let db_ops = DatabaseOperations::new(pool, metrics);

    (db_ops, temp_dir)
}

/// Benchmark database operations for creating entities and querying emotional states.
fn bench_database_operations(c: &mut Criterion) {
    let rt = setup_runtime();
    let (db_ops, _temp_dir) = rt.block_on(setup_test_db());
    let db_ops = Arc::new(db_ops);

    let mut group = c.benchmark_group("database");
    group.sample_size(10);

    // Benchmark: Insert a new memory record
    group.bench_function("insert_memory_record", |b| {
        b.to_async(&rt).iter_with_setup(
            || {
                // Setup for each iteration: new content and ID
                (
                    Uuid::new_v4().to_string(),
                    "Test content for insertion benchmark".to_string(),
                    chrono::Utc::now()
                )
            },
            |(id, content, timestamp)| async {
                db_ops.insert_memory_record(
                    &id,
                    &content,
                    &timestamp,
                    "episodic",
                    Some("neutral"),
                    None,
                ).await.unwrap();
            }
        )
    });

    group.finish();
}


/// Benchmark the core consciousness subsystems.
fn bench_consciousness_subsystems(c: &mut Criterion) {
    let rt = setup_runtime();
    let mut group = c.benchmark_group("consciousness_subsystems");

    // --- Global Workspace Benchmark ---
    let workspace = GlobalWorkspace::new();
    let long_content = "This is a meaningful and sufficiently long content to ensure it gets broadcasted.".to_string();
    group.bench_function("global_workspace_broadcast", |b| {
        b.to_async(&rt).iter(|| async {
            workspace.broadcast(&long_content).await.unwrap();
        })
    });

    // --- Higher-Order Thought Benchmark ---
    let hot = HigherOrderThought::new();
    let introspective_content = WorkspaceContent {
        id: Uuid::new_v4(),
        content: "I think about my own thinking process and reflect on my awareness.".to_string(),
        source: "benchmark".to_string(),
        priority: 0.9,
        timestamp: chrono::Utc::now(),
    };
    group.bench_function("higher_order_thought_process", |b| {
        b.to_async(&rt).iter(|| hot.process(blackbox(&introspective_content)))
    });

    // --- Integrated Information (Phi) Benchmark ---
    let phi_calculator = PhiCalculator::new();
    let complex_content = WorkspaceContent {
        id: Uuid::new_v4(),
        content: "This content is designed to be complex, with punctuation! And varied structure to test Phi calculation.".to_string(),
        source: "benchmark".to_string(),
        priority: 0.9,
        timestamp: chrono::Utc::now(),
    };
    group.bench_function("phi_calculation", |b| {
        b.to_async(&rt).iter(|| phi_calculator.calculate(blackbox(&complex_content)))
    });

    // --- Predictive Processing Benchmark ---
    let predictive_processor = PredictiveProcessor::new();
    let thought_for_prediction = "The system is observing multiple data streams, a decision must be made.";
    group.bench_function("predictive_processing", |b| {
        b.iter(|| predictive_processor.process(blackbox(thought_for_prediction)))
    });

    group.finish();
}


criterion_group!(
    benches,
    bench_database_operations,
    bench_consciousness_subsystems,
);
criterion_main!(benches);