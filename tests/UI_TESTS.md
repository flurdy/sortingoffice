# UI Tests for SortingOffice

This document describes the functional UI tests for the SortingOffice application, which test the web interface using Selenium WebDriver.

## Overview

The UI tests are designed to verify that the web interface works correctly from a user's perspective. They test:

- Page loading and navigation
- Form interactions and validation
- User workflows (create, read, update, delete operations)
- Responsive design
- Accessibility features
- Performance metrics

## Test Structure

### Basic UI Tests (`src/tests/ui.rs`)

These tests cover fundamental UI functionality:

- **Page Loading**: Verifies that all pages load correctly
- **Navigation**: Tests menu navigation and breadcrumbs
- **Responsive Design**: Checks that the interface works on different screen sizes
- **Error Handling**: Tests 404 pages and error states
- **Accessibility**: Basic accessibility checks
- **Cross-browser Compatibility**: Tests different viewport sizes

### Advanced UI Tests (`src/tests/ui_advanced.rs`)

These tests cover more complex user interactions:

- **Form Workflows**: Complete user workflows for creating domains, users, and aliases
- **Form Validation**: Tests client-side and server-side validation
- **Table Features**: Sorting, pagination, and search functionality
- **Modal Dialogs**: Tests confirmation dialogs and popups
- **Keyboard Navigation**: Tests accessibility and keyboard shortcuts
- **Performance**: Measures page load times and responsiveness

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
fantoccini = "0.19"  # WebDriver client
tokio = { version = "1", features = ["full"] }
anyhow = "1.0"
```

### 2. Environment Configuration

The tests expect the following environment variables:

- `DATABASE_URL`: MySQL connection string for test database
- `RUST_TEST_THREADS`: Set to 1 for sequential test execution
- `RUST_LOG`: Logging level (set to "debug" for detailed output)

### 3. Test Environment

The tests use a Docker Compose setup (`docker-compose.test.yml`) that includes:

- **Selenium Standalone Chrome**: WebDriver server
- **MySQL Database**: Test database
- **Application Container**: Running SortingOffice for testing

## Running the Tests

### Quick Start

Use the provided script to run all UI tests:

```bash
./run_ui_tests.sh
```

This script will:
1. Start the test environment (Selenium, database, application)
2. Wait for all services to be ready
3. Run the UI tests
4. Clean up the environment

### Manual Setup

If you prefer to run tests manually:

1. **Start the test environment**:
   ```bash
   docker-compose -f docker-compose.test.yml up -d selenium test-db
   ```

2. **Wait for services to be ready**:
   ```bash
   # Wait for Selenium
   until curl -s http://localhost:4444/status > /dev/null; do sleep 2; done
   
   # Wait for database
   until docker-compose -f docker-compose.test.yml exec -T test-db mysqladmin ping -h localhost -u root -ppassword > /dev/null; do sleep 2; done
   ```

3. **Start the application**:
   ```bash
   docker-compose -f docker-compose.test.yml up -d app
   ```

4. **Run the tests**:
   ```bash
   export DATABASE_URL="mysql://root:password@localhost:3307/sortingoffice_test"
   export RUST_TEST_THREADS=1
   cargo test --test ui -- --nocapture
   cargo test --test ui_advanced -- --nocapture
   ```

### Running Specific Tests

Run only basic UI tests:
```bash
cargo test --test ui
```

Run only advanced UI tests:
```bash
cargo test --test ui_advanced
```

Run a specific test:
```bash
cargo test --test ui test_homepage_loads
```

## Test Configuration

### WebDriver Configuration

The tests use Chrome in headless mode with the following configuration:

```rust
let mut chrome_opts = serde_json::map::Map::new();
chrome_opts.insert("args".to_string(), serde_json::Value::Array(vec![
    serde_json::Value::String("--headless".to_string()),
    serde_json::Value::String("--no-sandbox".to_string()),
    serde_json::Value::String("--disable-dev-shm-usage".to_string()),
    serde_json::Value::String("--disable-gpu".to_string()),
    serde_json::Value::String("--window-size=1920,1080".to_string()),
]));
```

### Timeouts

- **Element wait timeout**: 5 seconds
- **Page load timeout**: 10 seconds
- **Test execution timeout**: 30 seconds per test

### Test Data

Tests generate unique test data using timestamps to avoid conflicts:

```rust
let domain_name = format!("test-domain-{}", chrono::Utc::now().timestamp());
let username = format!("testuser{}", chrono::Utc::now().timestamp());
```

## Test Utilities

### Helper Functions

The tests include several utility functions for common operations:

- `setup_client()`: Creates and configures WebDriver client
- `wait_for_element()`: Waits for an element to appear
- `wait_for_text()`: Waits for specific text to appear
- `fill_form_field()`: Fills form fields
- `click_element()`: Clicks elements
- `get_element_text()`: Gets text from elements

### Selector Strategies

Tests use multiple selector strategies to handle different HTML structures:

```rust
let selectors = [
    "input[name='domain']",
    "input[placeholder*='domain']",
    "#domain",
    "input[type='text']"
];
```

This approach makes tests more robust and less brittle.

## Debugging

### Visual Debugging

To see the browser during test execution, modify the Chrome options:

```rust
// Remove --headless to see the browser
chrome_opts.insert("args".to_string(), serde_json::Value::Array(vec![
    serde_json::Value::String("--no-sandbox".to_string()),
    serde_json::Value::String("--disable-dev-shm-usage".to_string()),
]));
```

### VNC Access

The Selenium container includes VNC for remote debugging:

```bash
# Access VNC viewer at localhost:7900
# Password: secret
```

### Logging

Enable detailed logging:

```bash
export RUST_LOG=debug
cargo test --test ui -- --nocapture
```

### Screenshots

Add screenshot capture to tests:

```rust
use std::fs;

