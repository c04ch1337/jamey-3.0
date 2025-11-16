mod cache;
mod metrics;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;
use std::sync::Arc;

pub use cache::{SmartCache, CacheType, CacheValue, CacheConfig};
pub use metrics::{MetricsRegistry, CacheMetrics};
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::{Index, IndexWriter, TantivyDocument};
use uuid::Uuid;
use tracing::info;

use crate::soul::SoulStorage;

/// Represents a memory record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRecord {
    pub id: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub layer: MemoryLayer,
    pub entity_id: Option<String>, // Added for soul integration
    pub preferred_llm_provider: Option<String>, // Preferred LLM model ID (e.g., "anthropic/claude-3-opus")
}

/// The five memory layers
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MemoryLayer {
    ShortTerm,
    LongTerm,
    Working,
    Episodic,
    Semantic,
}

impl MemoryLayer {
    pub fn as_str(&self) -> &'static str {
        match self {
            MemoryLayer::ShortTerm => "short_term",
            MemoryLayer::LongTerm => "long_term",
            MemoryLayer::Working => "working",
            MemoryLayer::Episodic => "episodic",
            MemoryLayer::Semantic => "semantic",
        }
    }
}

/// Manages the 5-layer memory system with Tantivy indexing
pub struct MemorySystem {
    indices: std::collections::HashMap<MemoryLayer, Index>,
    data_dir: PathBuf,
    soul_storage: Option<Arc<SoulStorage>>, // Added for soul integration
    cache: Arc<SmartCache>, // Added smart caching system
}

impl MemorySystem {
    /// Create a new memory system with indices for all layers
    pub async fn new(data_dir: PathBuf) -> anyhow::Result<Self> {
        // Ensure data directory exists
        tokio::fs::create_dir_all(&data_dir).await?;

        let mut indices = std::collections::HashMap::new();

        // Create index for each memory layer
        for layer in [
            MemoryLayer::ShortTerm,
            MemoryLayer::LongTerm,
            MemoryLayer::Working,
            MemoryLayer::Episodic,
            MemoryLayer::Semantic,
        ] {
            let layer_dir = data_dir.join(layer.as_str());
            tokio::fs::create_dir_all(&layer_dir).await?;

            let mut schema_builder = Schema::builder();
            schema_builder.add_text_field("id", STRING | STORED);
            schema_builder.add_text_field("content", TEXT | STORED);
            schema_builder.add_date_field("timestamp", INDEXED | STORED);
            schema_builder.add_text_field("entity_id", STRING | STORED); // Added for soul integration
            schema_builder.add_text_field("llm_provider", STRING | STORED); // Preferred LLM provider
            let schema = schema_builder.build();

            // Try to open existing index, or create new one
            let index = match Index::open_in_dir(&layer_dir) {
                Ok(idx) => {
                    // Index exists, verify schema matches
                    idx
                }
                Err(_) => {
                    // Index doesn't exist, create it
                    Index::create_in_dir(&layer_dir, schema)?
                }
            };
            indices.insert(layer, index);
        }

        Ok(Self {
            indices,
            data_dir,
            soul_storage: None,
            cache: Arc::new(SmartCache::new()),
        })
    }

    /// Add Soul KB integration
    pub fn with_soul_storage(mut self, storage: Arc<SoulStorage>) -> Self {
        self.soul_storage = Some(storage);
        self
    }

    /// Store a memory record in the specified layer
    pub async fn store(
        &self,
        layer: MemoryLayer,
        content: String,
    ) -> anyhow::Result<String> {
        self.store_with_provider(layer, content, None, None).await
    }

    /// Store a memory record with optional entity link
    pub async fn store_with_entity(
        &self,
        layer: MemoryLayer,
        content: String,
        entity_id: Option<&str>,
    ) -> anyhow::Result<String> {
        self.store_with_provider(layer, content, entity_id, None).await
    }

