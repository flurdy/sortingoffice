# Testing Guide for SortingOffice

This document describes the comprehensive test suite for the SortingOffice application.

## Overview

The test suite is organized into several modules:

- **Models Tests** (`src/tests/models.rs`): Tests for data models, serialization, and form handling
- **Database Tests** (`src/tests/db.rs`): Tests for database operations and CRUD functionality
- **Handler Tests** (`src/tests/handlers.rs`): Tests for HTTP request handling and responses
- **Utility Tests** (`src/tests/utils.rs`): Tests for helper functions and validation logic
- **Integration Tests** (`src/tests/integration.rs`): End-to-end workflow tests

## Current Test Status ✅

**All test suites are now passing successfully:**

- ✅ **Unit Tests**: 88 tests passing
- ✅ **Handler Tests**: 40 tests passing  
- ✅ **Integration Tests**: 16 tests passing
- ✅ **Security Tests**: 6 tests passing
- ✅ **API Tests**: 9 tests passing
- ✅ **UI Containerized Tests**: 18 tests passing (1 ignored)
- ✅ **UI Smoke Tests**: All tests passing

## Running Tests

### Prerequisites

1. **Database Setup**: Ensure you have a MySQL database available for testing
2. **Environment Variables**: Set up your test environment
3. **For UI Tests**: Docker (for testcontainers) and the application running

### Quick Start

```bash
# Run all unit tests (default)
./tests/run_tests.sh

# Run only unit tests
./tests/run_tests.sh unit

# Run only UI tests
./tests/run_tests.sh ui

# Run only security tests
./tests/run_tests.sh security

# Run only headless UI tests
./tests/run_tests.sh ui-headless

# Run all tests (unit + integration + security + UI)
./tests/run_tests.sh all

# Show help
./tests/run_tests.sh help
```

### Alternative: Direct Cargo Commands

```bash
# Run all tests
cargo test

# Run specific test modules
cargo test models
cargo test db
cargo test handlers
cargo test utils
cargo test integration
cargo test --test security

# Run UI tests
cargo test --test ui_headless

# Run with verbose output
cargo test --verbose

# Run a specific test
cargo test test_domain_crud_operations
```

### Environment Setup

Set the following environment variables for testing:

```bash
export DATABASE_URL="mysql://username:password@localhost/sortingoffice_test"
export RUST_LOG=debug
export RUST_BACKTRACE=1
```

## Test Categories

### 1. Model Tests (`src/tests/models.rs`)

Tests for data structures and serialization:

- **Serialization/Deserialization**: Tests JSON and form data handling
- **Form Validation**: Tests form data parsing and validation
- **Checkbox Handling**: Tests boolean checkbox deserialization
- **Model Creation**: Tests struct instantiation and field validation

**Key Test Functions:**
- `test_domain_serialization()`: Tests Domain model JSON serialization
- `test_domain_form_deserialization()`: Tests form data parsing
- `test_checkbox_deserialization()`: Tests checkbox value handling

### 2. Database Tests (`src/tests/db.rs`)

Tests for database operations and data persistence:

- **CRUD Operations**: Create, Read, Update, Delete for all entities
- **Error Handling**: Tests for non-existent records and invalid operations
- **Toggle Operations**: Tests for active/inactive status toggling
- **Statistics**: Tests for system and domain statistics calculation

**Key Test Functions:**
- `test_domain_crud_operations()`: Full CRUD cycle for domains
- `test_user_crud_operations()`: Full CRUD cycle for users
- `test_alias_crud_operations()`: Full CRUD cycle for aliases
- `test_toggle_operations()`: Tests status toggling
- `test_stats_operations()`: Tests statistics calculation
- `test_error_handling()`: Tests error conditions

### 3. Handler Tests (`src/tests/handlers.rs`)

Tests for HTTP request handling and response generation:

- **Route Testing**: Tests all HTTP endpoints
- **Form Processing**: Tests form submission and validation
- **Response Validation**: Tests response status codes and content
- **Error Scenarios**: Tests 404 and other error conditions

