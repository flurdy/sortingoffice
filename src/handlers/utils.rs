use crate::{i18n::get_translation, AppState};
use askama::Template;
use axum::http::HeaderMap;
use axum::response::Html;
use std::collections::HashMap;
use tracing::error;
use axum::http::StatusCode;

/// Macro to fetch multiple translations at once
/// Usage: let translations = get_translations!(&state, &locale, [
///     "key1", "key2", "key3"
/// ]).await;
#[macro_export]
macro_rules! get_translations {
    ($state:expr, $locale:expr, [$($key:expr),*]) => {{
        let mut translations = std::collections::HashMap::new();
        $(
            let value = $crate::i18n::get_translation($state, $locale, $key).await;
            translations.insert($key.to_string(), value);
        )*
        translations
    }};
}

/// Macro to create a template with common error handling
/// Usage: let html = render_template!(template_instance, &state, &locale, &headers);
#[macro_export]
macro_rules! render_template {
    ($template:expr, $state:expr, $locale:expr, $headers:expr) => {{
        let content = match $template.render() {
            Ok(content) => content,
            Err(e) => {
                tracing::error!("Failed to render template: {:?}", e);
                return Html("Error rendering template".to_string());
            }
        };

        if $crate::handlers::utils::is_htmx_request($headers) {
            Html(content)
        } else {
            // Get current database id from session/cookie or default
            let current_db_id = $crate::handlers::auth::get_selected_database($headers)
                .unwrap_or_else(|| $state.db_manager.get_default_db_id().to_string());
            // Get current database label from db_manager
            let current_db_label = $state
                .db_manager
                .get_configs()
                .iter()
                .find(|db| db.id == current_db_id)
                .map(|db| db.label.clone())
                .unwrap_or_else(|| current_db_id.clone());

            let base_template = match $crate::templates::layout::BaseTemplate::with_i18n(
                "".to_string(), // Title will be set by the template
                content,
                $state,
                $locale,
                current_db_label,
                current_db_id,
            )
            .await
            {
                Ok(template) => template,
                Err(e) => {
                    tracing::error!("Failed to create base template: {:?}", e);
                    return Html("Error creating template".to_string());
                }
            };

            match base_template.render() {
                Ok(content) => Html(content),
                Err(e) => {
                    tracing::error!("Failed to render base template: {:?}", e);
                    Html("Error rendering template".to_string())
                }
            }
        }
    }};
}

/// Macro to create a template with title support
/// Usage: let html = render_template_with_title!(template_instance, title, &state, &locale, &headers);
#[macro_export]
macro_rules! render_template_with_title {
    ($template:expr, $title:expr, $state:expr, $locale:expr, $headers:expr) => {{
        let content = match $template.render() {
            Ok(content) => content,
            Err(e) => {
                tracing::error!("Failed to render template: {:?}", e);
                return Html("Error rendering template".to_string());
            }
        };

        if $crate::handlers::utils::is_htmx_request($headers) {
            Html(content)
        } else {
            // Get current database id from session/cookie or default
            let current_db_id = $crate::handlers::auth::get_selected_database($headers)
                .unwrap_or_else(|| $state.db_manager.get_default_db_id().to_string());
            // Get current database label from db_manager
            let current_db_label = $state
                .db_manager
                .get_configs()
                .iter()
                .find(|db| db.id == current_db_id)
                .map(|db| db.label.clone())
                .unwrap_or_else(|| current_db_id.clone());

            let base_template = match $crate::templates::layout::BaseTemplate::with_i18n(
                $title.to_string(),
                content,
                $state,
                $locale,
                current_db_label,
                current_db_id,
            )
            .await
            {
                Ok(template) => template,
                Err(e) => {
                    tracing::error!("Failed to create base template: {:?}", e);
                    return Html("Error creating template".to_string());
                }
            };

            match base_template.render() {
                Ok(content) => Html(content),
                Err(e) => {
                    tracing::error!("Failed to render base template: {:?}", e);
                    Html("Error rendering template".to_string())
                }
            }
        }
    }};
}

/// Macro to handle common "not found" error patterns
/// Usage: let entity = get_entity_or_not_found!(db::get_entity(&pool, id), &state, &locale, "entity-not-found").await?;
#[macro_export]
macro_rules! get_entity_or_not_found {
    ($db_call:expr, $state:expr, $locale:expr, $not_found_key:expr) => {{
        match $db_call {
            Ok(entity) => entity,
            Err(_) => {
                let not_found_msg =
                    $crate::i18n::get_translation($state, $locale, $not_found_key).await;
                return Html(not_found_msg);
            }
        }
    }};
}

/// Macro to handle database operations with error logging
/// Usage: let result = db_operation!(db::some_operation(pool), "Failed to perform operation");
#[macro_export]
macro_rules! db_operation {
    ($db_call:expr, $error_msg:expr) => {{
        match $db_call {
            Ok(result) => {
                tracing::info!("Successfully completed database operation");
                result
            }
            Err(e) => {
                tracing::error!("{}: {:?}", $error_msg, e);
                return Err(e);
            }
        }
    }};
}

