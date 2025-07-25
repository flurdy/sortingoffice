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

For clients and relays, the status is either OK, or REJECT, not to be confused with the enabled field.

In the UI I seem unable to edit a relay and relocated.

Create onboarding doc, detailing step by step what to do, fork code, build image or download, connect dbs, configure config.toml, run sql to add modified and created fields if missing, ensure migrations are turned off, start app, log in

On the configuration page there is functionality to a run migrations. We do not need this. Migrations are only run on dev env. In prod no migrations are allowed and this could lead to accidental migrate runs.

Add a backup function to backup a database and download it as file locally.

Ensure no migration is run automatically on starting the app.
