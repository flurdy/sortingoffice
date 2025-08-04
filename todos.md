# TODO List

- ✅ **Updated deprecated Selenium methods**: Replaced deprecated `driver.switch_to().alert()`, `alert.accept()`, and `driver.switch_to().default_content()` with modern `driver.accept_alert()` and `driver.enter_default_frame()` methods in tests/ui_helpers.rs

- ✅ **Refactor long functions and complex logic**: Applied extraction and guard clause patterns to domains.rs
  - ✅ Extracted validation logic from domain create function using guard clauses  
  - ✅ Extracted alias filtering logic into separate helper function
  - ✅ Simplified complex conditional logic using early returns
  - ✅ **Consolidated error handling patterns**: Applied functional programming approach to database operations
  - ✅ Created reusable database operation helpers with consistent error handling
  - ✅ Refactored domain create and delete functions to use functional error handling patterns
  - ✅ **Extracted database operations**: Created focused database operation functions with structured error handling
  - ✅ Extracted repetitive database operations into reusable helper functions
  - ✅ Applied structured error handling patterns from Rust error handling guide
  - ✅ Refactored domain list and show functions to use focused database operations

- ✅ **Improved error handling and logging**: Applied structured error handling patterns
  - ✅ Created custom error types using thiserror for database and validation operations
  - ✅ Replaced unwrap() calls with proper error handling in critical database functions
  - ✅ Added structured error handling patterns from Rust error handling guide
  - ✅ Implemented consistent error response patterns with custom error types
  - Add error recovery mechanisms

- ✅ **Performance optimizations**: Reduced unnecessary cloning and improved database operations
  - ✅ Created optimized helper functions to avoid config data cloning
  - ✅ Replaced Vec<String> with HashSet<&str> for faster alias name lookups
  - ✅ Added reference-based form creation helpers to reduce memory allocations
  - ✅ Optimized domain operations to use references instead of cloning where possible
  - Reduce unnecessary cloning in database operations
  - Implement connection pooling improvements
  - Add caching for frequently accessed data

- ✅ **Code organization improvements**: Split large utils.rs file into focused modules
  - ✅ Created errors.rs module for error types and error page helpers
  - ✅ Created database_ops.rs module for database-related helpers and error handling
  - ✅ Created performance.rs module for performance-optimized helpers
  - ✅ Updated module declarations in handlers/mod.rs
  - ✅ Fixed all compilation errors - updated all references to use new error module
  - Extract magic numbers to named constants
    - Explain the changes first, and get approval before changing magic numbers
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

## Recovery Branch Commits to Implement

- **94281b8** - Refactors database utilities into dedicated module
  - ✅ **COMPLETED** - Moved get_entity_or_handle_error, handle_entity_operation, handle_entity_operation_redirect, and handle_db_crud_operation from utils.rs to database_ops.rs
  - ✅ **COMPLETED** - Updated all call sites in aliases.rs, clients.rs, domain_backup.rs, relays.rs, relocated.rs, and users.rs to use the new database_ops module
  - ✅ **COMPLETED** - Added proper imports for database_ops functions in all affected files
  - ✅ **COMPLETED** - Maintained compilation and functionality while improving code organization
  - ✅ **COMPLETED** - Removed duplicate database functions from utils.rs (get_entity_or_not_found, handle_db_operation, get_entity_list_with_pagination, get_paginated_domains_with_fallback, get_backups_with_fallback, get_domain_with_not_found_handling, get_domain_aliases_with_fallback)
  - ✅ **COMPLETED** - Kept essential database functions in utils.rs to maintain backward compatibility
  - ✅ **COMPLETED** - Cleaned up unused imports and reduced code duplication
  - **DATABASE REFACTORING COMPLETE** - Successfully implemented a smaller, more manageable version of the original database_helpers refactor

- **475de2c** - Rendering refactor  
  - ✅ **COMPLETED** - Created rendering.rs module with generic template rendering functions
  - ✅ **COMPLETED** - Added render_form_template, render_list_template, render_show_template to rendering.rs
  - ✅ **COMPLETED** - Added rendering module to handlers/mod.rs
  - ✅ **COMPLETED** - Maintained compilation and functionality
  - ✅ **COMPLETED** - Moved render_domain_list_page from utils.rs to rendering.rs
  - ✅ **COMPLETED** - Updated 3 call sites in domains.rs to use the new rendering module
  - ✅ **COMPLETED** - Moved render_alias_list_page, render_alias_show_page, render_alias_form_page from utils.rs to rendering.rs
  - ✅ **COMPLETED** - Updated 15+ call sites in aliases.rs to use the new rendering module
  - ✅ **COMPLETED** - Removed alias rendering functions from utils.rs
  - **IN PROGRESS** - Continue moving remaining render_*_page functions from utils.rs to rendering.rs (relays, users, clients, backups, relocated)
  - **IN PROGRESS** - Update remaining call sites to use the new rendering module
  - **IN PROGRESS** - Ensure template compatibility with the current codebase

- **474eb69** - Adds a refactoring tool for Rust codebase
  - ✅ **COMPLETED** - Added refactor.sh and refactor_generic.sh scripts
  - ✅ **COMPLETED** - Added comprehensive README.md for refactoring tools

- Github actions apart from smoke test still fail
  - https://github.com/flurdy/sortingoffice/actions/  
  
- When in show domain page, and you add a new alias, you are shown the domain page again. But the alias report is then missing.
