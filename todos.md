# TODO List

## ✅ COMPLETED TASKS

### Infrastructure & Testing
- Database backup functionality working and tested
- All test suites passing (88 unit, 40 handler, 16 integration, 6 security, 9 API, UI tests)
- Test cleanup infrastructure with automatic MySQL container cleanup
- Test migration to new helpers with shared network support
- Fixed missing alias report issue in domain show page
- Fixed GitHub Actions CI failures with timeout optimizations

### Shell Command Dependencies
- ✅ mysqldump hardcoded paths → portable detection across systems
- ✅ Linux-specific `ip` commands → cross-platform detection
- ✅ Cross-platform support for Linux, macOS, Windows

### Code Quality
- ✅ Applied `cargo fmt` for consistent style
- ✅ Fixed 45 clippy warnings automatically, resolved 3 manually
- ✅ Cross-platform compatibility improvements

### Documentation
- ✅ Updated test implementation summary
- ✅ Comprehensive UI tests documentation
- ✅ Complete troubleshooting guide for common test issues

### Test Infrastructure Consolidation
- ✅ Consolidated duplicate test setup functions
- ✅ Removed unused AppConfig and related functions
- ✅ Streamlined testcontainer_helpers.rs to focus on Selenium setup
- ✅ Fixed import conflicts in ui_smoke.rs tests

### Test Infrastructure Optimization
- ✅ Optimized test container setup and teardown
- ✅ Implemented parallel container cleanup operations
- ✅ Added environment-aware timeouts (CI vs local)
- ✅ Consolidated duplicate setup logic into core functions
- ✅ Added configurable health check intervals

### Code Cleanup & Optimization
- ✅ Consolidated duplicate test setup functions
- ✅ Removed unused AppConfig and related functions
- ✅ Streamlined testcontainer_helpers.rs to focus on Selenium setup
- ✅ Fixed import conflicts in ui_smoke.rs tests
- ✅ Cleaned up unused functions and static variables
- ✅ Simplified test cleanup and lifecycle management
- ✅ Removed redundant code and improved maintainability

### Test Infrastructure Improvements
- ✅ **Investigate alternatives to remaining docker CLI calls**
  - Created Rust-based container management module using structured docker CLI output
  - Implemented fallback mechanisms for robustness
  - Replaced shell-based cleanup with Rust-structured container operations
  - Maintained cross-platform compatibility without external dependencies
- ✅ **Fixed integration test cleanup failures**
  - Restored proper cleanup functionality after runtime panic fixes
  - Implemented blocking cleanup functions for panic and ctrl-c handlers
  - Verified cleanup is working consistently across multiple test runs
  - No more orphaned database containers or test networks
- ✅ **Resolved integration test runtime panics**
  - Fixed TestContainer Drop implementation that was creating new runtimes
  - Implemented blocking cleanup without runtime creation
  - Added retry logic for database schema creation and migrations
  - Improved connection pool management with timeouts and limits
  - Enhanced shared container initialization to prevent race conditions
- ✅ **Optimized integration test performance**
  - Removed complex retry logic that was slowing down tests
  - Simplified test setup to be faster and more reliable
  - Fixed database schema conflicts by ensuring clean schema creation
  - Leveraged existing Makefile/script cleanup instead of redundant Rust cleanup
  - **Performance improvement**: From 4+ minutes to ~1 minute (4x faster)
- ✅ **Enhanced cleanup safety**
  - Replaced dangerously broad `docker prune` commands with targeted cleanup
  - Cleanup now only removes resources specifically related to this app's tests
  - Prevents accidental removal of unrelated containers, networks, and volumes
  - Uses specific naming patterns to identify test resources safely
- ✅ **Fixed premature test suite finalization**
  - Removed problematic `test_suite_finalization` functions that ran cleanup mid-test
  - These functions were marked with `#[tokio::test]` causing them to run as regular tests
  - Cleanup now only happens at the end via `run_tests.sh` as intended
  - Eliminates interference with tests that are still running
- ✅ **Fixed handler test database connection failures**
  - Identified root cause: tests were creating two separate test databases
  - Fixed by modifying `create_test_app()` to return `TestContainer` directly
  - Tests now use `container.get_pool()` instead of `TestUtils::setup_test_db_pool(&state)`
  - All 40 handler tests now pass successfully
  - Eliminates "Unknown database" errors that were causing test failures

## 📋 PENDING TASKS

### High Priority
- **None currently**

### Medium Priority

#### Test Infrastructure Improvements
- **None currently - all completed!**

### Low Priority

#### Future Enhancements
- **Add more comprehensive UI test coverage**
  - Consider edge cases not currently covered
  - Add performance testing for UI interactions
  - Consider accessibility testing

- **Explore performance improvements for test execution**
  - Benchmark current test execution times
  - Identify bottlenecks in container setup/teardown
  - Consider test parallelization strategies

- **Add integration tests for new features**
  - As new features are developed, ensure they have proper test coverage
  - Maintain test isolation and cleanup patterns

## 🐛 KNOWN ISSUES

### Current Issues
- **Test database connection failures** (intermittent)
  - Some tests fail with "Unknown database" errors
  - Appears to be test infrastructure timing issues
  - May need investigation into container lifecycle management

- The readme has some outdated parts
  - The database schema is outdated. And does not really belong in the readme.
  - The project structure is outdated. And too granular. It should not include individual files apart from the absolute core project ones.
  - In general too much detail. If the section refers to a md file in /docs then the root readme should have minimal detail to avoid updating two different docs or becoming outdated.
