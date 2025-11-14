# Jamey 3.0 - General & Guardian

Jamey 3.0 is the digital mirror and guardian system, part of the Eternal Hive architecture. It features a Conscience Engine for moral evaluation and a 5-Layer Memory System for persistent knowledge storage.

## Architecture

- **Backend**: Rust with Tokio, Axum, SQLx
- **Frontend**: React 18, TypeScript, Vite, TanStack Query
- **AI**: Conscience Engine with weighted moral rules
- **Memory**: 5-Layer Memory System (Short-term, Long-term, Working, Episodic, Semantic)
- **Database**: SQLite with SQLx
- **Real-time**: MQTT async client (to be implemented)

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

3. Start the development server:
```bash
npm run dev
```

The frontend will be available at `http://localhost:5173` (or the port Vite assigns)

### Environment Variables

Create a `.env` file in the frontend directory (optional):
```
VITE_API_URL=http://localhost:3000
```

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

## License

Part of the Eternal Hive project - Transform Army AI

