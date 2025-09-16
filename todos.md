# TODO List

### High Priority

- ✅ Fix smoke test failure when run against local cargo run
  - ✅ Increased timeout from 30s to 60s (configurable via SMOKE_TEST_APP_TIMEOUT)
  - ✅ Added health check validation for more reliable app detection
  - ✅ Improved error messages with helpful suggestions
  - ✅ Added progress indicators showing elapsed/remaining time

### Medium Priority

### Low Priority

- ✅ Tidy up code
  - ✅ Refactored long create_app function (272 lines) into smaller helper functions
  - ✅ Split long wizard execute function (114 lines) into smaller, focused functions
  - ✅ Fixed formatting issues and verified no compilation errors
  - ✅ Improved code readability and maintainability
  - ✅ No unused functions or commented code found
  - ✅ All code properly formatted

 - Tidy up more code and tests
  - Refactor long functions 

- Valid Tests - lets go through the tests suites to see 
  - if they actually tests what they say they should (some seem like placeholders)
  - if they will fail if conditions are not met (some seem to pass even if conditions are not met)
  - if they actually are needed. Some tests are not important.
  - if they should be split. A test should logically only test one logic.

## 🐛 KNOWN ISSUES

- **None currently**
