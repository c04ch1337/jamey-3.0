# Jamey 3.0 - General & Guardian

<<<<<<< HEAD
<div align="center">

![Version](https://img.shields.io/badge/version-3.0.0-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)
![Node](https://img.shields.io/badge/node-18+-green.svg)
![License](https://img.shields.io/badge/license-Eternal%20Hive-purple.svg)
![Status](https://img.shields.io/badge/status-active-success.svg)

[![Rust](https://img.shields.io/badge/Backend-Rust-orange?logo=rust)](https://www.rust-lang.org/)
[![React](https://img.shields.io/badge/Frontend-React-blue?logo=react)](https://react.dev/)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.9-blue?logo=typescript)](https://www.typescriptlang.org/)
[![SQLite](https://img.shields.io/badge/Database-SQLite-blue?logo=sqlite)](https://www.sqlite.org/)
[![MQTT](https://img.shields.io/badge/Protocol-MQTT-orange?logo=eclipsemosquitto)](https://mqtt.org/)

[![Axum](https://img.shields.io/badge/Web%20Framework-Axum-0.7-green)](https://github.com/tokio-rs/axum)
[![Tokio](https://img.shields.io/badge/Async-Tokio-1.0-blue)](https://tokio.rs/)
[![Vite](https://img.shields.io/badge/Build%20Tool-Vite-7.2-yellow?logo=vite)](https://vitejs.dev/)
[![TanStack Query](https://img.shields.io/badge/State-TanStack%20Query-5.62-red)](https://tanstack.com/query)

[![Conscience Engine](https://img.shields.io/badge/Feature-Conscience%20Engine-purple)](./src/conscience/)
[![Memory System](https://img.shields.io/badge/Feature-5--Layer%20Memory-blue)](./src/memory/)
[![Soul KB](https://img.shields.io/badge/Feature-Soul%20KB-pink)](./src/soul/)
[![MQTT Client](https://img.shields.io/badge/Feature-MQTT%20Client-orange)](./src/mqtt/)

[![API](https://img.shields.io/badge/API-REST%20API-green)](./docs/API_REFERENCE.md)
[![CLI](https://img.shields.io/badge/CLI-Interactive%20Chat-blue)](./src/bin/jamey-cli.rs)
[![Frontend](https://img.shields.io/badge/Frontend-React%20App-61DAFB?logo=react)](./frontend/)
[![Documentation](https://img.shields.io/badge/Docs-Comprehensive-blue)](./docs/)

[![Eternal Hive](https://img.shields.io/badge/Part%20of-Eternal%20Hive-purple)](https://github.com/TransformArmyAI/Eternal-Hive)
[![Transform Army AI](https://img.shields.io/badge/Transform-Army%20AI-black)](https://github.com/TransformArmyAI)

</div>

=======
>>>>>>> origin/main
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

<<<<<<< HEAD
**Easiest Way - Start Backend + CLI Together:**

**Linux/Mac:**
```bash
./scripts/start-with-cli.sh
```

**Windows (PowerShell):**
```powershell
.\scripts\start-with-cli.ps1
```

This starts the backend in the background and launches the CLI automatically.

**Connect to Running Backend (SSH-like):**

If you already have a backend running:

**Linux/Mac:**
```bash
./scripts/connect.sh
```

**Windows (PowerShell):**
```powershell
.\scripts\connect.ps1
```

Or manually:
```bash
cargo run --bin jamey-cli connect
```

Connect to remote backend:
```bash
cargo run --bin jamey-cli connect --url http://remote-server:3000 --api-key jamey_your-key
```

**Standalone CLI (Direct LLM Access):**

Run CLI without backend (uses OpenRouter directly):

```bash
cargo run --bin jamey-cli chat
=======
Start an interactive chat with Jamey 3.0:

```bash
cargo run --bin jamey-cli
>>>>>>> origin/main
```

Or use the helper script:
```bash
./scripts/chat.sh
```

<<<<<<< HEAD
**Note**: Standalone chat mode requires your OpenRouter API key to be set in `.env` file.
=======
**Note**: The CLI requires your OpenRouter API key to be set in `.env` file.
>>>>>>> origin/main

**CLI Commands:**
- `/help` - Show available commands
- `/exit` or `/quit` - Exit the chat
- `/clear` - Clear conversation history
- `/rules` - Show all moral rules
<<<<<<< HEAD
- `/memory` - Show recent memories (standalone mode only)
- `/conscience <text>` - Evaluate text with conscience engine (standalone mode only)

**See [CLI Usage Guide](docs/CLI_USAGE.md) for complete documentation.**
=======
- `/memory` - Show recent memories
- `/conscience <text>` - Evaluate text with conscience engine
>>>>>>> origin/main

### Frontend Setup

1. Navigate to frontend directory:
```bash
cd frontend
```

2. Install dependencies:
```bash
npm install
```

<<<<<<< HEAD
3. Configure environment (see [Frontend Quick Start](docs/FRONTEND_QUICK_START.md)):
```bash
cp .env.example .env
# Edit .env and add your API key if needed
```

4. Start the development server:
=======
3. Start the development server:
>>>>>>> origin/main
```bash
npm run dev
```

The frontend will be available at `http://localhost:5173` (or the port Vite assigns)

<<<<<<< HEAD
**Quick Setup**: See [Frontend Quick Start Guide](docs/FRONTEND_QUICK_START.md) for 5-minute setup.

**Multiple Frontends**: See [Multiple Frontends Guide](docs/MULTIPLE_FRONTENDS.md) for setting up local desktop + remote frontends.

**Any Framework**: See [Universal Frontend Integration Guide](docs/FRONTEND_INTEGRATION.md) for React, Vue, Angular, vanilla JS, desktop, and mobile apps.
=======
### Environment Variables

Create a `.env` file in the frontend directory (optional):
```
VITE_API_URL=http://localhost:3000
```
>>>>>>> origin/main

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
<<<<<<< HEAD
- **docs/phase_4_6_architecture.md** - Soul KB and emoji emotion system design
- **docs/BADGES.md** - Complete badge and icon reference guide
- **docs/BADGES_QUICK_REFERENCE.md** - Quick copy-paste badge collection
=======
- `docs/phase_4_6_architecture.md` - Soul KB and emoji emotion system design
- **docs/deployment/containerization.md** - Guide to setting up and using the containerized development environment and CI/CD pipeline.
>>>>>>> origin/main

## License

Part of the Eternal Hive project - Transform Army AI

