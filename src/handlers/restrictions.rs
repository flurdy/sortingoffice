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