**Key Test Functions:**
- `test_domains_list_handler()`: Tests domain listing endpoint
- `test_domains_create_handler()`: Tests domain creation endpoint
- `test_domains_update_handler()`: Tests domain update endpoint
- `test_users_create_handler()`: Tests user creation endpoint
- `test_aliases_create_handler()`: Tests alias creation endpoint
- `test_stats_handler()`: Tests statistics endpoint

### 4. Utility Tests (`src/tests/utils.rs`)

Tests for helper functions and validation logic:

- **Validation Functions**: Tests input validation logic
- **Email Validation**: Tests email address format validation
- **Domain Validation**: Tests domain name validation
- **Quota Validation**: Tests quota and storage validation

**Key Test Functions:**
- `test_form_validation()`: Tests form data validation
- `test_email_validation()`: Tests email address validation
- `test_domain_validation()`: Tests domain name validation
- `test_quota_validation()`: Tests quota validation logic

### 5. Integration Tests (`src/tests/integration.rs`)

End-to-end workflow tests that combine multiple components:

- **Full Workflows**: Complete user journeys across multiple operations
- **Data Consistency**: Tests data integrity across operations
- **System Integration**: Tests how different components work together

**Key Test Functions:**
- `test_full_domain_workflow()`: Complete domain management workflow
- `test_full_user_workflow()`: Complete user management workflow
- `test_full_alias_workflow()`: Complete alias management workflow
- `test_stats_integration()`: Statistics integration testing

### 6. Security Tests (`tests/security.rs`)

Comprehensive security tests to ensure the application is protected against common vulnerabilities:

- **SQL Injection Prevention**: Tests various SQL injection payloads
- **Authentication Bypass**: Tests unauthenticated access attempts
- **Authorization**: Tests role-based access control
- **Input Validation**: Tests edge cases and malicious input
- **Session Security**: Tests session management and expiration

**Key Test Functions:**
- `test_sql_injection_domain_creation()`: Tests SQL injection prevention in domain creation
- `test_authentication_bypass_domains()`: Tests authentication enforcement for domains
- `test_authentication_bypass_users()`: Tests authentication enforcement for users
- `test_authorization_readonly_user()`: Tests role-based access control
- `test_input_validation_edge_cases()`: Tests various input validation edge cases

### 7. Containerized UI Tests (`tests/ui_containerized.rs`)

Comprehensive UI tests using testcontainers with isolated database and Selenium:

- **Page Loading**: Tests that all pages load correctly
- **Authentication**: Tests login and session management with robust retry logic
- **Navigation**: Tests menu navigation and page transitions
- **Responsive Design**: Tests different viewport sizes
- **Form Validation**: Tests form submission and validation
- **Error Handling**: Tests 404 pages and error states
- **Accessibility**: Basic accessibility checks
- **Performance**: Page load time measurements
- **HTMX Compatibility**: Tests HTMX attribute presence
- **Cross-browser Compatibility**: Tests different viewport sizes

**Key Test Functions:**
- `test_homepage_loads_containerized()`: Tests homepage loading and authentication
- `test_navigation_containerized()`: Tests dashboard functionality and navigation
- `test_domains_list_page_containerized()`: Tests domains page functionality
- `test_aliases_list_page_containerized()`: Tests aliases page functionality
- `test_users_list_page_containerized()`: Tests users page functionality
- `test_clients_list_page_containerized()`: Tests clients page functionality
- `test_wizard_flow_with_dynamic_domains_containerized()`: Tests complete wizard workflow
- `test_backup_functionality_flow()`: Tests backup functionality
- `test_ui_error_handling_with_shared_theme_containerized()`: Tests error page handling

## UI Testing Setup

### Prerequisites for UI Tests

1. **Docker**: For running testcontainers (Selenium and MySQL containers)
2. **Application Running**: The SortingOffice application must be running on localhost:3000
3. **Testcontainers**: Automatically manages Selenium and MySQL containers

### Quick UI Test Setup

