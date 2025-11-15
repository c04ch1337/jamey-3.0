# Attention System Improvements Technical Specification

## 1. Focus Heuristics Enhancement

### 1.1 Semantic Content Analysis
```rust
pub struct SemanticAnalyzer {
    // Word embeddings for semantic similarity
    embeddings: HashMap<String, Vec<f32>>,
    // Topic modeling weights
    topic_weights: Vec<f32>,
    // Context window size
    context_window: usize,
}
```

### 1.2 Contextual Relevance Scoring
```rust
pub struct RelevanceScore {
    // Semantic similarity score (0.0-1.0)
    semantic_score: f32,
    // Topic relevance score (0.0-1.0)
    topic_score: f32,
    // Historical context score (0.0-1.0)
    context_score: f32,
    // Temporal relevance decay
    temporal_decay: f32,
}
```

### 1.3 Priority Weighting Framework
```rust
pub struct AttentionWeights {
    semantic_weight: f32,
    topic_weight: f32,
    context_weight: f32,
    temporal_weight: f32,
    urgency_weight: f32,
    novelty_weight: f32,
}
```

## 2. Concurrent Attention Management

### 2.1 Attention State Machine
```rust
pub enum AttentionState {
    Focused(String),      // Current focus
    Switching(String),    // Transitioning to new focus
    Divided(Vec<String>), // Multiple concurrent foci
    Idle,                 // No specific focus
}

pub struct AttentionStateMachine {
    current_state: Arc<RwLock<AttentionState>>,
    state_history: VecDeque<(AttentionState, Instant)>,
    max_concurrent_foci: usize,
    switch_cooldown: Duration,
}
```

### 2.2 Thread-safe Attention Queue
```rust
pub struct AttentionQueue {
    queue: Arc<RwLock<BinaryHeap<AttentionItem>>>,
    capacity: usize,
    processing_threshold: f32,
}

pub struct AttentionItem {
    content: String,
    priority: f32,
    timestamp: Instant,
    source: String,
}
```

### 2.3 Conflict Resolution
```rust
pub struct ConflictResolver {
    priority_threshold: f32,
    min_focus_duration: Duration,
    max_concurrent_items: usize,
    conflict_strategies: Vec<Box<dyn ConflictStrategy>>,
}

pub trait ConflictStrategy: Send + Sync {
    fn resolve(&self, current: &AttentionState, competing: &AttentionItem) -> Resolution;
}
```

## 3. Focus Point History

### 3.1 History Storage
```rust
pub struct AttentionHistory {
    records: Arc<RwLock<VecDeque<AttentionRecord>>>,
    max_history_size: usize,
    retention_period: Duration,
}

pub struct AttentionRecord {
    focus: String,
    state: AttentionState,
    duration: Duration,
    context: HashMap<String, f32>,
    timestamp: DateTime<Utc>,
}
```

### 3.2 Temporal Context Integration
```rust
pub struct TemporalContext {
    short_term: VecDeque<AttentionRecord>,
    medium_term: Vec<AttentionPattern>,
    long_term: HashMap<String, AttentionStatistics>,
}

pub struct AttentionPattern {
    sequence: Vec<String>,
    frequency: usize,
    avg_duration: Duration,
    last_seen: DateTime<Utc>,
}
```

### 3.3 Cleanup and Retention
```rust
pub struct RetentionPolicy {
    short_term_limit: Duration,
    medium_term_limit: Duration,
    long_term_sample_rate: f32,
    importance_threshold: f32,
}
```

## Implementation Guidelines

1. **Focus Heuristics Enhancement**
   - Implement semantic analysis using word embeddings
   - Calculate contextual relevance using sliding windows
   - Apply weighted scoring for priority calculation
   - Implement attention decay using exponential decay function

2. **Concurrent Attention Management**
   - Use atomic operations for state transitions
   - Implement priority queue with thread-safe access
   - Add deadlock prevention in conflict resolution
   - Maintain consistency with global workspace

3. **Focus History Integration**
   - Use efficient circular buffer for history storage
   - Implement periodic cleanup with configurable retention
   - Add compression for long-term storage
   - Maintain indices for fast pattern matching

## Migration Strategy

1. **Phase 1: Focus Heuristics**
   ```rust
   // Update AttentionSchema struct
   pub struct AttentionSchema {
       current_focus: Arc<RwLock<String>>,
       semantic_analyzer: SemanticAnalyzer,
       relevance_scorer: RelevanceScore,
       weights: AttentionWeights,
   }
   ```

2. **Phase 2: Concurrent Management**
   ```rust
   // Extend AttentionSchema
   pub struct AttentionSchema {
       state_machine: AttentionStateMachine,
       queue: AttentionQueue,
       conflict_resolver: ConflictResolver,
       // ... existing fields
   }
   ```

3. **Phase 3: History System**
   ```rust
   // Add history components
   pub struct AttentionSchema {
       history: AttentionHistory,
       temporal_context: TemporalContext,
       retention_policy: RetentionPolicy,
       // ... existing fields
   }
   ```

## Performance Considerations

1. **Memory Usage**
   - Implement circular buffers for history
   - Use reference counting for shared state
   - Compress historical data periodically

2. **Thread Safety**
   - Use fine-grained locking
   - Implement lock-free algorithms where possible
   - Add deadlock detection and prevention

3. **Scalability**
   - Shard history storage for large datasets
   - Implement batch processing for updates
   - Add caching for frequent operations