# TODO List


  - ✅ **Dutch translations**: Added comprehensive Dutch (nl-NL) translations including navigation, forms, validation messages, error pages, domains, backups, aliases, and core UI elements

  - ✅ **Dutch link in dropdown**: Added Dutch language option to both base.html and login.html templates, updated all template structs to include language_dutch field, and ensured language-dutch key is translated in all language files

- ✅ **Fixed Dutch language persistence**: Added Dutch (nl-NL) to the validation lists in both set_language and get_user_locale functions, and added a test to verify Dutch language support works correctly

- ✅ **Updated deprecated Selenium methods**: Replaced deprecated `driver.switch_to().alert()`, `alert.accept()`, and `driver.switch_to().default_content()` with modern `driver.accept_alert()` and `driver.enter_default_frame()` methods in tests/ui_helpers.rs

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
