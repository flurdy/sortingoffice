# TODO List

## High Priority Epics

- ✅ On the list domains page, the backups list also needs to be paged independently of the domains list.
  - ✅ We do not want a separate list backup domains page.
  - ✅ We need to remember which page the domains list and the backup domains list is, when clicking on the paging buttons for both the domains and backup domains list.
  - ✅ Since the SQL lookups are eventually cached this should not be much of further delay in rendering times.

## High Priority Minor and bugs 🐛

## Medium Priority Epics

- ✅ Can we make delete resource only be clickable if a resource is disabled

## Medium Priority Minor and bugs 🐛

- ✅ bug: links to domains in domain alias matrix report are wrong. IDs are not string

- ✅ If on domains list I go to the next page of domains, the backup lists is empty

- ✅ More feature toggle unit tests for various list and show resource pages
  - ✅ For global and per db features
  - ✅ As I noticed when globally it is set to read only, on the show domain there are still add relay and add alias.

- ✅ In show domain there is no add user button in the users section

- If global or db is set to read only
  - on show domain 
    - the edit button should be disabled
    - the disable domain button should be disabled
    - on the relays rows the enable/disable button should be disabled
    - on the existing aliases rows the enable/disable button should be disabled
    - on the user rows the enable/disable button should be disabled
  - on show alias 
    - the edit button should be disabled
    - the disable button should be disabled
  - on show relay 
    - the edit button should be disabled
    - the disable button should be disabled
  - on show user 
    - the edit button should be disabled
    - the disable button should be disabled
    - the change password button should be disabled
    - the require password Change button should be disabled
  - on list users
    - on the user rows the enable/disable button should be disabled
  - I presume there are similar issues on show relocated

- on the show user page there should be a link to the domain, like in show alias

- on the show relay page there should be a link to the domain, like in show alias

- Converting a domain to a backup domain when finished redirects to the new backup domain, but the path prefix is wrong.

## Low Priority Epics

- ✅ Refactor database helper functions for better maintainability. **COMPLETED**
  - ✅ Consolidate similar database pool retrieval functions in `src/handlers/database_ops.rs`
  - ✅ Create generic functions to reduce code duplication
  - ✅ Improve error handling consistency across database operations
  - ✅ Add comprehensive documentation for database operation patterns
  - ✅ Add paginated helpers for PaginatedResult<T>
  - ✅ Migrate 6+ handlers to unified patterns
  - ✅ Add 23 comprehensive unit tests

- ✅ On a show backup domain page add a button to change it from a backup domain to a normal domain.
  - ✅ And a button to change from domain to backup domain

## Low Priority Minor and bugs 🐛 

- ✅ In the show domain page, can the users be below the aliases

- ✅ Add i18n translations for disabled delete button tooltips

- ✅ Long strings in a domain's DKIM or DMARC sections are not wrapped as expected and changes the width of the page.

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
