# Testing Guide for SortingOffice

This document describes the comprehensive test suite for the SortingOffice application.

## Overview

The test suite is organized into several modules:

- **Models Tests** (`src/tests/models.rs`): Tests for data models, serialization, and form handling
- **Database Tests** (`src/tests/db.rs`): Tests for database operations and CRUD functionality
- **Handler Tests** (`src/tests/handlers.rs`): Tests for HTTP request handling and responses
- **Utility Tests** (`src/tests/utils.rs`): Tests for helper functions and validation logic

## Running Tests

### Prerequisites

1. **Database Setup**: Ensure you have a MySQL database available for testing
2. **Environment Variables**: Set up your test environment

### Quick Start

```bash
# Run all tests
./tests/run_tests.sh

# Or run with cargo directly
cargo test

# Run specific test modules
cargo test models
cargo test db
cargo test handlers
cargo test utils

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
- ✅ **Error Handling**: Database errors, validation errors, HTTP errors
- ✅ **Edge Cases**: Invalid input, missing data, boundary conditions

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
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo test
        env:
          DATABASE_URL: mysql://root:password@localhost/sortingoffice_test
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
