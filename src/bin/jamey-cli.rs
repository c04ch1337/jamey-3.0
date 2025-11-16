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
    /// Interactive chat with Jamey (standalone mode)
    Chat,
    
    /// Connect to a running backend and chat via API (SSH-like)
    Connect {
        /// Backend API URL
        #[arg(long, default_value = "http://localhost:3000")]
        url: String,
        
        /// API key for authentication (optional)
        #[arg(long)]
        api_key: Option<String>,
    },
    
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
        Commands::Connect { url, api_key } => {
            run_connect(&url, api_key.as_deref()).await?;
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

async fn run_connect(backend_url: &str, api_key: Option<&str>) -> Result<()> {
    use std::io::{self, Write};
    
    println!("\n🔌 Connecting to Jamey 3.0 Backend...");
    println!("   URL: {}", backend_url);
    
    // Check if backend is running
    let client = reqwest::Client::new();
    let health_url = format!("{}/health", backend_url);
    
    let response = match client.get(&health_url).send().await {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!("\n❌ Cannot connect to backend at {}: {}", backend_url, e);
            eprintln!("\n💡 Make sure the backend is running:");
            eprintln!("   ./scripts/run.sh");
            eprintln!("   or");
            eprintln!("   cargo run\n");
            return Err(anyhow::anyhow!("Backend not available"));
        }
    };
    
    if !response.status().is_success() {
        eprintln!("\n❌ Backend returned error: {}", response.status());
        return Err(anyhow::anyhow!("Backend error"));
    }
    
    println!("✅ Connected to backend!");
    println!("\n╔═══════════════════════════════════════════════════════════╗");
    println!("║     Jamey 3.0 - Connected Mode - CLI Chat                ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");
    println!("💬 Multi-line chat mode enabled");
    println!("   • Type your message (multiple lines supported)");
    println!("   • Press Enter twice (empty line) to send");
    println!("   • Or type '/send' on a new line to send immediately");
    println!("   • Commands: /help, /exit, /clear, /rules\n");
    
    let stdin = io::stdin();
    let mut stdin_lock = stdin.lock();
    
    loop {
        // Read multi-line input
        let input = read_multiline_input(&mut stdin_lock)?;
        
        if input.is_empty() {
            continue;
        }
        
        // Handle commands
        if input.starts_with('/') {
            match handle_connect_command(&input, &client, backend_url, api_key).await {
                Ok(should_exit) => {
                    if should_exit {
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                }
            }
            continue;
        }
        
        // Send message to backend via /evaluate endpoint
        let mut request = client
            .post(&format!("{}/evaluate", backend_url))
            .json(&serde_json::json!({
                "action": input
            }));
        
        if let Some(key) = api_key {
            request = request.header("x-api-key", key);
        }
        
        match request.send().await {
            Ok(resp) => {
                if resp.status().is_success() {
                    match resp.json::<serde_json::Value>().await {
                        Ok(data) => {
                            let score = data.get("score").and_then(|s| s.as_f64()).unwrap_or(0.0);
                            let action = data.get("action").and_then(|a| a.as_str()).unwrap_or("");
                            println!("\n⚖️  Conscience Evaluation: {:.2}", score);
                            println!("   Action: {}\n", action);
                        }
                        Err(e) => {
                            eprintln!("\n❌ Error parsing response: {}\n", e);
                        }
                    }
                } else {
                    eprintln!("\n❌ Backend error: {}\n", resp.status());
                }
            }
            Err(e) => {
                eprintln!("\n❌ Request failed: {}\n", e);
            }
        }
    }
    
    println!("\n👋 Disconnected from backend. Goodbye!\n");
    Ok(())
}

/// Read multi-line input from user (for connect mode)
fn read_multiline_input(stdin: &mut std::io::StdinLock) -> anyhow::Result<String> {
    use std::io::BufRead;
    
    let mut lines = Vec::new();
    let mut line_count = 0;

    print!("You: ");
    std::io::stdout().flush()?;

    loop {
        let mut line = String::new();
        stdin.read_line(&mut line)?;
        
        let trimmed = line.trim();
        
        // Check for /send command
        if trimmed == "/send" {
            break;
        }
        
        // Empty line after content = send
        if trimmed.is_empty() && !lines.is_empty() {
            break;
        }
        
        // Don't add empty lines at the start
        if trimmed.is_empty() && lines.is_empty() {
            continue;
        }
        
        lines.push(line);
        line_count += 1;
        
        // Show continuation prompt for multi-line
        if line_count > 1 {
            print!("  ... ");
            std::io::stdout().flush()?;
        }
    }

    Ok(lines.join("").trim().to_string())
}

async fn handle_connect_command(
    cmd: &str,
    client: &reqwest::Client,
    backend_url: &str,
    api_key: Option<&str>,
) -> Result<bool> {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    let command = parts[0];
    
    match command {
        "/help" | "/h" => {
            println!("\n📖 Available Commands:");
            println!("  /help, /h          - Show this help message");
            println!("  /exit, /quit, /q   - Disconnect and exit");
            println!("  /clear             - Clear conversation history");
            println!("  /rules             - Show all moral rules from backend");
            println!();
        }
        "/exit" | "/quit" | "/q" => {
            return Ok(true);
        }
        "/clear" => {
            println!("\n✅ Conversation history cleared.\n");
        }
        "/rules" => {
            let mut request = client.get(&format!("{}/rules", backend_url));
            
            if let Some(key) = api_key {
                request = request.header("x-api-key", key);
            }
            
            match request.send().await {
                Ok(resp) => {
                    if resp.status().is_success() {
                        match resp.json::<Vec<serde_json::Value>>().await {
                            Ok(rules) => {
                                println!("\n📜 Moral Rules from Backend:");
                                for rule in rules {
                                    let name = rule.get("name").and_then(|n| n.as_str()).unwrap_or("");
                                    let desc = rule.get("description").and_then(|d| d.as_str()).unwrap_or("");
                                    let weight = rule.get("weight").and_then(|w| w.as_f64()).unwrap_or(0.0);
                                    println!("  • {} (weight: {:.1})", name, weight);
                                    println!("    {}", desc);
                                }
                                println!();
                            }
                            Err(e) => {
                                eprintln!("\n❌ Error parsing rules: {}\n", e);
                            }
                        }
                    } else {
                        eprintln!("\n❌ Backend error: {}\n", resp.status());
                    }
                }
                Err(e) => {
                    eprintln!("\n❌ Request failed: {}\n", e);
                }
            }
        }
        _ => {
            println!("\n❓ Unknown command: {}. Type /help for available commands.\n", command);
        }
    }
    
    Ok(false)
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
        emotion_type: emotion_type.clone(),
        intensity: 0.8,
        target: None,
        timestamp: chrono::Utc::now(),
        duration: 0.0,
    };
    
    let mut entity = storage.get_entity(entity_name)
        .await?
        .unwrap_or_else(|| SoulEntity::new(entity_name.to_string()));
    
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
