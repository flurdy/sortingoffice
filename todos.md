# TODO List

### High Priority

- 

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

- ✅ Duplicate domain - add tests

- ✅ The status in Add/Edit Relays and Client since it can only be OK or REJECT, it should probably be radio buttons. Or a better toggle?

- ✅ UI tests 
  - ✅ Does the run_single_ui_test function in the run_tests.sh also allow running a test in thewizard_tests.rs?
  - ✅ Does the run_ui_test function in the run_tests.sh include the wizard tests

- ✅ DRY UI tests - Make sure the two wizard UI test files uses the shared helper functions, and not duplicating their own functions that does the same.

- Refactor database helper functions for better maintainability. Postponed for now.
  - Consolidate similar database pool retrieval functions in `src/handlers/database_ops.rs`
  - Create generic functions to reduce code duplication
  - Improve error handling consistency across database operations
  - Add comprehensive documentation for database operation patterns

- ✅ Smoke test
  - ✅ In run_test.sh the run_smoke_test function seem to check for localhost, even it that is not the environment to smoke test.
  - ✅ Instead can the makefile alias `test-smoke` take one argument which is the URL to test, instead of the env var.

- ✅ Some of the readme and docs in /docs lack a Table of Contents

## 🐛 Bugs and KNOWN ISSUES
