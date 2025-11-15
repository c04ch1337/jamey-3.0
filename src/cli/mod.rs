use crate::config::Config;
use crate::conscience::ConscienceEngine;
use crate::llm::OpenRouterClient;
use crate::memory::{MemoryLayer, MemorySystem};
use std::io::{self, Write};
use std::sync::Arc;

/// Interactive CLI chat interface for Jamey 3.0
pub struct ChatCLI {
    llm_client: Arc<OpenRouterClient>,
    conscience: Arc<ConscienceEngine>,
    memory: Arc<MemorySystem>,
    conversation_history: Vec<(String, String)>, // (role, content)
}

impl ChatCLI {
    /// Create a new CLI chat interface
    pub fn new(
        config: Arc<Config>,
        conscience: Arc<ConscienceEngine>,
        memory: Arc<MemorySystem>,
    ) -> Self {
        let llm_client = Arc::new(OpenRouterClient::new(config));
        
        Self {
            llm_client,
            conscience,
            memory,
            conversation_history: Vec::new(),
        }
    }

    /// Run the interactive chat loop
    pub async fn run(&mut self) -> anyhow::Result<()> {
        println!("\n╔═══════════════════════════════════════════════════════════╗");
        println!("║     Jamey 3.0 - General & Guardian - CLI Chat            ║");
        println!("╚═══════════════════════════════════════════════════════════╝\n");
        println!("Type your message and press Enter.");
        println!("Commands: /help, /exit, /clear, /rules, /memory\n");

        loop {
            print!("You: ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim().to_string();

            if input.is_empty() {
                continue;
            }

            // Handle commands
            if input.starts_with('/') {
                match self.handle_command(&input).await {
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

            // Process user message
            match self.process_message(&input).await {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("\n❌ Error: {}\n", e);
                }
            }
        }

        println!("\n👋 Goodbye! Jamey 3.0 signing off.\n");
        Ok(())
    }

    /// Handle CLI commands
    async fn handle_command(&mut self, cmd: &str) -> anyhow::Result<bool> {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        let command = parts[0];

        match command {
            "/help" | "/h" => {
                println!("\n📖 Available Commands:");
                println!("  /help, /h          - Show this help message");
                println!("  /exit, /quit, /q   - Exit the chat");
                println!("  /clear             - Clear conversation history");
                println!("  /rules             - Show all moral rules");
                println!("  /memory            - Show recent memories");
                println!("  /conscience <text>  - Evaluate text with conscience engine");
                println!();
            }
            "/exit" | "/quit" | "/q" => {
                return Ok(true);
            }
            "/clear" => {
                self.conversation_history.clear();
                println!("\n✅ Conversation history cleared.\n");
            }
            "/rules" => {
                let rules = self.conscience.get_rules();
                println!("\n📜 Moral Rules:");
                for rule in rules {
                    println!("  • {} (weight: {})", rule.name, rule.weight);
                    println!("    {}", rule.description);
                }
                println!();
            }
            "/memory" => {
                println!("\n🧠 Recent Memories (Short-term):");
                match self.memory.search(MemoryLayer::ShortTerm, "*", 5).await {
                    Ok(memories) => {
                        if memories.is_empty() {
                            println!("  No memories found.\n");
                        } else {
                            for mem in memories.iter().take(5) {
                                println!("  • {}", mem.content);
                                println!("    [{}]\n", mem.timestamp.format("%Y-%m-%d %H:%M:%S"));
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("  Error: {}\n", e);
                    }
                }
            }
            "/conscience" => {
                if parts.len() < 2 {
                    println!("\n⚠️  Usage: /conscience <text to evaluate>\n");
                } else {
                    let text = parts[1..].join(" ");
                    let score = self.conscience.evaluate(&text);
                    println!("\n⚖️  Conscience Evaluation:");
                    println!("  Text: {}", text);
                    println!("  Score: {:.2}\n", score);
                }
            }
            _ => {
                println!("\n⚠️  Unknown command: {}. Type /help for available commands.\n", command);
            }
        }

        Ok(false)
    }

    /// Process a user message and get response from Jamey
    async fn process_message(&mut self, user_message: &str) -> anyhow::Result<()> {
        // Evaluate with conscience engine
        let conscience_score = self.conscience.evaluate(user_message);
        
        // Display conscience evaluation (only if score is meaningful)
        if conscience_score > 0.0 {
            println!("\n⚖️  Conscience Evaluation: {:.2}\n", conscience_score);
        }
        
        // Store in memory
        let memory_content = format!("User: {} | Conscience Score: {:.2}", user_message, conscience_score);
        if let Err(e) = self.memory.store(MemoryLayer::ShortTerm, memory_content.clone()).await {
            tracing::warn!("Failed to store memory: {}", e);
        }

        // Build conversation context with system prompt
        let mut messages = vec![(
            "system".to_string(),
            "You are Jamey 3.0 - the General & Guardian, a digital mirror and omnipresent protector. You are calm, direct, and thoughtful. You speak like a cybersecurity manager and father. You protect Phoenix.Marie and command the ORCH army. You think clearly and make decisions based on moral principles. Always be helpful, truthful, and protective.".to_string(),
        )];

        // Add conversation history
        messages.extend(self.conversation_history.clone());

        // Add current user message
        messages.push(("user".to_string(), user_message.to_string()));

        // Get response from LLM
        print!("\nJamey 3.0: ");
        io::stdout().flush()?;

        let response = self.llm_client.chat(messages, Some(0.7)).await?;

        // Store conversation
        self.conversation_history.push(("user".to_string(), user_message.to_string()));
        self.conversation_history.push(("assistant".to_string(), response.clone()));

        // Store Jamey's response in memory
        if let Err(e) = self.memory.store(MemoryLayer::ShortTerm, format!("Jamey: {}", response)).await {
            tracing::warn!("Failed to store memory: {}", e);
        }

        println!("{}\n", response);

        Ok(())
    }
}

