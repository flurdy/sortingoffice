# TODO List

## High Priority Epics

## High Priority Minor and bugs 🐛

- ✅ In prod, reports were throwing 500 errors:
  - ✅ Orphan report blew up with a 500
    - Already optimized to O(n+m) with HashSet/HashMap lookups  
    - **FIXED**: Added 10,000 record safety limit with detailed error logging
    - If limit exceeded, logs will show exact breakdown (aliases/users/relays/relocated)
    - Can use "hide_disabled" filter to reduce count before rendering
  - ✅ Domain statistics report blew up with a 500
    - **FIXED**: Rewritten from 6000+ individual queries to just 5 bulk queries
    - Before: For 1000 domains = ~6000 SQL queries (6 per domain)
    - After: 5 queries total + HashMap aggregation in Rust
    - Added comprehensive logging and timing
  - ✅ Recent changes report blew up with a 500 
    - Already limited to 50 items per table (350 max records)
    - **FIXED**: Added detailed timing logs and error handling
    - **ACTION NEEDED**: Add database indexes on `modified` columns for better performance
    - See `/docs/PERFORMANCE_INDEXES.md` for SQL statements to create indexes
  - All fixes ready for production testing with comprehensive logging
  - Logs will now show exact timing and record counts for diagnosis

## Medium Priority Epics

- ✅ Can the exclude enabled/disabled filter on domains actually be a 3 state toggle.
  - Similar looks to the enable/disable toggle on edit resources
  - With all, only disabled and only enabled as states?
  - ✅ Implemented for domains
  - ✅ Implemented for aliases
  - ✅ Implemented for relays
  - ✅ Implemented for relocated
  - ✅ Implemented for users
  - ✅ Implemented for clients

## Medium Priority Minor and bugs 🐛

- ✅ Lets check the tests (e.g. make test-unit, make test-smoke-containerized etc)
  - ✅ Unit tests: 112/112 passed
  - ✅ Integration tests: 77/77 passed (13 + 45 handlers + 16 integration + 3 testcontainers)
  - ✅ Security tests: 6/6 passed
  - ✅ API tests: 9/9 passed
  - ✅ UI tests: 21/21 passed (18 UI + 2 duplicate wizard + 1 wizard, 1 ignored)
  - Note: Smoke test has minor alert cleanup issue (test infrastructure, not application bug)
  - Note: Some wizard tests show timeout warnings but still verify all resources correctly

- ✅ The paging seems not to be on the mx reports anymore? Related to the recent filtering, maybe?

- ✅ In the mx servers report, can it be shown if a domain:
  - ✅ enabled, disabled
  - ✅ normal domain or backup domain 

- ✅ On a prod site, the orphaned report keep blowing up. It has a lot of domain and aliases.
  - Optimized to use HashSet/HashMap lookups instead of individual DB queries per record
  - Performance improved from O(n*m) to O(n+m) where n=records, m=domains
  - Added comprehensive error handling and logging at each stage of report generation
  - Added timing logs to identify bottlenecks (start time, elapsed time, record counts)
  - Added detailed error logging with context at every database operation
  - All unit tests (112/112) pass
  - Ready for production testing - logs will now show exactly where any error occurs

## Low Priority Epics

- ✅ Add html head title and description to all the pages. So that tabs can be distinguished when not wide E.g
   - ✅ show domain could have: 'DOMAINNAME domain at DB db - Sorting Office' 

- ✅ In show alias, at the bottom replicate the Alias across domains report for that alias

- ✅ Can the orphaned report also check relays and relocated entries.

