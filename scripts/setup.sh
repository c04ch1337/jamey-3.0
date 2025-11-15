#!/bin/bash
# Setup script for Jamey 3.0
# This script helps set up the database and create an initial API key

set -e

echo "🚀 Jamey 3.0 Setup Script"
echo "========================="
echo ""

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Error: Cargo not found. Please install Rust first."
    echo "   Visit: https://rustup.rs/"
    exit 1
fi

echo "✅ Rust/Cargo found"

# Check if sqlx-cli is installed (optional, for manual migrations)
if ! command -v sqlx &> /dev/null; then
    echo "⚠️  sqlx-cli not found. Installing..."
    cargo install sqlx-cli --no-default-features --features sqlite
fi

echo "✅ sqlx-cli found"

# Create data directory if it doesn't exist
echo ""
echo "📁 Creating data directory..."
mkdir -p data/memory
mkdir -p backups
echo "✅ Data directories created"

# Check if database exists
if [ -f "data/jamey.db" ]; then
    echo ""
    echo "📊 Database already exists at data/jamey.db"
    echo "   To reset the database, delete it and run this script again:"
    echo "   rm data/jamey.db"
else
    echo ""
    echo "📊 Database will be created automatically on first run"
fi

# Run migrations (this will also create the database if it doesn't exist)
echo ""
echo "🔄 Running database migrations..."
if sqlx migrate run --database-url "sqlite:data/jamey.db" 2>/dev/null; then
    echo "✅ Migrations applied successfully"
else
    echo "⚠️  Note: Migrations will run automatically when you start the application"
    echo "   This is normal if the database doesn't exist yet"
fi

# Check for .env file
if [ ! -f ".env" ]; then
    echo ""
    echo "📝 Creating .env file template..."
    cat > .env << EOF
# Jamey 3.0 Configuration
# Copy this file and update with your values

# OpenRouter API Key (optional, for LLM features)
# OPENROUTER_API_KEY=your-key-here

# Database URL (optional, defaults to sqlite:data/jamey.db)
# DATABASE_URL=sqlite:data/jamey.db

# Server Configuration (optional)
# HOST=0.0.0.0
# PORT=3000

# Logging (optional)
# RUST_LOG=info
EOF
    echo "✅ .env file template created"
    echo "   Please edit .env and add your configuration"
else
    echo ""
    echo "✅ .env file exists"
fi

echo ""
echo "🎉 Setup complete!"
echo ""
echo "Next steps:"
echo "1. Edit .env file with your configuration (if needed)"
echo "2. Run the application: cargo run --release"
echo "3. Create an initial API key (see docs/IMPLEMENTATION_SUMMARY.md)"
echo ""
echo "The database will be automatically initialized when you start the application."
echo "All migrations will run automatically on first startup."