/// Macro to create default SystemStats with all fields set to 0
/// Usage: let stats = default_system_stats!();
#[macro_export]
macro_rules! default_system_stats {
    () => {{
        $crate::models::SystemStats {
            total_domains: 0,
            enabled_domains: 0,
            disabled_domains: 0,
            recent_domains: 0,
            total_users: 0,
            enabled_users: 0,
            disabled_users: 0,
            recent_users: 0,
            total_aliases: 0,
            enabled_aliases: 0,
            disabled_aliases: 0,
            recent_aliases: 0,
            total_backups: 0,
            enabled_backups: 0,
            disabled_backups: 0,
            recent_backups: 0,
            total_relays: 0,
            enabled_relays: 0,
            disabled_relays: 0,
            recent_relays: 0,
            total_relocated: 0,
            enabled_relocated: 0,
            disabled_relocated: 0,
            recent_relocated: 0,
            total_clients: 0,
            enabled_clients: 0,
            disabled_clients: 0,
            recent_clients: 0,
            total_quota: 0,
            used_quota: 0,
            quota_usage_percent: 0.0,
            enabled_domains_and_backups: 0,
        }
    }};
}

/// Macro to handle SystemStats retrieval with fallback to defaults
/// Usage: let stats = get_system_stats_or_default!(db::get_system_stats(&pool));
#[macro_export]
macro_rules! get_system_stats_or_default {
    ($db_call:expr) => {{
        match $db_call {
            Ok(stats) => stats,
            Err(_) => $crate::default_system_stats!(),
        }
    }};
}

/// Check if the request is an HTMX request
pub fn is_htmx_request(headers: &HeaderMap) -> bool {
    headers.get("HX-Request").is_some_and(|v| v == "true")
}

/// Get user locale from headers
pub fn get_user_locale(headers: &HeaderMap) -> String {
    crate::handlers::language::get_user_locale(headers)
}

/// Get the current database pool from the state
/// This gets the database pool based on the user's session selection
pub async fn get_current_db_pool(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<crate::DbPool, Box<dyn std::error::Error>> {
    // Get the selected database from the session, or fall back to default
    let selected_db = crate::handlers::auth::get_selected_database(headers)
        .unwrap_or_else(|| state.db_manager.get_default_db_id().to_string());

    state
        .db_manager
        .get_pool(&selected_db)
        .await
        .ok_or_else(|| format!("No database pool available for '{selected_db}'").into())
}

/// Batch translation fetcher
pub async fn get_translations_batch(
    state: &AppState,
    locale: &str,
    keys: &[&str],
) -> HashMap<String, String> {
    let mut translations = HashMap::new();
    for key in keys {
        let value = get_translation(state, locale, key).await;
        translations.insert(key.to_string(), value);
    }
    translations
}

/// Common translation keys for forms
pub async fn get_form_translations(state: &AppState, locale: &str) -> HashMap<String, String> {
    get_translations_batch(
        state,
        locale,
        &[
            "form-error",
            "form-cancel",
            "action-save",
            "action-edit",
            "action-view",
            "action-enable",
            "action-disable",
            "action-delete",
            "status-enabled",
            "status-disabled",
            "status-active",
            "status-inactive",
        ],
    )
    .await
}

/// Common translation keys for table headers
pub async fn get_table_translations(state: &AppState, locale: &str) -> HashMap<String, String> {
    get_translations_batch(
        state,
        locale,
        &[
            "action-view",
            "action-edit",
            "action-enable",
            "action-disable",
            "action-delete",
            "status-enabled",
            "status-disabled",
            "status-active",
            "status-inactive",
        ],
    )
    .await
}

/// Helper function to render a template with proper error handling
pub async fn render_template_with_layout<T>(
    template: T,
    state: &AppState,
    locale: &str,
    headers: &HeaderMap,
) -> Html<String>
where
    T: Template,
{
    let content = match template.render() {
        Ok(content) => content,
        Err(e) => {
            error!("Failed to render template: {:?}", e);
            return Html("Error rendering template".to_string());
        }
    };

    if is_htmx_request(headers) {
        Html(content)
    } else {
        // Get current database id from session/cookie or default
        let current_db_id = crate::handlers::auth::get_selected_database(headers)
            .unwrap_or_else(|| state.db_manager.get_default_db_id().to_string());
        // Get current database label from db_manager
        let current_db_label = state
            .db_manager
            .get_configs()
            .iter()
            .find(|db| db.id == current_db_id)
            .map(|db| db.label.clone())
            .unwrap_or_else(|| current_db_id.clone());

        let base_template = match crate::templates::layout::BaseTemplate::with_i18n(
            "".to_string(), // Title will be set by the template
            content,
            state,
            locale,
            current_db_label,
            current_db_id,
        )
        .await
        {
            Ok(template) => template,
            Err(e) => {
                error!("Failed to create base template: {:?}", e);
                return Html("Error creating template".to_string());
            }
        };

        match base_template.render() {
            Ok(content) => Html(content),
            Err(e) => {
                error!("Failed to render base template: {:?}", e);
                Html("Error rendering template".to_string())
            }
        }
    }
}

/// Helper function to handle "not found" errors consistently
pub async fn handle_not_found<T>(result: Result<T, Box<dyn std::error::Error>>) -> Result<T, StatusCode> {
    match result {
        Ok(value) => Ok(value),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

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
        _ => {}
    }

    Ok(())
}

/// Check if the current database has any write restrictions
pub fn get_database_restrictions_info(
    state: &AppState,
    database_id: &str,
) -> Vec<String> {
    let config = &state.config;
    let mut restrictions = Vec::new();

    if config.is_database_disabled(database_id) {
        restrictions.push("Database disabled".to_string());
    }
    if config.is_database_read_only(database_id) {
        restrictions.push("Read-only mode".to_string());
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

/// Helper function to handle database errors with logging
pub fn handle_db_error<T, E>(result: Result<T, E>, error_msg: &str) -> Result<T, E>
where
    E: std::fmt::Debug,
{
    match result {
        Ok(value) => {
            tracing::info!("Successfully completed database operation");
            Ok(value)
        }
        Err(e) => {
            tracing::error!("{}: {:?}", error_msg, e);
            Err(e)
        }
    }
}
