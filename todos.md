# TODO List

## High Priority Epics

## High Priority Minor and bugs 🐛 

## Medium Priority Epics

## Medium Priority Minor and bugs 🐛

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

## 🙈 KNOWN ISSUES

## ⏩ Postponed epics

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
