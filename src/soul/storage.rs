use sqlx::{SqlitePool, Row};
use anyhow::Result;
use super::entity::SoulEntity;
use super::emotion::Emotion;
use uuid::Uuid;

pub struct SoulStorage {
    pool: SqlitePool,
}

impl SoulStorage {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
    
    /// Upsert (insert or update) a soul entity
    pub async fn upsert_entity(&self, entity: &SoulEntity) -> Result<i64> {
        let result = sqlx::query(
            r#"
            INSERT INTO soul_entities (entity_name, trust_score, decay_rate, last_interaction, created_at)
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT(entity_name) DO UPDATE SET
                trust_score = excluded.trust_score,
                decay_rate = excluded.decay_rate,
                last_interaction = excluded.last_interaction
            "#
        )
        .bind(&entity.entity_name)
        .bind(entity.trust_score)
        .bind(entity.decay_rate)
        .bind(entity.last_interaction)
        .bind(entity.created_at)
        .execute(&self.pool)
        .await?;
        
        // Get the entity ID
        let entity_id = if result.rows_affected() > 0 {
            let row = sqlx::query("SELECT id FROM soul_entities WHERE entity_name = ?")
                .bind(&entity.entity_name)
                .fetch_one(&self.pool)
                .await?;
            row.get::<i64, _>("id")
        } else {
            entity.id
        };
        
        // Update emotions
        for (emotion, count) in &entity.emotions {
            self.record_emotion(entity_id, *emotion, *count).await?;
        }
        
        // Update memory links
        for memory_id in &entity.linked_memories {
            self.link_memory(entity_id, *memory_id).await?;
        }
        
        Ok(entity_id)
    }
    
    /// Get an entity by name
    pub async fn get_entity(&self, name: &str) -> Result<Option<SoulEntity>> {
        let row = sqlx::query(
            "SELECT id, entity_name, trust_score, decay_rate, last_interaction, created_at FROM soul_entities WHERE entity_name = ?"
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;
        
        match row {
            Some(row) => {
                let entity_id: i64 = row.get("id");
                
                // Load emotions
                let emotions = self.get_emotions(entity_id).await?;
                
                // Load memory links
                let linked_memories = self.get_entity_memories(entity_id).await?;
                
                Ok(Some(SoulEntity {
                    id: entity_id,
                    entity_name: row.get("entity_name"),
                    trust_score: row.get("trust_score"),
                    decay_rate: row.get("decay_rate"),
                    last_interaction: row.get("last_interaction"),
                    created_at: row.get("created_at"),
                    emotions,
                    linked_memories,
                }))
            }
            None => Ok(None),
        }
    }
    
    /// Record an emotion for an entity
    pub async fn record_emotion(&self, entity_id: i64, emotion: Emotion, count: u32) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO soul_emotions (entity_id, emotion, count)
            VALUES (?, ?, ?)
            ON CONFLICT(entity_id, emotion) DO UPDATE SET
                count = excluded.count
            "#
        )
        .bind(entity_id)
        .bind(emotion.name())
        .bind(count as i64)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    /// Get emotions for an entity
    async fn get_emotions(&self, entity_id: i64) -> Result<std::collections::HashMap<Emotion, u32>> {
        let rows = sqlx::query(
            "SELECT emotion, count FROM soul_emotions WHERE entity_id = ?"
        )
        .bind(entity_id)
        .fetch_all(&self.pool)
        .await?;
        
        let mut emotions = std::collections::HashMap::new();
        for row in rows {
            let emotion_str: String = row.get("emotion");
            if let Some(emotion) = Emotion::from_str(&emotion_str) {
                let count: i64 = row.get("count");
                emotions.insert(emotion, count as u32);
            }
        }
        
        Ok(emotions)
    }
    
    /// Link a memory UUID to an entity
    pub async fn link_memory(&self, entity_id: i64, memory_id: Uuid) -> Result<()> {
        sqlx::query(
            r#"
            INSERT OR IGNORE INTO soul_memory_links (entity_id, memory_id)
            VALUES (?, ?)
            "#
        )
        .bind(entity_id)
        .bind(memory_id.to_string())
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    /// Get all memory IDs linked to an entity
    pub async fn get_entity_memories(&self, entity_id: i64) -> Result<Vec<Uuid>> {
        let rows = sqlx::query(
            "SELECT memory_id FROM soul_memory_links WHERE entity_id = ?"
        )
        .bind(entity_id)
        .fetch_all(&self.pool)
        .await?;
        
        let mut links = Vec::new();
        for row in rows {
            let memory_id_str: String = row.get("memory_id");
            if let Ok(uuid) = Uuid::parse_str(&memory_id_str) {
                links.push(uuid);
            }
        }
        
        Ok(links)
    }
    
    /// Get all entities
    pub async fn get_all_entities(&self) -> Result<Vec<SoulEntity>> {
        let rows = sqlx::query(
            "SELECT entity_name FROM soul_entities"
        )
        .fetch_all(&self.pool)
        .await?;
        
        let mut entities = Vec::new();
        for row in rows {
            let name: String = row.get("entity_name");
            if let Some(entity) = self.get_entity(&name).await? {
                entities.push(entity);
            }
        }
        
        Ok(entities)
    }
    
    /// Delete an entity
    pub async fn delete_entity(&self, name: &str) -> Result<()> {
        sqlx::query("DELETE FROM soul_entities WHERE entity_name = ?")
            .bind(name)
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }
    
    /// Apply decay to all entities based on time since last interaction
    pub async fn apply_global_decay(&self) -> Result<usize> {
        let entities = self.get_all_entities().await?;
        let mut updated = 0;
        
        for mut entity in entities {
            let now = chrono::Utc::now();
            let days = (now - entity.last_interaction).num_days() as f64;
            
            if days > 0.0 {
                entity.apply_decay(days);
                self.upsert_entity(&entity).await?;
                updated += 1;
            }
        }
        
        Ok(updated)
    }
}