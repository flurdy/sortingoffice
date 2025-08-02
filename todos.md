# TODO List

## 🎉 **COMPLETED: Major Test Infrastructure Improvements!** ✅

### **UI Test Improvements**
- **✅ Fixed Environment Smoke Test**: Resolved `net::ERR_NAME_NOT_RESOLVED` by using host bridge IP
- **✅ Fixed Domain/Alias/User Creation**: All HTMX forms now work perfectly with pagination
- **✅ Fixed Pagination Handling**: Created reusable `check_item_in_paginated_list` function
- **✅ Fixed Authentication Issues**: Added proper session handling and re-authentication
- **✅ Fixed Deletion Functions**: All deletion functions work with paginated lists and alerts
- **✅ Fixed Validation Issues**: Email/domain validation and password requirements working
- **✅ Eliminated Code Duplication**: Consolidated duplicate functions between test files
- **✅ Added Button IDs**: Improved Selenium reliability by adding IDs to wizard buttons
- **✅ Enhanced Wizard Test**: Added comprehensive verification of created resources

### **Code Quality Improvements**
- **✅ Cleaned up unused functions**: Removed unused macros and functions
- **✅ Fixed compilation warnings**: Resolved all compiler warnings
- **✅ Improved test reliability**: Better error handling and fallback mechanisms

## **Remaining Tasks** 📋

### **HIGH PRIORITY**

1. **✅ COMPLETED: Fix duplicate `get_current_db_info` function**
   - **Location**: Found duplicate in `src/handlers/users.rs` and `src/handlers/utils.rs`
   - **Action**: Removed private function from `users.rs`, imported public function from `utils.rs`
   - **Result**: Eliminated code duplication, all tests passing

### **MEDIUM PRIORITY**

2. **Consolidate translation functions**
   - **Issue**: Handlers may be duplicating translation fetching logic
   - **Action**: Create shared translation helper functions
   - **Benefit**: Reduce code duplication and improve maintainability

3. **Consolidate handler code duplication**
   - **Issue**: Some POST/PUT handlers may duplicate show/list code
   - **Action**: Extract shared logic into reusable functions
   - **Benefit**: Reduce code duplication and improve maintainability

### **LOW PRIORITY**

4. **Fix untranslated strings**
   - `validation-password-required`
   - `form-enabled`
   - **Action**: Add missing translations to locale files

5. **Consider bundling translation fetching**
   - **Action**: Create helper functions for common translation patterns
   - **Benefit**: Reduce boilerplate in handlers

## **Recent Achievements** 🏆

- **Wizard Test Enhancement**: Added comprehensive verification that all domains, aliases, and users are actually created
- **Button ID Implementation**: Added reliable IDs to wizard buttons for better Selenium test stability
- **Error Handling Improvements**: Made tests more robust with better error recovery and fallback mechanisms
- **Code Duplication Elimination**: Consolidated duplicate functions between `ui_containerized.rs` and `ui_helpers.rs`
