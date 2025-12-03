#!/bin/bash
set -e

echo "Formatting Rust code..."
cargo fmt --all

echo "[SUCCESS] Code formatted successfully!"
echo ""
echo "Run this to check formatting:"
echo "  cargo fmt --all -- --check"
