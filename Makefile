.PHONY: help dev build test clean docker-build docker-format docker-test lint format check install release

BINARY_NAME := opencli
CARGO := cargo
DOCKER_COMPOSE := docker compose

help:
	@echo "OpenCLI - Makefile Commands"
	@echo ""
	@echo "Development:"
	@echo "  make dev              - Start development container"
	@echo "  make watch            - Start file watcher"
	@echo "  make format           - Format code with rustfmt"
	@echo "  make lint             - Run clippy linter"
	@echo "  make check            - Check code compilation"
	@echo ""
	@echo "Building:"
	@echo "  make build            - Build release binary"
	@echo "  make build-debug      - Build debug binary"
	@echo "  make build-min        - Build minimum size binary"
	@echo ""
	@echo "Testing:"
	@echo "  make test             - Run all tests"
	@echo "  make test-unit        - Run unit tests only"
	@echo "  make test-integration - Run integration tests"
	@echo "  make test-docker      - Run Docker-based tests"
	@echo ""
	@echo "Docker:"
	@echo "  make docker-build     - Build Docker images"
	@echo "  make docker-dev       - Start Docker dev environment"
	@echo "  make docker-format    - Format code using Docker"
	@echo "  make docker-test      - Run tests in Docker"
	@echo "  make docker-clean     - Clean Docker resources"
	@echo ""
	@echo "CI/CD:"
	@echo "  make ci               - Run full CI pipeline locally"
	@echo "  make security-audit   - Run security audit"
	@echo "  make coverage         - Generate test coverage"
	@echo ""
	@echo "Utilities:"
	@echo "  make install          - Install binary to system"
	@echo "  make clean            - Clean build artifacts"
	@echo "  make release          - Create release build"

dev:
	$(DOCKER_COMPOSE) -f docker-compose.yml up dev

watch:
	$(DOCKER_COMPOSE) -f docker-compose.yml up watch

build:
	$(CARGO) build --release

build-debug:
	$(CARGO) build

build-min:
	$(CARGO) build --profile min-size

test:
	$(CARGO) test --release --verbose

test-unit:
	$(CARGO) test --release --lib

test-integration:
	$(CARGO) test --release --test '*'

test-docker:
	$(DOCKER_COMPOSE) -f docker-compose.test.yml up --abort-on-container-exit

format:
	$(CARGO) fmt --all

lint:
	$(CARGO) clippy --all-targets --all-features -- -D warnings

check:
	$(CARGO) check --all-targets --all-features

docker-build:
	$(DOCKER_COMPOSE) build

docker-dev:
	$(DOCKER_COMPOSE) -f docker-compose.yml up dev

docker-format:
	$(DOCKER_COMPOSE) -f docker-compose.yml up format --abort-on-container-exit

docker-test:
	$(DOCKER_COMPOSE) -f docker-compose.test.yml up --abort-on-container-exit

docker-clean:
	$(DOCKER_COMPOSE) down -v --remove-orphans
	docker system prune -f

ci:
	@echo "Running CI pipeline locally..."
	$(DOCKER_COMPOSE) -f docker-compose.ci.yml up lint --abort-on-container-exit
	$(DOCKER_COMPOSE) -f docker-compose.ci.yml up test --abort-on-container-exit
	$(DOCKER_COMPOSE) -f docker-compose.ci.yml up build-release --abort-on-container-exit
	$(DOCKER_COMPOSE) -f docker-compose.ci.yml up security-audit --abort-on-container-exit
	@echo "CI pipeline completed successfully"

security-audit:
	$(DOCKER_COMPOSE) -f docker-compose.ci.yml up security-audit --abort-on-container-exit

coverage:
	$(DOCKER_COMPOSE) -f docker-compose.ci.yml up coverage --abort-on-container-exit

install: build
	sudo cp target/release/$(BINARY_NAME) /usr/local/bin/

clean:
	$(CARGO) clean
	rm -rf target/
	rm -rf artifacts/
	rm -rf coverage/

release: clean
	$(CARGO) build --release
	strip target/release/$(BINARY_NAME)
	@echo "Release binary: target/release/$(BINARY_NAME)"
	@ls -lh target/release/$(BINARY_NAME)

benchmark:
	$(CARGO) bench

size-check:
	$(DOCKER_COMPOSE) -f docker-compose.ci.yml up size-check --abort-on-container-exit

setup-test-scenarios:
	@mkdir -p test-scenarios/{install,remove,build,legacy,versions,integration}
	@for dir in test-scenarios/*; do \
		echo 'main() { print("Test scenario"); }' > $$dir/gamemode.pwn; \
	done
	@echo "Test scenarios created"
