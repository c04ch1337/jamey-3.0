#!/bin/bash
<<<<<<< HEAD
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
=======

echo "Setting up Jamey 3.0 - General & Guardian"
echo "=========================================="

# Check for Rust
if ! command -v cargo &> /dev/null; then
    echo "⚠️  Rust/Cargo not found. Please install Rust:"
    echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Check for Node.js
if ! command -v node &> /dev/null; then
    echo "⚠️  Node.js not found. Please install Node.js 18+"
    exit 1
fi

echo "✅ Rust found: $(cargo --version)"
echo "✅ Node.js found: $(node --version)"

# Create data directory
echo ""
echo "Creating data directories..."
mkdir -p data/memory
mkdir -p data/short_term
mkdir -p data/long_term
mkdir -p data/working
mkdir -p data/episodic
mkdir -p data/semantic

# Build Rust backend
echo ""
echo "Building Rust backend..."
cargo build

if [ $? -eq 0 ]; then
    echo "✅ Backend built successfully"
else
    echo "❌ Backend build failed"
    exit 1
fi

# Setup frontend
echo ""
echo "Setting up frontend..."
cd frontend
npm install

if [ $? -eq 0 ]; then
    echo "✅ Frontend dependencies installed"
else
    echo "❌ Frontend setup failed"
    exit 1
fi

cd ..

echo ""
echo "=========================================="
echo "✅ Setup complete!"
echo ""
echo "To run the backend:"
echo "  cargo run"
echo ""
echo "To run the frontend (in another terminal):"
echo "  cd frontend && npm run dev"
echo ""
echo "Backend will be available at: http://localhost:3000"
echo "Frontend will be available at: http://localhost:5173"

>>>>>>> origin/main
