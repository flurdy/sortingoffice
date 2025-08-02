# Sorting Office Makefile
# Provides convenient shortcuts for common tasks

# Include database management Makefile
include Makefile.db

.PHONY: help build up down restart logs dev dev-down clean status shell db-shell test test-unit test-ui test-all test-smoke test-smoke-containerized test-help run-watch run

# Default target
help:
	@echo "🚀 Sorting Office - Available Commands"
	@echo "======================================="
	@echo ""
	@echo "Docker Commands:"
	@echo "  make build      - Build Docker images"
	@echo "  make up         - Start all services"
	@echo "  make down       - Stop all services"
	@echo "  make restart    - Restart all services"
	@echo "  make logs       - Show logs from all services"
	@echo "  make status     - Show service status"
	@echo "  make clean      - Remove all containers and volumes"
	@echo "  make test-clean - Remove all test containers"
	@echo ""
	@echo "Development:"
	@echo "  make dev        - Start development environment"
	@echo "  make dev-down   - Stop development environment"
	@echo "  make run        - Run application with cargo"
	@echo "  make run-watch  - Run application with cargo watch"
	@echo ""
	@echo "Shell Access:"
	@echo "  make shell      - Open shell in application container"
	@echo "  make db-shell   - Open MySQL shell"
	@echo ""
	@echo "Database Management:"
	@echo "  make db-help        - Show all database commands"
	@echo "  make migrate        - Run pending migrations"
	@echo "  make seed           - Seed database with initial data"
	@echo "  make prod-db-setup  - Setup production database (migrations only, no seeding)"
	@echo "  make dev-db-setup   - Setup development database (includes seeding)"
	@echo "  make test-db-setup  - Setup test database"
	@echo ""
	@echo "Local Development:"
	@echo "  make install    - Install dependencies"
	@echo "  make test       - Run all tests (unit + integration + security + api + UI)"
	@echo "  make test-unit  - Run only unit tests"
	@echo "  make test-integration - Run only integration tests (set TEST_THREADS=N for parallelism, default 8)"
	@echo "  make test-security - Run security tests (SQL injection, auth bypass, etc.)"
	@echo "  make test-api    - Run API tests (authentication, authorization, etc.)"
	@echo "  make test-ui    - Run containerized UI tests (app + db in containers)"
	@echo "  make test-smoke - Run end-to-end smoke test against running app"
	@echo "  make test-smoke-containerized - Run end-to-end smoke test with testcontainers"
	@echo "  make test-all   - Run all tests (unit + integration + security + api + UI)"
	@echo ""
	@echo "SSH Tunnel Management:"
	@echo "  make tunnel-prod            - Start production SSH tunnel"
	@echo "  make tunnel-staging         - Start staging SSH tunnel"
	@echo "  make tunnel-backup          - Start backup SSH tunnel"
	@echo "  make tunnel-all             - Start all SSH tunnels"
	@echo "  make tunnel-stop            - Stop all SSH tunnels"
	@echo "  make tunnel-status          - Show tunnel status"
	@echo "  make tunnel-logs            - Show tunnel logs"
	@echo ""
	@echo "Code Quality:"
	@echo "  make fmt        - Format code with cargo fmt"
	@echo "  make check      - Check code compilation"
	@echo "  make clippy     - Run clippy linter"
	@echo "  make pre-commit - Run formatting and compilation checks"
	@echo ""
	@echo "For detailed testing information, run: make test-help"

# Test help target
test-help:
	@echo "🧪 Sorting Office - Testing Commands"
	@echo "===================================="
	@echo ""
	@echo "Test Types:"
	@echo "  Unit Tests:"
	@echo "    make test-unit          - Run only unit tests (80 tests)"
	@echo "    cargo test --lib        - Alternative: run unit tests directly"
	@echo ""
	@echo "  Integration Tests:"
	@echo "    make test-integration   - Run integration tests with database"
	@echo "    TEST_THREADS=N make test-integration  - Set parallelism (default: 8)"
	@echo "    cargo test --test integration  - Alternative: run directly"
	@echo ""
	@echo "  Security Tests:"
	@echo "    make test-security      - Run security tests (SQL injection, auth bypass)"
	@echo "    cargo test --test security  - Alternative: run directly"
	@echo ""
	@echo "  API Tests:"
	@echo "    make test-api           - Run API tests (authentication, authorization)"
	@echo "    cargo test --test api   - Alternative: run directly"
	@echo ""
	@echo "  UI Tests:"
	@echo "    make test-ui            - Run containerized UI tests (Selenium)"
	@echo "    cargo test --test ui_containerized  - Alternative: run directly"
	@echo ""
	@echo "  Smoke Tests:"
	@echo "    make test-smoke         - Run end-to-end smoke test"
	@echo "    make test-smoke-containerized - Run end-to-end smoke test with testcontainers"
	@echo "    cargo test ui_smoke_e2e_flow -- --ignored  - Alternative: run directly"
	@echo "    cargo test ui_smoke_e2e_flow_testcontainers -- --ignored  - Alternative: run testcontainers directly"
	@echo ""
	@echo "  Test Data Utilities:"
	@echo "    cargo test --test test_data_utilities  - Run test data utility tests"
	@echo ""
	@echo "  All Tests:"
	@echo "    make test               - Run all tests (unit + integration + security + api + UI)"
	@echo "    make test-all           - Run all tests with explicit breakdown"
	@echo "    ./tests/run_tests.sh all  - Alternative: run via test runner"
	@echo ""
	@echo "Test Infrastructure:"
	@echo "  Selenium Setup (deprecated - now using testcontainers):"
	@echo "    Selenium is now automatically managed by testcontainers"
	@echo "    No manual setup required for smoke tests"
	@echo ""
	@echo "  Test Database:"
	@echo "    make test-db-setup      - Setup test database"
	@echo "    docker compose --profile test up -d  - Start test containers"
	@echo ""
	@echo "  Test Cleanup:"
	@echo "    make test-clean         - Remove all test containers"
	@echo "    make clean-rust         - Clean Rust artifacts"
	@echo ""
	@echo "  Test Runner:"
	@echo "    ./tests/run_tests.sh help  - Show test runner help"
	@echo "    ./tests/run_tests.sh unit  - Run specific test type"

