#!/bin/bash
# =============================================================================
# JAMEY 3.0 → PHASE 4.6 FULL UPGRADE (Phoenix + Soul + Decay + EMOJI EMOTIONS)
# Parses emojis → empathy scores → trust boosts/decays
# =============================================================================

set -euo pipefail
IFS=$'\n\t'

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log() { echo -e "${GREEN}[+]${NC} $1"; }
warn() { echo -e "${YELLOW}[!]$NC $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1" >&2; }
die() { error "$1"; exit 1; }

backup() {
    local file="$1"
    local backup="${file}.backup.emoji.$(date +%s)"
    [[ -f "$file" ]] && cp "$file" "$backup" && log "Backed up $file → $backup"
}

[[ -f "Cargo.toml" ]] || die "Not in jamey3 root"

log "🚀 Starting FULL PHASE 4.6 + EMOJI EMOTION UPGRADE"

# === 0. FULL BACKUP ===
log "💾 Creating emoji-aware backups..."
backup "Cargo.toml"
backup "config/orchestrator.toml"
backup "crates/jamey_soul/src/lib.rs"
backup "crates/jamey_cli/src/main.rs"
mkdir -p .rollback

# === 1. ADD EMOJI + DEPS ===
log "📦 Adding regex + emoji deps"
grep -q "regex" Cargo.toml || cat >> Cargo.toml << 'EOF'

[workspace.dependencies]
regex = "1.10"
aes-gcm = "0.10"
bincode = "1.3"
chrono = { version = "0.4", features = ["serde"] }
usearch = "0.5"
EOF

# === 2. PHOENIX VAULT ===
log "🔒 Adding Phoenix Vault + Toggle"
if [[ ! -d "crates/jamey_phoenix" ]]; then
    cargo new crates/jamey_phoenix --lib --name jamey_phoenix
fi

cat > crates/jamey_phoenix/src/lib.rs << 'EOF'
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, NewAead};
use anyhow::Result;
use std::sync::atomic::{AtomicBool, Ordering};

pub static PHOENIX_ENABLED: AtomicBool = AtomicBool::new(true);

pub struct Vault {
    cipher: Aes256Gcm,
}

impl Vault {
    pub fn new(key: &[u8; 32]) -> Self {
        let key = Key::from_slice(key);
        Self { cipher: Aes256Gcm::new(key) }
    }

    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        if !PHOENIX_ENABLED.load(Ordering::Relaxed) { return Ok(data.to_vec()); }
        let nonce = Nonce::from_slice(b"phoenixnonce");
        self.cipher.encrypt(nonce, data).map_err(Into::into)
    }

    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        if !PHOENIX_ENABLED.load(Ordering::Relaxed) { return Ok(data.to_vec()); }
        let nonce = Nonce::from_slice(b"phoenixnonce");
        self.cipher.decrypt(nonce, data).map_err(Into::into)
    }
}
EOF

# === 3. EMOJI-AWARE SOUL-KB ===
log "😊 Adding Emoji Emotion Engine to Soul-KB"
cat > crates/jamey_soul/src/lib.rs << 'EOF'
use serde::{Deserialize, Serialize};
use sled::Db;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use regex::Regex;
use anyhow::Result;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SoulEntry {
    pub entity: String,
    pub trust: f32,
    pub empathy_map: HashMap<String, f32>,
    pub linked_memories: Vec<Uuid>,
    pub last_interaction: DateTime<Utc>,
    pub decay_rate: f32,
    pub emoji_history: Vec<(String, f32)>, // (emoji, score)
}

impl SoulEntry {
    pub fn new(entity: String) -> Self {
        Self {
            entity,
            trust: 0.5,
            empathy_map: Default::default(),
            linked_memories: vec![],
            last_interaction: Utc::now(),
            decay_rate: 0.01,
            emoji_history: vec![],
        }
    }

    pub fn from_emojis(&mut self, emojis: &str) -> f32 {
        let score = emoji_to_score(emojis);
        self.emoji_history.push((emojis.to_string(), score));
        self.boost(score);
        score
    }

