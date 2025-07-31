# Smoke Tests for SortingOffice

This document provides detailed information about the smoke tests in SortingOffice, which are comprehensive end-to-end tests that validate complete user workflows.

## Overview

Smoke tests are the highest-level tests in the SortingOffice test suite. They validate complete user workflows from authentication through resource creation and cleanup, ensuring that the entire application works correctly from a user's perspective.

## What Smoke Tests Validate

### Complete User Workflow
1. **Authentication**: Login with test credentials
2. **Domain Creation**: Create a new test domain with random data
3. **Alias Creation**: Create two aliases pointing to the test user
4. **User Creation**: Create a test user for the domain
5. **Reports Verification**: Check that the reports page loads correctly
6. **Cleanup**: Delete all created resources (user, aliases, domain)

### Key Validations
- **Database Operations**: All CRUD operations work correctly
- **UI Interactions**: Forms, navigation, and dynamic updates function properly
- **Data Integrity**: Created resources are properly stored and retrieved
- **Error Handling**: Application handles errors gracefully
- **Resource Cleanup**: Test isolation is maintained
- **Performance**: Operations complete within reasonable timeframes

## Test Implementation

### File Location
- **Main Test File**: `tests/ui_smoke.rs`
- **CI Workflow**: `.github/workflows/smoke-tests.yml`

### Test Functions

#### `ui_smoke_e2e_flow()`
Standard smoke test with default configuration:
```rust
#[tokio::test]
#[ignore]
async fn ui_smoke_e2e_flow() -> Result<()> {
    let config = SmokeTestConfig::default();
    run_smoke_test_with_config(config).await
}
```

#### `ui_smoke_e2e_flow_testcontainers()`
Smoke test with testcontainers support for CI environments:
```rust
#[tokio::test]
#[ignore]
async fn ui_smoke_e2e_flow_testcontainers() -> Result<()> {
    run_smoke_test_with_testcontainers().await
}
```

#### `run_smoke_test_with_config()`
Parameterized smoke test execution with flexible configuration:
```rust
pub async fn run_smoke_test_with_config(config: SmokeTestConfig) -> Result<()>
```

#### `run_smoke_test_with_testcontainers()`
CI-ready smoke test with database isolation:
```rust
pub async fn run_smoke_test_with_testcontainers() -> Result<()>
```

## Configuration

### SmokeTestConfig Structure

```rust
pub struct SmokeTestConfig {
    pub app_url: String,           // Application URL to test
    pub headless: bool,            // Headless mode for CI
    pub timeout_seconds: u64,      // Total timeout in seconds
    pub enable_vnc: bool,          // VNC debugging support
}
```

### Environment Variables

The smoke tests support configuration via environment variables:

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

### Default Configuration

```rust
impl Default for SmokeTestConfig {
    fn default() -> Self {
        Self {
            app_url: std::env::var("SMOKE_TEST_APP_URL")
                .unwrap_or_else(|_| "http://host.docker.internal:3000".to_string()),
            headless: std::env::var("SMOKE_TEST_HEADLESS")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            timeout_seconds: std::env::var("SMOKE_TEST_TIMEOUT")
                .unwrap_or_else(|_| "300".to_string())
                .parse()
                .unwrap_or(300),
            enable_vnc: std::env::var("SMOKE_TEST_VNC")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
        }
    }
}
```

## Running Smoke Tests

### Local Execution

```bash
# Run with default configuration
cargo test ui_smoke_e2e_flow -- --ignored --nocapture

# Run with testcontainers support
cargo test ui_smoke_e2e_flow_testcontainers -- --ignored --nocapture

# Run with custom configuration
SMOKE_TEST_APP_URL=http://localhost:3000 \
SMOKE_TEST_HEADLESS=false \
SMOKE_TEST_TIMEOUT=300 \
cargo test ui_smoke_e2e_flow -- --ignored --nocapture
```

### Makefile Commands

```bash
# Run smoke tests
make test-smoke

# Run all tests including smoke tests
make test-all
```

### CI/CD Execution

The smoke tests are automatically executed in CI/CD via GitHub Actions:

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

## Test Data Generation

### Random Data Strategy

The smoke test uses random data generation to avoid conflicts:

```rust
fn rand_str() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::rngs::ThreadRng::default();
    (0..8)
        .map(|_| {
            let idx = (rng.next_u32() as usize) % CHARSET.len();
            CHARSET[idx] as char
        })
        .collect()
}
```

### Generated Test Data

- **Domain**: `{random}.test.com` (e.g., `abc123.test.com`)
- **Aliases**: 
  - `alias1-{random}@{domain}` (e.g., `alias1-xyz789@abc123.test.com`)
  - `alias2-{random}@{domain}` (e.g., `alias2-def456@abc123.test.com`)
- **User**: `user-{random}@{domain}` (e.g., `user-ghi012@abc123.test.com`)
- **Maildir**: `{domain}/user-{random}/` (e.g., `abc123.test.com/user-ghi012/`)

## Error Handling

### Timeout Management

The smoke test implements comprehensive timeout handling:

- **Total Timeout**: 5 minutes (configurable via `SMOKE_TEST_TIMEOUT`)
- **Step Timeouts**: 60 seconds for major operations, 30 seconds for minor operations
- **WebDriver Timeout**: 30 seconds for WebDriver creation
- **Cleanup Timeout**: 10 seconds for WebDriver shutdown