# Docker commands
build:
	./docker.sh build

up:
	./docker.sh up

down:
	./docker.sh down

restart:
	./docker.sh restart

logs:
	./docker.sh logs

status:
	./docker.sh status

clean:
	./docker.sh clean

test-clean:
	./docker.sh test-clean

# Development environment
dev:
	./docker.sh dev

dev-down:
	./docker.sh dev-down

# Shell access
shell:
	./docker.sh shell

db-shell:
	./docker.sh db-shell

# Local development
install:
	cargo install diesel_cli --no-default-features --features mysql

test: 
	./tests/run_tests.sh all

.PHONY: test-unit
test-unit:
	@echo "Running unit tests..."
	@tests/run_tests.sh unit

.PHONY: test-integration
test-integration:
	@echo "Running integration tests..."
	@TEST_THREADS=$${TEST_THREADS:-8} tests/run_tests.sh integration

.PHONY: test-security
test-security:
	@echo "Running security tests..."
	@tests/run_tests.sh security

.PHONY: test-api
test-api:
	@echo "Running API tests..."
	@tests/run_tests.sh api

.PHONY: test-ui
test-ui:
	@echo "Running UI tests..."
	@tests/run_tests.sh ui

.PHONY: test-smoke
test-smoke:
	@echo "Running end-to-end smoke test..."
	@echo "Prerequisites:"
	@echo "  1. Start app: cargo run (in another terminal)"
	@echo "  2. Ensure app is running on http://localhost:3000"
	@echo "  (Selenium is automatically managed by testcontainers)"
	@echo ""
	@tests/run_tests.sh smoke

.PHONY: test-smoke-containerized
test-smoke-containerized:
	@echo "Running end-to-end smoke test with testcontainers..."
	@echo "This will start its own isolated environment with:"
	@echo "  - Testcontainers database"
	@echo "  - Testcontainers app container"
	@echo "  - Testcontainers selenium container"
	@echo ""
	# @echo "Building Docker image first..."
	# @make build
	# @echo ""
	@tests/run_tests.sh smoke-containerized

.PHONY: test-all
test-all: test-unit test-integration test-security test-api test-ui
	@echo "All tests completed!"

run-watch:
	cargo watch -d 5 -w src -w static -w templates -w cargo.toml --why -x run

run:
	cargo run

# Utility commands
fmt:
	cargo fmt

check:
	cargo check

clippy:
	cargo clippy

# Pre-commit checks (same as git hook)
pre-commit: fmt check
	@echo "✅ All pre-commit checks passed!"

# Production build
release:
	cargo build --release

# Clean Rust artifacts
clean-rust:
	cargo clean

# Show project info
info:
	@echo "Sorting Office - Mail Server Admin Tool"
	@echo "======================================"
	@echo "Version: $(shell grep '^version =' Cargo.toml | cut -d'"' -f2)"
	@echo "Rust Version: $(shell rustc --version)"
	@echo "Cargo Version: $(shell cargo --version)"
	@echo ""
	@echo "Services:"
	@echo "  - Sorting Office: http://localhost:3000"
	@echo "  - phpMyAdmin: http://localhost:8080"
	@echo "  - MySQL: localhost:3306"
	@echo ""
	@echo "SSH Tunnels:"
	@echo "  - Production: localhost:3306"
	@echo "  - Staging: localhost:3307"
	@echo "  - Backup: localhost:3308"

# SSH Tunnel Management
tunnel-prod:
	@echo "Starting production SSH tunnel..."
	@docker-compose -f docker-compose.tunnels.yml up -d tunnel-prod

tunnel-staging:
	@echo "Starting staging SSH tunnel..."
	@docker-compose -f docker-compose.tunnels.yml up -d tunnel-staging

tunnel-backup:
	@echo "Starting backup SSH tunnel..."
	@docker-compose -f docker-compose.tunnels.yml up -d tunnel-backup

tunnel-all:
	@echo "Starting all SSH tunnels..."
	@docker-compose -f docker-compose.tunnels.yml up -d

tunnel-stop:
	@echo "Stopping all SSH tunnels..."
	@docker-compose -f docker-compose.tunnels.yml down

tunnel-status:
	@echo "SSH Tunnel Status:"
	@docker-compose -f docker-compose.tunnels.yml ps

tunnel-logs:
	@echo "SSH Tunnel Logs:"
	@docker-compose -f docker-compose.tunnels.yml logs

# Test organization:
#   src/tests/           - Unit and integration test modules
#   tests/ui.rs          - Basic UI tests (testcontainers Selenium)
#   tests/ui_advanced.rs - Advanced UI tests (testcontainers Selenium)
#   tests/README.md      - Test documentation
#   tests/run_tests.sh   - Unified test runner 

test-ui-failfast:
	./tests/run_tests.sh ui --fail-fast
