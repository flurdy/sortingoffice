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

## Medium Priority Epics

- Add a remove domain wizard. Postponed for now.
  - Delete or disable?
  - Delete all users
  - Delete relays and relocated
  - Delete all aliases with it in the mail field
  - Delete from domains or backup table.
  - Lists all entries to be deleted/disabled in the review step
  - Add tests

## Medium Priority Minor and bugs

## Low Priority Epics

- Full search page. Postponed for now.
  - Search across all fields and tables.

- Recent changes report
  - list most recent added and updated resources

## Low Priority Minor and bugs

- Can the alias `make prod-run` run on a different port than the default

- Refactor database helper functions for better maintainability. Postponed for now.
  - Consolidate similar database pool retrieval functions in `src/handlers/database_ops.rs`
  - Create generic functions to reduce code duplication
  - Improve error handling consistency across database operations
  - Add comprehensive documentation for database operation patterns

- On the statistics the Domain Statistics table belongs as a separate report instead.

- On the Domain alias matrix report
  - Can the row styles be striped
  - Can the icons/emojiis be smaller?

- On the cross database domain matrix report
  - The domain names should be clickable and go to that domain page.
  - There is no back to reports button.
  - The domain table header key is not translated

- On the Orphaned alias and users report
  - The domain names should be clickable, if the domain exists, even if not enabled 
  - The alias should be clickable and go to theat alias' page
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

## 🐛 Other Bugs and KNOWN ISSUES
