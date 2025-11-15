use anyhow::Result;
use clap::{Parser, Subcommand};
use jamey_3::cli::ChatCLI;
use jamey_3::config::Config;
use jamey_3::conscience::ConscienceEngine;
use jamey_3::db;
use jamey_3::memory::MemorySystem;
use jamey_3::soul::{Emotion, SoulEntity, SoulStorage};
use jamey_3::soul::emotion::EmotionType;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::info;
use tracing_subscriber;

#[derive(Parser)]
#[command(name = "jamey", version = "3.0.0")]
#[command(about = "Jamey 3.0 - AI Assistant with Soul", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Interactive chat with Jamey
    Chat,
    
    /// Soul knowledge base commands
    #[command(subcommand)]
    Soul(SoulCommands),
}

#[derive(Subcommand)]
enum SoulCommands {
    /// Add or update an entity with a trust score
    Upsert {
        /// Entity name
        entity: String,
        /// Trust score (0.0 to 1.0)
        trust: f32,
    },
    
    /// Record an emotion for an entity
    Record {
        /// Entity name
        entity: String,
        /// Emotion (joy/😊, sad/😢, angry/😡, neutral/😐, love/😍)
        emotion: String,
    },
    
    /// Show soul status for entity(ies)
    Status {
        /// Entity name (optional, shows all if not provided)
        entity: Option<String>,
    },
    
    /// Apply trust decay to all entities based on time since last interaction
    Decay,
    
