# TODO List

## ✅ COMPLETED
- Database backup functionality is now working and tested
- All unit tests are passing (88 tests)
- All handler tests are passing (40 tests)
- All integration tests are passing (16 tests)
- All security tests are passing (6 tests)
- All API tests are passing (9 tests)
- UI containerized tests are passing (including backup functionality)
- UI smoke tests are now passing after fixing the missing home field issue
- Code formatting is clean (cargo fmt passed)
- All test suites are now passing successfully
- UI tests tidy up completed:
  - Fixed compilation errors in ui_containerized.rs
  - Removed unused functions (setup_selenium_on_shared_network, setup_selenium_host, seed_schema)
  - Fixed unused variable warnings
  - Added #[allow(dead_code)] attributes for intentionally shared helper functions
  - Fixed module import/declaration order
  - Cleaned up trailing whitespace
  - All tests now compile without warnings
- Test cleanup infrastructure completed:
  - Individual test runs now automatically clean up MySQL containers and resources
  - Created Makefile aliases: `make test-single TEST=name` and `make test-single-ui TEST=name`
  - Enhanced run_tests.sh with single/single-ui/cleanup commands
  - Fixed container accumulation issue for debugging individual UI tests
  - Improved make test-clean to use comprehensive cleanup instead of basic Docker cleanup
  - Removed unused should_cleanup_schema/should_cleanup_container methods
- Test migration to new helpers completed:
  - Migrated ui_smoke.rs tests to use setup_selenium_with_default_args() and setup_selenium_with_custom_args()
  - Migrated ui_containerized.rs to use setup_selenium_on_shared_network() for shared network tests
  - Replaced manual container drop() calls with proper env.cleanup() method calls
  - Enhanced testcontainer helpers with shared network support
  - Improved cleanup patterns across test suite for consistency
- **Fixed missing alias report issue** (NEW):
  - When adding a new alias from the domain show page, the alias report was missing after redirect
  - Fixed alias creation handler to properly fetch and pass alias report data when redirecting to domain show page
  - Fixed toggle_enabled_domain_show function to also properly fetch alias data
  - Added missing import for analytics::find_database_common_aliases function
  - Now the domain show page displays the complete alias report after alias operations

- **Fixed GitHub Actions CI Failures** (NEW):
  - Increased timeout limits: Main CI (20→30 min), UI Tests (15→25 min)
  - Added granular step timeouts (5-15 minutes per step)
  - Implemented CI-specific test optimizations:
    - Reduced test parallelism: 8 threads → 2 threads for CI
    - Single-threaded UI tests to prevent container resource contention
    - Faster container timeouts: 300s → 180s for CI environments
  - Added CI environment variables: `CI=true`, `RUST_TEST_THREADS=1`, `TEST_THREADS=2`
  - Optimized container startup: Disabled unnecessary Chrome features, reduced Selenium sessions
  - All workflows now have proper timeout controls and CI-specific optimizations

## 🔄 IN PROGRESS
- None currently

## 📋 NEXT PRIORITIES
- ✅ ~~**Fix GitHub Actions CI Failures** (High Priority)~~ (COMPLETED: Implemented comprehensive CI optimizations)
  - ~~CI tests are failing due to timeout issues~~
  - ~~Need to investigate and adjust timeout configurations for the CI environment~~
  - ~~This affects development workflow and CI/CD pipeline~~

- **Reduce Shell Command Dependencies** (Medium Priority)
  - ✅ Replace hardcoded paths in mysqldump calls (COMPLETED: Implemented portable mysqldump detection)
  - ✅ Clean up problematic shell commands in smoke tests (COMPLETED: Made IP detection cross-platform)
  - ✅ Investigate alternatives to docker CLI calls where possible (COMPLETED: Most are unavoidable for test infrastructure)
  - ✅ Make the application more portable across different server environments (COMPLETED: Cross-platform support implemented)

- **Code Quality Improvements Completed** (NEW):
  - ✅ Code formatting: Applied `cargo fmt` to ensure consistent style
  - ✅ Linting: Fixed 42 clippy warnings automatically, resolved remaining 3 manually
  - ✅ Cross-platform compatibility: Fixed doc comment formatting and unwrap_or_default usage
  - ✅ Code style: All code now follows Rust best practices and passes clippy checks

- Documentation enhancements
  - ✅ Update test documentation to reflect current test status (COMPLETED: Updated TEST_IMPLEMENTATION_SUMMARY.md)
  - ✅ Document the shared network approach for UI tests (COMPLETED: Comprehensive UI_TESTS.md)
  - ✅ Add troubleshooting guide for common test issues (COMPLETED: Created TEST_TROUBLESHOOTING.md)

- **Documentation Enhancements Completed** (NEW):
  - **Test Implementation Summary**: Updated to reflect current status and recent improvements
  - **UI Tests Documentation**: Comprehensive coverage of smoke tests and containerized tests
  - **Troubleshooting Guide**: Complete guide covering critical issues, common problems, and solutions
  - **Cross-Platform Support**: Documented environment variables and platform-specific behavior
  - **CI/CD Optimization**: Documented timeout management and test parallelism settings

- Code cleanup and optimization
  - ✅ Address unused function warnings in test helpers (completed)
  - Consider consolidating duplicate test setup functions
  - Optimize test container setup and teardown

- Future enhancements
  - Consider adding more comprehensive UI test coverage
  - Explore performance improvements for test execution
  - Add integration tests for new features

## 🐛 KNOWN ISSUES
- ✅ ~~**Github actions apart from smoke test still fail** (High Priority)~~ (FIXED: Implemented comprehensive CI optimizations)
  - ~~https://github.com/flurdy/sortingoffice/actions/~~
  - ~~I think the individual test timeouts are not long enough for the CI environment.~~

- ✅ ~~When in show domain page, and you add a new alias, you are shown the domain page again. But the alias report is then missing.~~ (FIXED: Now properly fetches and displays alias report data)

- ✅ ~~In integration and UI tests, the db container is not removed~~ (FIXED: Now automatically cleaned up)

- In the rust code, lets reduce or remove if possible and shelling out to CLI commands.
  - There are some calls to docker CLI. It may by unavoidable.
  - And some calls to mysqldump which seem fragile but possibly only solution.
    - But the current hardcodes the path which will not work on other servers.
  - There may be others
  - Maybe some can be replaced with library code?
  - This is mostly to ensure it can be run anywhere.
  - ✅ ~~Removed the bogus `cargo run --cleanup-tests` call~~ (FIXED)

- ✅ ~~Maybe we should extract a testcontainer helper function to assist the tests in creating and removing mysql, selenium, app testcontainers~~ (COMPLETED: Created testcontainer_helpers.rs with unified API)
