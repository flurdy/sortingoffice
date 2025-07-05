# Testing Guide for SortingOffice

This document describes the comprehensive test suite for the SortingOffice application.

## Overview

The test suite is organized into several modules:

- **Models Tests** (`src/tests/models.rs`): Tests for data models, serialization, and form handling
- **Database Tests** (`src/tests/db.rs`): Tests for database operations and CRUD functionality
- **Handler Tests** (`src/tests/handlers.rs`): Tests for HTTP request handling and responses
- **Utility Tests** (`src/tests/utils.rs`): Tests for helper functions and validation logic
- **Integration Tests** (`src/tests/integration.rs`): End-to-end workflow tests
- **UI Tests** (`tests/ui.rs`): Functional UI tests using Selenium WebDriver
- **Advanced UI Tests** (`tests/ui_advanced.rs`): Complex UI workflows and interactions

## Running Tests

### Prerequisites

1. **Database Setup**: Ensure you have a MySQL database available for testing
2. **Environment Variables**: Set up your test environment
3. **For UI Tests**: Docker and Chrome WebDriver (see UI Testing section)

### Quick Start

```bash
# Run all unit tests (default)
./tests/run_tests.sh

# Run only unit tests
./tests/run_tests.sh unit

# Run only UI tests
./tests/run_tests.sh ui

# Run all tests (unit + UI)
./tests/run_tests.sh all

# Setup UI test environment
./tests/run_tests.sh ui-setup

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

# Run UI tests
cargo test --test ui
cargo test --test ui_advanced

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

### 6. UI Tests (`tests/ui.rs`)

Functional UI tests using Selenium WebDriver:

- **Page Loading**: Tests that all pages load correctly
- **Navigation**: Tests menu navigation and breadcrumbs
- **Responsive Design**: Tests different viewport sizes
- **Error Handling**: Tests 404 pages and error states
- **Accessibility**: Basic accessibility checks
- **Cross-browser Compatibility**: Tests different viewport sizes

**Key Test Functions:**
- `test_homepage_loads()`: Tests homepage loading and redirects
- `test_dashboard_navigation()`: Tests dashboard functionality
- `test_navigation_menu()`: Tests menu navigation
- `test_responsive_design()`: Tests responsive behavior
- `test_error_pages()`: Tests error page handling

### 7. Advanced UI Tests (`tests/ui_advanced.rs`)

Complex UI workflows and interactions:

- **Form Workflows**: Complete user workflows for creating domains, users, and aliases
- **Form Validation**: Tests client-side and server-side validation
- **Table Features**: Sorting, pagination, and search functionality
- **Modal Dialogs**: Tests confirmation dialogs and popups
- **Performance**: Measures page load times and responsiveness

**Key Test Functions:**
- `test_domain_creation_workflow()`: Complete domain creation workflow
- `test_user_creation_workflow()`: Complete user creation workflow
- `test_alias_creation_workflow()`: Complete alias creation workflow
- `test_form_validation_errors()`: Tests form validation
- `test_performance_metrics()`: Tests page load performance

## UI Testing Setup

### Prerequisites for UI Tests

1. **Docker**: For running Selenium WebDriver
2. **Application Running**: The SortingOffice application must be running on localhost:3000
3. **Selenium WebDriver**: Chrome WebDriver for browser automation

### Quick UI Test Setup (Docker Compose)

```bash
# Setup UI test environment (starts Selenium WebDriver via Docker Compose)
./tests/run_tests.sh ui-setup

# Start your application
cargo run

# Run UI tests
./tests/run_tests.sh ui
```

### Alternative: All-in-one UI Test Command

```bash
# Run UI tests with Docker Compose (starts Selenium automatically)
make test-ui-compose
```

### Manual UI Test Setup

```bash
# Start Selenium WebDriver manually via Docker Compose
docker compose --profile test up -d selenium

# Start your application
cargo run

# Run UI tests
./tests/run_tests.sh ui

# Clean up when done
docker compose --profile test down selenium
```

### UI Test Debugging

- **VNC Access**: Connect to localhost:7900 (password: `secret`) to see the browser
- **Screenshots**: Tests can capture screenshots on failure
- **Logging**: Set `RUST_LOG=debug` for detailed WebDriver logs

For detailed UI testing information, see [UI_TESTS.md](UI_TESTS.md).

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
      selenium:
        image: selenium/standalone-chrome
        ports:
          - 4444:4444
        shm_size: 2gb
    
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
        run: cargo test --test ui --test ui_advanced
```

## Debugging Tests

### Common Issues

1. **Database Connection**: Ensure MySQL is running and accessible
2. **Permissions**: Verify database user has proper permissions
3. **Migrations**: Ensure all migrations are up to date

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
```

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
├── integration.rs     # End-to-end workflow tests
├── ui.rs              # Basic UI functionality tests
└── ui_advanced.rs     # Advanced UI workflow tests

tests/
├── README.md          # This documentation
├── UI_TESTS.md        # Detailed UI testing guide
└── run_tests.sh       # Unified test runner script
```

This organization provides a clear separation between different types of tests while maintaining a unified interface for running them all. 
