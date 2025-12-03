#!/bin/bash
set -e

echo ""
echo "========================================"
echo "Checking Code Quality"
echo "========================================"
echo ""

echo "[1/3] Checking code formatting..."
if cargo fmt --all -- --check; then
    echo "[OK] Code formatting is correct"
else
    echo "[ERROR] Code is not formatted correctly"
    echo ""
    echo "Run this to fix:"
    echo "  cargo fmt --all"
    echo "  or: ./scripts/format.sh"
    exit 1
fi
echo ""

echo "[2/3] Running Clippy linter..."
if cargo clippy --all-targets --all-features -- -D warnings; then
    echo "[OK] No Clippy warnings"
else
    echo "[ERROR] Clippy found issues"
    exit 1
fi
echo ""

echo "[3/3] Checking compilation..."
if cargo check --all-targets --all-features; then
    echo "[OK] Code compiles successfully"
else
    echo "[ERROR] Compilation check failed"
    exit 1
fi
echo ""

echo "========================================"
echo "[SUCCESS] All checks passed!"
echo "========================================"