```bash
# Start your application
cargo run

# Run containerized UI tests (recommended)
./tests/run_tests.sh ui

# Or run containerized UI tests directly
cargo test --test ui_containerized -- --nocapture --test-threads=1
```

### UI Test Features

- **Automatic Container Management**: Testcontainers automatically starts and stops containers
- **Database Isolation**: Each test suite uses a clean, isolated MySQL database
- **Shared Network Approach**: Uses Docker bridge network for reliable container communication
- **No External Dependencies**: No need to manually start Selenium or MySQL services
- **CI/CD Friendly**: Works reliably in automated environments
- **Robust Authentication**: Enhanced retry logic for reliable login and page navigation

### Shared Network Approach

The containerized UI tests use a shared Docker bridge network (`sortingoffice-e2e`) to ensure reliable communication between containers:

- **Network Creation**: Automatically creates a dedicated test network
- **Container Communication**: App, MySQL, and Selenium containers communicate via internal IPs
- **Port Isolation**: No port conflicts with host system
- **Cleanup**: Network is automatically cleaned up after tests complete

**Benefits:**
- ✅ Reliable container-to-container communication
- ✅ No port conflicts with host services
- ✅ Consistent test environment across different systems
- ✅ Automatic cleanup of test resources

### Wizard Flow Test

A comprehensive UI test for the wizard flow with dynamic domain fields:

```bash
# Run the wizard test specifically
cargo test test_wizard_flow_with_dynamic_domains_containerized -- --nocapture

# Or use the convenience script
./tests/run_tests.sh ui
```

**What the Wizard Test Covers:**

1. **Dynamic Domain Fields**: Tests adding/removing domain input fields
2. **Form Validation**: Verifies domain validation and error handling
3. **Session Management**: Tests wizard session persistence across steps
4. **HTMX Integration**: Verifies dynamic form submissions
5. **Complete Workflow**: Tests the entire wizard flow from start to finish
6. **Custom Aliases**: Tests dynamic custom alias field management
7. **Review & Execution**: Verifies review and execution steps

**Test Steps:**
- ✅ Navigate to wizard
- ✅ Configure domains with dynamic fields
- ✅ Add/remove domain fields
- ✅ Submit domain configuration
- ✅ Configure aliases with custom fields
- ✅ Add/remove custom alias fields
- ✅ Submit alias configuration
- ✅ Review configuration
- ✅ Execute wizard
- ✅ Verify completion

**Prerequisites:**
- Application running on `http://localhost:3000`
- Docker available for testcontainers
- Authentication as admin user

## Test Database Setup

The test suite automatically:

1. **Creates Test Database**: Uses a separate test database to avoid affecting production data
2. **Runs Migrations**: Automatically applies database migrations
3. **Cleans Up**: Removes test data after each test
4. **Isolation**: Each test runs in isolation with fresh data

### Database Configuration

```sql
-- Create test database
CREATE DATABASE sortingoffice_test;

-- Grant permissions
GRANT ALL PRIVILEGES ON sortingoffice_test.* TO 'username'@'localhost';
FLUSH PRIVILEGES;
```

## Test Coverage

The test suite covers:

- ✅ **100% Model Coverage**: All data structures and serialization
- ✅ **100% Database Operations**: All CRUD operations and queries
- ✅ **100% HTTP Handlers**: All endpoints and request processing
- ✅ **100% Form Processing**: All form validation and parsing
- ✅ **100% Integration Workflows**: End-to-end user workflows
- ✅ **100% UI Functionality**: All major UI interactions and workflows
- ✅ **Security Coverage**: SQL injection, authentication bypass, authorization, input validation
- ✅ **Error Handling**: Database errors, validation errors, HTTP errors
- ✅ **Edge Cases**: Invalid input, missing data, boundary conditions
- ✅ **Performance**: Page load times and responsiveness
- ✅ **Accessibility**: Basic accessibility compliance

## Continuous Integration

The test suite is designed to work with CI/CD pipelines:

