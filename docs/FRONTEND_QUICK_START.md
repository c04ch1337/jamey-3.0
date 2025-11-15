# Frontend Quick Start Guide

5-minute setup guide for the React frontend included in this repository.

## Prerequisites

- Node.js 18+ and npm
- Backend running (see [Backend Setup](../README.md#backend-setup))
- API key (optional, if backend requires authentication)

## Step 1: Install Dependencies (1 minute)

```bash
cd frontend
npm install
```

## Step 2: Create API Key (2 minutes)

If your backend requires API key authentication:

**Option A: Using Script** (from project root)
```bash
# From project root directory
./scripts/create-api-key.sh frontend-key 60
```

**Option B: Programmatically**
```rust
// In backend code or CLI
let key_manager = ApiKeyManager::new(pool);
let key = key_manager.create_key("frontend-key", None, Some(60)).await?;
println!("API Key: {}", key);
```

**Save the API key** - you'll need it in the next step!

## Step 3: Configure Environment (1 minute)

Create `.env` file in the `frontend/` directory:

```bash
cd frontend
cp .env.example .env
```

Edit `.env` and add your configuration:

```bash
# Backend URL
VITE_API_URL=http://localhost:3000

# API Key (from Step 2)
VITE_API_KEY=jamey_your-api-key-here
```

**Note**: If backend doesn't require authentication, you can leave `VITE_API_KEY` empty.

## Step 4: Start Backend (if not running)

In a separate terminal, start the backend:

```bash
# From project root
cargo run --release
```

Backend should be running on `http://localhost:3000`

## Step 5: Start Frontend (1 minute)

```bash
# From frontend directory
npm run dev
```

Frontend will start on `http://localhost:5173` (or the port Vite assigns).

## Step 6: Verify Connection

1. Open browser to `http://localhost:5173`
2. You should see the Jamey 3.0 interface
3. Try evaluating an action:
   - Enter: "I will help others"
   - Click "Evaluate"
   - You should see a moral score

## Troubleshooting

### Frontend Can't Connect to Backend

**Check**:
1. Backend is running: `curl http://localhost:3000/health`
2. `VITE_API_URL` in `.env` matches backend URL
3. No firewall blocking connection

**Fix**: Update `VITE_API_URL` in `frontend/.env` to match your backend URL.

### Authentication Error (401)

**Check**:
1. `VITE_API_KEY` is set in `.env`
2. API key is correct (no typos)
3. API key exists in backend database

**Fix**: 
- Verify API key: Check backend database or recreate key
- Update `.env` with correct key

### CORS Error

**Check**: Backend CORS configuration

**Fix**: Backend allows all origins in development. If you see CORS errors:
- Check backend is running
- Verify backend URL in `VITE_API_URL`
- Check browser console for specific error

### Rate Limit Exceeded (429)

**Check**: Too many requests

**Fix**: 
- Wait a minute before retrying
- Request higher rate limit for your API key
- Implement retry logic with backoff

## Next Steps

- **Customize UI**: Edit `frontend/src/App.tsx`
- **Add Features**: See [Frontend Integration Guide](FRONTEND_INTEGRATION.md)
- **Multiple Frontends**: See [Multiple Frontends Guide](MULTIPLE_FRONTENDS.md)
- **API Reference**: See [API Reference](API_REFERENCE.md)

## Production Build

Build for production:

```bash
npm run build
```

Deploy the `dist/` folder to your web server.

**Important**: Set environment variables in your production environment, as Vite embeds them at build time.

