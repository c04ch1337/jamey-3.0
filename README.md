# Jamey 3.0 - General & Guardian

Jamey 3.0 is the digital mirror and guardian system, part of the Eternal Hive architecture. It features a Conscience Engine for moral evaluation and a 5-Layer Memory System for persistent knowledge storage.

## Architecture

- **Backend**: Rust with Tokio, Axum, SQLx
- **Frontend**: React 18, TypeScript, Vite, TanStack Query
- **AI**: Conscience Engine with weighted moral rules
- **Memory**: 5-Layer Memory System (Short-term, Long-term, Working, Episodic, Semantic)
- **Database**: SQLite with SQLx
- **Real-time**: MQTT async client with authentication and pub/sub capabilities
- **Soul KB**: Emoji-based emotion tracking with trust scoring (Phase 4.6)

## Project Structure

```
jamey-3.0/
├── src/                    # Rust backend source
│   ├── main.rs            # Application entry point
│   ├── lib.rs             # Library root
│   ├── api/               # Axum API routes
│   ├── conscience/        # Conscience Engine
│   ├── memory/            # 5-Layer Memory System
│   └── db/                # Database layer
├── frontend/              # React frontend
│   ├── src/
│   │   ├── api/          # API client
│   │   ├── App.tsx       # Main app component
│   │   └── main.tsx      # React entry point
│   └── package.json
├── migrations/            # SQLx database migrations
├── data/                  # Database and memory indices
└── Cargo.toml            # Rust dependencies
```

## Getting Started

### Prerequisites

- Rust (latest stable)
- Node.js 18+ and npm
- SQLite

### Backend Setup

1. Install Rust dependencies:
```bash
cargo build
```

2. Run the backend server:
```bash
cargo run
```

Or use the helper script (automatically kills any existing server on port 3000):
```bash
./scripts/run.sh
```

The server will start on `http://localhost:3000`

### CLI Chat Interface

Start an interactive chat with Jamey 3.0:

```bash
cargo run --bin jamey-cli
```

Or use the helper script:
```bash
./scripts/chat.sh
```

**Note**: The CLI requires your OpenRouter API key to be set in `.env` file.

**CLI Commands:**
- `/help` - Show available commands
- `/exit` or `/quit` - Exit the chat
- `/clear` - Clear conversation history
- `/rules` - Show all moral rules
- `/memory` - Show recent memories
- `/conscience <text>` - Evaluate text with conscience engine

### Frontend Setup

1. Navigate to frontend directory:
```bash
cd frontend
```

2. Install dependencies:
```bash
npm install
```

3. Configure environment (see [Frontend Quick Start](docs/FRONTEND_QUICK_START.md)):
```bash
cp .env.example .env
# Edit .env and add your API key if needed
```

4. Start the development server:
```bash
npm run dev
```

The frontend will be available at `http://localhost:5173` (or the port Vite assigns)

**Quick Setup**: See [Frontend Quick Start Guide](docs/FRONTEND_QUICK_START.md) for 5-minute setup.

**Multiple Frontends**: See [Multiple Frontends Guide](docs/MULTIPLE_FRONTENDS.md) for setting up local desktop + remote frontends.

**Any Framework**: See [Universal Frontend Integration Guide](docs/FRONTEND_INTEGRATION.md) for React, Vue, Angular, vanilla JS, desktop, and mobile apps.

## API Endpoints

- `GET /` - Health check
- `POST /evaluate` - Evaluate an action's morality
  - Body: `{ "action": "string" }`
  - Returns: `{ "score": f32, "action": "string" }`
- `GET /rules` - Get all moral rules
- `POST /rules` - Add a new moral rule
  - Body: `{ "name": "string", "description": "string", "weight": f32 }`

## Memory System

The 5-Layer Memory System stores memories in separate Tantivy indices:

1. **Short-term**: Immediate actions and evaluations
2. **Long-term**: Persistent moral learnings
3. **Working**: Current context and reasoning
4. **Episodic**: Event sequences and experiences
5. **Semantic**: Conceptual knowledge and rules

## Soul Knowledge Base (Phase 4.6)

The Soul KB tracks entities with emotion-based trust scoring, empathy calculation, and automatic trust decay over time.

### Emotion System

Five core emotions with emoji representation and scoring:
- 😍 **Love** (1.0) - Highest empathy, slows trust decay
- 😊 **Joy** (0.8) - Positive interaction
- 😐 **Neutral** (0.5) - Balanced interaction
- 😢 **Sadness** (0.2) - Negative interaction
- 😡 **Anger** (0.1) - Lowest empathy, accelerates trust decay

### Trust & Empathy

- **Trust Score**: 0.0 to 1.0, starts at 0.5 by default
- **Empathy Score**: Weighted average of recorded emotions
- **Trust Boost**: High empathy (>0.7) increases trust and slows decay
- **Trust Decay**: Time-based decay, rate adjusted by empathy level
- **Memory Links**: Connect memories to entities for context

### CLI Commands

```bash
# Add or update entity with trust score
jamey-cli soul upsert alice 0.7

# Record emotion (text or emoji)
jamey-cli soul record alice joy
jamey-cli soul record alice 😊

# Show entity status with emoji
jamey-cli soul status alice

# Show all entities
jamey-cli soul status

# Apply time-based trust decay
jamey-cli soul decay

# Delete entity
jamey-cli soul delete bob
```

### Configuration

Configure in `.env`:
```bash
SOUL_DEFAULT_TRUST=0.5
SOUL_BASE_DECAY_RATE=0.01
SOUL_PRUNE_THRESHOLD=0.1
SOUL_EMPATHY_THRESHOLD=0.7
SOUL_AUTO_RECORD=true
```

### Integration

The soul KB integrates with:
- **Conscience Engine**: Automatically records emotions based on moral evaluation scores
- **Memory System**: Links memories to entities for personalized context
- **Configuration**: Fully configurable trust and decay parameters

For detailed architecture, see [`docs/phase_4_6_architecture.md`](docs/phase_4_6_architecture.md).

## Conscience Engine

The Conscience Engine evaluates actions against weighted moral rules. Default rules include:
- `no-harm` (weight: 10.0) - Do not cause physical or emotional harm
- `truth` (weight: 8.0) - Be honest and truthful

Rules can be added, removed, and customized through the API.

## Development

### Running Tests

```bash
cargo test
```

### Building for Production

Backend:
```bash
cargo build --release
```

Frontend:
```bash
cd frontend
npm run build
```

## Documentation

- **TEMPLATE.md** - Comprehensive codebase template and reference
- **docs/architecture.md** - System architecture details
- **docs/mqtt_architecture.md** - MQTT system architecture and design
- **docs/MQTT_USAGE.md** - MQTT client usage and configuration guide
- **docs/setup/** - Setup and configuration guides
- `docs/phase_4_6_architecture.md` - Soul KB and emoji emotion system design

## License

Part of the Eternal Hive project - Transform Army AI