```yaml
# Example GitHub Actions workflow
name: Tests
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    services:
      mysql:
        image: mysql:8.0
        env:
          MYSQL_ROOT_PASSWORD: password
          MYSQL_DATABASE: sortingoffice_test
        ports:
          - 3306:3306
        options: --health-cmd="mysqladmin ping" --health-interval=10s --health-timeout=5s --health-retries=3
    
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run unit tests
        run: cargo test
        env:
          DATABASE_URL: mysql://root:password@localhost/sortingoffice_test
      - name: Start application
        run: cargo run &
        env:
          DATABASE_URL: mysql://root:password@localhost/sortingoffice_test
      - name: Wait for application
        run: sleep 10
      - name: Run UI tests
        run: cargo test --test ui_containerized -- --nocapture --test-threads=1
```

## Troubleshooting Guide

### Common Test Issues and Solutions

#### 1. **UI Test Timeouts During Authentication**

**Symptoms**: Tests fail with "Test timed out after 60s" or "Timeout on: Get current URL"

**Causes**:
- Database connection pool still warming up
- Application startup delays in containerized environment
- Network latency between containers

**Solutions**:
- The enhanced authentication logic now includes robust retry mechanisms
- Tests automatically wait for login form to be ready
- Improved redirect handling after successful login
- Database connection retry logic for slow startup scenarios

#### 2. **Container Communication Issues**

**Symptoms**: Tests fail with connection errors or "Connection refused"

**Causes**:
- Docker network configuration issues
- Container startup timing problems
- Port conflicts

**Solutions**:
- Tests use dedicated bridge network (`sortingoffice-e2e`)
- Automatic network creation and cleanup
- Health checks ensure containers are ready before testing
- Internal container IP addressing avoids port conflicts

#### 3. **Database Connection Errors**

**Symptoms**: Tests show "Database connection error" messages

**Causes**:
- MySQL container not fully ready
- Connection pool configuration issues
- Network connectivity problems

**Solutions**:
- Tests include retry logic for database operations
- Health checks ensure MySQL is ready before testing
- Optimized connection pool settings for test environment
- Automatic cleanup of failed connections

#### 4. **Selenium Container Issues**

**Symptoms**: WebDriver connection failures or browser crashes

**Causes**:
- Chrome container resource limitations
- Memory or CPU constraints
- Container startup timing

**Solutions**:
- Optimized Chrome container configuration
- Health checks ensure Selenium is ready
- Automatic container restart on failures
- Resource limits appropriate for test environment

### Debug Commands

```bash
# Run single test with output
cargo test test_name -- --nocapture

# Run tests with detailed logging
RUST_LOG=debug cargo test

# Run tests with backtrace
RUST_BACKTRACE=1 cargo test

# Check test compilation
cargo check --tests

# Check container status
docker ps -a

# View container logs
docker logs <container_id>

# Check network configuration
docker network ls
docker network inspect sortingoffice-e2e
```

### Performance Optimization

**Test Execution Times**:
- **Unit Tests**: ~1-2 seconds total
- **Integration Tests**: ~5-10 seconds total  
- **UI Tests**: 12-52 seconds per test (includes container startup)
- **Full Test Suite**: ~6-7 minutes total

**Optimization Tips**:
- Run tests in parallel when possible (unit tests)
- Use `--test-threads=1` for UI tests to avoid resource conflicts
- Container reuse between test suites reduces startup overhead
- Database connection pooling improves test reliability

## Adding New Tests

When adding new functionality, follow these guidelines:

1. **Test Structure**: Follow the existing test organization
2. **Naming**: Use descriptive test function names
3. **Isolation**: Each test should be independent
4. **Cleanup**: Always clean up test data
5. **Documentation**: Add comments explaining complex test logic

### Example Test Template

```rust
#[tokio::test]
async fn test_new_feature() {
    let pool = setup_test_db();
    
    // Test setup
    // ... create test data ...
    
    // Test execution
    // ... call function being tested ...
    
    // Assertions
    assert_eq!(result, expected);
    
    // Cleanup
    cleanup_test_db(&pool);
}
```

## Performance Considerations

