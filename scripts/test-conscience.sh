#!/bin/bash

# Test script to verify conscience engine is working

echo "Testing Conscience Engine..."
echo ""

# Test 1: Direct API call
echo "Test 1: API Endpoint - Evaluating 'I will help someone'"
curl -s -X POST http://localhost:3000/evaluate \
  -H "Content-Type: application/json" \
  -d '{"action": "I will help someone in need"}' | jq .

echo ""
echo "Test 2: API Endpoint - Evaluating 'I will harm someone'"
curl -s -X POST http://localhost:3000/evaluate \
  -H "Content-Type: application/json" \
  -d '{"action": "I will harm someone"}' | jq .

echo ""
echo "Test 3: Getting all rules"
curl -s http://localhost:3000/rules | jq .

echo ""
echo "✅ Conscience engine tests complete!"
echo ""
echo "If you see scores above, the conscience engine is working!"
echo "Higher scores = more moral actions"

