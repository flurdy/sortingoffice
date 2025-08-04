# TODO List


  - ✅ **Dutch translations**: Added comprehensive Dutch (nl-NL) translations including navigation, forms, validation messages, error pages, domains, backups, aliases, and core UI elements

  - ✅ **Dutch link in dropdown**: Added Dutch language option to both base.html and login.html templates, updated all template structs to include language_dutch field, and ensured language-dutch key is translated in all language files

- ✅ **Fixed Dutch language persistence**: Added Dutch (nl-NL) to the validation lists in both set_language and get_user_locale functions, and added a test to verify Dutch language support works correctly

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

- ✅ **Italian translations**: Added comprehensive Italian (it-IT) translations including navigation, forms, validation messages, error pages, domains, backups, aliases, and core UI elements

- ✅ **Italian link in dropdown**: Added Italian language option to both base.html and login.html templates, updated all template structs to include language_italian field, and ensured language-italian key is translated in all language files

- ✅ **Fixed Italian language persistence**: Fixed language handler to properly recognize and persist Italian (it-IT) language selection by adding it to the validation list in both set_language and get_user_locale functions

- ✅ **Fixed compilation warnings**: Removed unused askama::Template imports from handlers/about.rs, handlers/aliases.rs, handlers/auth.rs, handlers/contact.rs, and handlers/not_found.rs

- ✅ **Fixed HTML select warnings**: Updated clients/form.html to use value attribute on select element instead of selected attribute on option elements to prevent browser warnings

- ✅ **Restored Italian translations**: Recreated the comprehensive Italian (it-IT) translations file with all essential UI elements, navigation, forms, validation messages, and error pages

- ✅ **Fixed login page crash**: Resolved missing translation keys in get_login_translations function that was causing the login page to fail. Added missing Italian and Dutch language keys and restored proper login translation keys.

- ✅ **Complete Italian translation success**: Successfully expanded the Italian translation file from 394 to 916 lines (522 new lines) by adding all missing sections and specific keys:
  - Users section (complete) + password management
  - Domains section (complete) + missing aliases, catch-all
  - Reports section (complete)
  - About section (complete)
  - Aliases section (complete)
  - Statistics section (complete)
  - Clients section (complete)
  - Relays section (complete)
  - Relocated section (complete)
  - Backups section (complete)
  - Contact section (expanded)
  - Config section (expanded)
  - Wizard section (complete) + all steps and descriptions
  - Database backup section (complete)
  - Pagination section (complete)
  - Action buttons (edit, delete, enable, disable)
  - **Italian file now has 916 lines - ahead of most other languages!** (vs 400+ line gap before)

- ✅ **Dashboard title clarification**: The dashboard title shows "Dashboard" because the Italian translation file has `dashboard-title = Dashboard` (keeping it in English, which is common practice for technical terms)

- ✅ **Dashboard translation complete**: The dashboard is now 100% translated in Italian, showing "Domini e Backup" in quick action cards and "Domini e Backup Abilitati" in the domains count box.

- IGNORE: Add a Portuguese translations and link

- Github actions apart from smoke test still fail
  - https://github.com/flurdy/sortingoffice/actions/  
