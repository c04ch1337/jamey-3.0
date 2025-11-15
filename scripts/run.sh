#!/bin/bash

# Quick script to kill any existing server on port 3000 and start Jamey 3.0

echo "🔍 Checking for existing server on port 3000..."

# Kill any process using port 3000
if lsof -ti:3000 > /dev/null 2>&1; then
    echo "🛑 Killing existing process on port 3000..."
    lsof -ti:3000 | xargs kill -9 2>/dev/null
    sleep 1
    echo "✅ Port 3000 cleared"
else
    echo "✅ Port 3000 is free"
fi

echo ""
echo "🚀 Starting Jamey 3.0..."
echo ""

# Run the application
cargo run

