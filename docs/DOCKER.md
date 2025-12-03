# Docker Guide

OpenCLI uses Alpine Linux for production images

**Image sizes:**
- Production: ~15-20 MB
- Development: ~500 MB

---

## Quick Start

Run OpenCLI with Docker:

```bash
# Show help
docker run --rm -v $(pwd):/workspace ghcr.io/mxp96/open-cli:latest --help

# Setup your project
docker run --rm -v $(pwd):/workspace ghcr.io/mxp96/open-cli:latest setup

# Build
docker run --rm -v $(pwd):/workspace ghcr.io/mxp96/open-cli:latest build
```

Or use Docker Compose:

```bash
# Start development
docker compose up dev

# Build binary
docker compose up build

# Run tests
docker compose -f docker-compose.test.yml up --abort-on-container-exit
```

---

## Development

Start the dev container with hot reload:

```bash
docker compose up watch
```

This will auto-recompile when you save files.

For interactive development:

```bash
docker compose up -d dev
docker exec -it opencli-dev bash

# Inside container
cargo check
cargo test
cargo run -- build
```

---

## Testing

First, create test scenarios:

```bash
make setup-test-scenarios
```

Run all tests:

```bash
docker compose -f docker-compose.test.yml up --abort-on-container-exit
```

Run specific test:

```bash
# Test package installation
docker compose -f docker-compose.test.yml up test-package-install --abort-on-container-exit

# Test build workflow
docker compose -f docker-compose.test.yml up test-build-workflow --abort-on-container-exit
```

Available tests:
- `test-package-install`
- `test-package-remove`
- `test-build-workflow`
- `test-legacy-plugins`
- `test-version-constraints`
- `integration-test`

---

## CI Pipeline

Run the full CI pipeline locally:

```bash
make ci
```

This runs lint, test, build, and security audit.

Or run individual steps:

```bash
docker compose -f docker-compose.ci.yml up lint --abort-on-container-exit
docker compose -f docker-compose.ci.yml up test --abort-on-container-exit
docker compose -f docker-compose.ci.yml up build-release --abort-on-container-exit
```

---

## Volumes

OpenCLI uses these volumes:

- `opencli-cargo-cache` - Cargo dependencies
- `opencli-target-cache` - Build cache
- `opencli-config` - User config
- `opencli-test-config` - Test config

Clean volumes:

```bash
docker volume rm $(docker volume ls -q | grep opencli)
```

Or use the Makefile:

```bash
make docker-clean
```

---

## Production

Pull the latest image:

```bash
docker pull ghcr.io/mxp96/open-cli:latest
```

Run in production:

```bash
docker run -d \
  --name opencli \
  --restart unless-stopped \
  -v /path/to/project:/workspace \
  ghcr.io/mxp96/open-cli:latest build
```

Check if it's running:

```bash
docker exec opencli opencli --version
```

---

## Troubleshooting

**Build issues?** Clean everything and rebuild:

```bash
docker compose down -v
docker system prune -a
docker compose build --no-cache
```

**Permission problems?** Fix ownership:

```bash
docker run --rm -v $(pwd):/workspace alpine chown -R $(id -u):$(id -g) /workspace
```

**Need logs?**

```bash
docker compose logs -f
```

---

## Makefile

Common commands:

```bash
make dev              # Start dev container
make build            # Build release
make test             # Run tests
make test-docker      # Run Docker tests
make ci               # Run CI pipeline
make docker-clean     # Clean everything
```

Run `make help` to see all commands.
