# Sorting Office Makefile
# Provides convenient shortcuts for common tasks

# Include database management Makefile
include Makefile.db

.PHONY: help build up down restart logs dev dev-down clean status shell db-shell test test-unit test-ui test-all test-ui-setup test-ui-compose test-ui-cleanup test-ui-failfast

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
	@echo "  make prod-db-setup  - Setup production database with seed data"
	@echo "  make test-db-setup  - Setup test database"
	@echo ""
	@echo "Local Development:"
	@echo "  make install    - Install dependencies"
	@echo "  make test       - Run all tests (unit + UI)"
	@echo "  make test-unit  - Run only unit/integration tests"
	@echo "  make test-ui    - Run only UI tests"
	@echo "  make test-all   - Run all tests (unit + UI)"
	@echo "  make test-ui-setup - Setup Selenium for UI tests"
	@echo "  make test-ui-compose - Run UI tests with Docker Compose"
	@echo "  make test-ui-cleanup - Clean up UI test environment"
	@echo "  make run        - Run locally with cargo watch (auto-restart on changes)"


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

test: test-db-setup
	./tests/run_tests.sh all

test-unit: test-db-setup
	./tests/run_tests.sh unit

test-ui:
	./tests/run_tests.sh ui

test-all: test-db-setup
	./tests/run_tests.sh all

test-ui-setup:
	@echo "🔧 Setting up UI test environment..."
	./tests/run_tests.sh ui-setup

# Docker Compose UI test commands
test-ui-compose:
	@echo "🧪 Running UI tests with Docker Compose..."
	docker compose --profile test up -d selenium
	@echo "⏳ Waiting for Selenium to be ready..."
	@sleep 5
	./tests/run_tests.sh ui

test-ui-cleanup:
	@echo "🧹 Cleaning up UI test environment..."
	docker compose --profile test down selenium

run:
	cargo watch -d 5 -x run



# Utility commands
fmt:
	cargo fmt

check:
	cargo check

clippy:
	cargo clippy

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
