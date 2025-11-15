#!/bin/bash

# SSH-like interface: Connect to a running Jamey 3.0 backend
# This connects to the backend API and provides an interactive CLI

BACKEND_URL="${JAMEY_API_URL:-http://localhost:3000}"
API_KEY="${JAMEY_API_KEY:-}"

echo "🔌 Connecting to Jamey 3.0 Backend..."
echo "   URL: $BACKEND_URL"
echo ""

# Check if backend is running
if ! curl -s "$BACKEND_URL/health" > /dev/null 2>&1; then
    echo "❌ Backend is not running at $BACKEND_URL"
    echo ""
    echo "💡 To start the backend:"
    echo "   ./scripts/run.sh"
    echo "   or"
    echo "   cargo run"
    echo ""
    exit 1
fi

echo "✅ Connected to backend!"
echo ""
echo "💬 Starting interactive chat..."
echo "   (Type /help for commands, /exit to disconnect)"
echo ""

# Use the CLI in connect mode
if [ -n "$API_KEY" ]; then
    cargo run --bin jamey-cli connect --url "$BACKEND_URL" --api-key "$API_KEY"
else
    cargo run --bin jamey-cli connect --url "$BACKEND_URL"
fi

