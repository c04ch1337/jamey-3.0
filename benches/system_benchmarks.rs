//! System Performance Benchmarks
//!
//! Comprehensive benchmarks for measuring and optimizing system performance
//! across all major components.

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId, Throughput, black_box};
use pprof::criterion::{Output, PProfProfiler};
use tokio::runtime::Runtime;
use std::sync::Arc;
use std::path::PathBuf;
use uuid::Uuid;
use futures::future::join_all;
use std::time::Duration;
use sysinfo::{System, SystemExt};
use tracing::info;
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::fs;

/// Custom allocator wrapper for tracking allocations
#[global_allocator]
static ALLOCATOR: AllocationTracker = AllocationTracker::new();

struct AllocationTracker {
    allocator: System,
    allocation_count: AtomicUsize,
    bytes_allocated: AtomicUsize,
}

impl AllocationTracker {
    const fn new() -> Self {
        Self {
            allocator: System,
            allocation_count: AtomicUsize::new(0),
            bytes_allocated: AtomicUsize::new(0),
        }
    }

    fn get_stats(&self) -> (usize, usize) {
        (
            self.allocation_count.load(Ordering::Relaxed),
            self.bytes_allocated.load(Ordering::Relaxed),
        )
    }

    fn reset_stats(&self) {
        self.allocation_count.store(0, Ordering::Relaxed);
        self.bytes_allocated.store(0, Ordering::Relaxed);
    }
}

unsafe impl GlobalAlloc for AllocationTracker {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.allocation_count.fetch_add(1, Ordering::Relaxed);
        self.bytes_allocated.fetch_add(layout.size(), Ordering::Relaxed);
        self.allocator.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.bytes_allocated.fetch_sub(layout.size(), Ordering::Relaxed);
        self.allocator.dealloc(ptr, layout)
    }
}

// Jamey 3.0 components
use jamey_3::consciousness::{
    ConsciousnessEngine,
    global_workspace::{GlobalWorkspace, WorkspaceContent},
    higher_order::HigherOrderThought,
    integrated_info::PhiCalculator,
    predictive::PredictiveProcessor,
    cache::ConsciousnessCache,
};
use jamey_3::memory::{MemorySystem, MemoryLayer, holographic::HolographicMemory};
use jamey_3::soul::emotion::EmotionManager;
use jamey_3::monitoring::MetricsCollector;
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


/// Benchmark cache operations with detailed performance metrics
fn bench_cache_operations(c: &mut Criterion) {
    let rt = setup_runtime();
    let metrics = Arc::new(MetricsCollector::new("cache_bench"));
    let cache = Arc::new(ConsciousnessCache::new(metrics));

    let mut group = c.benchmark_group("cache_operations");
    group.throughput(Throughput::Elements(1));
    group.sample_size(100);
    
    // Track memory usage before benchmarks
    let initial_memory = get_memory_usage();

    // Benchmark: Cache Set Operation with size tracking
    group.bench_function("cache_set", |b| {
        b.to_async(&rt).iter_with_setup(
            || {
                (
                    format!("key_{}", Uuid::new_v4()),
                    format!("value_{}", Uuid::new_v4())
                )
            },
            |(key, value)| async {
                black_box(cache.set(&key, value).await);
            }
        )
    });

    // Benchmark: Cache Get Operation with hit ratio tracking
    let test_key = "test_key".to_string();
    let test_value = "test_value".to_string();
    rt.block_on(cache.set(&test_key, test_value.clone()));

    let mut hits = 0;
    let mut total = 0;
    group.bench_function("cache_get", |b| {
        b.to_async(&rt).iter(|| async {
            total += 1;
            if let Some(_) = black_box(cache.get(&test_key).await) {
                hits += 1;
            }
        })
    });

    // Benchmark: Cache Get with Miss and latency tracking
    group.bench_function("cache_get_miss", |b| {
        b.to_async(&rt).iter_with_setup(
            || format!("missing_key_{}", Uuid::new_v4()),
            |key| async {
                let start = std::time::Instant::now();
                black_box(cache.get(&key).await);
                metrics::histogram!("cache_miss_latency", start.elapsed().as_secs_f64());
            }
        )
    });

    // Benchmark: Cache under load (concurrent operations)
    group.bench_function("cache_concurrent_ops", |b| {
        b.to_async(&rt).iter_with_setup(
            || {
                let mut futures = Vec::new();
                for i in 0..100 {
                    let cache = cache.clone();
                    let key = format!("concurrent_key_{}", i);
                    let value = format!("concurrent_value_{}", i);
                    futures.push(async move {
                        cache.set(&key, value).await;
                        cache.get(&key).await
                    });
                }
                futures
            },
            |futures| async {
                black_box(join_all(futures).await);
            }
        )
    });

    // Benchmark: Cache eviction behavior
    group.bench_function("cache_eviction", |b| {
        b.to_async(&rt).iter_with_setup(
            || {
                // Fill cache beyond capacity
                for i in 0..12_000 {
                    let key = format!("eviction_key_{}", i);
                    let value = format!("eviction_value_{}", i);
                    rt.block_on(cache.set(&key, value));
                }
            },
            |_| async {
                // Measure time to access keys that should trigger eviction
                let start = std::time::Instant::now();
                for i in 0..100 {
                    let key = format!("eviction_key_{}", i);
                    black_box(cache.get(&key).await);
                }
                metrics::histogram!("cache_eviction_latency", start.elapsed().as_secs_f64());
            }
        )
    });

    // Report cache performance metrics
    let final_memory = get_memory_usage();
    let memory_delta = final_memory - initial_memory;
    let hit_ratio = hits as f64 / total as f64;

    info!(
        "Cache Benchmark Results:",
        "hit_ratio" = hit_ratio,
        "memory_usage_delta_bytes" = memory_delta,
    );

    group.finish();
}