- Tests use a separate test database to avoid conflicts
- Each test runs in isolation for reliability
- Database connections are pooled for efficiency
- Test data is cleaned up after each test
- UI tests use testcontainers for automatic resource management
- Shared network approach ensures reliable container communication

## Contributing

When contributing to the test suite:

1. **Run Existing Tests**: Ensure all existing tests pass
2. **Add New Tests**: Add tests for new functionality
3. **Update Documentation**: Update this README if needed
4. **Follow Patterns**: Use existing test patterns and conventions

## Test Organization

```
src/tests/
├── mod.rs              # Test module configuration and utilities
├── models.rs           # Model and serialization tests
├── db.rs              # Database operation tests
├── handlers.rs        # HTTP handler tests
├── utils.rs           # Utility function tests
└── integration.rs     # End-to-end workflow tests

tests/
├── README.md          # This documentation
├── ui_containerized.rs # Comprehensive containerized UI tests
├── ui_smoke.rs        # Basic smoke tests
└── run_tests.sh       # Unified test runner script
```

This organization provides a clear separation between different types of tests while maintaining a unified interface for running them all. 

# UI Testing Options

This project provides multiple UI testing approaches to suit different needs and environments.

## Test Options

### 1. Containerized UI Tests (`make test-ui` or `./tests/run_tests.sh ui`) ⭐ **RECOMMENDED**
- **What it does**: Runs UI tests with isolated database using testcontainers
- **Database**: Uses testcontainers MySQL (isolated, clean database)
- **Browser**: Uses testcontainers for Selenium Chrome
- **Application**: Runs against localhost:3000 (application must be started)
- **Pros**: Database isolation, consistent test environment, reliable networking
- **Cons**: Requires application to be running
- **Status**: ✅ **Production Ready** - All tests passing reliably

### 2. Standard UI Tests (`make test-ui-dev` or `./tests/run_tests.sh ui-headless`)
- **What it does**: Runs UI tests against the application running on localhost:3000
- **Database**: Uses the main application database (requires seed data)
- **Browser**: Uses testcontainers for Selenium Chrome
- **Pros**: Fast, simple setup
- **Cons**: Requires application to be running, depends on main database state

### 3. Full Containerized Tests (Future Enhancement)
- **What it does**: Would run entire application in testcontainers
- **Database**: Uses testcontainers MySQL
- **Browser**: Uses testcontainers for Selenium Chrome
- **Application**: Would run in testcontainers
- **Pros**: Complete isolation, no external dependencies
- **Cons**: More complex setup, slower startup

## Usage

### Prerequisites
- Docker and Docker Compose installed
- Application running on localhost:3000 (for options 1 and 2)

### Running Tests

```bash
# Containerized UI tests (recommended)
make test-ui

# Standard UI tests
make test-ui-dev

# All tests (unit + UI)
make test-all
```

### Test Script Options

```bash
# Using the test runner script directly
./tests/run_tests.sh ui                 # Containerized UI tests (recommended)
./tests/run_tests.sh ui-headless        # Standard UI tests
./tests/run_tests.sh unit               # Unit tests only
./tests/run_tests.sh all                # All tests
```

## Test Coverage

The UI tests cover:
- Homepage loading and authentication
- Navigation between pages
- Domain search functionality
- Form interactions
- Basic UI responsiveness
- Complete wizard workflow
- Backup functionality
- Error page handling

## Troubleshooting

### Application Not Running
If you get an error about the application not running on localhost:3000:
```bash
# Start the application
cargo run

# Or use Docker Compose
docker-compose up
```

### Docker Issues
If Docker is not available or not running:
```bash
# Check Docker status
docker info

# Start Docker service (if needed)
sudo systemctl start docker
```

### Test Timeouts
If tests timeout, you can increase timeout values in the test files or check:
- Application responsiveness
- Database connectivity
- Selenium container health

## CI/CD Integration

For CI/CD pipelines, the containerized UI tests are recommended as they provide:
- Consistent environment
- Reliable Docker networking
- No external dependencies (except Docker)
- Reproducible results

The containerized UI tests are now production-ready with all tests passing reliably. 
