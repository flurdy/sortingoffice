# Sorting Office Makefile
# Provides convenient shortcuts for common tasks

# Include database management Makefile
include Makefile.db

.PHONY: help build up down restart logs dev dev-down clean status shell db-shell test test-unit test-ui test-all test-smoke run-watch selenium-up selenium-down selenium-logs selenium-clean

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
	@echo "  make test-all   - Run all tests (unit + integration + security + api + UI)"
	@echo "  make run-watch  - Run locally with cargo watch (auto-restart on changes)"
	@echo ""
	@echo "Code Quality:"
	@echo "  make fmt        - Format code with cargo fmt"
	@echo "  make check      - Check code compilation"
	@echo "  make clippy     - Run clippy linter"
	@echo "  make pre-commit - Run formatting and compilation checks"


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
	@echo "  1. Start Selenium: make selenium-up"
	@echo "  2. Start app: cargo run (in another terminal)"
	@echo "  3. Ensure app is running on http://localhost:3000"
	@echo ""
	@tests/run_tests.sh smoke

.PHONY: test-all
test-all: test-unit test-integration test-security test-api test-ui
	@echo "All tests completed!"

run-watch:
	cargo watch -d 5 -x run



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

# Test organization:
#   src/tests/           - Unit and integration test modules
#   tests/ui.rs          - Basic UI tests (Selenium)
#   tests/ui_advanced.rs - Advanced UI tests (Selenium)
#   tests/README.md      - Test documentation
#   tests/run_tests.sh   - Unified test runner 

test-ui-failfast:
	./tests/run_tests.sh ui --fail-fast

selenium-up:
	docker compose --profile test up -d selenium

selenium-down:
	docker compose --profile test stop selenium

selenium-logs:
	docker logs -f sortingoffice-selenium

selenium-clean:
	docker rm -f sortingoffice-selenium || true
