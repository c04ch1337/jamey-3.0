# Quick Start Guide - Running Jamey 3.0

**Date:** 2025-01-27

---

## Overview

Jamey 3.0 consists of two separate applications:
1. **Backend** - Rust server (runs on port 3000)
2. **Frontend** - React app (runs on port 5173)

You need to run **both** in separate terminals.

---

## Step-by-Step Setup

### 1. Start Backend Server

**Terminal 1:**
```bash
cargo run
```

The backend will:
- Start on `http://localhost:3000`
- Initialize database
- Set up API endpoints
- Connect to MQTT (if configured)

**Expected output:**
```
Starting Jamey 3.0 - General & Guardian
Server listening on http://0.0.0.0:3000
```

**Verify it's running:**
```bash
curl http://localhost:3000/health
```

---

### 2. Start Frontend Development Server

**Terminal 2:**
```bash
cd frontend
npm install  # First time only
npm run dev
```

The frontend will:
- Start on `http://localhost:5173` (or next available port)
- Connect to backend at `http://localhost:3000`
- Open in browser automatically (if configured)

**Expected output:**
```
  VITE v7.2.2  ready in 500 ms

  ➜  Local:   http://localhost:5173/
  ➜  Network: use --host to expose
```

---

## Quick Commands

### Backend Only
```bash
# Start backend
cargo run

# Or with helper script (kills existing process on port 3000)
./scripts/run.sh  # Linux/Mac
.\scripts\run.ps1  # Windows PowerShell
```

### Frontend Only
```bash
cd frontend
npm run dev
```

### Both Together (Manual)
Open two terminals:
- **Terminal 1:** `cargo run`
- **Terminal 2:** `cd frontend && npm run dev`

---

## Verification

### 1. Backend Health Check
```bash
curl http://localhost:3000/health
```

Should return:
```json
{
  "status": "ok",
  "service": "Jamey 3.0",
  "version": "3.0.0"
}
```

### 2. Frontend Access
Open browser: `http://localhost:5173`

You should see:
- "Jamey 3.0 - General & Guardian" header
- Action evaluation form
- Moral rules list

### 3. Test Connection
1. Enter an action in the frontend
2. Click "Evaluate"
3. You should see a moral score

---

## Troubleshooting

### Backend Won't Start

**Port 3000 already in use:**
```bash
# Linux/Mac
lsof -ti:3000 | xargs kill -9

# Windows PowerShell
Get-NetTCPConnection -LocalPort 3000 | ForEach-Object { Stop-Process -Id $_.OwningProcess -Force }
```

**Database errors:**
```bash
# Delete and recreate database
rm -rf data/jamey.db
cargo run  # Will recreate database
```

### Frontend Won't Connect

**Check backend is running:**
```bash
curl http://localhost:3000/health
```

**Check frontend .env:**
```bash
cd frontend
cat .env  # Should have VITE_API_URL=http://localhost:3000
```

**CORS errors:**
- Backend CORS is configured to allow `http://localhost:5173`
- If using different port, update backend `.env`:
  ```
  CORS_ALLOWED_ORIGINS=http://localhost:5173,http://localhost:3000
  ```

### Frontend Build Errors

**Missing dependencies:**
```bash
cd frontend
rm -rf node_modules package-lock.json
npm install
```

**TypeScript errors:**
```bash
cd frontend
npm run build  # Check for errors
```

---

## Development Workflow

### Typical Development Session

1. **Start backend** (Terminal 1):
   ```bash
   cargo run
   ```

2. **Start frontend** (Terminal 2):
   ```bash
   cd frontend
   npm run dev
   ```

3. **Make changes:**
   - Backend: Changes require restart (`Ctrl+C` then `cargo run`)
   - Frontend: Hot reload (changes appear automatically)

4. **Test:**
   - Backend: `curl http://localhost:3000/health`
   - Frontend: Open `http://localhost:5173` in browser

---

## Production Build

### Backend
```bash
cargo build --release
./target/release/jamey-3
```

### Frontend
```bash
cd frontend
npm run build
# Output in frontend/dist/
```

---

## Environment Variables

### Backend (.env in project root)
```env
# Optional - for LLM features
OPENROUTER_API_KEY=your-key-here

# Optional - MQTT
MQTT_BROKER_HOST=localhost
MQTT_BROKER_PORT=1883

# Optional - CORS
CORS_ALLOWED_ORIGINS=http://localhost:5173
```

### Frontend (frontend/.env)
```env
VITE_API_URL=http://localhost:3000
VITE_API_KEY=optional-api-key
```

---

## Summary

✅ **Backend:** `cargo run` (Terminal 1)  
✅ **Frontend:** `cd frontend && npm run dev` (Terminal 2)  
✅ **Access:** `http://localhost:5173` in browser

Both must be running for the full application to work!

---

**Last Updated:** 2025-01-27

