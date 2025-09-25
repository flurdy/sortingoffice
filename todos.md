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

- Integrate or link to DNS lookup for NS, MX, TXT, DKIM and similar for a domain. Postponed for now.
   - Initially just research and suggest how to
   - Leave actual implementation to another TODO

- ✅ List relays for a domain on the show domain page.

## Medium Priority Minor and bugs

- ✅ Write integration tests that
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

- ✅ Fix failing integration tests

- ✅ Domain statics report links to domains are broken. The URL should be i32 IDs not strings.

## Low Priority Epics

- ✅ Add caching for some DB queries - COMPLETED
  - ✅ System stats caching (5-minute TTL) - COMPLETED
  - ✅ Add caching for report functions (12 report functions need caching) - COMPLETED
  - ✅ Add caching for pagination functions (domains, aliases, users, etc.) - COMPLETED
  - ✅ Implement automatic cache invalidation on write operations - COMPLETED
  - ✅ Add cache management utilities (clear all caches, cache stats) - COMPLETED

- ✅ Verify that these caches handle different DBs
  - e.g. the statistics page values don't seem to change if the db changes

- ✅ The cache_management.rs handler seems to contain blocks of html. The other handlers do not do that. - COMPLETED

- ✅ Is cache management linked from anywhere? Perhaps inside the Configuration page? - COMPLETED

- Full search page. Postponed for now.
  - Search across all fields and tables.

- Recent changes report
  - list most recent added and updated resources

- If a db's feature is read only, add and edit buttons should be disabled. 
  - or other relevant feature toggles as well

- Refactor database helper functions for better maintainability. Postponed for now.
  - Consolidate similar database pool retrieval functions in `src/handlers/database_ops.rs`
  - Create generic functions to reduce code duplication
  - Improve error handling consistency across database operations
  - Add comprehensive documentation for database operation patterns

- On show domain, add info of the domain in the other DBs. 
  - E.g is domain or backup domain.  
  - E.g count of aliases and users
  - Enabled or disabled

## Low Priority Minor and bugs

- Fix failing UI test

- Add a single integration test alias to the makefile like the ui one. Unless the test-single already cover this

- ✅ "Database connection error"
  - If you have chosen a db in your session, and the server reboots, and possibly the db have since been renamed or disabled, when you reload the page, the session is still alive but then the app gives up and just displays: Database connection error, and you are stuck


## 🐛 Other Bugs and KNOWN ISSUES

- ✅ Requests can be slow.

- ✅ There are some test code warnings of unused functions. 
  - ✅ This should be checked against ui, integration, smoke tests and against ignored smoke tests if they are actually used.
  - ✅ Then either deleted or marked as dead code if used.
