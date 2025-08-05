# TODO List

- ✅ **Updated deprecated Selenium methods**: Replaced deprecated `driver.switch_to().alert()`, `alert.accept()`, and `driver.switch_to().default_content()` with modern `driver.accept_alert()` and `driver.enter_default_frame()` methods in tests/ui_helpers.rs

  - ✅ **Add error recovery mechanisms**: Implemented comprehensive error recovery mechanisms including retry logic with exponential backoff, circuit breaker pattern for database operations, and graceful degradation with fallback mechanisms. Added structured error handling with proper logging and user-friendly error messages.

- ✅ **Performance optimizations**: Reduced unnecessary cloning and improved database operations
  - ✅ Created optimized helper functions to avoid config data cloning
  - ✅ Replaced Vec<String> with HashSet<&str> for faster alias name lookups
  - ✅ Added reference-based form creation helpers to reduce memory allocations
  - ✅ Optimized domain operations to use references instead of cloning where possible
  - ✅ **Reduce unnecessary cloning in database operations**: Removed unnecessary `.clone()` calls in database operations including domain, backup, and user operations. Optimized string handling by using references where possible and avoiding redundant cloning in update operations.
  - ✅ **Implement connection pooling improvements**: Added comprehensive connection pooling configuration with configurable settings including max_size, min_idle, connection_timeout, idle_timeout, max_lifetime, and test_on_acquire/test_on_return options. Implemented health checks, pool statistics monitoring, and detailed health check endpoints. Added connection pool configuration to database configs with sensible defaults and example configurations.
  - Add caching for frequently accessed data

- ✅ **Code organization improvements**: Split large utils.rs file into focused modules
  - ✅ Created errors.rs module for error types and error page helpers
  - ✅ Created database_ops.rs module for database-related helpers and error handling
  - ✅ Created performance.rs module for performance-optimized helpers
  - ✅ Updated module declarations in handlers/mod.rs
  - ✅ Fixed all compilation errors - updated all references to use new error module
  - ✅ Consolidate duplicate configuration logic
  - ✅ Improve module structure and separation of concerns
  - Add comprehensive documentation

- Documentation enhancements
  - Remove duplication, where things are explained in detail in multiple places, instead refer to relevant file instead
  - Remove cruft.

- Testing enhancements
  - Add missing unit tests for utility functions
  - Improve test coverage for edge cases
  - Add integration tests for error scenarios
  - Implement property-based testing
  - ✅ **Remove tests that don't actually test anything**: Removed 8 tests from tests/utils.rs that were testing basic struct field assignment and string operations instead of actual functionality. Kept and improved the checkbox deserialization test to properly test the actual checkbox functionality.

- Security improvements
  - Audit and improve input validation
  - Add rate limiting for sensitive endpoints
  - Implement proper session management
  - Add security headers validation tests

- Remove dead code and unused imports
  - Clean up unused translation keys
  - Remove deprecated functions and modules
  - Consolidate duplicate utility functions
  - Remove commented-out code

- Configuration management
  - Extract hardcoded values to configuration
  - Implement environment-specific configs
  - Add configuration validation
  - Improve configuration documentation

- Github actions apart from smoke test still fail
  - https://github.com/flurdy/sortingoffice/actions/  
  
- When in show domain page, and you add a new alias, you are shown the domain page again. But the alias report is then missing.