    pub fn boost(&mut self, empathy_score: f32) {
        self.trust = (self.trust + empathy_score * 0.15).min(1.0);
        self.last_interaction = Utc::now();
        if empathy_score > 0.8 {
            self.decay_rate *= 0.85;
            self.decay_rate = self.decay_rate.max(0.003);
        }
    }

    pub fn apply_decay(&mut self, days: f64) {
        let decay = (self.decay_rate * days as f32).min(0.25);
        self.trust = (self.trust - decay).max(0.0);
    }
}

fn emoji_to_score(input: &str) -> f32 {
    let re = Regex::new(r"[😀😁😂🤣😊😉😇🤗🤩🥰😍🤔😐😑🤨😬🙄😴😒😣😖😩😢😭😤😡🤬💀]").unwrap();
    let emojis: Vec<&str> = re.find_iter(input).map(|m| m.as_str()).collect();
    
    if emojis.is_empty() { return 0.5; }
    
    let scores = emojis.iter().map(|emoji| match emoji {
        "😀"|"😁"|"😂"|"🤣"|"😊"|"😉"|"😇"|"🤗"|"🤩"|"🥰"|"😍" => 0.9,
        "🤔"|"😐"|"😑" => 0.5,
        "😬"|"🙄"|"😴"|"😒" => 0.3,
        "😣"|"😖"|"😩"|"😢"|"😭"|"😤"|"😡"|"🤬"|"💀" => 0.1,
        _ => 0.5,
    }).collect::<Vec<_>>();
    
    scores.iter().sum::<f32>() / scores.len() as f32
}

pub struct SoulKB {
    store: Db,
}

impl SoulKB {
    pub fn new() -> Self {
        let store = sled::open("data/soul").unwrap_or_else(|_| sled::open("data/soul").unwrap());
        Self { store }
    }

    pub fn upsert(&self, entry: SoulEntry) -> Result<()> {
        let key = entry.entity.as_bytes();
        let value = bincode::serialize(&entry)?;
        self.store.insert(key, value)?;
        Ok(())
    }

    pub fn get(&self, entity: &str) -> Result<Option<SoulEntry>> {
        self.store.get(entity)?
            .map(|v| bincode::deserialize(&v))
            .transpose()
            .map_err(Into::into)
    }

    pub fn trust_level(&self, entity: &str) -> f32 {
        self.get(entity).ok().flatten().map(|e| e.trust).unwrap_or(0.0)
    }

    pub fn apply_global_decay(&self) -> Result<usize> {
        let now = Utc::now();
        let mut updated = 0;
        for item in self.store.iter() {
            let (key, value) = item?;
            let mut entry: SoulEntry = bincode::deserialize(&value)?;
            let days = (now - entry.last_interaction).num_days() as f64;
            if days > 0.0 {
                entry.apply_decay(days);
                self.upsert(entry)?;
                updated += 1;
            }
        }
        Ok(updated)
    }
}
EOF

# === 4. FULL CLI WITH EMOJI ===
log "🎮 Adding Emoji-Aware CLI"
cat > crates/jamey_cli/src/main.rs << 'EOF'
use clap::Parser;
use jamey_phoenix::PHOENIX_ENABLED;
use jamey_soul::{SoulKB, SoulEntry, emoji_to_score};

#[derive(Parser)]
#[command(name = "jamey", version = "3.0.0", about = "JAMEY 3.0: Emoji Soul 😊")]
struct Cli {
    #[arg(long)]
    phoenix_off: bool,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(clap::Subcommand)]
