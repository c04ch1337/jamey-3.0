use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::{Index, IndexWriter, TantivyDocument};
use uuid::Uuid;

/// Represents a memory record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRecord {
    pub id: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub layer: MemoryLayer,
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
    #[allow(dead_code)]
    data_dir: PathBuf, // Kept for potential future use (backup paths, etc.)
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
            let schema = schema_builder.build();

            // Try to open existing index, or create new one if it doesn't exist
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

        Ok(Self { indices, data_dir })
    }

    /// Store a memory record in the specified layer
    pub async fn store(&self, layer: MemoryLayer, content: String) -> anyhow::Result<String> {
        let id = Uuid::new_v4().to_string();
        let timestamp = Utc::now();

        let record = MemoryRecord {
            id: id.clone(),
            content: content.clone(),
            timestamp,
            layer,
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

        // Create document
        let mut doc = TantivyDocument::default();
        doc.add_text(id_field, &record.id);
        doc.add_text(content_field, &record.content);
        // Convert chrono DateTime to Tantivy DateTime (Unix timestamp in seconds)
        let tantivy_timestamp = tantivy::DateTime::from_timestamp_secs(timestamp.timestamp());
        doc.add_date(timestamp_field, tantivy_timestamp);

        // Write to index
        let mut index_writer: IndexWriter = index.writer(50_000_000)?;
        index_writer.add_document(doc)?;
        index_writer.commit()?;

        Ok(id)
    }

    /// Search memories in a specific layer
    pub async fn search(
        &self,
        layer: MemoryLayer,
        query: &str,
        limit: usize,
    ) -> anyhow::Result<Vec<MemoryRecord>> {
        let index = self.indices.get(&layer).ok_or_else(|| {
            anyhow::anyhow!("Index not found for layer: {:?}", layer)
        })?;

        let schema = index.schema();
        let content_field = schema.get_field("content")?;
        let id_field = schema.get_field("id")?;
        let timestamp_field = schema.get_field("timestamp")?;

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

            results.push(MemoryRecord {
                id,
                content,
                timestamp,
                layer,
            });
        }

        Ok(results)
    }
}

