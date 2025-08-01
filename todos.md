# Sorting Office - TODO List

## Completed Tasks

### 2025-01-31
- [x] Restored useful runtime logging (auth failures, permission issues, unexpected flows)
- [x] Removed verbose debug logs from integration tests (full page source dumps)
- [x] Removed unused variables entirely instead of suppressing warnings
- [x] Fixed UI test duplicate data issues with INSERT IGNORE
- [x] Updated smoke test to use testcontainers for database isolation
- [x] Modified CI workflow to use testcontainers database approach
- [x] Added testcontainers for app containerization using GenericImage
- [x] Added testcontainers for selenium containerization using GenericImage
- [x] Updated CI workflow to remove manual selenium setup
- [x] Added Makefile target for testcontainers smoke test (`test-smoke-containerized`)
- [x] Updated config.toml to support environment variable substitution
- [x] Fixed environment-based smoke test by adding missing environment variables to .env file
- [x] Added dotenv loading to main.rs to properly load .env file
- [x] **REFACTORED: Eliminated code duplication between ui_smoke.rs and ui_containerized.rs**
  - [x] Created shared `tests/ui_helpers.rs` module with common functions
  - [x] Moved all duplicated helper functions to shared module
  - [x] Updated both test files to use shared helpers
  - [x] Reduced code duplication by ~70% (from ~2000 lines to ~600 lines)
  - [x] Improved maintainability by centralizing common test logic
- [x] **FIXED: Test separation issue with --ignored flag**
  - [x] Created separate test functions with different names to avoid conflicts
  - [x] Environment-based test: `ui_smoke_e2e_flow`
  - [x] Testcontainers test: `ui_smoke_containerized_e2e_flow`
  - [x] Tests now run independently when specified by name
- [x] **CLEANUP: Removed docker-compose selenium setup**
  - [x] Both smoke tests now use testcontainers for Selenium
  - [x] Removed selenium service from docker-compose.yml
  - [x] Removed selenium-related Makefile targets
  - [x] Updated test scripts to remove selenium dependency
  - [x] Updated CI workflow to use new test names
  - [x] Simplified test setup - no manual selenium management required
- [x] **UPDATED: Documentation to reflect current test architecture**
  - [x] Updated `docs/SMOKE_TESTS.md` with current test architecture
  - [x] Updated `docs/UI_TESTS.md` with shared infrastructure details
  - [x] Updated `docs/TEST_ORGANIZATION.md` with current organization
  - [x] Removed references to docker-compose selenium setup
  - [x] Updated test names and commands throughout documentation
  - [x] Added information about shared helpers and testcontainers Selenium

## Current Status

### ❌ **Failing: Environment-based Smoke Test**

### ❌ **Still Failing: Testcontainers Smoke Test**
- **Status**: ❌ **FAILING** with `App /health not healthy after 30s`
- **Command**: `cargo test ui_smoke_containerized_e2e_flow -- --ignored --nocapture`
- **Infrastructure**: Uses testcontainers for database, app, and selenium
- **Root Cause**: Application container starts but doesn't respond to health checks
- **Debugging Progress**:
  - ❌ Fixed config file mismatch (using `config.docker.toml`)
  - ✅ Fixed environment variable mismatch (using `DATABASE_URL`)
  - ✅ Fixed container networking (using `host.docker.internal`)
  - ✅ Confirmed Docker image has working environment variable substitution
  - ❌ **Remaining Issue**: Container mount not working as expected


## Next Steps

### Priority 1: Fix Testcontainers Smoke Test
1. **Investigate container mount issue**: The config file mount isn't working as expected
2. **Debug container startup**: Check why the application container isn't responding to health checks
3. **Test alternative approaches**: Consider using docker-compose for testcontainers setup

### Priority 2: Fix Selenium Issues
1. **Environment-based test**: Fix selenium user data directory conflict
2. **Add unique user data directories**: Ensure each test run uses a unique Chrome profile
3. **Improve selenium cleanup**: Better cleanup between test runs

### Priority 3: Improve Test Infrastructure
1. **Add more shared helpers**: Move remaining duplicated functions to ui_helpers.rs
2. **Create test utilities**: Add more common test patterns to shared module
3. **Documentation**: Update test documentation to reflect new shared structure

### Priority 4: CI/CD Integration
1. **Update CI workflow**: Ensure both smoke test approaches work in CI
2. **Add test reporting**: Better reporting for test failures
3. **Performance optimization**: Reduce test execution time
