# TODO List

### High Priority

- ✅ Duplicate domain is not in the left menu.

- ✅ Duplicate domain wizard seems not styled like the rest

- ✅ Final step in duplicate domain shows an error: Failed to deserialize form body: missing field `source_domain`

- ✅ Duplicate domain - List aliases and relays about to be created in the review step.

- ✅ Duplicate domain - ALiases listed on the review page should be for aliases to be created, not the one copied.

- ✅ Duplicate domain - Remove the checkboxes under "What to duplicate" in the first step.

- ✅ Duplicate domain - verify backup domains are also listed.

- ✅ Duplicate domain - Remove Transport as a field. Always copy the source domain's transport.

- ✅ Duplicate domain - Relays to be copied seems to include other domains relays? It should not

- ✅ Duplicate domain - for the domain, aliases and relays:
  - Do not show enabled as a text column
  - Instead for all entries show a toggle to change enabled/disabled

- ✅ Duplicate domain - on the review page, showing the Yes in green on the right for aliases and relays does not make sense since it is not optional.

### Medium Priority

- ✅ Add a duplicate domain wizard
  - ✅ It takes searchable domain or backup domain
  - ✅ It duplicates its aliases and destinations if not a backup domain
  - ✅ It duplicates its relays
  - ✅ It does not duplicate users
  - ✅ Add tests

- Add a remove domain wizard. Confirm before starting this todo. Currently this is postponed till later.
  - Delete or disable?
  - Delete all users
  - Delete relays and relocated
  - Delete all aliases with it in the mail field
  - Delete from domains or backup table.
  - Lists all entries to be deleted/disabled in the review step
  - Add tests

- ✅ The status in Relays is only OK or REJECT. Lets default to OK.
  Similar to what is in Clients.

### Low 

- Duplicate domain - Remove the enabled checkbox for the domain on the first page. 

- Duplicate domain - in the review page, clicking back does not retain any fields.

- Duplicate domain - add tests
  - Many unit tests, for a lot of scenarios, happy and sad.
  - A few integration tests
  - One or two UI tests

- Duplicate domain - Please ensure domain field is validated as valid text like in other forms, e.g lowercase only. No symbols apart from dot, hyphen and underscore

- Duplicate domain - Can on the review page the source and new domain box be 3 columns wide by default. To avoid the 2nd row for the enabled text/toggle. 


- Refactor database helper functions for better maintainability
  - Consolidate similar database pool retrieval functions in `src/handlers/database_ops.rs`
  - Create generic functions to reduce code duplication
  - Improve error handling consistency across database operations
  - Add comprehensive documentation for database operation patterns

- Valid Tests - lets go through the tests suites to see 
  - if they actually tests what they say they should (some seem like placeholders)
  - if they will fail if conditions are not met (some seem to pass even if conditions are not met)
  - if they actually are needed. Some tests are not important.
  - if they should be split. A test should logically only test one logic.

## 🐛 Bugs and KNOWN ISSUES

- When adding a new domain, the domain list is not updated

- In show domain page, when clicking add catch-call the field in the add alias form has 2 @s.

- Add relocated 
  - Save button does nothing
  - Title missing translation

- Add client - missing title translation