/// Get current memory usage
fn get_memory_usage() -> usize {
    let mut sys = System::new_all();
    sys.refresh_all();
    sys.used_memory() as usize
}

/// Benchmark async communication patterns
fn bench_async_communication(c: &mut Criterion) {
    let rt = setup_runtime();
    let workspace = Arc::new(GlobalWorkspace::new());
    
    let mut group = c.benchmark_group("async_communication");
    group.throughput(Throughput::Elements(1));
    group.sample_size(100);

    // Benchmark: Message Broadcasting
    group.bench_function("broadcast_message", |b| {
        b.to_async(&rt).iter_with_setup(
            || format!("test_message_{}", Uuid::new_v4()),
            |msg| async {
                black_box(workspace.broadcast(&msg).await.unwrap());
            }
        )
    });

    // Benchmark: Concurrent Message Processing
    group.bench_function("concurrent_processing", |b| {
        b.to_async(&rt).iter_with_setup(
            || {
                let mut futures = Vec::new();
                for _ in 0..10 {
                    let workspace = workspace.clone();
                    let msg = format!("concurrent_msg_{}", Uuid::new_v4());
                    futures.push(workspace.broadcast(&msg));
                }
                futures
            },
            |futures| async {
                black_box(join_all(futures).await);
            }
        )
    });

    group.finish();
}