// Take screenshot on failure
let screenshot = client.screenshot().await?;
fs::write("test_failure.png", screenshot).await?;
```

## Common Issues

### Connection Errors

**Problem**: "Failed to connect to WebDriver"
**Solution**: Ensure Selenium container is running and accessible

```bash
docker-compose -f docker-compose.test.yml ps
curl http://localhost:4444/status
```

### Timeout Errors

**Problem**: "Element not found within X seconds"
**Solution**: 
- Check if the application is running
- Verify the page is loading correctly
- Increase timeout values if needed
- Check for JavaScript errors

### Database Connection Issues

**Problem**: "Too many connections"
**Solution**: 
- Ensure database is running
- Check connection pool settings
- Clean up test data between tests

### Chrome Crashes

**Problem**: Chrome crashes during tests
**Solution**:
- Increase Docker memory allocation
- Add `--disable-dev-shm-usage` flag
- Use `--no-sandbox` flag

## Best Practices

### Test Design

1. **Isolation**: Each test should be independent
2. **Cleanup**: Clean up test data after tests
3. **Robustness**: Use multiple selector strategies
4. **Readability**: Use descriptive test names and comments

### Performance

1. **Parallelization**: Run tests sequentially to avoid conflicts
2. **Resource Management**: Close WebDriver clients properly
3. **Timeouts**: Use appropriate timeout values
4. **Caching**: Cache test data when possible

### Maintenance

1. **Selectors**: Use stable, semantic selectors
2. **Data**: Use unique, timestamped test data
3. **Documentation**: Keep test documentation updated
4. **Monitoring**: Monitor test execution times

## Integration with CI/CD

### GitHub Actions Example

```yaml
name: UI Tests
on: [push, pull_request]

jobs:
  ui-tests:
    runs-on: ubuntu-latest
    services:
      selenium:
        image: selenium/standalone-chrome
        ports:
          - 4444:4444
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
      - run: cargo test --test ui
      - run: cargo test --test ui_advanced
```

### Docker Compose for CI

```yaml
version: '3.8'
services:
  selenium:
    image: selenium/standalone-chrome
    ports:
      - "4444:4444"
    environment:
      - SE_NODE_MAX_SESSIONS=4
    shm_size: 2gb
  
  test-db:
    image: mysql:8.0
    environment:
      MYSQL_ROOT_PASSWORD: password
      MYSQL_DATABASE: sortingoffice_test
    ports:
      - "3306:3306"
```

## Future Enhancements

### Planned Features

1. **Visual Regression Testing**: Compare screenshots across versions
2. **Accessibility Testing**: Automated accessibility compliance checks
3. **Performance Testing**: Load testing and performance benchmarks
4. **Mobile Testing**: Test mobile-specific functionality
5. **Cross-browser Testing**: Test in multiple browsers

### Potential Improvements

1. **Test Data Management**: Better test data generation and cleanup
2. **Parallel Execution**: Run tests in parallel where possible
3. **Reporting**: Enhanced test reporting and analytics
4. **Monitoring**: Real-time test monitoring and alerting

## Contributing

When adding new UI tests:

1. Follow the existing test structure and naming conventions
2. Use the provided utility functions
3. Add appropriate documentation
4. Ensure tests are robust and handle edge cases
5. Update this documentation if needed

## Support

For issues with UI tests:

1. Check the troubleshooting section above
2. Review the test logs and error messages
3. Verify the test environment is properly configured
4. Check for recent changes that might affect the tests
5. Create an issue with detailed error information 
