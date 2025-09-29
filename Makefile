# Sorting Office Makefile
# Provides convenient shortcuts for common tasks

# Include sectioned Makefiles
include Makefile.db
include Makefile.test
include Makefile.tunnel
include Makefile.docker
include Makefile.dev
include Makefile.code

.PHONY: help shell db-shell

# Section helps
.PHONY: docker-help dev-help tunnel-help code-help

# docker-help moved to Makefile.docker

# dev-help moved to Makefile.dev

# tunnel-help moved to Makefile.tunnel

# code-help moved to Makefile.code

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
	@echo "  make test-single TEST=<name> - Run individual test with cleanup"
	@echo "  make test-single-ui TEST=<name> - Run individual UI test with cleanup"
	@echo "  make docker-help - Detailed Docker section help"
	@echo ""
	@echo "Development:"
	@echo "  make dev        - Start development environment"
	@echo "  make dev-down   - Stop development environment"
	@echo "  make run [PORT=3001] - Run application with cargo (optionally on different port)"
	@echo "  make prod-run [PROD_PORT=8080] [PORT=3001] - Run application with production config (PROD_PORT takes precedence over PORT)"
	@echo "  make run-watch [PORT=3001] - Run application with cargo watch (optionally on different port)"
	@echo "  make dev-help   - Detailed Development section help"
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
	@echo "  make test-smoke [URL] - Run end-to-end smoke test against running app (default: localhost:3000)"
	@echo "  make test-smoke-containerized - Run end-to-end smoke test with testcontainers"
	@echo "  make test-all   - Run all tests (unit + integration + security + api + UI)"
	@echo "  make test-help  - Detailed Testing section help"
	@echo ""
	@echo "SSH Tunnel Management:"
	@echo "  make tunnel-prod            - Start production SSH tunnel"
	@echo "  make tunnel-staging         - Start staging SSH tunnel"
	@echo "  make tunnel-backup          - Start backup SSH tunnel"
	@echo "  make tunnel-all             - Start all SSH tunnels"
	@echo "  make tunnel-stop            - Stop all SSH tunnels"
	@echo "  make tunnel-status          - Show tunnel status"
	@echo "  make tunnel-logs            - Show tunnel logs"
	@echo "  make tunnel-help            - Detailed Tunnel section help"
	@echo ""
	@echo "Code Quality:"
	@echo "  make fmt        - Format code with cargo fmt"
	@echo "  make check      - Check code compilation"
	@echo "  make clippy     - Run clippy linter"
	@echo "  make pre-commit - Run formatting and compilation checks"
	@echo "  make code-help  - Detailed Code Quality section help"
	@echo ""
	@echo "For detailed testing information, run: make test-help"
	@echo "For DB commands, run: make db-help"
	@echo "For Docker details, run: make docker-help"
	@echo "For Tunnel details, run: make tunnel-help"
	@echo "For Dev details, run: make dev-help"
	@echo "For Code Quality details, run: make code-help"

# test-help moved to Makefile.test

# Docker commands moved to Makefile.docker

# Development environment moved to Makefile.dev

# Shell access
shell:
	./docker.sh shell

# db-shell is provided by Makefile.db

# Local development
install:
	cargo install diesel_cli --no-default-features --features mysql

include Makefile.test

# test targets moved to Makefile.test

# dev run/watch and run/prod-run moved to Makefile.dev

# Utility/code-quality targets moved to Makefile.code

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

# SSH Tunnel targets moved to Makefile.tunnel

# Test organization:
#   src/tests/           - Unit and integration test modules
#   tests/ui.rs          - Basic UI tests (testcontainers Selenium)
#   tests/ui_advanced.rs - Advanced UI tests (testcontainers Selenium)
#   tests/README.md      - Test documentation
#   tests/run_tests.sh   - Unified test runner 

# test-ui-failfast moved to Makefile.test

# Individual test runners with cleanup
# individual test runners moved to Makefile.test
