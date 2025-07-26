✅ Can the integration tests be a separate call like make test-unit and test-ui? And not part of test-unit. But part of make test-all. - IMPLEMENTED

✅ Some resource pages (backups list, users list) have enabled as active/inactive not enabled/disabled
- in dark mode the colouring of these are inconsistent in some resource lists. - IMPLEMENTED

✅ In the run_tests.sh does run_unit_tests actually test anything different than run_integration_tests? - IMPLEMENTED
- No, they were running the same tests
- Fixed by:
  - Moving unit tests into source files (src/*.rs)
  - Moving integration tests to tests/ directory
  - Updating run_tests.sh to use --lib for unit tests and --test for integration tests
  - Updating Makefile to properly separate test types

✅ Provide some timings of the unit, integration, ui-headless and ui-containerized.

Why when compiling is there multiple axum versions? 

✅ In the run_tests.sh does run_ui_containerized which uses databases in testcontainers, but then uses the running app which probably don't use the testcontainers db? Or am I missing something? - IMPLEMENTED

✅ Fix integration tests that were failing due to "Too many connections" errors - IMPLEMENTED
- Reduced test threads from 8 to 2 for integration tests
- Reduced database pool sizes for test containers (from 50 to 5)
- Fixed failing login tests to use testcontainers instead of local database
- All tests now pass: unit (21), integration (49), UI (16)

✅ If a domain has a catchall then the required and common aliases are not missing in the alias reports. - IMPLEMENTED

✅ Can we reorder the English messages.ftl, there seems to be two about sections etc, it would be good if they where joined. And the same for other sections. - IMPLEMENTED

✅ Improve theme slightly - IMPLEMENTED
- add a subtle gray background where applicable
- still support the dark mode

Add wizard for onboarding a set of new domains, maybe with common aliases and destinations across all the new domains, maybe copy existing domain+aliases.

✅ First column of aliases list (mail) is missing translations. So is the alias-empty-title and description. - IMPLEMENTED
- Added missing translations to English messages.ftl:
  - aliases-table-header-mail = Mail
  - aliases-empty-title = No aliases  
  - aliases-empty-description = Get started by creating a new alias.
- Other language files already had these translations

✅ Can the resource lists be striped in row style for easier reading - IMPLEMENTED
- Added alternating row background colors to all resource list tables
- Applied to: aliases, domains, users, relays, clients, relocated lists
- Also applied to search result lists (aliases search, domain search)
- Uses: even rows = white/gray-800, odd rows = gray-50/gray-700
- Includes hover effects for better interactivity
- Maintains dark mode compatibility

✅ Like the users table where there is a domain column which is extracted from the id, can we add a similar domain column to the aliases list, extracted from the mail field? - IMPLEMENTED
- Added domain column to aliases list table
- Extracts domain from alias.mail using `alias.mail.split('@').last().unwrap_or("")`
- Added translations for "aliases-table-header-domain" in all languages:
  - English: "Domain"
  - Spanish: "Dominio" 
  - German: "Domain"
  - French: "Domaine"
  - Norwegian: "Domene"
- Updated template struct and handler to pass domain header translation
- Adjusted column widths to accommodate new domain column

✅ Fix aliases handler panic due to missing translation key - IMPLEMENTED
- Fixed "no entry found for key" panic in aliases list handler
- Added missing "aliases-table-header-domain" key to get_translations_batch call
- Verified fix with unit tests (21 passed), integration tests (49 passed), and manual curl testing
- Confirmed domain column header now displays correctly in aliases table

✅ Can the mail, destination and domain column header be clickable in the aliases to sort by that column. If clicking domain, it will secondary sort by mail. - IMPLEMENTED
- Added sorting parameters to PaginationParams struct (sort_by, sort_order)
- Updated get_aliases_paginated function to support sorting by mail, destination, and domain
- Implemented proper domain sorting by extracting domain from mail field and post-processing results
- Added secondary sorting by mail when domains are equal (as requested)
- Added secondary sorting by mail when destinations are equal (enhancement)
- Made column headers clickable with visual indicators (up/down arrows)
- Implemented toggle functionality (asc/desc) when clicking the same column
- Updated pagination links to preserve sorting parameters
- Added sorting state to AliasesListTemplate
- Verified functionality with unit tests (21 passed), integration tests (49 passed), and manual curl testing

✅ Create a complete end-to-end functional UI test, that headless, that creates a random domain, a few aliases, a user and runs a report.
- create an duplicate flow that is not headless but instead a smoke test, which argument is a running app (defaulting to localhost 3000) and shows this flow in the browser. This smoke test must I guess can be ignored by default to avoid accidental running in build tests? - IMPLEMENTED

✅ Any way we can ensure no seeding happens on prod databases? - IMPLEMENTED
- Added `no_seeding` feature flag to DatabaseFeatures struct
- Added `is_seeding_blocked()` method to Config to check for seeding restrictions
- Modified `make seed` to block seeding when ENVIRONMENT=production or ENVIRONMENT=prod
- Added `make seed-force` command to bypass production protection (use with caution)
- Updated `prod-db-setup` to not automatically seed (migrations only)
- Added `dev-db-setup` command for development database setup (includes seeding)
- Updated config.toml.example with production safety documentation
- Seeding is automatically blocked when:
  - ENVIRONMENT=production or ENVIRONMENT=prod is set
  - Global read_only = true is set
  - Database-specific no_seeding = true is set
  - Database-specific disabled = true is set
  - Database-specific read_only = true is set

✅ Any way we can block migrations on prod databases? - IMPLEMENTED
- Added `no_migrations` feature flag to DatabaseFeatures struct
- Added `is_migration_blocked()` method to Config to check for migration restrictions
- Modified `make migrate` to block migrations when ENVIRONMENT=production or ENVIRONMENT=prod
- Added force migration commands: `make migrate-force`, `make migrate-revert-force`, `make migrate-reset-force`, `make migrate-all-force`
- Updated migration functions in `src/db.rs` to check for migration restrictions before running
- Updated database handler to pass config to migration functions
- Updated config.toml.example with migration protection documentation
- Migrations are automatically blocked when:
  - ENVIRONMENT=production or ENVIRONMENT=prod is set
  - Global read_only = true is set
  - Database-specific no_migrations = true is set
  - Database-specific disabled = true is set
  - Database-specific read_only = true is set
- Force commands require explicit confirmation to bypass protection

✅ Update the github workflows. - IMPLEMENTED
- Removed database setup from all workflows since tests now use testcontainers
- Updated CI workflow to use `make test-unit` and `make test-integration` commands
- Updated UI tests workflow to use `make test-ui` command (containerized tests)
- Updated nightly workflow to include all test types (unit, integration, UI)
- Removed MySQL service dependencies and Diesel CLI installation
- Updated workflows README to reflect current test structure and testcontainers usage
- All workflows now rely on testcontainers for isolated database testing
- Simplified workflow configuration and reduced external dependencies

✅ Can we make certain tables optional? Relays, relocated and clients are not in every db. - IMPLEMENTED
- Added optional table feature flags to DatabaseFeatures struct (no_relays, no_relocated, no_clients)
- Added methods to Config to check table availability (is_relays_available, is_relocated_available, is_clients_available)
- Updated database functions to handle missing tables gracefully in stats collection
- Added helper functions to check if tables exist (table_exists, relays_table_exists, etc.)
- Updated handlers to check table availability before processing requests
- Added conditional rendering in UI templates (navigation menu, dashboard cards)
- Added translation keys for "not available" messages
- Updated configuration example with documentation for optional tables
- All tests pass: unit (21), integration (49)

✅ On the configuration page there is functionality to add and remove, promote and demote, required and common aliases. We do not need this. Configuring it in config.toml offline is enough. - IMPLEMENTED
- Removed required and common aliases management sections from config template
- Removed related fields from ConfigTemplate struct
- Removed unused translations from config handler
- Removed JavaScript functions for required/common alias management
- Kept domain overrides and feature toggles functionality
- Removed form action since no update handler exists
- All tests pass: unit (24), integration (49)

✅ For clients and relays, the status is either OK, or REJECT, not to be confused with the enabled field. - IMPLEMENTED
- Fixed clients templates to properly handle "OK" and "REJECT" status values from database
- Updated relays template to display status with proper styling and translations
- Added status-ok and status-reject translation keys to English messages
- Updated relays handler and template struct to pass status translations
- Status field now properly distinguished from enabled field in UI
- All tests pass: unit (24), integration (49)

✅ In the UI I seem unable to edit a relay and relocated. - IMPLEMENTED
- Fixed relay status display inconsistency between list and show pages
- Added missing status_ok and status_reject fields to RelayShowTemplate struct
- Updated show_relay handler to include status translations
- Updated relay show template to display status with proper styling (green for OK, red for REJECT)
- Status now displays consistently across list and show pages with proper translations
- Fixed missing "Back to Relays" button text by adding relays-back-to-list translations to all language files
- Database re-seeded with correct "OK"/"REJECT" values, templates reverted to original logic
- Renamed clients status translation keys to be consistent with database values: clients-status-ok and clients-status-reject
- Updated all language files (en-US, de-DE, es-ES, fr-FR, nb-NO) with new translation keys
- Updated clients handlers to use new translation keys for better consistency
- Updated get_status_translations helper function to return correct keys for clients entity
- Updated form templates to use new translation keys
- All tests pass: unit (24), integration (49)

✅ Create onboarding doc, detailing step by step what to do, fork code, build image or download, connect dbs, configure config.toml, run sql to add modified and created fields if missing, ensure migrations are turned off, start app, log in - IMPLEMENTED
- Created comprehensive ONBOARDING.md documentation in docs/ directory
- Covers all aspects: prerequisites, code acquisition, building, database setup, configuration, schema preparation, production safety, startup, first login, and troubleshooting
- Includes multiple deployment options: Docker, local development, systemd service
- Documents production safety features and force commands
- Added security considerations and next steps
- Updated README.md to reference the new onboarding guide
- All tests pass: unit (24), integration (49)

✅ On the configuration page there is functionality to a run migrations. We do not need this. Migrations are only run on dev env. In prod no migrations are allowed and this could lead to accidental migrate runs. - IMPLEMENTED
- Removed migration functionality from database selection page
- Removed /database/migrate route from router
- Removed run_migrations function from database handler
- Removed MigrationForm struct
- Removed migration management section from database selection template
- Migrations can now only be run via command line tools (make migrate, etc.)
- All tests pass: unit (24), integration (49)

✅ Add a backup function to backup a database and download it as file locally.
- Implemented complete database backup functionality with web interface
- Added backup creation using mysqldump with proper error handling
- Added backup download functionality with security checks
- Added backup listing and deletion capabilities
- Created backup management UI with database selection
- Added proper authentication and authorization for backup operations
- Implemented backup file naming with timestamps (format: database_id_database_YYYYMMDD_HHMMSS.sql)
- Added automatic backup directory creation
- Renamed "Backup" to "Database Backup" to avoid confusion with "Backups" table
- Updated all translation files (en-US, es-ES, de-DE, fr-FR, nb-NO) with new naming
- Fixed JSON error handling in backup functionality
- Fixed compilation errors with HTMX handler parameter order
- Updated mysqldump command to use minimal privileges (--no-tablespaces, --skip-lock-tables, etc.) to avoid PROCESS privilege requirements
- Verified backup functionality works with basic SELECT privileges
- Enhanced backup listing with prettified table showing database info, timestamps, file sizes
- Fixed UI refresh issues and JavaScript scope problems
- Fixed database name parsing for databases with underscores in names
- Fixed table layout and width issues for better usability
- Implemented complete translation system for table headers and action buttons
- Created comprehensive documentation in docs/DATABASE_BACKUP.md
- All tests pass: unit (24), integration (49)

✅ Create a little curl helpers in the scripts folder for logging in etc.
- Created comprehensive curl-helpers.sh script with authentication and API testing capabilities
- Implemented login/logout functionality with session cookie management
- Added backup operations (create, list, download, delete)
- Added resource listing operations (aliases, domains, users)
- Added system operations (health, stats, config)
- Implemented proper error handling and colored output
- Added environment variable support for configuration
- Created comprehensive documentation in scripts/README.md
- Tested all functionality with working application
- All features working: login, logout, backup operations, resource listing

✅ Ensure no migration is run automatically on starting the app.
- Verified that no migrations are run during application startup
- Confirmed DatabaseManager::new() only creates connection pools, no migrations
- Checked main.rs startup sequence: config loading → i18n init → db manager init → server start
- Verified no automatic migration calls in handlers or initialization code
- Confirmed migration functions exist but are only called manually via CLI commands
- Migration system has proper production protection with force commands
- All migration operations require explicit user action via make commands
- Application startup logs show no migration-related messages

✅ Review and refactor the codebase if there are things to clean, delete, simplify, standardise, rename, document, and other useful things to make the codebase easies to maintain in the future.
- Fixed all clippy warnings and code quality issues
- Applied automatic fixes for uninlined format args (12 fixes)
- Manually fixed manual_flatten warning in backup.rs
- Improved code formatting with cargo fmt
- Verified no TODO/FIXME comments or dead code
- Confirmed well-organized code structure with proper separation of concerns
- Validated translation system abstraction and reuse
- Checked for magic numbers and hardcoded values (found none problematic)
- Verified all unit tests pass (24 tests)
- Confirmed code follows Rust best practices and conventions
- No major refactoring needed - codebase is well-maintained

✅ Can any of the code in the integrations and ui tests be moved to helpers and shared or are they too different?
- Created comprehensive shared test utilities in src/test_helpers/test_utils.rs
- Implemented TestUtils struct with common HTTP request helpers and assertions
- Added TestData struct for generating unique test data and form data
- Created authentication helpers (create_auth_cookie, create_edit_auth_cookie, etc.)
- Added test app setup helpers (create_test_app_with_db, create_test_app_with_dbs)
- Implemented HTTP request helpers (make_get_request, make_post_request, make_delete_request)
- Added assertion helpers (assert_status, assert_body_contains, assert_body_not_contains)
- Created database configuration helpers with dynamic port replacement
- Added comprehensive documentation in src/test_helpers/README.md with usage examples
- All unit tests pass (24 tests)
- UI test helpers were attempted but had complex Selenium dependency issues
- Integration test helpers are ready for use and can significantly reduce code duplication

Clicking on database selection in another language switches back to english, for that page only.

✅ Lets work on getting github workflow passing again.
- https://github.com/flurdy/sortingoffice/actions
- Fixed all code formatting issues with cargo fmt
- Fixed all clippy warnings with cargo clippy --fix
- Verified all unit tests pass (24 tests)
- Verified all integration tests pass (49 tests)
- Verified all UI tests pass (17 tests)
- Verified Docker build works correctly
- All GitHub Actions workflows should now pass:
  - CI workflow (unit tests, integration tests, clippy, fmt, Docker build)
  - UI tests workflow (containerized UI tests)
  - Nightly workflow (comprehensive testing)
  - Security audit workflow 