/// Benchmark system-wide performance
fn bench_system_performance(c: &mut Criterion) {
    let rt = setup_runtime();
    let engine = Arc::new(ConsciousnessEngine::new());
    let metrics = Arc::new(MetricsCollector::new("system_bench"));

    let mut group = c.benchmark_group("system_performance");
    group.sample_size(50);
    group.measurement_time(Duration::from_secs(10));

    // Benchmark: Full System Pipeline
    group.bench_function("full_pipeline", |b| {
        b.to_async(&rt).iter_with_setup(
            || {
                let content = WorkspaceContent {
                    id: Uuid::new_v4(),
                    content: "Test content for full system pipeline".to_string(),
                    source: "benchmark".to_string(),
                    priority: 0.8,
                    timestamp: chrono::Utc::now(),
                };
                (engine.clone(), content)
            },
            |(engine, content)| async {
                black_box(engine.process_content(&content).await);
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


/// Benchmark I/O operations (file and network)
fn bench_io_operations(c: &mut Criterion) {
    let rt = setup_runtime();
    let mut group = c.benchmark_group("io_operations");
    group.sample_size(50);
    
    // Prepare temp directory for file operations
    let temp_dir = tempfile::tempdir().unwrap();
    let test_file_path = temp_dir.path().join("benchmark_test_file.txt");
    let test_data = "Test data for I/O benchmark".repeat(1000); // ~20KB of data
    
    // Benchmark: File Write Operation
    group.bench_function("file_write", |b| {
        b.iter_with_setup(
            || test_data.clone(),
            |data| {
                std::fs::write(&test_file_path, data).unwrap();
            }
        )
    });

    // Benchmark: File Read Operation
    group.bench_function("file_read", |b| {
        b.iter(|| {
            black_box(std::fs::read_to_string(&test_file_path).unwrap());
        })
    });

    // Benchmark: File Append Operation
    group.bench_function("file_append", |b| {
        b.iter_with_setup(
            || {
                std::fs::write(&test_file_path, "").unwrap(); // Clear file
                "Append test data\n".to_string()
            },
            |data| {
                use std::fs::OpenOptions;
                let mut file = OpenOptions::new()
                    .append(true)
                    .open(&test_file_path)
                    .unwrap();
                std::io::Write::write_all(&mut file, data.as_bytes()).unwrap();
            }
        )
    });

    // Benchmark: Async File Operations
    group.bench_function("async_file_ops", |b| {
        b.to_async(&rt).iter_with_setup(
            || test_data.clone(),
            |data| async {
                use tokio::fs;
                // Write
                fs::write(&test_file_path, &data).await.unwrap();
                // Read
                let _ = black_box(fs::read_to_string(&test_file_path).await.unwrap());
            }
        )
    });

    // Benchmark: Network Operations (HTTP requests)
    let client = reqwest::Client::new();
    group.bench_function("http_get", |b| {
        b.to_async(&rt).iter(|| async {
            let response = client.get("http://localhost:8080/health")
                .timeout(Duration::from_secs(1))
                .send()
                .await;
            
            // Log latency but don't fail benchmark on connection errors
            if let Ok(resp) = response {
                black_box(resp);
            }
        })
    });

    // Benchmark: TCP Socket Operations
    group.bench_function("tcp_echo", |b| {
        b.to_async(&rt).iter_with_setup(
            || {
                use tokio::net::{TcpListener, TcpStream};
                async {
                    // Start echo server
                    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
                    let addr = listener.local_addr().unwrap();
                    
                    let server = tokio::spawn(async move {
                        if let Ok((mut socket, _)) = listener.accept().await {
                            let mut buf = [0u8; 1024];
                            if let Ok(n) = socket.try_read(&mut buf) {
                                let _ = socket.try_write(&buf[..n]);
                            }
                        }
                    });
                    
                    (TcpStream::connect(addr).await.unwrap(), server)
                }
            },
            |(mut stream, _server)| async move {
                let data = b"benchmark test data";
                stream.write_all(data).await.unwrap();
                let mut buf = [0u8; 1024];
                let n = stream.read(&mut buf).await.unwrap();
                black_box(&buf[..n]);
            }
        )
    });

    group.finish();
}

/// Benchmark memory operations and allocation patterns
fn bench_memory_operations(c: &mut Criterion) {
    let rt = setup_runtime();
    let mut group = c.benchmark_group("memory_operations");
    group.sample_size(50);
    
    // Reset allocation tracking before benchmarks
    ALLOCATOR.reset_stats();

    // Benchmark: Vector Allocation Pattern
    group.bench_function("vector_allocation", |b| {
        b.iter_with_setup(
            || ALLOCATOR.reset_stats(),
            |_| {
                let mut data = Vec::with_capacity(1000);
                for i in 0..1000 {
                    data.push(i);
                }
                black_box(data);
                let (allocs, bytes) = ALLOCATOR.get_stats();
                metrics::gauge!("vector_allocations", allocs as f64);
                metrics::gauge!("vector_bytes_allocated", bytes as f64);
            }
        )
    });

    // Benchmark: String Operations
    group.bench_function("string_operations", |b| {
        b.iter_with_setup(
            || ALLOCATOR.reset_stats(),
            |_| {
                let mut s = String::with_capacity(1000);
                for _ in 0..100 {
                    s.push_str("benchmark test ");
                }
                black_box(s);
                let (allocs, bytes) = ALLOCATOR.get_stats();
                metrics::gauge!("string_allocations", allocs as f64);
                metrics::gauge!("string_bytes_allocated", bytes as f64);
            }
        )
    });

    // Benchmark: Memory Pressure Test
    group.bench_function("memory_pressure", |b| {
        b.iter_with_setup(
            || {
                ALLOCATOR.reset_stats();
                Vec::new()
            },
            |mut data| {
                // Allocate and deallocate in a pattern that creates memory pressure
                for _ in 0..10 {
                    // Grow
                    for _ in 0..1000 {
                        data.push(vec![0u8; 1000]);
                    }
                    // Shrink
                    for _ in 0..500 {
                        data.pop();
                    }
                }
                black_box(&data);
                let (allocs, bytes) = ALLOCATOR.get_stats();
                metrics::gauge!("pressure_test_allocations", allocs as f64);
                metrics::gauge!("pressure_test_bytes", bytes as f64);
            }
        )
    });

    // Benchmark: Arena Allocation Pattern
    group.bench_function("arena_allocation", |b| {
        use bumpalo::Bump;
        
        b.iter_with_setup(
            || {
                ALLOCATOR.reset_stats();
                Bump::new()
            },
            |bump| {
                for _ in 0..1000 {
                    let _data = bump.alloc_slice_fill_default::<u8>(100);
                }
                let (allocs, bytes) = ALLOCATOR.get_stats();
                metrics::gauge!("arena_allocations", allocs as f64);
                metrics::gauge!("arena_bytes_allocated", bytes as f64);
            }
        )
    });

    // Benchmark: Concurrent Allocation
    group.bench_function("concurrent_allocation", |b| {
        b.to_async(&rt).iter_with_setup(
            || {
                ALLOCATOR.reset_stats();
                let mut handles = Vec::new();
                for _ in 0..10 {
                    handles.push(tokio::spawn(async {
                        let mut data = Vec::new();
                        for _ in 0..100 {
                            data.push(vec![0u8; 100]);
                        }
                        black_box(data)
                    }));
                }
                handles
            },
            |handles| async {
                for handle in handles {
                    handle.await.unwrap();
                }
                let (allocs, bytes) = ALLOCATOR.get_stats();
                metrics::gauge!("concurrent_allocations", allocs as f64);
                metrics::gauge!("concurrent_bytes_allocated", bytes as f64);
            }
        )
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_database_operations,
    bench_consciousness_subsystems,
    bench_cache_operations,
    bench_async_communication,
    bench_system_performance,
    bench_io_operations,
    bench_memory_operations,
);
criterion_main!(benches);