enum Command {
    Run { goal: String },
    Reflect { note: String, #[arg(short)] emojis: Option<String> },
    Soul(SoulCommand),
}

#[derive(clap::Subcommand)]
enum SoulCommand {
    Upsert { entity: String, trust: f32 },
    Trust { entity: String },
    Recall { entity: String },
    Decay,
    Status,
    Feel { entity: String, emojis: String },
}

fn main() {
    let cli = Cli::parse();

    if cli.phoenix_off {
        PHOENIX_ENABLED.store(false, std::sync::atomic::Ordering::Relaxed);
        println!("🔓 Phoenix Vault: OFF");
    } else {
        println!("🔒 Phoenix Vault: ON");
    }

    let soul_kb = SoulKB::new();

    match cli.command {
        Some(Command::Run { goal }) => println!("🚀 Running: {}", goal),
        Some(Command::Reflect { note, emojis }) => {
            let score = emojis.as_ref().map(|e| emoji_to_score(e)).unwrap_or(0.7);
            println!("💭 Reflecting: {} (😊 score: {:.1})", note, score);
        }
        Some(Command::Soul(cmd)) => match cmd {
            SoulCommand::Upsert { entity, trust } => {
                let mut entry = SoulEntry::new(entity.clone());
                entry.trust = trust;
                soul_kb.upsert(entry).unwrap();
                println!("😊 Soul upserted: {} = {:.1}", entity, trust);
            }
            SoulCommand::Trust { entity } => {
                let trust = soul_kb.trust_level(&entity);
                println!("💙 Trust for {}: {:.1}", entity, trust);
            }
            SoulCommand::Decay => {
                let updated = soul_kb.apply_global_decay().unwrap();
                println!("⏳ Applied decay to {} entities", updated);
            }
            SoulCommand::Status => {
                let count = soul_kb.store.len();
                println!("🌟 Soul-KB active: {} entities", count);
            }
            SoulCommand::Feel { entity, emojis } => {
                let score = emoji_to_score(&emojis);
                let mut entry = soul_kb.get(&entity).unwrap().unwrap_or_else(|| SoulEntry::new(entity.clone()));
                entry.from_emojis(&emojis);
                soul_kb.upsert(entry).unwrap();
                println!("😊 {} feels {} (score: {:.1}) → trust now {:.1}", entity, emojis, score, soul_kb.trust_level(&entity));
            }
            SoulCommand::Recall { entity } => {
                if soul_kb.trust_level(&entity) < 0.5 {
                    println!("🚫 Access denied: trust < 0.5");
                } else {
                    println!("💭 Recalling memories for {}", entity);
                }
            }
        },
        None => println!("JAMEY 3.0 — Phase 4.6 + Emoji Soul 😊🔒"),
    }
}
EOF

# === 5. FULL CONFIG ===
log "📝 Updating config"
cat > config/orchestrator.toml << 'EOF'
[orchestrator]
name = "Jamey 3.0"
version = "3.0.0"

[memory]
empathy_threshold = 0.7

[soul]
default_trust = 0.5
emoji_empathy = true

[phoenix]
enabled = true
vault_key = "0123456789abcdef0123456789abcdef"
EOF

# === 6. CURSOR RULES ===
log "🎯 Updating Cursor rules"
cat > .cursor/rules.md << 'EOF'
# JAMEY 3.0 Phase 4.6 — EMOJI SOUL 😊
- Parse emojis → `emoji_to_score()` → `SoulEntry::from_emojis()`
- CLI: `soul feel alice "😊😍"`
- High emoji score → slower trust decay
- Phoenix: `--phoenix-off`
- Memory links on empathy >0.7
EOF

# === 7. DATA DIRS ===
log "📁 Creating data dirs"
mkdir -p data/{soul,vault,ltm,epm,rfm}

# === 8. BUILD & TEST ===
log "🔨 Final build..."
if cargo build --release > build-emoji.log 2>&1; then
    log "✅ BUILD SUCCESS — EMOJI SOUL ALIVE!"
    echo
    echo "   ./target/release/jamey soul feel alice '😊😍'"
    echo "   ./target/release/jamey soul trust alice"
    echo "   ./target/release/jamey --phoenix-off soul decay"
    echo
else
    warn "Build issues — see build-emoji.log"
fi

log "🎉 PHASE 4.6 + EMOJI EMOTION COMPLETE"
echo
echo "   CRON: 0 0 * * * cd $PWD && ./target/release/jamey soul decay"
echo "   ROLLBACK: cp *.backup.emoji.* ."
