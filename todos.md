# TODO List

- ✅ All Github actions apart from smoke test still fail
  - https://github.com/flurdy/sortingoffice/actions/  

- ✅ Create scrips/makefile or docker compose settings for launching ssh tunnels to databases for prod use.

## Code Quality & Refactoring Tasks

### High Priority
- ✅ Fix clippy warnings and formatting issues
  - ⚠️ Remaining: Deprecated Selenium methods (lower priority)

- ✅ Complete I18N implementation
  - ⚠️ Remaining: Locale detection and switching (lower priority)

### Medium Priority
- ✅ Refactor long functions and complex logic
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
  
✅ Sometimes refactoring has been done as an example but not applied to the rest of the codebase.
  - ✅ Applied database error handling refactoring to aliases.rs (all functions)
  - ✅ Applied database error handling refactoring to users.rs (all functions)  
  - ✅ Applied database error handling refactoring to domain_backup.rs (all functions)
  - ✅ Applied database error handling refactoring to stats.rs (all functions)
  - ✅ Applied database error handling refactoring to dashboard.rs (all functions)
  - ✅ Applied database error handling refactoring to clients.rs (all functions)
  - ✅ Applied database error handling refactoring to relays.rs (all functions)
  - ✅ Applied database error handling refactoring to relocated.rs (all functions)
  - ✅ Applied database error handling refactoring to reports.rs (all functions)
  - ✅ **COMPLETED: All handlers now use consistent database error handling patterns**

- ✅ We need to test if errors are shown in the UI with the shared theme.
 - ✅ These tests exist and are running but one is failing on theme consistency
 - ✅ Fixed error page rendering to use proper themed templates instead of plain text
 - ✅ Added missing translation key for users-not-found
 - ✅ Test now passing - error pages properly themed with consistent CSS classes

- Are there any other obvious localisation issues? Are there some other key language for mail servers that we should add?

- Sometimes refactoring has been done as an example but not applied to the rest of the codebase. THis is not about the error handling refactor but older ones.
