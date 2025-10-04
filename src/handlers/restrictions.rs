use crate::AppState;
use axum::http::StatusCode;

/// Check database feature restrictions and return error if operation is not allowed
pub fn check_database_restrictions(
    state: &AppState,
    database_id: &str,
    operation: &str,
) -> Result<(), StatusCode> {
    let config = &state.config;

    // Check if database is completely disabled
    if config.is_database_disabled(database_id) {
        tracing::warn!(
            "Operation '{}' blocked on database '{}': Database is disabled",
            operation,
            database_id
        );
        return Err(StatusCode::FORBIDDEN);
    }

    // Check read-only restriction
    if config.is_database_read_only(database_id) {
        tracing::warn!(
            "Operation '{}' blocked on database '{}': Database is read-only",
            operation,
            database_id
        );
        return Err(StatusCode::FORBIDDEN);
    }

    // Check specific operation restrictions
    match operation {
        "create_user" | "update_user" if config.is_new_users_blocked(database_id) => {
            tracing::warn!(
                "Operation '{}' blocked on database '{}': New users are not allowed",
                operation,
                database_id
            );
            return Err(StatusCode::FORBIDDEN);
        }
        "create_domain" | "update_domain" if config.is_new_domains_blocked(database_id) => {
            tracing::warn!(
                "Operation '{}' blocked on database '{}': New domains are not allowed",
                operation,
                database_id
            );
            return Err(StatusCode::FORBIDDEN);
        }
        "update_user" if config.is_password_updates_blocked(database_id) => {
            tracing::warn!(
                "Operation '{}' blocked on database '{}': Password updates are not allowed",
                operation,
                database_id
            );
            return Err(StatusCode::FORBIDDEN);
        }
        _ => {
            // Unknown operation, allow by default
        }
    }

    Ok(())
}

/// Get information about database restrictions for display purposes
pub fn get_database_restrictions_info(state: &AppState, database_id: &str) -> Vec<String> {
    let config = &state.config;
    let mut restrictions = Vec::new();

    if config.is_database_disabled(database_id) {
        restrictions.push("Database is completely disabled".to_string());
    }

    if config.is_database_read_only(database_id) {
        restrictions.push("Database is read-only".to_string());
    }

    if config.is_new_users_blocked(database_id) {
        restrictions.push("No new users".to_string());
    }
    if config.is_new_domains_blocked(database_id) {
        restrictions.push("No new domains".to_string());
    }
    if config.is_password_updates_blocked(database_id) {
        restrictions.push("No password updates".to_string());
    }

    restrictions
}

/// Helper function to check read-only restrictions and return HTML error page
pub async fn check_read_only_and_return_error(
    state: &crate::AppState,
    headers: &axum::http::HeaderMap,
    operation: &str,
) -> Option<axum::response::Html<String>> {
    let current_db_id = crate::handlers::auth::get_selected_database(headers)
        .unwrap_or_else(|| state.db_manager.get_default_db_id().to_string());

    if let Err(_status_code) = check_database_restrictions(state, &current_db_id, operation) {
        let locale = crate::handlers::language::get_user_locale(headers);
        let error_msg = crate::i18n::get_translation(state, &locale, "error-read-only-mode").await;
        return Some(axum::response::Html(format!(
            "<div class='text-center py-16'><h1 class='text-2xl font-bold text-red-600 mb-4'>Access Denied</h1><p class='text-lg text-gray-700 dark:text-gray-300'>{}</p></div>", 
            error_msg
        )));
    }

    None
}