### Resource Cleanup

The smoke test ensures proper resource cleanup:

```rust
// Always try to quit the driver
println!("[SMOKE TEST] Cleaning up WebDriver...");
match timeout(Duration::from_secs(10), driver.quit()).await {
    Ok(Ok(_)) => {
        println!("[SMOKE TEST] WebDriver quit successfully");
    }
    _ => {
        // Force cleanup if normal quit fails
        // ... cleanup code ...
    }
}
```

### Error Reporting

The smoke test provides detailed error reporting:

- **Step-by-step logging**: Each operation is logged with timestamps
- **Error context**: Detailed error messages with context
- **Artifact collection**: Screenshots and logs for debugging
- **Graceful degradation**: Continues cleanup even on failures

## Debugging

### Local Debugging

For local debugging, run the smoke test in non-headless mode:

```bash
SMOKE_TEST_HEADLESS=false cargo test ui_smoke_e2e_flow -- --ignored --nocapture
```

This will open a visible browser window where you can observe the test execution.

### CI Debugging

The CI workflow collects artifacts for debugging:

- **Application Logs**: `app.log` contains application output
- **Test Results**: Detailed test output with timestamps
- **Screenshots**: Available if VNC is enabled

### Common Issues

1. **Selenium Connection**: Ensure Selenium container is running
   ```bash
   docker compose --profile test up -d selenium
   ```

2. **Application Startup**: Check that the app is accessible
   ```bash
   curl -f http://localhost:3000/health
   ```

3. **Database Connection**: Verify database connectivity for testcontainers
   ```bash
   docker ps | grep mysql
   ```

4. **Timeout Issues**: Adjust timeout for slower environments
   ```bash
   SMOKE_TEST_TIMEOUT=600 cargo test ui_smoke_e2e_flow -- --ignored --nocapture
   ```

5. **Resource Cleanup**: Check for orphaned containers
   ```bash
   docker ps -a | grep selenium
   docker system prune -f
   ```

## Performance Considerations

### Test Duration
- **Typical Duration**: 3-5 minutes
- **CI Duration**: 4-6 minutes (due to container startup)
- **Local Duration**: 2-4 minutes (faster hardware)

### Resource Usage
- **Memory**: ~2GB RAM (Selenium + Chrome + Application)
- **CPU**: High CPU usage during test execution
- **Disk**: Minimal disk usage (temporary files only)

### Optimization Tips
- **Sequential Execution**: Run tests sequentially to avoid conflicts
- **Headless Mode**: Use headless mode for CI environments
- **Timeout Tuning**: Adjust timeouts based on environment performance
- **Resource Monitoring**: Monitor system resources during test execution

## Integration with CI/CD

### GitHub Actions Workflow

The smoke tests are integrated into the CI/CD pipeline via `.github/workflows/smoke-tests.yml`:

```yaml
name: Smoke Tests
on:
  push:
    branches: [ master ]
  pull_request:
    branches: [ master ]
  workflow_dispatch: # Allow manual triggering

jobs:
  smoke-test:
    name: Smoke Tests (End-to-End)
    runs-on: ubuntu-latest
    
    steps:
      - name: Checkout code
        uses: actions/checkout@v4
      
      - name: Start Selenium container
        run: docker compose --profile test up -d selenium
      
      - name: Build application
        run: cargo build --release
      
      - name: Start application
        run: cargo run --release &
      
      - name: Run smoke test
        run: cargo test ui_smoke_e2e_flow_testcontainers -- --ignored --nocapture
```

### CI Features
- **Automated Execution**: Runs on push, pull requests, and manual triggers
- **Selenium Container Setup**: Automatic Selenium WebDriver setup
- **Application Startup**: Automatic app startup and health checks
- **Artifact Collection**: Screenshots and logs for debugging
- **Resource Cleanup**: Proper cleanup of containers and processes
- **Timeout Management**: Configurable timeouts for CI environments

## Best Practices

### Test Design
1. **Isolation**: Each test should be independent and not affect others
2. **Cleanup**: Always clean up resources, even on failure
3. **Randomization**: Use random data to avoid conflicts
4. **Timeouts**: Set appropriate timeouts for each operation
5. **Logging**: Provide detailed logging for debugging

### CI Integration
1. **Headless Mode**: Use headless mode for CI environments
2. **Optimized Timeouts**: Set timeouts appropriate for CI performance
3. **Artifact Collection**: Collect logs and screenshots for debugging
4. **Resource Management**: Ensure proper cleanup of containers and processes
5. **Error Handling**: Provide clear error messages and context

### Maintenance
1. **Regular Updates**: Keep dependencies up to date
2. **Test Data**: Review and update test data as needed
3. **Performance Monitoring**: Monitor test performance and adjust timeouts
4. **Documentation**: Keep documentation up to date with changes
5. **Review Process**: Review smoke test changes as part of the development process

## Related Documentation

- [UI_TESTS.md](UI_TESTS.md): General UI testing documentation
- [TEST_DATABASE_SETUP.md](TEST_DATABASE_SETUP.md): Test database setup
- [TEST_IMPLEMENTATION_SUMMARY.md](TEST_IMPLEMENTATION_SUMMARY.md): Test implementation overview
- [TEST_ORGANIZATION.md](TEST_ORGANIZATION.md): Test organization and structure 
