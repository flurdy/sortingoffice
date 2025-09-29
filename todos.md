# TODO List

## High Priority Epics

- ✅ Search filtering on resource list.
  - E.g. on these pages: /domains and /aliases
  - as you type filtering by name only, not screen refresh.
  - Handy if paged across many pages 

## High Priority Minor and bugs

- ✅ What does the `disabled` feature toggle for a database actually do? 
  - Currently it seems to still show the db.
  - I was expecting a disabled db not to shown as a choice as all, or at least grayed out.

- ✅ Are clients listed? I have a db with clients  but none are shown in the app.

- ✅ Bug - In list domains, filtering by search, brings a list of matching domains, however the result seems to include double up of backups as if the whole page has been replicated inside the domains list box.

- ✅ It seems when filtering the domains, the backup domains are not filtered.
  - Please filter/search them as well at the same time. 
  - And do not create a separate filter box for just backup domains.

- ✅ Add some unit tests to test the new search/filtering of resources as well

- ✅ Github Actions workflow fails, the CI one: https://github.com/flurdy/sortingoffice/actions/workflows/ci.yml
  - Says:
    ``` process didn't exit successfully: `/home/runner/work/sortingoffice/sortingoffice/target/debug/deps/cross_database_domain_tests-68551967c8d47aff --test-threads=2` (signal: 11, SIGSEGV: invalid memory reference) ```

## Medium Priority Epics

- ✅ List relays for a domain on the show domain page.

## Medium Priority Minor and bugs

- ✅ Domain Wizard and Duplicate Domain pages does not seem to be translated? Or at least still showing in English when selected.

- ✅ Domain statics report links to domains are broken. The URL should be i32 IDs not strings.

- ✅ When viewing a resource and then in the DB drop down at the top selects another db. 
 - It will show a completely other resource with the same id in that DB
 - E.g viewing 'http://localhost:3000/domains/1' is not the same in other DBs 
 - This seems weird

## Low Priority Epics

- ✅ Recent changes report
  - list most recent added and updated resources

- ✅ If a db's feature is read only, add and edit buttons should be disabled. 
  - or other relevant feature toggles as well


- ✅ On show domain, add info of the domain in the other DBs. 
  - E.g is domain or backup domain.  
  - E.g count of aliases and users
  - Enabled or disabled

## Low Priority Minor and bugs

- ✅ There seems to be a domain backups item in the menu, under Users and above Relays. The text is missing but the icon is shown. It should not be there at all.

- ✅ Fix failing UI test

- ✅ Add a single integration test alias to the makefile like the ui one. Unless the test-single already cover this

- ✅ Check if the 'make help' is up to date and not too noisy. 
  - Maybe some aliases should just be listed in 'sub-help' eg 'make test-help','make db-help' etc?

## 🐛 Other Bugs and KNOWN ISSUES

## ⏩ Postponed epics

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

- Refactor database helper functions for better maintainability. Postponed for now.
  - Consolidate similar database pool retrieval functions in `src/handlers/database_ops.rs`
  - Create generic functions to reduce code duplication
  - Improve error handling consistency across database operations
  - Add comprehensive documentation for database operation patterns

- On a show backup domain page add a button to change it from a backup domain to a normal domain. Postponed for now.
  - This might be a wizard to add aliases etc like the current wizards

- Full search page. Postponed for now.
  - Search across all fields and tables.
