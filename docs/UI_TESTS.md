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

## Test Structure

### Basic UI Tests (`tests/ui.rs`)

These tests cover fundamental UI functionality with **enhanced Selenium readiness checks**:

- **Selenium Readiness**: Automatic verification that Selenium WebDriver and Chrome browser are ready
- **Page Loading**: Verifies that all pages load correctly
- **Navigation**: Tests menu navigation and breadcrumbs
- **Responsive Design**: Checks that the interface works on different screen sizes
- **Error Handling**: Tests 404 pages and error states
- **Accessibility**: Basic accessibility checks
- **Cross-browser Compatibility**: Tests different viewport sizes

### Advanced UI Tests (`tests/ui_advanced.rs`)

These tests cover more complex user interactions with **enhanced Selenium readiness checks**:

- **Form Workflows**: Complete user workflows for creating domains, users, and aliases
- **Form Validation**: Tests client-side and server-side validation
- **Table Features**: Sorting, pagination, and search functionality
- **Modal Dialogs**: Tests confirmation dialogs and popups
- **Keyboard Navigation**: Tests accessibility and keyboard shortcuts
- **Performance**: Measures page load times and responsiveness

### Headless UI Tests (`tests/ui_headless.rs`) - **NEW**

These tests use the **thirtyfour** WebDriver client for improved headless browser testing:

- **Modern WebDriver Client**: Uses thirtyfour instead of fantoccini for better stability
- **Enhanced Headless Mode**: Optimized Chrome configuration for headless operation
- **Improved Error Handling**: Better timeout and retry mechanisms
- **Same Test Coverage**: All the same functionality as regular UI tests
- **Faster Execution**: Optimized for CI/CD environments

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
- **MySQL Database**: Test database
- **Application Container**: Running SortingOffice for testing

## Running the Tests

### Quick Start

Use the provided script to run all UI tests:

```bash
./tests/run_tests.sh
```

This script will:
1. Start the test environment (Selenium, database, application)
2. Wait for all services to be ready
3. Run the UI tests
4. Clean up the environment

### Test Options

```bash
# Run only basic UI tests (with Selenium readiness checks)
./tests/run_tests.sh ui

# Run only headless UI tests (recommended for CI/CD)
./tests/run_tests.sh ui-headless

# Run all UI tests (basic + advanced + headless)
./tests/run_tests.sh ui

# Setup UI test environment
./tests/run_tests.sh ui-setup
```

### Manual Setup

If you prefer to run tests manually:

1. **Start the test environment**:
   ```bash
   docker compose --profile test up -d selenium
   ```

2. **Wait for services to be ready**:
   ```bash
   # Wait for Selenium
   until curl -s http://localhost:4444/status > /dev/null; do sleep 2; done
   ```

3. **Start the application**:
   ```bash
   cargo run
   ```

