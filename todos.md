# TODO List

## High Priority Epics

- ✅ Search filtering on resource list.
  - E.g. on these pages: /domains and /aliases
  - as you type filtering by name only, not screen refresh.
  - Handy if paged across many pages 

## High Priority Minor and bugs 🐛 

- ✅ Github Actions workflow fails, the CI one: https://github.com/flurdy/sortingoffice/actions/workflows/ci.yml
  - Says:
    ``` process didn't exit successfully: `/home/runner/work/sortingoffice/sortingoffice/target/debug/deps/cross_database_domain_tests-68551967c8d47aff --test-threads=2` (signal: 11, SIGSEGV: invalid memory reference) ```
    
- ✅ There are a lot 'Unknown database' in the Github Actions CI workflow fails
  - https://github.com/flurdy/sortingoffice/actions/runs/18103979585/job/51513804349
  - Investigate CI logs for which step emits "Unknown database"
  - Ensure test databases/schemas are created in CI before tests run
  - Align CI env/config with local `config.toml.example`/docker env
  - Verify CI run passes after changes

## Medium Priority Epics

- ✅ Integrate or link to DNS lookup for NS, MX, TXT, DKIM and similar for a domain.
   - ✅ Research completed - see docs/DNS_LOOKUP_RESEARCH.md

- Implement DNS lookup researched above
   - See docs/DNS_LOOKUP_RESEARCH.md
   - Adjust DNS lookup implementation plan:
     - There is no DB for this app to store any information (the app accesses client mail dbs, but they are not its own dbs)
     - There is in-memory caching that can be extended to avoid too many request.

- ✅ Full search page. 
  - ✅ Search across all fields and tables.

## Medium Priority Minor and bugs 🐛 

- ✅ Rename the 'Domain Wizard' to 'Bulk Add Domains' 
  - Because there are other wizards for domains, the original name is confusing
  - Or perhaps an even better/more suitable new name can be found

- ✅ When viewing a resource and then in the DB drop down at the top selects another db. 
 - It will show a completely other resource with the same id in that DB
 - E.g viewing 'http://localhost:3000/domains/1' is not the same in other DBs 
 - This seems weird

## Low Priority Epics

- ✅ Recent changes report
  - list most recent added and updated resources

## Low Priority Minor and bugs 🐛 

- ✅ Add a single integration test alias to the makefile like the ui one. Unless the test-single already cover this

- ✅ Check if the 'make help' is up to date and not too noisy. 
  - Maybe some aliases should just be listed in 'sub-help' eg 'make test-help','make db-help' etc?

- ✅ Make sure each section of makefile has a help for its part. 

- ✅ Move test aliases in the Makefile to a Makefile.test 
- ✅ Move tunnel aliases in the Makefile to a Makefile.tunnel 
- ✅ Move docker aliases in the Makefile to a Makefile.docker 
- ✅ Move some of the help aliases in the respective Makefile

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

- Refactor database helper functions for better maintainability. Postponed for now.
  - Consolidate similar database pool retrieval functions in `src/handlers/database_ops.rs`
  - Create generic functions to reduce code duplication
  - Improve error handling consistency across database operations
  - Add comprehensive documentation for database operation patterns

- On a show backup domain page add a button to change it from a backup domain to a normal domain. Postponed for now.
  - This might be a wizard to add aliases etc like the current wizards