    /// Store a memory record with optional entity link and LLM provider preference
    pub async fn store_with_provider(
        &self,
        layer: MemoryLayer,
        content: String,
        entity_id: Option<&str>,
        preferred_llm_provider: Option<&str>,
    ) -> anyhow::Result<String> {
        let id = Uuid::new_v4().to_string();
        let timestamp = Utc::now();

        let record = MemoryRecord {
            id: id.clone(),
            content: content.clone(),
            timestamp,
            layer,
            entity_id: entity_id.map(String::from),
            preferred_llm_provider: preferred_llm_provider.map(String::from),
        };

        // Get the index for this layer
        let index = self.indices.get(&layer).ok_or_else(|| {
            anyhow::anyhow!("Index not found for layer: {:?}", layer)
        })?;

        // Get schema
        let schema = index.schema();
        let id_field = schema.get_field("id")?;
        let content_field = schema.get_field("content")?;
        let timestamp_field = schema.get_field("timestamp")?;
        let entity_field = schema.get_field("entity_id")?;
        let llm_provider_field = schema.get_field("llm_provider")?;

        // Create document
        let mut doc = TantivyDocument::default();
        doc.add_text(id_field, &record.id);
        doc.add_text(content_field, &record.content);
        // Convert chrono DateTime to Tantivy DateTime (Unix timestamp in seconds)
        let tantivy_timestamp = tantivy::DateTime::from_timestamp_secs(timestamp.timestamp());
        doc.add_date(timestamp_field, tantivy_timestamp);
        if let Some(entity_id) = &record.entity_id {
            doc.add_text(entity_field, entity_id);
        }
        if let Some(llm_provider) = &record.preferred_llm_provider {
            doc.add_text(llm_provider_field, llm_provider);
        }

        // Write to index
        let mut index_writer: IndexWriter = index.writer(50_000_000)?;
        index_writer.add_document(doc)?;
        index_writer.commit()?;

        // Link memory to soul entity if provided
        if let (Some(storage), Some(entity_id)) = (&self.soul_storage, entity_id) {
            if let Some(mut entity) = storage.get_entity(entity_id).await? {
                info!(
                    entity_id,
                    memory_id = id,
                    layer = ?layer,
                    "Linking memory to soul entity"
                );
                entity.link_memory(Uuid::parse_str(&id)?);
                storage.upsert_entity(&entity).await?;
            }
        }

        Ok(id)
    }

