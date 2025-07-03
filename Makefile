# Sorting Office Makefile
# Provides convenient shortcuts for common tasks

.PHONY: help build up down restart logs dev dev-down clean status shell db-shell

# Default target
help:
	@echo "🚀 Sorting Office - Available Commands"
	@echo "======================================"
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
	@echo "Local Development:"
	@echo "  make install    - Install dependencies"
	@echo "  make test       - Run tests"
	@echo "  make run        - Run locally with cargo"

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

test:
	cargo test

run:
	cargo run

# Database operations
migrate:
	diesel migration run

migrate-revert:
	diesel migration revert

migrate-reset:
	diesel migration revert
	diesel migration run

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
