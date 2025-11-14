#!/bin/bash

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