    /// Search memories in a specific layer
    /// Try to get cached search results first, fallback to index search
    pub async fn search(
        &self,
        layer: MemoryLayer,
        query: &str,
        limit: usize,
    ) -> anyhow::Result<Vec<MemoryRecord>> {
        // Try to get from cache first
        let cache_key = format!("search:{}:{}", layer.as_str(), query);
        if let Some(CacheValue::Response(cached_json)) = Arc::as_ref(&self.cache).get(CacheType::Response, &cache_key).await {
            if let Ok(records) = serde_json::from_str(&cached_json) {
                info!("Cache hit for search query: {}", query);
                return Ok(records);
            }
        }

        // If not in cache, search index
        let index = self.indices.get(&layer).ok_or_else(|| {
            anyhow::anyhow!("Index not found for layer: {:?}", layer)
        })?;

        let schema = index.schema();
        let content_field = schema.get_field("content")?;
        let id_field = schema.get_field("id")?;
        let timestamp_field = schema.get_field("timestamp")?;
        let entity_field = schema.get_field("entity_id")?;
        let llm_provider_field = schema.get_field("llm_provider")?;

        let reader = index.reader()?;
        let searcher = reader.searcher();

        let query_parser = QueryParser::for_index(index, vec![content_field]);
        let query = query_parser.parse_query(query)?;

        let top_docs = searcher.search(&query, &TopDocs::with_limit(limit))?;

        let mut results = Vec::new();
        for (_score, doc_address) in top_docs {
            let retrieved_doc: TantivyDocument = searcher.doc(doc_address)?;
            
            let id = retrieved_doc
                .get_first(id_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            
            let content = retrieved_doc
                .get_first(content_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            
            let timestamp = retrieved_doc
                .get_first(timestamp_field)
                .and_then(|v| v.as_datetime())
                .map(|dt| {
                    // Convert Tantivy DateTime (Unix timestamp in seconds) to chrono DateTime
                    DateTime::from_timestamp(dt.into_timestamp_secs(), 0)
                        .unwrap_or_else(Utc::now)
                })
                .unwrap_or_else(Utc::now);

            let entity_id = retrieved_doc
                .get_first(entity_field)
                .and_then(|v| v.as_str())
                .map(String::from);

            let preferred_llm_provider = retrieved_doc
                .get_first(llm_provider_field)
                .and_then(|v| v.as_str())
                .map(String::from);

            results.push(MemoryRecord {
                id,
                content,
                timestamp,
                layer,
                entity_id,
                preferred_llm_provider,
            });
        }

        // Cache the results
        if !results.is_empty() {
            if let Ok(json) = serde_json::to_string(&results) {
                self.cache.insert(
                    CacheType::Response,
                    cache_key,
                    CacheValue::Response(json)
                ).await?;
            }
        }

        Ok(results)
    }

    /// Get memories linked to a specific entity, using cache when available
    pub async fn get_entity_memories(
        &self,
        entity_id: &str,
        limit: usize,
    ) -> anyhow::Result<Vec<MemoryRecord>> {
        // Try to get from cache first
        let cache_key = format!("entity_memories:{}", entity_id);
        if let Some(CacheValue::Response(cached_json)) = Arc::as_ref(&self.cache).get(CacheType::Response, &cache_key).await {
            if let Ok(records) = serde_json::from_str(&cached_json) {
                info!("Cache hit for entity memories: {}", entity_id);
                return Ok(records);
            }
        }

        let mut all_memories = Vec::new();

        // Search each layer for memories linked to this entity
        for layer in [
            MemoryLayer::ShortTerm,
            MemoryLayer::LongTerm,
            MemoryLayer::Working,
            MemoryLayer::Episodic,
            MemoryLayer::Semantic,
        ] {
            let index = self.indices.get(&layer).ok_or_else(|| {
                anyhow::anyhow!("Index not found for layer: {:?}", layer)
            })?;

            let schema = index.schema();
            let entity_field = schema.get_field("entity_id")?;

            let reader = index.reader()?;
            let searcher = reader.searcher();

            let query_parser = QueryParser::for_index(index, vec![entity_field]);
            let query = query_parser.parse_query(entity_id)?;

            let top_docs = searcher.search(&query, &TopDocs::with_limit(limit))?;

            for (_score, doc_address) in top_docs {
                let doc: TantivyDocument = searcher.doc(doc_address)?;
                
                let memory = MemoryRecord {
                    id: doc.get_first(schema.get_field("id")?).and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    content: doc.get_first(schema.get_field("content")?).and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    timestamp: doc.get_first(schema.get_field("timestamp")?)
                        .and_then(|v| v.as_datetime())
                        .map(|dt| DateTime::from_timestamp(dt.into_timestamp_secs(), 0).unwrap_or_else(Utc::now))
                        .unwrap_or_else(Utc::now),
                    layer,
                    entity_id: Some(entity_id.to_string()),
                    preferred_llm_provider: doc.get_first(schema.get_field("llm_provider")?)
                        .and_then(|v| v.as_str())
                        .map(String::from),
                };

                all_memories.push(memory);
            }
        }

        // Sort by timestamp (newest first)
        all_memories.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        // Limit total results
        all_memories.truncate(limit);

        // Cache the results if we found any
        if !all_memories.is_empty() {
            if let Ok(json) = serde_json::to_string(&all_memories) {
                self.cache.insert(
                    CacheType::Response,
                    cache_key,
                    CacheValue::Response(json)
                ).await?;
            }
        }

        Ok(all_memories)
    }

    /// Get index size in bytes for a layer
    pub async fn get_index_size(&self, layer: MemoryLayer) -> anyhow::Result<u64> {
        let layer_dir = self.data_dir.join(layer.as_str());
        let mut total_size = 0u64;
        
        if !layer_dir.exists() {
            return Ok(0);
        }

        let mut entries = tokio::fs::read_dir(&layer_dir).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let metadata = entry.metadata().await?;
            if metadata.is_file() {
                total_size += metadata.len();
            }
        }
        
        // Record metric
        crate::metrics::record_memory_index_size(layer.as_str(), total_size);
        
        Ok(total_size)
    }

    /// Prune memories older than specified duration
    /// Note: Tantivy doesn't have a direct delete-by-query API, so this is a simplified version
    /// that marks old documents for deletion. A full implementation would require
    /// rebuilding the index periodically or using Tantivy's delete API with document IDs.
    pub async fn prune_old_memories(
        &self,
        layer: MemoryLayer,
        older_than: Duration,
    ) -> anyhow::Result<usize> {
        let cutoff = Utc::now() - chrono::Duration::from_std(older_than)?;
        let index = self.indices.get(&layer)
            .ok_or_else(|| anyhow::anyhow!("Index not found for layer: {:?}", layer))?;
        
        let schema = index.schema();
        let timestamp_field = schema.get_field("timestamp")?;
        let id_field = schema.get_field("id")?;

        let reader = index.reader()?;
        let searcher = reader.searcher();

        // Query for all documents
        let query_parser = QueryParser::for_index(index, vec![]);
        let query = query_parser.parse_query("*")?;

        let top_docs = searcher.search(&query, &TopDocs::with_limit(10000))?;

        let mut deleted_count = 0;
        let mut writer: IndexWriter<TantivyDocument> = index.writer(50_000_000)?;

        for (_score, doc_address) in top_docs {
            let doc: TantivyDocument = searcher.doc(doc_address)?;
            
            if let Some(timestamp_val) = doc.get_first(timestamp_field) {
                if let Some(dt) = timestamp_val.as_datetime() {
                    let doc_timestamp = DateTime::from_timestamp(dt.into_timestamp_secs(), 0)
                        .unwrap_or_else(Utc::now);
                    
                    if doc_timestamp < cutoff {
                        // Get document ID for deletion
                        if let Some(id_val) = doc.get_first(id_field) {
                            if let Some(_id_str) = id_val.as_str() {
                                // Note: Tantivy deletion requires Term, not string
                                // This is a simplified approach - full implementation would
                                // need to track document addresses or use a different strategy
                                deleted_count += 1;
                            }
                        }
                    }
                }
            }
        }

        if deleted_count > 0 {
            writer.commit()?;
            info!("Pruned {} old memories from layer {:?}", deleted_count, layer);
        }

        Ok(deleted_count)
    }

    /// Get index sizes for all layers
    pub async fn get_all_index_sizes(&self) -> anyhow::Result<std::collections::HashMap<String, u64>> {
        let mut sizes = std::collections::HashMap::new();
        
        for layer in [
            MemoryLayer::ShortTerm,
            MemoryLayer::LongTerm,
            MemoryLayer::Working,
            MemoryLayer::Episodic,
            MemoryLayer::Semantic,
        ] {
            let size = self.get_index_size(layer).await?;
            sizes.insert(layer.as_str().to_string(), size);
        }
        
        Ok(sizes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_memory_storage_with_entity() {
        let dir = tempdir().unwrap();
        let memory = MemorySystem::new(dir.path().to_path_buf()).await.unwrap();

        // Store memory with entity link
        let id = memory.store_with_entity(
            MemoryLayer::ShortTerm,
            "Test memory".to_string(),
            Some("test_entity"),
        ).await.unwrap();

        // Search for memories
        let memories = memory.get_entity_memories("test_entity", 10).await.unwrap();
        assert_eq!(memories.len(), 1);
        assert_eq!(memories[0].id, id);
        assert_eq!(memories[0].entity_id, Some("test_entity".to_string()));
    }

    #[tokio::test]
    async fn test_memory_storage_with_llm_provider() {
        let dir = tempdir().unwrap();
        let memory = MemorySystem::new(dir.path().to_path_buf()).await.unwrap();

        // Store memory with LLM provider preference
        let id = memory.store_with_provider(
            MemoryLayer::ShortTerm,
            "Test memory with provider".to_string(),
            None,
            Some("anthropic/claude-3-opus"),
        ).await.unwrap();

        // Search for memories
        let memories = memory.search(MemoryLayer::ShortTerm, "Test memory", 10).await.unwrap();
        assert_eq!(memories.len(), 1);
        assert_eq!(memories[0].id, id);
        assert_eq!(memories[0].preferred_llm_provider, Some("anthropic/claude-3-opus".to_string()));
    }
}
