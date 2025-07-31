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

### Smoke Tests (`tests/ui_smoke.rs`)

**End-to-end smoke tests** that validate complete user workflows with **testcontainers support**:

- **Complete User Workflow**: Authentication → Domain → Aliases → User → Reports → Cleanup
- **Random Data Generation**: Avoids conflicts with existing data
- **Automatic Cleanup**: Ensures test isolation
- **Configurable Execution**: Supports both local and CI environments
- **Testcontainers Integration**: Database isolation for CI environments
- **Timeout Handling**: Comprehensive timeout management (5-minute total, 60s/30s step timeouts)
- **Error Reporting**: Detailed logging and debugging support

#### Smoke Test Configuration

The smoke tests support flexible configuration via environment variables:

```bash
# Application URL to test
SMOKE_TEST_APP_URL=http://localhost:3000

# Headless mode for CI environments
SMOKE_TEST_HEADLESS=true

# Total timeout in seconds
SMOKE_TEST_TIMEOUT=300

# Enable VNC for debugging
SMOKE_TEST_VNC=false
```

#### Smoke Test Functions

- `ui_smoke_e2e_flow()`: Standard smoke test with default configuration
- `ui_smoke_e2e_flow_testcontainers()`: Smoke test with testcontainers support
- `run_smoke_test_with_config()`: Parameterized smoke test execution
- `run_smoke_test_with_testcontainers()`: CI-ready smoke test with database isolation

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

## CI/CD Integration

### GitHub Actions Workflow

The smoke tests are integrated into CI/CD via `.github/workflows/smoke-tests.yml`:

```yaml
name: Smoke Tests
on: [push, pull_request, workflow_dispatch]

jobs:
  smoke-test:
    runs-on: ubuntu-latest
    steps:
      - name: Start Selenium container
        run: docker compose --profile test up -d selenium
      
      - name: Build and start application
        run: cargo build --release && cargo run --release &
      
      - name: Run smoke test with testcontainers
        run: cargo test ui_smoke_e2e_flow_testcontainers -- --ignored --nocapture
```

### CI Features

- **Automated Execution**: Runs on push, pull requests, and manual triggers
- **Selenium Container Setup**: Automatic Selenium WebDriver setup
- **Application Startup**: Automatic app startup and health checks
- **Artifact Collection**: Screenshots and logs for debugging
- **Resource Cleanup**: Proper cleanup of containers and processes
- **Timeout Management**: Configurable timeouts for CI environments

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

# Run smoke tests specifically
make test-smoke
```

### Manual Smoke Test Execution

```bash
# Run smoke test with default configuration
cargo test ui_smoke_e2e_flow -- --ignored --nocapture

# Run smoke test with testcontainers support
cargo test ui_smoke_e2e_flow_testcontainers -- --ignored --nocapture

# Run with custom configuration
SMOKE_TEST_APP_URL=http://localhost:3000 \
SMOKE_TEST_HEADLESS=true \
SMOKE_TEST_TIMEOUT=300 \
cargo test ui_smoke_e2e_flow -- --ignored --nocapture
```

This script will:
1. Start the test environment (Selenium, database, application)
2. Wait for all services to be ready
3. Run the UI tests
4. Clean up resources automatically

## Smoke Test Workflow

The smoke test performs the following end-to-end workflow:

1. **Authentication**: Login with test credentials
2. **Domain Creation**: Create a new test domain with random data
3. **Alias Creation**: Create two aliases pointing to the test user
4. **User Creation**: Create a test user for the domain
5. **Reports Verification**: Check that the reports page loads correctly
6. **Cleanup**: Delete all created resources (user, aliases, domain)

### Test Data Generation

The smoke test uses random data generation to avoid conflicts:

- **Domain**: `{random}.test.com`
- **Aliases**: `alias1-{random}@{domain}`, `alias2-{random}@{domain}`
- **User**: `user-{random}@{domain}`
- **Maildir**: `{domain}/user-{random}/`

### Error Handling

The smoke test includes comprehensive error handling:

- **Timeout Management**: 5-minute total timeout with step-specific timeouts
- **Resource Cleanup**: Automatic cleanup even on failure
- **WebDriver Cleanup**: Proper WebDriver shutdown with fallback cleanup
- **Detailed Logging**: Step-by-step progress logging
- **Artifact Collection**: Screenshots and logs for debugging

## Debugging

### Local Debugging

For local debugging, you can run the smoke test in non-headless mode:

```bash
SMOKE_TEST_HEADLESS=false cargo test ui_smoke_e2e_flow -- --ignored --nocapture
```

### CI Debugging

The CI workflow collects artifacts for debugging:

- **Application Logs**: `app.log` contains application output
- **Screenshots**: Available if VNC is enabled
- **Test Results**: Detailed test output with timestamps

### Common Issues

1. **Selenium Connection**: Ensure Selenium container is running
2. **Application Startup**: Check that the app is accessible at the configured URL
3. **Database Connection**: Verify database connectivity for testcontainers
4. **Timeout Issues**: Adjust `SMOKE_TEST_TIMEOUT` for slower environments
5. **Resource Cleanup**: Check for orphaned containers or processes

## Performance Considerations

- **Test Duration**: Smoke tests typically take 3-5 minutes
- **Resource Usage**: Requires significant RAM and CPU
- **Parallel Execution**: Tests should run sequentially to avoid conflicts
- **CI Optimization**: Use headless mode and optimized timeouts for CI 
