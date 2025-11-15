# Jamey 3.0 - Codebase Template

This document serves as a comprehensive template and reference for the Jamey 3.0 codebase structure, architecture, and conventions.

## 📋 Table of Contents

1. [Project Overview](#project-overview)
2. [Project Structure](#project-structure)
3. [Technology Stack](#technology-stack)
4. [Architecture](#architecture)
5. [Module Organization](#module-organization)
6. [API Structure](#api-structure)
7. [Configuration](#configuration)
8. [Development Workflow](#development-workflow)

---

## Project Overview

**Jamey 3.0** is the General & Guardian system, part of the Eternal Hive architecture. It serves as a digital mirror with:
- **Conscience Engine**: Moral rule evaluation system
- **5-Layer Memory System**: Hierarchical memory storage with Tantivy indexing
- **REST API**: Axum-based web interface
- **CLI Interface**: Interactive chat with OpenRouter LLM integration
- **React Frontend**: Real-time interaction interface

---

## Project Structure

```
jamey-3.0/
├── src/                          # Rust backend source code
│   ├── main.rs                  # Main server entry point
│   ├── lib.rs                   # Library root with module exports
│   ├── api/                     # REST API routes and handlers
│   │   └── mod.rs              # Axum routes, handlers, AppState
│   ├── bin/                     # Binary executables
│   │   └── jamey-cli.rs        # CLI chat interface entry point
│   ├── cli/                     # CLI chat interface
│   │   └── mod.rs              # Interactive chat CLI implementation
│   ├── config/                  # Configuration management
│   │   └── mod.rs              # Environment variable loading
│   ├── conscience/              # Conscience Engine
│   │   └── mod.rs              # Moral rule evaluation system
│   ├── db/                      # Database layer
│   │   └── mod.rs              # SQLite connection and migrations
│   ├── llm/                     # LLM client
│   │   └── mod.rs              # OpenRouter API client
│   └── memory/                  # Memory System
│       └── mod.rs              # 5-Layer Memory System with Tantivy
├── frontend/                     # React frontend application
│   ├── src/
│   │   ├── api/                # API client
│   │   │   └── client.ts      # Typed API client with Axios
│   │   ├── App.tsx            # Main application component
│   │   ├── App.css            # Application styles
│   │   ├── main.tsx           # React entry point with QueryClient
│   │   └── index.css          # Global styles
│   ├── package.json           # Frontend dependencies
│   ├── vite.config.ts        # Vite configuration with API proxy
│   └── tsconfig.json         # TypeScript configuration
├── migrations/                  # Database migrations
│   ├── _sqlx_migrations.json # SQLx migration metadata
│   └── 20241114000000_init.sql # Initial migration
├── scripts/                     # Utility scripts
│   ├── setup.sh              # Initial project setup
│   ├── run.sh                # Start backend server
│   ├── chat.sh               # Start CLI chat interface
│   ├── test-conscience.sh    # Test conscience engine
│   └── README.md             # Scripts documentation
├── docs/                        # Documentation
│   ├── architecture.md       # System architecture
│   └── setup/                # Setup documentation
│       ├── API_KEY_SETUP.md
│       ├── SETUP_DATABASE.md
│       ├── CONSCIENCE_VERIFICATION.md
│       └── TROUBLESHOOTING.md
├── data/                        # Runtime data (gitignored)
│   ├── jamey.db              # SQLite database
│   └── memory/               # Tantivy memory indices
│       ├── short_term/
│       ├── long_term/
│       ├── working/
│       ├── episodic/
│       └── semantic/
├── Cargo.toml                   # Rust project configuration
├── Cargo.lock                   # Dependency lock file
├── .env.example                 # Environment variable template
├── .gitignore                   # Git ignore patterns
├── .cursorrules                 # Cursor IDE rules
└── README.md                    # Main project documentation
```

---

## Technology Stack

### Backend (Rust)
- **Runtime**: Tokio (async runtime)
- **Web Framework**: Axum 0.7
- **Database**: SQLx 0.7 with SQLite
- **Search/Indexing**: Tantivy 0.22
- **HTTP Client**: Reqwest 0.12 (with rustls-tls)
- **Serialization**: Serde 1.0
- **Error Handling**: Anyhow 1.0, Thiserror 1.0
- **Logging**: Tracing 0.1
- **Collections**: DashMap 5.5 (thread-safe HashMap)
- **Environment**: Dotenvy 0.15

### Frontend (TypeScript/React)
- **Framework**: React 18
- **Build Tool**: Vite 7
- **State Management**: TanStack Query 5.62
- **HTTP Client**: Axios 1.7
- **Validation**: Zod 3.24
- **Language**: TypeScript (strict mode)

### AI/LLM
- **Provider**: OpenRouter API
- **Model**: Configurable (default: deepseek/deepseek-chat)
- **Integration**: Custom client in `src/llm/mod.rs`

---

## Architecture

### Core Components

1. **Conscience Engine** (`src/conscience/`)
   - Weighted moral rule evaluation
   - Thread-safe rule storage (DashMap)
   - Keyword-based matching algorithm
   - Default rules: `no-harm` (10.0), `truth` (8.0)

2. **5-Layer Memory System** (`src/memory/`)
   - Short-term: Immediate actions and evaluations
   - Long-term: Persistent moral learnings
   - Working: Current context and reasoning
   - Episodic: Event sequences and experiences
   - Semantic: Conceptual knowledge and rules
   - Each layer uses separate Tantivy index

3. **REST API** (`src/api/`)
   - Axum-based HTTP server
   - CORS enabled for development
   - State management via AppState
   - Endpoints: `/`, `/evaluate`, `/rules`

4. **CLI Interface** (`src/cli/`, `src/bin/jamey-cli.rs`)
   - Interactive chat with Jamey 3.0
   - OpenRouter LLM integration
   - Conscience evaluation on every message
   - Memory storage integration
   - Commands: `/help`, `/exit`, `/clear`, `/rules`, `/memory`, `/conscience`

5. **Database Layer** (`src/db/`)
   - SQLite with SQLx
   - Automatic migration system
   - Connection pooling
   - Path: `data/jamey.db`

6. **Configuration** (`src/config/`)
   - Environment variable loading
   - OpenRouter API key management
   - Optional/required configuration handling

---

## Module Organization

### Backend Modules (`src/lib.rs`)

```rust
pub mod api;        // REST API routes
pub mod cli;        // CLI chat interface
pub mod config;     // Configuration management
pub mod conscience; // Conscience Engine
pub mod db;         // Database layer
pub mod llm;        // OpenRouter LLM client
pub mod memory;     // 5-Layer Memory System
```

### Module Dependencies

```
main.rs / jamey-cli.rs
    ├── api::create_app()
    ├── db::init_db()
    ├── config::Config
    ├── conscience::ConscienceEngine
    ├── memory::MemorySystem
    └── cli::ChatCLI

api/mod.rs
    ├── conscience::ConscienceEngine
    └── memory::MemorySystem

cli/mod.rs
    ├── config::Config
    ├── llm::OpenRouterClient
    ├── conscience::ConscienceEngine
    └── memory::MemorySystem
```

---

## API Structure

### Endpoints

| Method | Path | Description | Request Body | Response |
|--------|------|-------------|--------------|----------|
| GET | `/` | Health check | - | `{status, service, version}` |
| POST | `/evaluate` | Evaluate action morality | `{action: string}` | `{score: f32, action: string}` |
| GET | `/rules` | Get all moral rules | - | `MoralRule[]` |
| POST | `/rules` | Add new moral rule | `{name, description, weight}` | `201 Created` |

### AppState

```rust
pub struct AppState {
    pub conscience: Arc<ConscienceEngine>,
    pub memory: Arc<MemorySystem>,
}
```

### Request/Response Types

```rust
// Evaluate Request
struct EvaluateRequest {
    action: String,
}

// Evaluate Response
struct EvaluateResponse {
    score: f32,
    action: String,
}

// Moral Rule
pub struct MoralRule {
    pub name: String,
    pub description: String,
    pub weight: f32,
}
```

---

## Configuration

### Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `OPENROUTER_API_KEY` | Yes (for CLI) | - | OpenRouter API key |
| `OPENROUTER_MODEL` | No | `deepseek/deepseek-chat` | LLM model to use |
| `OPENROUTER_API_URL` | No | `https://openrouter.ai/api/v1` | OpenRouter API endpoint |
| `RUST_LOG` | No | `info` | Logging level |
| `DATABASE_URL` | No | Auto | SQLite database path |

### Configuration Loading

```rust
// Optional (app can run without LLM)
let config = Config::from_env()?; // Returns Option<Config>

// Required (CLI needs LLM)
let config = Config::from_env_required()?; // Returns Config or error
```

---

## Development Workflow

### Running the Application

**Backend Server:**
```bash
cargo run
# or
./scripts/run.sh
```

**CLI Chat:**
```bash
cargo run --bin jamey-cli
# or
./scripts/chat.sh
```

**Frontend:**
```bash
cd frontend
npm install
npm run dev
```

### Building

**Backend:**
```bash
cargo build --release
```

**Frontend:**
```bash
cd frontend
npm run build
```

### Testing

```bash
# Run all tests
cargo test

# Test conscience engine
./scripts/test-conscience.sh
```

### Database Migrations

Migrations are automatically applied on startup. To create a new migration:

```bash
sqlx migrate add <migration_name>
```

---

## Code Conventions

### Rust

- **Edition**: 2021
- **Error Handling**: Use `anyhow::Result` for application errors
- **Logging**: Use `tracing::info!`, `tracing::error!`, etc.
- **Async**: Use `async/await` throughout
- **Modules**: Clear separation of concerns
- **Thread Safety**: Use `Arc<>` for shared state

### TypeScript/React

- **Strict Mode**: Enabled
- **Components**: Functional components with hooks
- **State**: TanStack Query for server state
- **Types**: Explicit typing, avoid `any`
- **API**: Typed client in `src/api/client.ts`

### File Naming

- **Rust**: `snake_case.rs`
- **TypeScript**: `PascalCase.tsx` for components, `camelCase.ts` for utilities
- **Scripts**: `kebab-case.sh`

---

## Key Design Patterns

1. **State Management**: Arc-wrapped shared state in AppState
2. **Error Handling**: Result types with anyhow for error propagation
3. **Memory Storage**: Automatic storage of evaluations and conversations
4. **Configuration**: Environment-based with sensible defaults
5. **Modularity**: Clear module boundaries with minimal coupling

---

## Future Enhancements

- [ ] MQTT async client integration
- [ ] Voice interface
- [ ] Dashboard for monitoring
- [ ] Failover logic for Phoenix.Marie
- [ ] ORCH node command interface
- [ ] TA-QR encryption channel
- [ ] IPFS/Arweave backup integration

---

## License

Part of the Eternal Hive project - Transform Army AI

