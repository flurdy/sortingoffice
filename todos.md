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

## 🔄 IN PROGRESS
- None currently

## 📋 NEXT PRIORITIES
- Documentation enhancements
  - Update test documentation to reflect current test status
  - Document the shared network approach for UI tests
  - Add troubleshooting guide for common test issues

- Code cleanup and optimization
  - Address unused function warnings in test helpers
  - Consider consolidating duplicate test setup functions
  - Optimize test container setup and teardown

- Future enhancements
  - Consider adding more comprehensive UI test coverage
  - Explore performance improvements for test execution
  - Add integration tests for new features

## 🐛 KNOWN ISSUES
- Github actions apart from smoke test still fail
  - https://github.com/flurdy/sortingoffice/actions/  
  - I think the individual test timeouts are not long enough for the CI environment.
  
- When in show domain page, and you add a new alias, you are shown the domain page again. But the alias report is then missing.

- In UI tests, if a test fails the db container is not removed. Now the whole suit shares the db container, so when either the suit finishes the db has to be removed, or somehow when individual test is run as well.

- In the rust code, lets reduce or remove if possible and shelling out to CLI commands. 
  - There are some calls to docker CLI. It may by unavoidable.
  - And some calls to mysqldump which seem fragile but possibly only solution.
    - But the current hardcodes the path which will not work on other servers.
  - There may be others
  - Maybe some can be replaced with library code?
  - This is mostly to ensure it can be run anywhere.
