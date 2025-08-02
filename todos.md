# TODO List

## Completed Tasks ✅

- **Fixed the environment smoke test**
  - **Problem**: `net::ERR_NAME_NOT_RESOLVED` error when trying to connect to `http://host.docker.internal:3000`
  - **Solution**: Implemented dynamic host IP detection using `ip route get 8.8.8.8` and `ip addr show` commands
  - **Result**: Test now successfully connects to existing localhost:3000 application using bridge IP (e.g., `192.168.11.42:3000`)

- **Fixed the containerized smoke test cleanup issues**
  - **Problem**: Created items not appearing in list pages during cleanup, leading to "Element not found" errors
  - **Solution**: 
    - Added explicit navigation back to list pages after creation
    - Added assertions to verify items appear in lists after creation
    - Made cleanup functions stricter (fail if deletion fails instead of accepting gracefully)
    - Improved deletion functions to use main content area instead of full page source
  - **Result**: Test now properly verifies creation success and handles cleanup failures

- **Fixed user deletion in cleanup**
  - **Problem**: User deletion failing with "unexpected alert" error
  - **Solution**: Added proper JavaScript alert dialog handling in `delete_user` function
  - **Result**: User deletion now successfully handles confirmation dialogs

- **Fixed email validation in smoke tests**
  - **Problem**: Test email addresses failing validation due to uppercase letters in domain names
  - **Solution**: Created `rand_domain_str()` function that generates only lowercase letters and numbers
  - **Result**: Test domains now pass strict email validation rules

- **Fixed user creation in smoke tests**
  - **Problem**: User creation failing with "validation-password-required" error
  - **Solution**: Added password field to user creation form
  - **Result**: User creation now works successfully

- **Improved alias creation handler architecture**
  - **Problem**: Massive code duplication in alias create handler recreating domain show page
  - **Solution**: 
    - Created shared `render_domain_show_page()` function in domains handler
    - Added `redirect_to` parameter to `AliasForm` to support smart redirect logic
    - Refactored alias create handler to use shared function and implement redirect logic
  - **Result**: Much cleaner, more maintainable code following DRY principle

- **Fixed authentication issues during cleanup**
  - **Problem**: Cleanup functions failing due to expired authentication sessions
  - **Solution**: Added authentication verification and re-authentication logic to all deletion functions
  - **Result**: User deletion now works perfectly with proper authentication handling

## Current Issues 🔍

- **CRITICAL**: Fix alias creation form interaction issue
  - **Problem**: Alias input field is "not interactable" during alias creation
  - **Root Cause**: Same HTMX form interaction issue that we fixed for domain creation
  - **Evidence**: 
    - ✅ Domain creation is now working perfectly
    - ✅ Domain verification is working
    - ✅ Test progresses to alias creation step
    - ❌ Alias creation fails with "element not interactable" error
  - **Next Steps**:
    1. Apply the same HTMX form handling fixes to alias creation
    2. Add proper waiting for HTMX form loading
    3. Use the same form interaction approach that worked for domains

- **CRITICAL**: Fix database visibility issue in creation/deletion
  - **Problem**: Items are created successfully and visible in specific contexts (e.g., domain show page), but not visible in list pages during cleanup
  - **Root Cause**: This appears to be a database transaction or context issue where items are created in one database context but not visible in list operations
  - **Evidence**: 
    - Aliases are created and visible in domain show page ✅
    - Aliases are not found in aliases list page ❌
    - Domains are created and visible in domain show page ✅
    - Domains are not found in domains list page ❌
    - Users are created and visible in users list page ✅
  - **Next Steps**:
    1. Investigate database connection handling in list operations vs show operations
    2. Check if there are transaction isolation issues
    3. Verify database context is consistent between creation and list operations
    4. Consider adding database connection debugging to understand the issue

## Future/Remaining Tasks 📋

- See where the UI tests are failing and fix them
- Fix the database visibility issue in the cleanup functions
- Make sure both UI and smoke tests use the helpers and don't double up on similar functions
- Remove unused `timeout10s` macro
- Consider consolidating duplicate setup functions between `ui_smoke.rs` and `ui_containerized.rs`
- If a similarly named function exists in several files, perhaps consider moving it to a helper function
- Clean up unused functions in `ui_helpers.rs`
- Consider bundling translation fetching into helper functions to reduce repetitive code
- validation-password-required seems untranslated
