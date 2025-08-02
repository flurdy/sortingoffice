# TODO List

- ✅ All Github actions apart from smoke test still fail
  - https://github.com/flurdy/sortingoffice/actions/  

- ✅ Create scrips/makefile or docker compose settings for launching ssh tunnels to databases for prod use.

## Code Quality & Refactoring Tasks

### High Priority
- ✅ Fix clippy warnings and formatting issues
  - ✅ Fix uninlined format args in domain_backup.rs:406
  - ✅ Run `cargo fmt --all` to fix formatting issues
  - ✅ Address unused variable warnings in test files
  - ⚠️ Remaining: Deprecated Selenium methods (lower priority)

- ✅ Complete I18N implementation
  - ✅ Handlers updated (aliases, users, domains, backups)
  - ✅ Resource-specific helper functions implemented
  - ✅ Translation keys properly integrated
  - ✅ BaseTemplate with i18n support working
  - ⚠️ Remaining: Locale detection and switching (lower priority)

### Medium Priority
- ✅ Refactor long functions and complex logic
  - ✅ Break down wizard.rs functions (complete function refactored from 100+ lines to smaller functions)
  - Extract repeated database error handling patterns
  - Consolidate similar validation logic across handlers
  - Simplify complex conditional logic in handlers

- ✅ Improve error handling and logging
  - ✅ Add safe template rendering helper function
  - Replace unwrap()/expect() with proper error handling
  - Add structured logging with context
  - Implement consistent error response patterns
  - Add error recovery mechanisms

- ✅ Performance optimizations
  - ✅ Add optimized database info helper function to reduce cloning
  - Reduce unnecessary cloning in database operations
  - Optimize database queries with proper indexing
  - Implement connection pooling improvements
  - Add caching for frequently accessed data

### Low Priority
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

### Technical Debt
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
