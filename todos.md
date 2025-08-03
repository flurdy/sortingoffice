# TODO List

- ✅ All Github actions apart from smoke test still fail
  - https://github.com/flurdy/sortingoffice/actions/  


- ⚠️ Remaining: Deprecated Selenium methods (lower priority)


- Refactor long functions and complex logic
  - Consolidate similar validation logic across handlers
  - Simplify complex conditional logic in handlers

- Improve error handling and logging
  - Replace unwrap()/expect() with proper error handling
  - Add structured logging with context
  - Implement consistent error response patterns
  - Add error recovery mechanisms

- Performance optimizations
  - Reduce unnecessary cloning in database operations
  - Optimize database queries with proper indexing
  - Implement connection pooling improvements
  - Add caching for frequently accessed data

### Low Priority
- Code organization improvements
  - Extract magic numbers to named constants
  - Consolidate duplicate configuration logic
  - Improve module structure and separation of concerns
  - Add comprehensive documentation

- Testing enhancements
  - Add missing unit tests for utility functions
  - Improve test coverage for edge cases
  - Add integration tests for error scenarios
  - Implement property-based testing

- Security improvements
  - Audit and improve input validation
  - Add rate limiting for sensitive endpoints
  - Implement proper session management
  - Add security headers validation tests

### Technical Debt
- Remove dead code and unused imports
  - Clean up unused translation keys
  - Remove deprecated functions and modules
  - Consolidate duplicate utility functions
  - Remove commented-out code

- Configuration management
  - Extract hardcoded values to configuration
  - Implement environment-specific configs
  - Add configuration validation
  - Improve configuration documentation

- ✅ Are there any other obvious localisation issues? Are there some other key language for mail servers that we should add?
  - ✅ **Analysis completed**: Found 88+ missing translation keys and 27+ orphaned keys
  - ✅ **Current languages**: en-US, de-DE, fr-FR, es-ES, nb-NO (5 languages)
  - ✅ **Recommendations for additional languages**:
    - **High priority**: Dutch (nl-NL) - Very common in European hosting
    - **Medium priority**: Italian (it-IT), Portuguese (pt-BR/pt-PT), Russian (ru-RU)
    - **Lower priority**: Japanese (ja-JP), Chinese (zh-CN)
  - ✅ **Immediate issues to fix**: Add missing translation keys to all language files
  - ✅ **All missing translation keys added**: 81 keys added to en-US/messages.ftl
  - ✅ **All missing keys added to other languages**: 
    - ✅ German (de-DE): All keys added
    - ✅ French (fr-FR): All keys added  
    - ✅ Spanish (es-ES): All keys added
    - ✅ Norwegian (nb-NO): All keys added
  - ✅ **Orphaned keys removed**: 33 orphaned keys removed from all language files
  - ✅ **Duplicate keys removed**: All duplicate keys removed from all language files

- ✅ Sometimes refactoring has been done as an example but not applied to the rest of the codebase. This is not about the error handling 
  - ✅ **Investigation completed**: Found several helper functions that are not consistently used
  - ✅ **Template rendering safety**: Added `render_template_to_html` helper and started replacing `unwrap()` calls
  - ✅ **Template rendering refactoring completed**: Replaced all `unwrap()` calls in template rendering across all handlers with proper error handling
  - ✅ **Database operation helpers**: Applied `handle_entity_operation` and `handle_db_operation` consistently across handlers (reverted over-complicated changes in domains.rs)
  - ✅ **Template rendering helpers**: Extracted `render_backup_form_page_with_error` helper and refactored `domain_backup.rs` to use existing `render_backup_show_page` and `render_backup_form_page` helpers
  - ⚠️ **Form validation helpers**: Use `validate_form_and_handle_error` where applicable (complex due to lifetime issues)
  - ⚠️ **Pagination helpers**: Apply `get_entity_list_with_pagination` consistently (requires adapting for `PaginatedResult<T>` vs `Vec<T>`)

- Add a Dutch translations

- The utils.rs file is huge. Can it be split?

- Add a Italian translations

- Add a Portuguese translations
