# UI Tests for SortingOffice

This document describes the functional UI tests for the SortingOffice application, which test the web interface using Selenium WebDriver and headless browser testing.

## Overview

The UI tests are designed to verify that the web interface works correctly from a user's perspective. They test:

- Page loading and navigation
- Form interactions and validation
- User workflows (create, read, update, delete operations)
- Responsive design
- Accessibility features
- Performance metrics
- Security features

## Test Structure

### Containerized UI Tests (`tests/ui_containerized.rs`)

These tests cover comprehensive UI functionality with **testcontainers for database isolation**:

- **Database Isolation**: Each test gets its own MySQL container
- **Page Loading**: Verifies that all pages load correctly
- **Navigation**: Tests menu navigation and breadcrumbs
- **Form Workflows**: Complete user workflows for creating domains, users, and aliases
- **Form Validation**: Tests client-side and server-side validation
- **Responsive Design**: Checks that the interface works on different screen sizes
- **Error Handling**: Tests 404 pages and error states
- **Accessibility**: Basic accessibility checks
- **Cross-browser Compatibility**: Tests different viewport sizes
- **Wizard Flow**: Tests the complete wizard workflow with dynamic fields

### Security Tests (`tests/security.rs`)

Comprehensive security tests to ensure the application is protected:

- **SQL Injection Prevention**: Tests various SQL injection payloads
- **Authentication Bypass**: Tests unauthenticated access attempts
- **Authorization**: Tests role-based access control
- **Input Validation**: Tests edge cases and malicious input
- **Session Security**: Tests session management and expiration

## Prerequisites

### Required Software

1. **Docker and Docker Compose**: For running the test environment
2. **Rust**: For running the tests
3. **Chrome/Chromium**: The tests use Chrome WebDriver

### System Requirements

- At least 4GB RAM (for running Selenium and Chrome)
- Docker with at least 2GB memory allocation
- Network access for downloading Docker images

## Setup

### 1. Install Dependencies

The UI tests use the following Rust dependencies (already added to `Cargo.toml`):

```toml
[dev-dependencies]
fantoccini = "0.22.0"     # WebDriver client for basic/advanced tests
thirtyfour = "0.32.0"     # WebDriver client for headless tests
testcontainers = "0.15"   # For database isolation
testcontainers-modules = { version = "0.3", features = ["mysql"] }
tokio = { version = "1", features = ["full"] }
anyhow = "1.0"
reqwest = { version = "0.11.27", features = ["json"] }
```

### 2. Environment Configuration

The tests expect the following environment variables:

- `DATABASE_URL`: MySQL connection string for test database
- `RUST_TEST_THREADS`: Set to 1 for sequential test execution
- `RUST_LOG`: Logging level (set to "debug" for detailed output)
- `APP_URL`: Application URL (defaults to http://localhost:3000)

### 3. Test Environment

The tests use a Docker Compose setup (`docker-compose.yml`) that includes:

- **Selenium Standalone Chrome**: WebDriver server
- **MySQL Database**: Test database (isolated per test)
- **Application Container**: Running SortingOffice for testing

## Running the Tests

### Quick Start

Use the provided script to run all tests:

```bash
# Run all tests (unit + integration + security + UI)
./tests/run_tests.sh all

# Run only UI tests
./tests/run_tests.sh ui

# Run only security tests
./tests/run_tests.sh security

# Run only unit tests
./tests/run_tests.sh unit

# Show help
./tests/run_tests.sh help
```

### Alternative: Makefile Commands

```bash
# Run all tests
make test-all

# Run only UI tests
make test-ui

# Run only security tests
make test-security

# Run only unit tests
make test-unit
```

This script will:
1. Start the test environment (Selenium, database, application)
2. Wait for all services to be ready
3. Run the UI tests
4. Clean up resources automatically 
