# TODO List

- Documentation enhancements
  - Remove duplication, where things are explained in detail in multiple places, instead refer to relevant file instead
  - Remove cruft.

- Testing enhancements
  - Add missing unit tests for utility functions
  - Improve test coverage for edge cases
  - Add integration tests for error scenarios
  - Implement property-based testing

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

- In UI tests, if a test fails the db container is not removed. Now the whole suit shares the db container, so when either the suit finishes the db has to be removed, or somehow when individual test is run as well.
