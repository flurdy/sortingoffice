# TODO List

## High Priority Epics

## High Priority Minor and bugs 🐛 

- On the list domains page, the backups list also needs to be paged independently of the domains list.
  - We do not want a separate list backup domains page.
  - We need to remember which page the domains list and the backup domains list is, when clicking on the paging buttons for both the domains and backup domains list.
  - Since the SQL lookups are eventually cached this should not be much of further delay in rendering times.

## Medium Priority Epics

- ✅ Can we make delete resource only be clickable if a resource is disabled

## Medium Priority Minor and bugs 🐛

- ✅ bug: links to domains in domain alias matrix report are wrong. IDs are not string

- ✅ If on domains list I go to the next page of domains, the backup lists is empty

## Low Priority Epics

- ✅ Refactor database helper functions for better maintainability. **COMPLETED**
  - ✅ Consolidate similar database pool retrieval functions in `src/handlers/database_ops.rs`
  - ✅ Create generic functions to reduce code duplication
  - ✅ Improve error handling consistency across database operations
  - ✅ Add comprehensive documentation for database operation patterns
  - ✅ Add paginated helpers for PaginatedResult<T>
  - ✅ Migrate 6+ handlers to unified patterns
  - ✅ Add 23 comprehensive unit tests

## Low Priority Minor and bugs 🐛 

- ✅ In the show domain page, can the users be below the aliases

- ✅ Add i18n translations for disabled delete button tooltips

## 🙈 KNOWN ISSUES

## ⏩ Postponed epics

- Have a cached/timebased undo feature for deletion

- Add a remove domain wizard. Postponed for now.
  - Delete or disable?
  - Delete all users
  - Delete relays and relocated
  - Delete all aliases with it in the mail field
  - Delete from domains or backup table.
  - Lists all entries to be deleted/disabled in the review step
  - Add tests

- On a show backup domain page add a button to change it from a backup domain to a normal domain. Postponed for now.
  - This might be a wizard to add aliases etc like the current wizards
