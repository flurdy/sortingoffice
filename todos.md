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

- Write integration tests that
  - has 3 databases (in test containers)
  - And the purpose is to test domains across these dbs in /domains and reports
  - This can be split across several tests.
  - Creates domain1 in db1
  - Creates domain1 in db2
  - Creates domain1 as a backup domain in db3
  - Creates domain2 in db2
  - Creates domain3 in db3
  - Creates domain4 as a backup domain in db1
  - Creates domain4 as a domain in db2
  - Verify if listed correctly in /domains when relevant db is the active db
  - And not listed when not in the relevant db
  - Verify that the cross domain report lists them all, and correctly
  - Verify that the cross domain report links to the correct domain/backup domain if on the relevant db

- Fix failing integration tests

## Low Priority Epics

- Add caching for some DB queries.
  - Mostly for reports.
  - But maybe also when domain and aliases gets paged etc.
  - Invalidated as soon as a write operation happens or a certain time.

- Full search page. Postponed for now.
  - Search across all fields and tables.

- Recent changes report
  - list most recent added and updated resources

- If a db's feature is read only, add and edit buttons should be disabled. 
  - or other relevant feature toggles as well

## Low Priority Minor and bugs

- Add a single integration test alias to the makefile like the ui one.

- Refactor database helper functions for better maintainability. Postponed for now.
  - Consolidate similar database pool retrieval functions in `src/handlers/database_ops.rs`
  - Create generic functions to reduce code duplication
  - Improve error handling consistency across database operations
  - Add comprehensive documentation for database operation patterns

- ✅ On the statistics the Domain Statistics table belongs as a separate report instead.
  - COMPLETED: Successfully moved the Domain Statistics table to a separate report at /reports/domain-statistics. The table now appears as a dedicated report card in the reports list and has been removed from the main statistics page.

- ✅ On the cross database domain matrix report
  - ✅ The domain names should be clickable and go to that domain page if in the current db.
    - This seems to not quite work on other dbs
  - ✅ There is no back to reports button. 
  - ✅ The domain table header key is not translated 
  - ✅ Fixed striping issues -
  

- ✅ On the Orphaned alias and users report
  - ✅ The domain names should be clickable, if the domain exists, even if not enabled 
  - ✅ The alias should be clickable and go to that alias' page
  - ✅ The users should be clickable and go to that user's page
  - ✅ The created column seems wrong. Most of these have no created value at the moment. And not needed.
  - ✅ Can they be ordered by domain and then mail?
  - ✅ Can the row styles be striped

- ✅ On the External Forwarders report
  - ✅ The domain names should be clickable, if the domain exists, even if not enabled 
  - ✅ The alias should be clickable and go to theat alias' page
  - ✅ The created column seems wrong. Most of these have no created value at the moment. And not needed
  - ✅ Can they be ordered by domain and then mail?
  - ✅ Can the row styles be striped

- ✅ On the Cross database user distribution report
  - ✅ The users should be clickable and go to that user's page, if present on this db.
  
- ✅ On the Alias across domains report
  - ✅ The alias should be clickable and go to that alias' page.

- ✅ On the main report page is the Alias Across Domains title looks odd?

- "Database connection error"
  - If you have chosen a db in your session, and the server reboots, and possibly the db have since been renamed or disabled, when you reload the page, the session is still alive but then the app gives up and just displays: Database connection error, and you are stuck

- On the show user page, the not-available key for created and modified is not translated
- On the show alias page, the not-available key for created and modified is not translated
- On the show domain page, the not-available key for created and modified is not translated
- On the show relay page, the not-available key for created and modified is not translated

## 🐛 Other Bugs and KNOWN ISSUES

- Requests can be slow.
