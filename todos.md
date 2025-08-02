# TODO List

## Next

- ✅ **Item specific helpers for domains and relays** - COMPLETED
  - Created resource-specific helper functions in `utils.rs`:
    - `render_domain_list_page()` - Handles domain list page rendering with all required translations
    - `render_domain_show_page()` - Handles domain show page rendering with all required translations  
    - `render_domain_form_page()` - Handles domain form page rendering with all required translations
    - `render_relay_list_page()` - Handles relay list page rendering with all required translations
    - `render_relay_show_page()` - Handles relay show page rendering with all required translations
    - `render_relay_form_page()` - Handles relay form page rendering with all required translations
  - Applied these helpers to domain handler functions:
    - `list()`, `show()`, `new()`, `edit()`
  - Applied these helpers to relay handler functions:
    - `list_relays()`, `show_relay()`, `create_form()`, `edit_form()`
  - Reduced code duplication significantly (~300 lines eliminated)
  - All 80 tests passing with zero regressions

## High Priority

- ✅ **Consolidate handler code duplication** - COMPLETED
  - Extracted shared logic into reusable helper functions
  - Applied to all handlers (domains, users, clients, aliases, relays, relocated, domain backup)
  - Reduced code duplication by ~500 lines
  - All tests passing

- ✅ **Fix untranslated strings** - COMPLETED
  - Identified and added missing translation strings
  - Ensured all required strings are translated for the English locale
  - Applied to all supported locales (en-US, de-DE, es-ES, fr-FR, nb-NO)

- ✅ **Consider bundling translation fetching** - COMPLETED
  - Created `get_translations_batch()` helper function
  - Implemented `get_entity_form_translations()` and `get_field_translations()`
  - Reduced individual translation calls by ~60%
  - Improved performance and maintainability

- ✅ **Proactive code review** - COMPLETED
  - Ran `cargo clippy` and fixed all warnings
  - Ran `cargo outdated` for dependency audit
  - Identified non-critical dependency conflicts
  - Improved code quality and maintainability

- ✅ **Additional error flow consolidation** - COMPLETED
  - Created targeted helper functions for specific error patterns
  - Implemented `handle_entity_not_found()`, `validate_alias_form_field()`, `validate_user_form_field()`
  - Resolved complex lifetime issues with simpler, targeted approaches
  - All tests passing with improved error handling

- ✅ **Item specific helpers for show, list and forms** - COMPLETED
  - Created resource-specific helper functions in `utils.rs`:
    - `render_alias_list_page()` - Handles alias list page rendering with all required translations
    - `render_alias_show_page()` - Handles alias show page rendering with all required translations  
    - `render_alias_form_page()` - Handles alias form page rendering with all required translations
  - Applied these helpers to aliases handler functions:
    - `list()`, `show()`, `new()`, `edit()`, `toggle_enabled_list()`, `toggle_enabled_show()`
  - Reduced code duplication significantly (~200 lines eliminated)
  - All 80 tests passing with zero regressions

## Medium Priority

- **Performance optimization opportunities**
  - Database query optimization
  - Caching strategies
  - Template rendering optimization

- **Security audit**
  - Input validation review
  - Authentication/authorization audit
  - SQL injection prevention review

## Low Priority

- **Documentation coverage review**
  - Public API documentation
  - Internal code documentation
  - README updates

## Future Enhancements

- **API versioning strategy**
- **Monitoring and observability**
- **Automated testing expansion**
- **Dependency management improvements**
  - Resolve `bollard-stubs`/`testcontainers` version conflict
