#!/bin/bash
# Comprehensive Test Runner for Jamey 3.0
# Runs all test suites: unit, integration, and benchmarks

set -e

echo "🧪 Running comprehensive test suite for Jamey 3.0..."
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Track failures
FAILED=0

# Run unit tests
echo -e "${YELLOW}📦 Running unit tests...${NC}"
if cargo test --lib --all-features; then
    echo -e "${GREEN}✅ Unit tests passed${NC}"
else
    echo -e "${RED}❌ Unit tests failed${NC}"
    FAILED=1
fi
echo ""

# Run integration tests
echo -e "${YELLOW}🔗 Running integration tests...${NC}"
if cargo test --test '*' --all-features; then
    echo -e "${GREEN}✅ Integration tests passed${NC}"
else
    echo -e "${RED}❌ Integration tests failed${NC}"
    FAILED=1
fi
echo ""

# Run benchmarks (if available)
echo -e "${YELLOW}⚡ Running benchmarks...${NC}"
if cargo bench --all-features 2>/dev/null; then
    echo -e "${GREEN}✅ Benchmarks completed${NC}"
else
    echo -e "${YELLOW}⚠️  Benchmarks skipped (not critical)${NC}"
fi
echo ""

# Run clippy
echo -e "${YELLOW}🔍 Running clippy...${NC}"
if cargo clippy --all-targets --all-features -- -D warnings; then
    echo -e "${GREEN}✅ Clippy checks passed${NC}"
else
    echo -e "${RED}❌ Clippy checks failed${NC}"
    FAILED=1
fi
echo ""

# Check formatting
echo -e "${YELLOW}📝 Checking code formatting...${NC}"
if cargo fmt -- --check; then
    echo -e "${GREEN}✅ Code formatting OK${NC}"
else
    echo -e "${RED}❌ Code formatting issues found${NC}"
    echo "   Run 'cargo fmt' to fix"
    FAILED=1
fi
echo ""

# Summary
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✅ All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}❌ Some tests failed${NC}"
    exit 1
fi

