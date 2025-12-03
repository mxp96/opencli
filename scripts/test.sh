#!/bin/bash
set -e

echo ""
echo "========================================"
echo "Running Tests"
echo "========================================"
echo ""

echo "Running unit tests..."
cargo test --release --verbose

echo ""
echo "========================================"
echo "[SUCCESS] All tests passed!"
echo "========================================"

