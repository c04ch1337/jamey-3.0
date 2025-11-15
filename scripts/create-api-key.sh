#!/bin/bash
# Script to create an initial API key for Jamey 3.0
# This requires the application to be running or uses sqlx-cli

set -e

if [ -z "$1" ]; then
    echo "Usage: $0 <key-name> [rate-limit-per-minute]"
    echo ""
    echo "Example:"
    echo "  $0 initial-key 60"
    echo ""
    echo "This will create an API key named 'initial-key' with a rate limit of 60 requests per minute"
    exit 1
fi

KEY_NAME="$1"
RATE_LIMIT="${2:-60}"

# Generate a secure random key
API_KEY="jamey_$(openssl rand -hex 16)"

# Hash the key (using Python for portability, or you can use other tools)
HASH=$(echo -n "$API_KEY" | sha256sum | cut -d' ' -f1)

echo "🔑 Creating API key: $KEY_NAME"
echo ""

# Check if database exists
if [ ! -f "data/jamey.db" ]; then
    echo "❌ Error: Database not found at data/jamey.db"
    echo "   Please run the application first to initialize the database:"
    echo "   cargo run"
    exit 1
fi

# Insert the key into the database
sqlite3 data/jamey.db <<EOF
INSERT INTO api_keys (key_hash, name, created_at, rate_limit_per_minute)
VALUES ('$HASH', '$KEY_NAME', datetime('now'), $RATE_LIMIT);
EOF

if [ $? -eq 0 ]; then
    echo "✅ API key created successfully!"
    echo ""
    echo "⚠️  IMPORTANT: Save this key now - it cannot be retrieved later!"
    echo ""
    echo "API Key: $API_KEY"
    echo "Name: $KEY_NAME"
    echo "Rate Limit: $RATE_LIMIT requests/minute"
    echo ""
    echo "Usage:"
    echo "  curl -H 'x-api-key: $API_KEY' http://localhost:3000/evaluate \\"
    echo "    -H 'Content-Type: application/json' \\"
    echo "    -d '{\"action\": \"test action\"}'"
    echo ""
    echo "⚠️  Store this key securely - it will be hashed in the database and cannot be recovered!"
else
    echo "❌ Error: Failed to create API key"
    exit 1
fi

