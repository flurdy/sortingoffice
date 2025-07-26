# Test Helpers

This directory contains shared test utilities that can be used across integration and UI tests to reduce code duplication and improve maintainability.

## Modules

### `common.rs`
Basic test utilities for database setup and cleanup:
- `setup_test_db()` - Sets up a test database with migrations
- `cleanup_test_db()` - Cleans up test data
- `unique_test_id()` - Generates unique test identifiers

### `testcontainers_setup.rs`
Testcontainers-specific utilities for isolated database testing:
- `setup_test_db()` - Creates a test database in a container
- `TestContainer` - Wrapper for containerized test databases
- `get_shared_mysql_port()` - Gets a shared MySQL container port

### `test_utils.rs`
Common test utilities for HTTP requests and assertions:
- `TestUtils` - HTTP request helpers and assertions
- `TestData` - Test data generators

### `ui_utils.rs` (test-only)
UI test utilities for Selenium-based testing:
- `UiTestUtils` - Selenium WebDriver helpers and assertions

## Usage Examples

### Integration Tests

```rust
use sortingoffice::test_helpers::test_utils::{TestUtils, TestData};
use sortingoffice::test_helpers::testcontainers_setup::setup_test_db;

#[tokio::test]
async fn test_domain_creation() {
    // Setup test database
    let container = setup_test_db().await;
    let db_url = format!("mysql://root@127.0.0.1:{}/{}", 
                        container.get_port(), container.get_schema());
    
    // Create test app
    let (app, state) = TestUtils::create_test_app_with_db(&db_url, "test").await;
    
    // Generate test data
    let domain = TestData::unique_domain();
    let form_data = TestData::domain_form_data(&domain, "smtp:test", true);
    
    // Make authenticated request
    let auth_cookie = TestUtils::create_edit_auth_cookie();
    let response = TestUtils::make_post_request(
        &app, &state, "/domains", &form_data, Some(auth_cookie)
    ).await.unwrap();
    
    // Assert response
    TestUtils::assert_status(&response, StatusCode::OK);
    TestUtils::assert_body_contains(response, &domain).await;
}
```

### UI Tests

```rust
use sortingoffice::test_helpers::ui_utils::UiTestUtils;

#[tokio::test]
async fn test_login_flow() -> Result<()> {
    // Setup Selenium
    let (selenium_container, driver, _port) = 
        UiTestUtils::setup_selenium_container_and_driver().await?;
    
    // Authenticate
    UiTestUtils::authenticate_driver(&driver, "http://localhost:3000").await?;
    
    // Assert dashboard loaded
    UiTestUtils::assert_page_title(&driver, "Dashboard").await?;
    UiTestUtils::assert_element_present(&driver, "h1").await?;
    
    Ok(())
}
```

## Benefits

1. **Reduced Code Duplication**: Common patterns are shared across test files
2. **Consistent Testing**: All tests use the same utilities and patterns
3. **Easier Maintenance**: Changes to test utilities benefit all tests
4. **Better Readability**: Test code focuses on test logic, not setup boilerplate
5. **Type Safety**: Shared utilities provide type-safe interfaces

## Migration Guide

To migrate existing tests to use shared helpers:

1. Replace manual auth cookie creation with `TestUtils::create_edit_auth_cookie()`
2. Replace manual app setup with `TestUtils::create_test_app_with_db()`
3. Replace manual HTTP requests with `TestUtils::make_*_request()` methods
4. Replace manual assertions with `TestUtils::assert_*()` methods
5. Replace manual test data generation with `TestData::*()` methods

## Future Improvements

- Add more specialized test utilities as needed
- Consider adding test fixtures for common scenarios
- Add performance testing utilities
- Add load testing helpers 