    /// Delete an entity from the soul KB
    Delete {
        /// Entity name
        entity: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    
    match cli.command {
        Commands::Chat => {
            run_chat().await?;
        }
        Commands::Soul(soul_cmd) => {
            handle_soul_command(soul_cmd).await?;
        }
    }
    
    Ok(())
}

async fn run_chat() -> Result<()> {
    info!("Starting Jamey 3.0 CLI - General & Guardian");

    // Load configuration (required for CLI)
    let config = match Config::from_env_required() {
        Ok(cfg) => {
            cfg.validate()?;
            info!("OpenRouter API key loaded");
            Arc::new(cfg)
        }
        Err(e) => {
            eprintln!("\n❌ Configuration Error: {}\n", e);
            eprintln!("📝 To fix this:");
            eprintln!("   1. Create a .env file in the project root");
            eprintln!("   2. Add: OPENROUTER_API_KEY=your-api-key-here");
            eprintln!("   3. Get your key from: https://openrouter.ai/keys\n");
            eprintln!("   See API_KEY_SETUP.md for detailed instructions.\n");
            return Err(e);
        }
    };

    // Initialize database (optional, but good to have)
    if let Err(e) = jamey_3::db::init_db().await {
        tracing::warn!("Database initialization failed: {}. Continuing without database.", e);
    } else {
        info!("Database initialized");
    }

    // Initialize conscience engine
    let conscience = Arc::new(ConscienceEngine::new());
    info!("Conscience Engine initialized");

    // Initialize memory system
    let data_dir = PathBuf::from("data/memory");
    let memory = match MemorySystem::new(data_dir).await {
        Ok(mem) => {
            info!("Memory System initialized");
            Arc::new(mem)
        }
        Err(e) => {
            eprintln!("⚠️  Warning: Memory system initialization failed: {}", e);
            eprintln!("   Continuing without memory system...\n");
            // Create a dummy memory system - this is not ideal but allows CLI to run
            return Err(anyhow::anyhow!("Memory system required for CLI"));
        }
    };

    // Create and run CLI
    let mut cli = ChatCLI::new(config, conscience, memory);
    cli.run().await?;

    Ok(())
}

async fn handle_soul_command(cmd: SoulCommands) -> Result<()> {
    // Initialize database with migrations
    let pool = db::init_db().await?;
    let storage = SoulStorage::new(pool);
    
    match cmd {
        SoulCommands::Upsert { entity, trust } => {
            soul_upsert(&storage, &entity, trust).await?;
        }
        SoulCommands::Record { entity, emotion } => {
            soul_record(&storage, &entity, &emotion).await?;
        }
        SoulCommands::Status { entity } => {
            soul_status(&storage, entity.as_deref()).await?;
        }
        SoulCommands::Decay => {
            soul_decay(&storage).await?;
        }
        SoulCommands::Delete { entity } => {
            soul_delete(&storage, &entity).await?;
        }
    }
    
    Ok(())
}

async fn soul_upsert(storage: &SoulStorage, entity_name: &str, trust: f32) -> Result<()> {
    // Validate trust score
    if trust < 0.0 || trust > 1.0 {
        anyhow::bail!("Trust score must be between 0.0 and 1.0");
    }
    
    let mut entity = storage.get_entity(entity_name)
        .await?
        .unwrap_or_else(|| SoulEntity::new(entity_name.to_string()));
    
    entity.trust_score = trust;
    storage.upsert_entity(&entity).await?;
    
    println!("\n✅ Entity '{}' updated:", entity_name);
    println!("   Trust: {:.2}", entity.trust_score);
    println!("   Decay Rate: {:.4}\n", entity.decay_rate);
    
    Ok(())
}

async fn soul_record(storage: &SoulStorage, entity_name: &str, emotion_str: &str) -> Result<()> {
    let emotion_type = match emotion_str.to_lowercase().as_str() {
        "joy" => EmotionType::Joy,
        "love" => EmotionType::PaternalLove,
        "protect" => EmotionType::ProtectiveConcern,
        "pride" => EmotionType::Pride,
        "worry" => EmotionType::Worry,
        "calm" => EmotionType::Calm,
        "focus" => EmotionType::Focus,
        _ => EmotionType::General(emotion_str.to_string()),
    };
    
    let emotion = Emotion {
        id: uuid::Uuid::new_v4(),
        emotion_type,
        intensity: 0.8,
        target: None,
        timestamp: chrono::Utc::now(),
        duration: 0.0,
    };
    
    let mut entity = storage.get_entity(entity_name)
        .await?
        .unwrap_or_else(|| SoulEntity::new(entity_name.to_string()));
    
    let emotion_type = emotion.emotion_type.clone();
    entity.record_emotion(emotion);
    
    // Update empathy and trust
    entity.boost_trust();
    
    storage.upsert_entity(&entity).await?;
    
    println!("\n{} Recorded {} for '{}'", emotion_type.emoji(), emotion_type.name(), entity_name);
    println!("   Updated Trust: {:.2}", entity.trust_score);
    println!("   Empathy: {:.2}\n", entity.empathy_score());
    
    Ok(())
}

async fn soul_status(storage: &SoulStorage, entity_name: Option<&str>) -> Result<()> {
    match entity_name {
        Some(name) => {
            // Show single entity
            match storage.get_entity(name).await? {
                Some(entity) => print_entity_status(&entity),
                None => println!("\n⚠️  Entity '{}' not found\n", name),
            }
        }
        None => {
            // Show all entities
            let entities = storage.get_all_entities().await?;
            if entities.is_empty() {
                println!("\n📭 No entities tracked yet\n");
            } else {
                println!("\n👥 Tracked Entities ({})\n", entities.len());
                
                // Sort by trust score descending
                let mut sorted = entities;
                sorted.sort_by(|a, b| b.trust_score.partial_cmp(&a.trust_score).unwrap());
                
                // Show top 5 or all if less than 5
                let display_count = sorted.len().min(5);
                for entity in sorted.iter().take(display_count) {
                    print_entity_summary(entity);
                }
                
                if sorted.len() > 5 {
                    println!("\n   ... and {} more entities", sorted.len() - 5);
                    println!("   Use 'jamey soul status <entity>' for details\n");
                }
            }
        }
    }
    
    Ok(())
}

fn print_entity_status(entity: &SoulEntity) {
    let emoji = entity.dominant_emotion()
        .map(|e| e.emoji())
        .unwrap_or("😐");
    
    println!("\n╔══════════════════════════════════════╗");
    println!("║  {} Entity: {:20} ║", emoji, entity.entity_name);
    println!("╚══════════════════════════════════════╝");
    println!("Trust: {:.2} {}", entity.trust_score, trust_emoji(entity.trust_score));
    println!("Decay Rate: {:.4}/day", entity.decay_rate);
    println!("Last Seen: {}", entity.last_interaction.format("%Y-%m-%d %H:%M:%S"));
    println!("Empathy: {:.2}", entity.empathy_score());
    
    if !entity.emotions.is_empty() {
        println!("\nEmotions:");
        for (emotion, count) in &entity.emotions {
            let bar = "█".repeat((*count as usize).min(20));
            println!("  {} {:8} {} ({})", emotion.emoji(), emotion.name(), bar, count);
        }
    }
    
    if let Some(dominant) = entity.dominant_emotion() {
        println!("\nDominant: {} {}", dominant.emoji(), dominant.name());
    }
    
    println!("Linked Memories: {}\n", entity.linked_memories.len());
}

fn print_entity_summary(entity: &SoulEntity) {
    let emoji = entity.dominant_emotion()
        .map(|e| e.emoji())
        .unwrap_or("😐");
    
    let empathy = entity.empathy_score();
    
    println!("  {} {} | Trust: {:.2} | Empathy: {:.2} | Decay: {:.3}",
        emoji,
        entity.entity_name,
        entity.trust_score,
        empathy,
        entity.decay_rate
    );
}

fn trust_emoji(trust: f32) -> &'static str {
    if trust > 0.8 { "🌟" }
    else if trust > 0.6 { "✨" }
    else if trust > 0.4 { "💫" }
    else if trust > 0.2 { "⚠️" }
    else { "❌" }
}

async fn soul_decay(storage: &SoulStorage) -> Result<()> {
    println!("\n⏰ Applying trust decay to all entities...");
    
    let count = storage.apply_global_decay().await?;
    
    println!("✅ Updated {} entities\n", count);
    
    Ok(())
}

async fn soul_delete(storage: &SoulStorage, entity_name: &str) -> Result<()> {
    storage.delete_entity(entity_name).await?;
    println!("\n✅ Entity '{}' deleted\n", entity_name);
    
    Ok(())
}
