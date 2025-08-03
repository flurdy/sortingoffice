# TODO List


  - ✅ **Dutch translations**: Added comprehensive Dutch (nl-NL) translations including navigation, forms, validation messages, error pages, domains, backups, aliases, and core UI elements

  - ✅ **Dutch link in dropdown**: Added Dutch language option to both base.html and login.html templates, updated all template structs to include language_dutch field, and ensured language-dutch key is translated in all language files

- Deprecated Selenium methods (lower priority)

- Refactor long functions and complex logic
  - Consolidate similar validation logic across handlers
  - Simplify complex conditional logic in handlers

- Improve error handling and logging
  - Replace unwrap()/expect() with proper error handling
  - Add structured logging with context
  - Implement consistent error response patterns
  - Add error recovery mechanisms

- Performance optimizations
  - Reduce unnecessary cloning in database operations
  - Optimize database queries with proper indexing
  - Implement connection pooling improvements
  - Add caching for frequently accessed data

- Code organization improvements
  - Extract magic numbers to named constants
  - Consolidate duplicate configuration logic
  - Improve module structure and separation of concerns
  - Add comprehensive documentation

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


- The utils.rs file is huge. Can it be split?

- Add a Italian translations and link

- Add a Portuguese translations and link

- Github actions apart from smoke test still fail
  - https://github.com/flurdy/sortingoffice/actions/  
