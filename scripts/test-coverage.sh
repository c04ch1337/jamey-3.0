#!/bin/bash
# Test Coverage Script for Jamey 3.0
# Generates test coverage reports using cargo-tarpaulin

set -e

echo "🔍 Running test coverage analysis..."

# Check if cargo-tarpaulin is installed
if ! command -v cargo-tarpaulin &> /dev/null; then
    echo "📦 Installing cargo-tarpaulin..."
    cargo install cargo-tarpaulin
fi

# Create coverage directory
mkdir -p coverage

# Run tests with coverage
echo "🧪 Running tests with coverage..."
cargo tarpaulin \
    --out Xml \
    --out Html \
    --output-dir coverage \
    --all-features \
    --exclude-files 'src/bin/*' \
    --exclude-files 'tests/*' \
    --exclude-files 'benches/*' \
    --timeout 120

# Check coverage threshold (70%)
COVERAGE=$(grep -oP 'line-rate="\K[0-9.]+' coverage/cobertura.xml | head -1)
COVERAGE_PERCENT=$(echo "$COVERAGE * 100" | bc | cut -d. -f1)

echo ""
echo "📊 Coverage Results:"
echo "   Line Coverage: ${COVERAGE_PERCENT}%"
echo "   Target: 70%"
echo ""

if [ "$COVERAGE_PERCENT" -lt 70 ]; then
    echo "❌ Coverage below threshold (70%)"
    echo "   Current: ${COVERAGE_PERCENT}%"
    exit 1
else
    echo "✅ Coverage meets threshold!"
    echo "   HTML report: coverage/tarpaulin-report.html"
    echo "   XML report: coverage/cobertura.xml"
fi

