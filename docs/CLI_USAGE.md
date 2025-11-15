# Jamey 3.0 CLI Usage Guide

Complete guide for using the Jamey 3.0 command-line interface.

## Quick Start

### Option 1: Start Backend + CLI Together (Easiest)

```bash
./scripts/start-with-cli.sh
```

This script:
- Starts the backend server in the background
- Waits for it to be ready
- Launches the CLI chat interface
- Automatically stops the backend when you exit the CLI

**Backend logs**: `/tmp/jamey-backend.log`

### Option 2: Connect to Running Backend (SSH-like)

If you already have a backend running:

```bash
./scripts/connect.sh
```

Or manually:

```bash
cargo run --bin jamey-cli connect
```

Connect to a remote backend:

```bash
cargo run --bin jamey-cli connect --url http://remote-server:3000 --api-key jamey_your-key
```

### Option 3: Standalone CLI (Direct LLM Access)

Run CLI without backend (uses OpenRouter directly):

```bash
cargo run --bin jamey-cli chat
```

Or use the script:

```bash
./scripts/chat.sh
```

## CLI Modes

### 1. Chat Mode (Standalone)

**Command**: `cargo run --bin jamey-cli chat`

- Direct connection to OpenRouter LLM
- Uses local conscience engine and memory
- No backend server required
- Requires `OPENROUTER_API_KEY` in `.env`

**Use when**: You want a standalone chat experience without running the backend server.

### 2. Connect Mode (SSH-like)

**Command**: `cargo run --bin jamey-cli connect [--url URL] [--api-key KEY]`

- Connects to a running backend via HTTP API
- Uses backend's conscience engine and memory
- Backend must be running separately
- Can connect to local or remote backends

**Use when**: 
- Backend is already running
- You want to connect to a remote backend
- You want to use the backend's shared memory and state

**Example**:
```bash
# Connect to local backend
cargo run --bin jamey-cli connect

# Connect to remote backend with API key
cargo run --bin jamey-cli connect \
  --url https://jamey-api.example.com \
  --api-key jamey_your-key-here
```

### 3. Soul Commands

**Command**: `cargo run --bin jamey-cli soul <command>`

Manage the Soul Knowledge Base:

```bash
# Add/update entity
cargo run --bin jamey-cli soul upsert alice 0.7

# Record emotion
cargo run --bin jamey-cli soul record alice joy

# Show status
cargo run --bin jamey-cli soul status alice
cargo run --bin jamey-cli soul status  # Show all

# Apply trust decay
cargo run --bin jamey-cli soul decay

# Delete entity
cargo run --bin jamey-cli soul delete bob
```

## Chat Commands

When in chat mode, use these commands:

- `/help` or `/h` - Show available commands
- `/exit`, `/quit`, or `/q` - Exit the chat
- `/clear` - Clear conversation history
- `/rules` - Show all moral rules
- `/memory` - Show recent memories (standalone mode only)
- `/conscience <text>` - Evaluate text with conscience engine (standalone mode only)

## Environment Variables

### For Standalone Chat Mode

```bash
OPENROUTER_API_KEY=your-key-here
```

### For Connect Mode

```bash
# Optional - defaults to http://localhost:3000
JAMEY_API_URL=http://localhost:3000

# Optional - for authenticated backends
JAMEY_API_KEY=jamey_your-key-here
```

## Examples

### Start Everything Together

```bash
# Easiest way - starts backend and CLI together
./scripts/start-with-cli.sh
```

### Connect to Running Backend

```bash
# Terminal 1: Start backend
./scripts/run.sh

# Terminal 2: Connect to it
./scripts/connect.sh
```

### Remote Connection

```bash
# Connect to remote backend
cargo run --bin jamey-cli connect \
  --url https://jamey-api.production.com \
  --api-key jamey_production-key
```

### Standalone Chat

```bash
# Direct LLM access, no backend needed
cargo run --bin jamey-cli chat
```

## Troubleshooting

### "Backend is not running"

**Problem**: Connect mode can't reach the backend.

**Solutions**:
1. Start the backend: `./scripts/run.sh` or `cargo run`
2. Check the URL: `cargo run --bin jamey-cli connect --url http://localhost:3000`
3. Verify backend is running: `curl http://localhost:3000/health`

### "Configuration Error" in Standalone Mode

**Problem**: Missing `OPENROUTER_API_KEY`.

**Solutions**:
1. Create `.env` file in project root
2. Add: `OPENROUTER_API_KEY=your-key-here`
3. Get key from: https://openrouter.ai/keys

### "Cannot connect to backend"

**Problem**: Backend URL is incorrect or backend is not accessible.

**Solutions**:
1. Check backend is running: `curl http://localhost:3000/health`
2. Verify URL is correct
3. Check firewall/network if connecting remotely
4. Verify API key if backend requires authentication

## Scripts Reference

| Script | Purpose |
|--------|---------|
| `./scripts/start-with-cli.sh` | Start backend + CLI together |
| `./scripts/connect.sh` | Connect to running backend |
| `./scripts/chat.sh` | Start standalone CLI chat |
| `./scripts/run.sh` | Start backend server only |

## Advanced Usage

### Custom Backend URL

```bash
JAMEY_API_URL=https://my-backend.com cargo run --bin jamey-cli connect
```

### With API Key

```bash
JAMEY_API_KEY=jamey_key cargo run --bin jamey-cli connect
```

### Background Backend

```bash
# Start backend in background
nohup cargo run > backend.log 2>&1 &

# Connect to it
./scripts/connect.sh
```

---

**See Also**:
- [Quick Start Guide](QUICK_START.md)
- [Configuration Guide](CONFIGURATION.md)
- [API Reference](API_REFERENCE.md)