4. **Run the tests**:
   ```bash
   # Basic UI tests with readiness checks
   cargo test --test ui -- --nocapture
   
   # Advanced UI tests with readiness checks
   cargo test --test ui_advanced -- --nocapture
   
   # Headless UI tests (recommended)
   cargo test --test ui_headless -- --nocapture
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

Run only headless UI tests:
```bash
cargo test --test ui_headless
```

Run a specific test:
```bash
cargo test --test ui test_homepage_loads
cargo test --test ui_headless test_homepage_loads_headless
```

## Test Configuration

### Selenium Readiness Checks

**NEW**: All UI tests now include automatic Selenium readiness verification:

```rust
async fn check_selenium_readiness() -> Result<()> {
    // Check if Selenium service is responding
    let client = reqwest::Client::new();
    let status_url = "http://localhost:4444/status";
    
    match client.get(status_url).timeout(Duration::from_secs(10)).send().await {
        Ok(response) => {
            if response.status().is_success() {
                println!("✅ Selenium WebDriver service is responding");
            } else {
                return Err(anyhow::anyhow!("Selenium WebDriver service returned status: {}", response.status()));
            }
        }
        Err(e) => {
            return Err(anyhow::anyhow!("Failed to connect to Selenium WebDriver: {}", e));
        }
    }
    
    // Try to create a test session to verify Chrome is ready
    match ClientBuilder::native()
        .connect("http://localhost:4444")
        .await
    {
        Ok(client) => {
            println!("✅ Chrome browser is ready");
            client.close().await?;
            Ok(())
        }
        Err(e) => {
            Err(anyhow::anyhow!("Chrome browser is not ready: {}", e))
        }
    }
}
```

### Headless Browser Configuration

The headless tests use optimized Chrome configuration:

```rust
let mut caps = DesiredCapabilities::chrome();
caps.add_arg("--headless=new")?;
caps.add_arg("--no-sandbox")?;
caps.add_arg("--disable-dev-shm-usage")?;
caps.add_arg("--disable-gpu")?;
caps.add_arg("--window-size=1920,1080")?;
caps.add_arg("--disable-web-security")?;
caps.add_arg("--allow-running-insecure-content")?;
```

### Timeouts

- **Element wait timeout**: 5 seconds
- **Page load timeout**: 10 seconds
- **Test execution timeout**: 30 seconds per test
- **Selenium readiness timeout**: 10 seconds

### Test Data

Tests generate unique test data using timestamps to avoid conflicts:

```rust
let domain_name = format!("test-domain-{}", chrono::Utc::now().timestamp());
let username = format!("testuser{}", chrono::Utc::now().timestamp());
```

## Test Utilities

### Helper Functions

The tests include several utility functions for common operations:

- `setup_client()` / `setup_driver()`: Creates and configures WebDriver client
- `check_selenium_readiness()`: Verifies Selenium and Chrome are ready
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
caps.add_arg("--no-sandbox")?;
caps.add_arg("--disable-dev-shm-usage")?;
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
let screenshot = driver.screenshot().await?;
fs::write("test_failure.png", screenshot).await?;
```

## Common Issues

### Connection Errors

**Problem**: "Failed to connect to WebDriver"
**Solution**: The new readiness checks will automatically detect and report this issue

```bash
# Check if Selenium is running
curl http://localhost:4444/status

# Restart Selenium if needed
docker compose --profile test restart selenium
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
- Try the headless tests which have optimized Chrome configuration

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
      - run: cargo test --test ui_headless  # Use headless tests for CI
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

1. **Choose the right test file**:
   - `tests/ui.rs` for basic functionality
   - `tests/ui_advanced.rs` for complex workflows
   - `tests/ui_headless.rs` for headless testing

2. **Include readiness checks**: All tests should verify Selenium is ready

3. **Use proper timeouts**: Include appropriate timeout handling

4. **Add documentation**: Document any new test utilities or patterns

5. **Test locally first**: Always test locally before committing

## Migration Guide

### From Old UI Tests

If you have existing UI tests, they should continue to work with the new readiness checks. The main changes are:

1. **Automatic readiness verification**: Tests now check Selenium status automatically
2. **Better error messages**: More descriptive error messages for connection issues
3. **New headless option**: Consider using `tests/ui_headless.rs` for new tests

### To Headless Tests

To migrate from Selenium tests to headless tests:

1. **Copy test logic**: Copy test functions from `ui.rs` to `ui_headless.rs`
2. **Update API calls**: Replace `find_element` with `find`, `find_elements` with `find_all`
3. **Update window sizing**: Replace `set_window_size` with `set_window_rect`
4. **Test thoroughly**: Verify all functionality works in headless mode

## Troubleshooting

### Selenium Not Starting

```bash
# Check Docker logs
docker compose --profile test logs selenium

# Restart Selenium
docker compose --profile test restart selenium

# Check system resources
docker stats
```

### Tests Hanging

```bash
# Check if Selenium is responsive
curl http://localhost:4444/status

# Check application logs
cargo run -- --log-level debug

# Use headless tests which are more stable
cargo test --test ui_headless
```

### Memory Issues

```bash
# Increase Docker memory allocation
# In Docker Desktop: Settings -> Resources -> Memory -> 4GB

# Or use headless tests which use less memory
cargo test --test ui_headless
``` 
