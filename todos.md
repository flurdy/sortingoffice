# TODO List

### High Priority

- What does the `disabled` feature toggle for a database do? Currently it seems to still show the db.

### Medium Priority

- Add a remove domain wizard. Confirm before starting this todo. Currently this is postponed till later.
  - Delete or disable?
  - Delete all users
  - Delete relays and relocated
  - Delete all aliases with it in the mail field
  - Delete from domains or backup table.
  - Lists all entries to be deleted/disabled in the review step
  - Add tests


### Low 

- Refactor database helper functions for better maintainability. Postponed for now.
  - Consolidate similar database pool retrieval functions in `src/handlers/database_ops.rs`
  - Create generic functions to reduce code duplication
  - Improve error handling consistency across database operations
  - Add comprehensive documentation for database operation patterns

## 🐛 Bugs and KNOWN ISSUES
