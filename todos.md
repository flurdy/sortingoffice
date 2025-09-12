# TODO List

## ✅ COMPLETED TASKS

- ✅ Fix UI tests
- ✅ Extend unit and integration tests
  - ✅ Added 20 comprehensive edge case tests covering unicode, special characters, extreme lengths, malformed data, concurrent requests, authentication edge cases, and database error handling
- ✅ Update README
  - ✅ Made README concise and current by removing outdated database schema and detailed project structure
  - ✅ Now references /docs files instead of duplicating information

## 📋 PENDING TASKS

### High Priority

- ✅ Fix smoke test failure when run against local cargo run
  - ✅ Increased timeout from 30s to 60s (configurable via SMOKE_TEST_APP_TIMEOUT)
  - ✅ Added health check validation for more reliable app detection
  - ✅ Improved error messages with helpful suggestions
  - ✅ Added progress indicators showing elapsed/remaining time

### Medium Priority

### Low Priority

- Tidy up code
  -- Remove commented out code.
  -- Remove unused functions. But checking all tests including ignored ones if they are used.
  -- Remove orphaned white space.
  -- Evaluate underscored / suppressed warnings if they should just be deleted. But fully test along the way.
  -- Scan code for only partially implemented refactoring.
  -- Scan code for long functions, and split them up.
  -- Scan code for long files and see if more granular files is better.

## 🐛 KNOWN ISSUES

- **None currently**