- ✅ Add a remove domain wizard.
  - ✅ Created RemoveWizardStep enum with all steps
  - ✅ Created RemoveDomainSession model to track wizard state
  - ✅ Created DisabledResourceCount and DeletedResourceCount tracking models
  - ✅ Created handler with all wizard steps (domain_selection, review_affected, disable_resources, review_disabled, confirm_delete, execute_deletion)
  - ✅ Added all routes (GET and POST)
  - ✅ Created helper functions to find affected resources
  - ✅ Implemented disable flow (domain → aliases → users → relays → relocated)
  - ✅ Implemented delete flow (relocated → relays → users → aliases → domain)
  - ✅ Added cross-database domain detection
  - ✅ Added orphaned aliases detection (aliases with domain in destination field)
  - ✅ Added logic to skip disable step if all resources already disabled
  - ✅ Basic HTML rendering functions (to be upgraded to proper templates)
  - ✅ Created 5 Askama template structs (selection, review_affected, review_disabled, confirm_delete, complete)
  - ✅ Created 4 HTML template files (selection, review_disabled, confirm_delete, complete)
  - ✅ Added `nav-remove-wizard` translation key to all 7 locales
  - ✅ Added navigation link to wizard in sidebar with trash icon
  - ✅ Added 38 translation keys for wizard pages in English (en-US)
  - ✅ Translated wizard keys to all 6 other languages (de-DE, es-ES, fr-FR, it-IT, nb-NO, nl-NL)
  - ✅ All 7 locales now have complete Remove Wizard translations
  - ✅ Upgraded ALL rendering functions to use proper Askama templates with full i18n
  - ✅ Selection page uses RemoveDomainSelectionTemplate with all translations
  - ✅ Review Affected page uses RemoveReviewAffectedTemplate with all translations
  - ✅ Review Disabled page uses RemoveReviewDisabledTemplate with all translations
  - ✅ Confirm Delete page uses RemoveConfirmDeleteTemplate with all translations
  - ✅ Complete page uses RemoveCompleteTemplate with all translations
  - ✅ All 5 HTML templates created with modern UI matching app style
  - ✅ Orphaned aliases shown in proper table with clickable links to alias details
  - ✅ Table shows alias mail, destination, and enabled/disabled status
  - ✅ **FIXED**: Orphaned list now correctly excludes aliases that will be deleted (have domain in mail field)
  - ✅ Only shows aliases with domain in destination field that won't be deleted
  - ✅ **FIXED**: Cross-database list now correctly excludes current database
  - ✅ Only shows OTHER databases where this domain exists
  - ✅ Added 6 unit tests for wizard logic (session storage, filtering, serialization)
  - ✅ Added 9 integration tests for form validation and resource counts
  - ✅ All 118 unit tests pass (up from 112)
  - ✅ All 9 integration tests pass
  - Remember to use htmx where suitable
  - Remember same style as the rest of the app
  - Remember this is a multi DB app but the removal is from one db per wizard flow
     - If session DB is changed mid flow then abort

## Low Priority Minor and bugs 🐛 

- ✅ In show backup domains, the disable/enable buttons for relays are not translated
  - They use "Enable Alias" / "Disable Alias" but should say "Enable Relay" / "Disable Relay"
  - **FIXED**: Added `relays-enable` and `relays-disable` keys to all 7 locales
  - Updated handler to use relay-specific translation keys
  - Buttons now correctly say "Enable Relay" / "Disable Relay" in all languages

- ✅ Like in some of the reports, can the we add filters to the domain and backup domain lists? 
   - ✅ enabled/disabled
   - ✅ subdomain

- ✅ In show alias, on the domain row, can we add a tiny icon if the domain is enabled or not

- ✅ In show alias, if the alias is a catch all, please do not include the alias occurrences report.

- ✅ On the domain statistics report, remove the quota columns. Add relays and relocated.
  - With enabled and disabled shown as well?

- ✅ On show backup domain, the relay rows do not have the usual buttons.
  - Added Actions column header
  - Added View button to each relay row
  - Added Enable/Disable toggle button to each relay row
  - Buttons respect read-only database mode
  - All buttons use HTMX for dynamic updates

## 🙈 KNOWN ISSUES

* Adding alias/user full email as domain field in a backup domain is an edge case 

## ⏩ Postponed epics

- Have a cached/timebased undo feature for deletion
