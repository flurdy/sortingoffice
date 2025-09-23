# TODO List

## High Priority Epics

- Search filtering on resource list.
  - E.g. on these pages: /domains and /aliases
  - as you type filtering by name only, not screen refresh.
  - Handy if paged across many pages 

## High Priority Minor and bugs

- ✅ What does the `disabled` feature toggle for a database actually do? 
  - Currently it seems to still show the db.
  - I was expecting a disabled db not to shown as a choice as all, or at least grayed out.

- ✅ Are clients listed? I have a db with clients  but none are shown in the app.
  - RESOLVED: Clients are actually being displayed correctly. Found 8 clients in the database including IP addresses and domain names with proper status and enabled/disabled states.

## Medium Priority Epics

- Add a remove domain wizard. Postponed for now.
  - Delete or disable?
  - Delete all users
  - Delete relays and relocated
  - Delete all aliases with it in the mail field
  - Delete from domains or backup table.
  - Lists all entries to be deleted/disabled in the review step
  - Add tests

- Integrate or link to DNS lookup for NS, MX, TXT, DKIM and similar for a domain
   - Initially just research and suggest how to
   - Leave actual implementation to another TODO

- ✅ List relays for a domain on the show domain page.

## Medium Priority Minor and bugs

- ✅ /reports are mostly not translated - COMPLETED
  - ✅ Fixed hardcoded titles in Orphaned Aliases, External Forwarders, and Alias Cross-Domain reports
  - ✅ Completely translated Cross-Database Feature Toggle report
  - ✅ Completely translated Cross-Database Migration report
  - ✅ Completely translated Cross-Database User Distribution report
  - ✅ Completely translated Cross-Database Matrix report
  - ✅ Completely translated Matrix report
  - ✅ All 8 reports now have proper translation support with no hardcoded strings remaining

## Low Priority Epics

- Full search page. Postponed for now.
  - Search across all fields and tables.

- Recent changes report
  - list most recent added and updated resources

- If a db's feature is read only, add and edit buttons should be disabled. 
  - or other relevant feature toggles as well

## Low Priority Minor and bugs

- ✅  Can the alias `make prod-run` run on a different port than the default
  - RESOLVED: The functionality already exists and works perfectly. You can use:
    - `make prod-run` (default port 3000)
    - `PORT=8080 make prod-run` (uses PORT environment variable)
    - `PROD_PORT=9090 make prod-run` (PROD_PORT takes precedence over PORT)
    - The Makefile uses `${PROD_PORT:-$(PORT)}` syntax for proper precedence

- Refactor database helper functions for better maintainability. Postponed for now.
  - Consolidate similar database pool retrieval functions in `src/handlers/database_ops.rs`
  - Create generic functions to reduce code duplication
  - Improve error handling consistency across database operations
  - Add comprehensive documentation for database operation patterns

- ✅ On the statistics the Domain Statistics table belongs as a separate report instead.
  - COMPLETED: Successfully moved the Domain Statistics table to a separate report at /reports/domain-statistics. The table now appears as a dedicated report card in the reports list and has been removed from the main statistics page.

- On the Domain alias matrix report
  - Can the row styles be striped
  - Can the icons/emojiis be smaller?

- On the cross database domain matrix report
  - The domain names should be clickable and go to that domain page.
  - There is no back to reports button.
  - The domain table header key is not translated

- On the Orphaned alias and users report
  - The domain names should be clickable, if the domain exists, even if not enabled 
  - The alias should be clickable and go to that alias' page
  - The users should be clickable and go to that user's page
  - The created column seems wrong. Most of these have no created value at the moment. And not needed.
  - Can they be ordered by domain and then mail?
  - Can the row styles be striped

- On the External Forwarders report
  - The domain names should be clickable, if the domain exists, even if not enabled 
  - The alias should be clickable and go to theat alias' page
  - The created column seems wrong. Most of these have no created value at the moment. And not needed
  - Can they be ordered by domain and then mail?
  - Can the row styles be striped

- On the Cross database user distribution report
  - The users should be clickable and go to that user's page, if present on this db.
  
- On the Alias across domains report
  - The alias should be clickable and go to that alias' page.

- "Database connection error"
  - If you have chosen a db in your session, and the server reboots, and possibly the db have since been renamed or disabled, when you reload the page, the session is still alive but then the app gives up and just displays: Database connection error, and you are stuck

- On the show user page, the not-available key for created and modified is not translated
- On the show alias page, the not-available key for created and modified is not translated
- On the show domain page, the not-available key for created and modified is not translated
- On the show relay page, the not-available key for created and modified is not translated

## 🐛 Other Bugs and KNOWN ISSUES

- Requests can be slow.
