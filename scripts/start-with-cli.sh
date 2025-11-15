#!/bin/bash

# Start Jamey 3.0 backend and CLI together
# This script starts the backend in the background and then launches the CLI

echo "🚀 Starting Jamey 3.0 Backend + CLI..."

# Kill any existing server on port 3000
if lsof -ti:3000 > /dev/null 2>&1; then
    echo "🛑 Killing existing process on port 3000..."
    lsof -ti:3000 | xargs kill -9 2>/dev/null
    sleep 1
fi

# Start backend in background
echo "📡 Starting backend server..."
cargo run > /tmp/jamey-backend.log 2>&1 &
BACKEND_PID=$!

# Wait for backend to be ready
echo "⏳ Waiting for backend to start..."
for i in {1..30}; do
    if curl -s http://localhost:3000/health > /dev/null 2>&1; then
        echo "✅ Backend is ready!"
        break
    fi
    if [ $i -eq 30 ]; then
        echo "❌ Backend failed to start. Check /tmp/jamey-backend.log"
        kill $BACKEND_PID 2>/dev/null
        exit 1
    fi
    sleep 1
done

# Start CLI
echo ""
echo "💬 Starting CLI chat interface..."
echo "   (Backend running in background, PID: $BACKEND_PID)"
echo "   (Backend logs: /tmp/jamey-backend.log)"
echo ""

# Trap to cleanup on exit
trap "echo ''; echo '🛑 Stopping backend...'; kill $BACKEND_PID 2>/dev/null; exit" INT TERM

# Start CLI
cargo run --bin jamey-cli chat

# Cleanup
echo ""
echo "🛑 Stopping backend..."
kill $BACKEND_PID 2>/dev/null

