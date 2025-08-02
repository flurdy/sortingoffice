# TODO List

## 🎉 **COMPLETED: Smoke Tests Fixed!** ✅

All major smoke test issues have been resolved! The environment smoke test now works perfectly:

- **✅ Environment Smoke Test**: Fully working
- **✅ Domain Creation**: Working with HTMX forms and pagination
- **✅ Alias Creation**: Working with HTMX forms and pagination  
- **✅ User Creation**: Working perfectly
- **✅ Cleanup**: Working with pagination and alert handling
- **✅ Pagination**: Reusable function for searching paginated lists
- **✅ Authentication**: Proper handling of expired sessions
- **✅ Alert Handling**: JavaScript confirmation dialogs handled correctly

## Completed Tasks ✅

- **✅ Fixed Environment Smoke Test**: 
  - Fixed `net::ERR_NAME_NOT_RESOLVED` by using host bridge IP
  - Added resilient app detection with 30-second timeout
  - Extracted app URL finding logic into separate function
  - Test now properly detects and connects to existing environment

- **✅ Fixed Domain Creation**: 
  - Fixed HTMX form loading and waiting
  - Fixed "element not interactable" issue for domain creation
  - Fixed domain verification with pagination support
  - Domain creation now works perfectly

- **✅ Fixed Alias Creation**: 
  - Fixed HTMX form loading and waiting
  - Fixed "element not interactable" issue for alias creation
  - Fixed alias verification with pagination support
  - Alias creation now works perfectly

- **✅ Fixed Pagination Handling**: 
  - Created reusable `check_item_in_paginated_list` function
  - Properly handles `/domains?page={{ page_num }}&per_page=25` URLs
  - Correctly searches through pages until item is found or no more pages
  - Applied to both domain and alias verification and deletion

- **✅ Fixed Authentication Issues**: 
  - Added authentication check and re-authentication logic to cleanup functions
  - Fixed "unexpected alert open" during user deletion
  - Added proper alert handling for confirmation dialogs

- **✅ Fixed Validation Issues**: 
  - Fixed email/domain validation by using lowercase-only domain names
  - Fixed password requirement in user creation
  - Added comprehensive post-creation validation checks

- **✅ Fixed Deletion Functions**: 
  - Fixed domain deletion to use pagination and handle alerts
  - Fixed alias deletion to use pagination and handle alerts
  - All deletion functions now work with paginated lists

## Remaining Tasks 📋

- **MEDIUM**: Clean up unused functions in ui_helpers.rs
- **MEDIUM**: Remove unused timeout10s macro
- **MEDIUM**: Consider consolidating duplicate setup functions between ui_smoke.rs and ui_containerized.rs
- **LOW**: Consider bundling translation fetching into helper functions
- **LOW**: Fix untranslated strings (validation-password-required, form-enabled)
