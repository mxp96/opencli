# Contributing Guide

## Code Formatting

Before committing, format your code:

```bash
# Local formatting
cargo fmt --all

# Using Docker
docker compose -f docker-compose.yml up format --abort-on-container-exit
# or
make docker-format
```

Check if formatting is correct:

```bash
cargo fmt --all -- --check

# Using Docker
docker compose -f docker-compose.ci.yml up format-check --abort-on-container-exit
```

## Linting

Run Clippy to check for common mistakes:

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

## Testing

Run unit tests:

```bash
cargo test --release
```

Run integration tests with Docker:

```bash
# Setup test scenarios
make setup-test-scenarios

# Run all tests
docker compose -f docker-compose.test.yml up --abort-on-container-exit
```

## Before Pushing

Run these commands to ensure CI will pass:

```bash
# Format code
cargo fmt --all

# Check lints
cargo clippy --all-targets --all-features -- -D warnings

# Run tests
cargo test --release

# (Optional) Run full CI locally
make ci
```

## GitHub Actions

The project has three workflows:

### test.yml (Automatic)
Runs on:
- Every push to master
- Every PR to master
- Weekly schedule (Sunday midnight)
- Manual trigger

Includes:
- Code formatting check
- Clippy linting
- Unit tests
- Docker integration tests
- Security audit

### build.yml (Manual)
- Builds binaries for all platforms
- Linux (x86_64, musl)
- Windows (x86_64)
- macOS (x86_64, ARM64)
- Docker image

### release.yml (Manual)
- Creates GitHub release
- Builds all platform binaries
- Publishes Docker image
- Generates changelog

## Common Issues

### Formatting check fails
```bash
# Fix formatting
cargo fmt --all
```

### Clippy warnings
```bash
# See warnings
cargo clippy --all-targets --all-features

# Fix automatically where possible
cargo clippy --fix --all-targets --all-features
```

### Test failures
```bash
# Run with verbose output
cargo test --release -- --nocapture

# Run specific test
cargo test test_name -- --nocapture
```

