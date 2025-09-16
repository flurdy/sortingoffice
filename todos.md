# TODO List

### High Priority

- ✅ Fix smoke test failure when run against local cargo run
  - ✅ Increased timeout from 30s to 60s (configurable via SMOKE_TEST_APP_TIMEOUT)
  - ✅ Added health check validation for more reliable app detection
  - ✅ Improved error messages with helpful suggestions
  - ✅ Added progress indicators showing elapsed/remaining time

### Medium Priority

- ✅ Add a duplicate domain wizard
  - ✅ It takes searchable domain or backup domain
  - ✅ It duplicates its aliases and destinations if not a backup domain
  - ✅ It duplicates its relays
  - ✅ It does not duplicate users
  - ✅ Add tests

- Add a remove domain wizard
  - Delete all users
  - Delete relays and relocated
  - Delete all aliases with it in the mail field
  - Delete from domains or backup table.
  - Add tests

### Low Priority

- ✅ Tidy up code
  - Refactored long create_app function into smaller helper functions
  - ✅ Split long wizard execute function into smaller, focused functions
  - ✅ Fixed formatting issues and verified no compilation errors
  - ✅ Improved code readability and maintainability
  - ✅ No unused functions or commented code found
  - ✅ All code properly formatted

 - ✅ Tidy up more code and tests
  - ✅ Refactor long functions 

- Refactor database helper functions for better maintainability
  - Consolidate similar database pool retrieval functions in `src/handlers/database_ops.rs`
  - Create generic functions to reduce code duplication
  - Improve error handling consistency across database operations
  - Add comprehensive documentation for database operation patterns

- Valid Tests - lets go through the tests suites to see 
  - if they actually tests what they say they should (some seem like placeholders)
  - if they will fail if conditions are not met (some seem to pass even if conditions are not met)
  - if they actually are needed. Some tests are not important.
  - if they should be split. A test should logically only test one logic.

## 🐛 KNOWN ISSUES

- **None currently**
