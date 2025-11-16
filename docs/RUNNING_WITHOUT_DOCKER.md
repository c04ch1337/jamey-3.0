# Running Jamey 3.0 Locally

This guide explains how to run Jamey 3.0 locally.

## Quick Start (Windows)

### Option 1: Development Script (Recommended)
```powershell
.\scripts\dev-start.ps1
```

This script:
- Starts the backend on `http://localhost:3000`
- Starts the frontend on `http://localhost:5173`
- Opens two separate terminal windows
- Automatically checks for port conflicts

### Option 2: Manual Start

**Terminal 1 - Backend:**
```powershell
cargo run
```


## Quick Start (Linux/Mac)

```bash
# Terminal 1 - Backend
./scripts/run.sh

```

## Troubleshooting

### Backend Not Running Error
If you see "Network error - unable to reach server", the backend isn't running:

1. **Check if backend is running:**
   ```powershell
   # Windows
   Get-NetTCPConnection -LocalPort 3000
   
   # Linux/Mac
   lsof -i:3000
   ```

2. **Start the backend:**
   ```powershell
   cargo run
   ```

3. **Verify backend is healthy:**
   ```powershell
   curl http://localhost:3000/health
   ```

### Port Already in Use

**Windows:**
```powershell
# Kill process on port 3000
Get-NetTCPConnection -LocalPort 3000 | ForEach-Object { Stop-Process -Id $_.OwningProcess -Force }

# Then start backend
cargo run
```

**Linux/Mac:**
```bash
# Kill process on port 3000
lsof -ti:3000 | xargs kill -9

# Then start backend
cargo run
```

## Running the Backend

If you only need the backend API:

```powershell
cargo run
```

Then access the API at:
- Health: `http://localhost:3000/health`
- Metrics: `http://localhost:3000/metrics`
- API endpoints: `http://localhost:3000/evaluate`, etc.

## Environment Variables

Make sure you have a `.env` file with required variables:

```env
# Required
OPENROUTER_API_KEY=your_key_here

# Optional
SERVER_HOST=0.0.0.0
SERVER_PORT=3000
RUST_LOG=info
```

See [`.env.example`](.env.example) for all available options.

## Next Steps

- [API Reference](API_REFERENCE.md) - Available endpoints
- [Configuration Guide](CONFIGURATION.md) - Environment variables
- [CLI Usage](CLI_USAGE.md) - Command-line interface