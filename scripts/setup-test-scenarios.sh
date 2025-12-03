#!/bin/bash
set -e

echo "Setting up test scenarios..."

# Create test scenario dirs
mkdir -p test-scenarios/install
mkdir -p test-scenarios/remove
mkdir -p test-scenarios/build
mkdir -p test-scenarios/legacy
mkdir -p test-scenarios/versions
mkdir -p test-scenarios/integration

# Create sample gamemode.pwn in each dir
for dir in test-scenarios/*/; do
    echo 'main() { print("Test scenario"); }' > "${dir}gamemode.pwn"
done

echo "Test scenarios created successfully"

