# Sorting Office TODOs

## Completed Tasks ✅

- [x] Remove `StatusCode::NOT_FOUND` as acceptable response from API tests
- [x] Fix `cargo fmt` issues (trailing whitespace in `tests/ui_containerized.rs`)
- [x] Fix `cargo clippy` warnings (unused variables in `src/handlers/wizard.rs`)
- [x] Rename backup handler and template files to align with path changes
  - [x] Renamed `backup.rs` to `database_backup.rs`
  - [x] Renamed `backups.rs` to `domain_backup.rs`
  - [x] Updated all module declarations and imports
  - [x] Updated template paths in Rust code
  - [x] Moved HTML template directories
- [x] Update configuration page description
- [x] Fix incorrect version displayed on about page
- [x] Improve test data utilities
  - [x] Added enhanced test data factories with randomization
  - [x] Added comprehensive form data generators
  - [x] Added test dataset generators (edge cases, security, Unicode)
  - [x] Added improved cleanup utilities
  - [x] Added test data management utilities
  - [x] Created comprehensive test suite for new utilities

## Current Tasks 🔄

- [ ] **Improve test organization** (IN PROGRESS - has compilation errors)
  - [ ] Group related tests into modules
  - [ ] Add more descriptive test names
  - [ ] Add test documentation
  - [ ] Fix compilation errors in integration tests

## Pending Tasks 📋

- [ ] Review ignored smoke test
  - [ ] Evaluate `ui_smoke_e2e_flow` for CI integration

## Future Improvements 🚀

- [ ] Add more comprehensive test coverage
- [ ] Implement performance benchmarks
- [ ] Add integration tests for new features
- [ ] Improve error handling in test utilities
- [ ] Add test data validation utilities
- [ ] Create test data persistence utilities
