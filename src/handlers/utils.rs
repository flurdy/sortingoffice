use crate::models::PaginatedResult;
use crate::{i18n::get_translation, AppState};
use askama::Template;
use axum::http::HeaderMap;
use axum::http::StatusCode;
use axum::response::Html;
use diesel::result::Error;
use std::collections::HashMap;
use tracing::debug;
use tracing::error;
use tracing::info;

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

/// Helper function to fetch multiple form-related translations at once
pub async fn get_entity_form_translations(
    state: &AppState,
    locale: &str,
    entity: &str,
) -> HashMap<String, String> {
    let mut translations = HashMap::new();

    // Load common form translations
    let common_keys = vec![
        "form-error",
        "form-cancel",
        "action-save",
        "action-cancel",
        "form-enabled",
        "form-disabled",
        "form-create-domain",
        "form-update-domain",
        "form-placeholder-domain",
        "form-placeholder-transport",
        "form-tooltip-domain",
        "form-tooltip-transport",
        "form-tooltip-enable",
    ];

    for key in common_keys {
        let value = get_translation(state, locale, key).await;
        translations.insert(key.to_string(), value);
    }

    // Generate entity-specific keys
    let singular = if entity == "aliases" {
        "alias"
    } else {
        entity.trim_end_matches('s')
    };

    let entity_keys = vec![
        format!("{entity}-add-title"),
        format!("{entity}-edit-title"),
        format!("{entity}-new-{singular}"),
        format!("{entity}-edit-{singular}"),
    ];

    for key in entity_keys {
        let value = get_translation(state, locale, &key).await;
        translations.insert(key, value);
    }

    debug!("Final translations map: {translations:#?}");
    translations
}

/// Helper function to fetch field-related translations
pub async fn get_field_translations(
    state: &AppState,
    locale: &str,
    entity: &str,
    fields: &[&str],
) -> HashMap<String, String> {
    let mut translations = HashMap::new();

    for field in fields {
        let field_keys = [
            format!("{entity}-field-{field}"),
            format!("{entity}-field-{field}-help"),
            format!("{entity}-placeholder-{field}"),
        ];

        for key in field_keys {
            let value = get_translation(state, locale, &key).await;
            translations.insert(key, value);
        }
    }

    translations
}

/// Helper function to fetch status-related translations
pub async fn get_status_translations(
    state: &AppState,
    locale: &str,
    entity: &str,
) -> HashMap<String, String> {
    let mut translations = HashMap::new();

    // Entity-specific status keys
    let entity_status_keys = if entity == "clients" {
        vec![
            format!("{entity}-status-ok"),
            format!("{entity}-status-reject"),
            format!("{entity}-status-enabled"),
            format!("{entity}-status-disabled"),
            format!("{entity}-enabled-yes"),
            format!("{entity}-enabled-no"),
        ]
    } else {
        vec![
            format!("{entity}-status-allowed"),
            format!("{entity}-status-blocked"),
            format!("{entity}-status-enabled"),
            format!("{entity}-status-disabled"),
            format!("{entity}-enabled-yes"),
            format!("{entity}-enabled-no"),
        ]
    };

    // Common status keys
    let common_status_keys = [
        "status-enabled",
        "status-disabled",
        "status-active",
        "status-inactive",
        "status-ok",
        "status-reject",
    ];

    // Combine entity-specific and common status keys
    let all_keys = [
        entity_status_keys,
        common_status_keys.iter().map(|s| s.to_string()).collect(),
    ]
    .concat();

    for key in all_keys {
        let value = get_translation(state, locale, &key).await;
        translations.insert(key, value);
    }

    translations
}

/// Helper function to fetch action-related translations
pub async fn get_action_translations(
    state: &AppState,
    locale: &str,
    entity: &str,
) -> HashMap<String, String> {
    let mut translations = HashMap::new();

    // Entity-specific action keys
    let entity_action_keys = [
        format!("{entity}-action-view"),
        format!("{entity}-action-edit"),
        format!("{entity}-action-enable"),
        format!("{entity}-action-disable"),
        format!("{entity}-action-delete"),
        format!("{entity}-action-cancel"),
        format!("{entity}-delete-confirm"),
    ];

    // Common action keys
    let common_action_keys = [
        "action-view",
        "action-edit",
        "action-enable",
        "action-disable",
        "action-delete",
        "action-save",
        "action-cancel",
    ];

    // Combine entity-specific and common action keys
    let all_keys = [
        entity_action_keys
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
        common_action_keys.iter().map(|s| s.to_string()).collect(),
    ]
    .concat();

    for key in all_keys {
        let value = get_translation(state, locale, &key).await;
        translations.insert(key, value);
    }

    translations
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
pub async fn handle_not_found<T>(
    result: Result<T, Box<dyn std::error::Error>>,
) -> Result<T, StatusCode> {
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
pub fn get_database_restrictions_info(state: &AppState, database_id: &str) -> Vec<String> {
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

/// Helper function to handle database errors consistently
pub async fn handle_database_error(
    state: &AppState,
    locale: &str,
    error: diesel::result::Error,
    entity: &str,
    identifier: &str,
) -> String {
    match error {
        diesel::result::Error::DatabaseError(
            diesel::result::DatabaseErrorKind::UniqueViolation,
            _,
        ) => {
            let key = format!("error-duplicate-{entity}");
            get_translation(state, locale, &key)
                .await
                .replace("{identifier}", identifier)
        }
        diesel::result::Error::DatabaseError(
            diesel::result::DatabaseErrorKind::CheckViolation,
            _,
        ) => get_translation(state, locale, "error-constraint-violation").await,
        _ => get_translation(state, locale, "error-unexpected").await,
    }
}

/// Helper function to create BaseTemplate with common pattern
pub async fn create_base_template(
    state: &AppState,
    locale: &str,
    title: String,
    content: String,
    headers: &HeaderMap,
) -> Result<Html<String>, Box<dyn std::error::Error>> {
    let current_db_id = crate::handlers::auth::get_selected_database(headers)
        .unwrap_or_else(|| state.db_manager.get_default_db_id().to_string());
    let current_db_label = state
        .db_manager
        .get_configs()
        .iter()
        .find(|db| db.id == current_db_id)
        .map(|db| db.label.clone())
        .unwrap_or_else(|| current_db_id.clone());

    match crate::templates::layout::BaseTemplate::with_i18n(
        title,
        content,
        state,
        locale,
        current_db_label,
        current_db_id,
    )
    .await
    {
        Ok(template) => Ok(Html(template.render()?)),
        Err(e) => {
            error!("Failed to create BaseTemplate with i18n: {:?}", e);
            Err(e)
        }
    }
}

/// Helper function to render form template with error handling
pub async fn render_form_template<T>(
    template: T,
    state: &AppState,
    locale: &str,
    headers: &HeaderMap,
    title: String,
) -> Html<String>
where
    T: askama::Template,
{
    let content = match template.render() {
        Ok(content) => content,
        Err(e) => {
            error!("Failed to render form template: {:?}", e);
            return Html("Error rendering template".to_string());
        }
    };

    if is_htmx_request(headers) {
        Html(content)
    } else {
        match create_base_template(state, locale, title, content, headers).await {
            Ok(html) => html,
            Err(e) => {
                error!(
                    "Failed to create base template in render_form_template: {:?}",
                    e
                );
                Html("Error creating template".to_string())
            }
        }
    }
}

/// Helper function to get current database info
pub async fn get_current_db_info(state: &AppState, headers: &HeaderMap) -> (String, String) {
    let current_db_id = crate::handlers::auth::get_selected_database(headers)
        .unwrap_or_else(|| state.db_manager.get_default_db_id().to_string());
    let current_db_label = state
        .db_manager
        .get_configs()
        .iter()
        .find(|db| db.id == current_db_id)
        .map(|db| db.label.clone())
        .unwrap_or_else(|| current_db_id.clone());

    (current_db_label, current_db_id)
}

pub async fn render_list_template<T>(
    template: T,
    state: &AppState,
    locale: &str,
    headers: &HeaderMap,
) -> Html<String>
where
    T: Template,
{
    let content = template.render().unwrap();

    if is_htmx_request(headers) {
        Html(content)
    } else {
        let current_db_id = crate::handlers::auth::get_selected_database(headers)
            .unwrap_or_else(|| state.db_manager.get_default_db_id().to_string());
        let current_db_label = state
            .db_manager
            .get_configs()
            .iter()
            .find(|db| db.id == current_db_id)
            .map(|db| db.label.clone())
            .unwrap_or_else(|| current_db_id.clone());
        let template = crate::templates::layout::BaseTemplate::with_i18n(
            get_translation(state, locale, "aliases-title").await,
            content,
            state,
            locale,
            current_db_label,
            current_db_id,
        )
        .await
        .unwrap();
        Html(template.render().unwrap())
    }
}

pub async fn render_show_template<T>(
    template: T,
    state: &AppState,
    locale: &str,
    headers: &HeaderMap,
) -> Html<String>
where
    T: Template,
{
    let content = template.render().unwrap();

    if is_htmx_request(headers) {
        Html(content)
    } else {
        let current_db_id = crate::handlers::auth::get_selected_database(headers)
            .unwrap_or_else(|| state.db_manager.get_default_db_id().to_string());
        let current_db_label = state
            .db_manager
            .get_configs()
            .iter()
            .find(|db| db.id == current_db_id)
            .map(|db| db.label.clone())
            .unwrap_or_else(|| current_db_id.clone());
        let template = crate::templates::layout::BaseTemplate::with_i18n(
            get_translation(state, locale, "aliases-show-title").await,
            content,
            state,
            locale,
            current_db_label,
            current_db_id,
        )
        .await
        .unwrap();
        Html(template.render().unwrap())
    }
}

/// Helper function to fetch entity-specific translations for list pages
pub async fn get_entity_list_translations(
    state: &AppState,
    locale: &str,
    entity: &str,
) -> HashMap<String, String> {
    let mut translations = HashMap::new();

    // Common entity list translations
    let entity_keys = vec![
        format!("{entity}-title"),
        format!("{entity}-add"),
        format!("{entity}-list-description"),
        format!("{entity}-empty-title"),
        format!("{entity}-empty-description"),
        format!("{entity}-not-found"),
        format!("{entity}-not-available"),
    ];

    // Table header translations
    let table_header_keys = vec![
        format!("{entity}-table-header-id"),
        format!("{entity}-table-header-name"),
        format!("{entity}-table-header-domain"),
        format!("{entity}-table-header-email"),
        format!("{entity}-table-header-enabled"),
        format!("{entity}-table-header-status"),
        format!("{entity}-table-header-actions"),
        format!("{entity}-table-header-created"),
        format!("{entity}-table-header-modified"),
    ];

    // Combine all keys
    let all_keys = [entity_keys, table_header_keys].concat();

    for key in all_keys {
        let value = get_translation(state, locale, &key).await;
        translations.insert(key, value);
    }

    translations
}

/// Helper function to fetch entity-specific translations for show pages
pub async fn get_entity_show_translations(
    state: &AppState,
    locale: &str,
    entity: &str,
) -> HashMap<String, String> {
    let mut translations = HashMap::new();

    // Common show page translations
    let show_keys = vec![
        format!("{entity}-show-title"),
        format!("{entity}-show-title-label"),
        format!("{entity}-info-title"),
        format!("{entity}-info-description"),
        format!("{entity}-view-edit-settings"),
        format!("{entity}-back-to-list"),
        format!("{entity}-back-to-{entity}"),
        format!("{entity}-delete-confirm"),
    ];

    // Field translations
    let field_keys = vec![
        format!("{entity}-field-id"),
        format!("{entity}-field-name"),
        format!("{entity}-field-domain"),
        format!("{entity}-field-email"),
        format!("{entity}-field-enabled"),
        format!("{entity}-field-status"),
        format!("{entity}-field-created"),
        format!("{entity}-field-modified"),
        format!("{entity}-field-recipient"),
        format!("{entity}-field-old-address"),
        format!("{entity}-field-new-address"),
    ];

    // Combine all keys
    let all_keys = [show_keys, field_keys].concat();

    for key in all_keys {
        let value = get_translation(state, locale, &key).await;
        translations.insert(key, value);
    }

    translations
}

/// Helper function to fetch entity-specific error translations
pub async fn get_entity_error_translations(
    state: &AppState,
    locale: &str,
    entity: &str,
) -> HashMap<String, String> {
    let mut translations = HashMap::new();

    let error_keys = vec![
        format!("{entity}-create-error"),
        format!("{entity}-update-error"),
        format!("{entity}-delete-error"),
        format!("{entity}-toggle-error"),
        format!("{entity}-not-found"),
        format!("{entity}-not-available"),
    ];

    for key in error_keys {
        let value = get_translation(state, locale, &key).await;
        translations.insert(key, value);
    }

    translations
}

/// Helper function to fetch all common translations for an entity
pub async fn get_entity_all_translations(
    state: &AppState,
    locale: &str,
    entity: &str,
) -> HashMap<String, String> {
    let mut all_translations = HashMap::new();

    // Get all entity-specific translations
    let list_translations = get_entity_list_translations(state, locale, entity).await;
    let show_translations = get_entity_show_translations(state, locale, entity).await;
    let error_translations = get_entity_error_translations(state, locale, entity).await;
    let form_translations = get_entity_form_translations(state, locale, entity).await;

    // Get common translations
    let status_translations = get_status_translations(state, locale, entity).await;
    let action_translations = get_action_translations(state, locale, entity).await;

    // Merge all translations
    all_translations.extend(list_translations);
    all_translations.extend(show_translations);
    all_translations.extend(error_translations);
    all_translations.extend(form_translations);
    all_translations.extend(status_translations);
    all_translations.extend(action_translations);

    all_translations
}

/// Helper function to fetch login-related translations
pub async fn get_login_translations(state: &AppState, locale: &str) -> HashMap<String, String> {
    let login_keys = vec![
        "login-title",
        "login-user-id",
        "login-password",
        "login-sign-in",
        "login-error-empty-fields",
        "login-error-invalid-credentials",
        "app-title",
        "app-subtitle",
        "language-selector",
        "theme-toggle",
        "language-english",
        "language-spanish",
        "language-french",
        "language-norwegian",
        "language-german",
    ];

    let mut translations = HashMap::new();
    for key in login_keys {
        let value = get_translation(state, locale, key).await;
        translations.insert(key.to_string(), value);
    }

    translations
}

/// Helper function to fetch reports-related translations
pub async fn get_reports_translations(state: &AppState, locale: &str) -> HashMap<String, String> {
    let reports_keys = vec![
        "reports-cross-db-matrix-title",
        "reports-cross-db-matrix-description",
        "reports-domain-header",
        "reports-database-header",
        "reports-primary-domain",
        "reports-backup-domain",
        "reports-not-present",
        "reports-legend-title",
        "reports-no-domains",
        "reports-no-domains-description",
        "reports-list-title",
        "reports-list-description",
        "reports-matrix-title",
        "reports-matrix-description",
        "reports-orphaned-aliases-title",
        "reports-orphaned-aliases-description",
        "reports-external-forwarders-title",
        "reports-external-forwarders-description",
        "reports-alias-cross-domain-title",
        "reports-alias-cross-domain-description",
        "reports-cross-db-user-distribution-title",
        "reports-cross-db-user-distribution-description",
        "reports-cross-db-feature-toggle-title",
        "reports-cross-db-feature-toggle-description",
        "reports-cross-db-migration-title",
        "reports-cross-db-migration-description",
        "reports-view-report",
        "reports-user-header",
        "reports-present",
        "reports-no-users",
        "reports-no-users-description",
        "reports-disabled",
        "reports-database-status-header",
        "reports-read-only",
        "reports-no-new-users",
        "reports-no-new-domains",
        "reports-no-password-updates",
        "reports-enabled",
        "reports-status-header",
        "reports-last-migration-header",
        "reports-migration-count-header",
    ];

    let mut translations = HashMap::new();
    for key in reports_keys {
        let value = get_translation(state, locale, key).await;
        translations.insert(key.to_string(), value);
    }

    translations
}

/// Helper function to fetch not-found related translations
pub async fn get_not_found_translations(state: &AppState, locale: &str) -> HashMap<String, String> {
    let not_found_keys = vec!["not-found-title", "not-found-message"];

    let mut translations = HashMap::new();
    for key in not_found_keys {
        let value = get_translation(state, locale, key).await;
        translations.insert(key.to_string(), value);
    }

    translations
}

/// Helper function to fetch pagination-related translations
pub async fn get_pagination_translations(
    state: &AppState,
    locale: &str,
) -> HashMap<String, String> {
    let pagination_keys = vec![
        "pagination-previous",
        "pagination-next",
        "pagination-showing",
        "pagination-to",
        "pagination-of",
        "pagination-results",
    ];

    let mut translations = HashMap::new();
    for key in pagination_keys {
        let value = get_translation(state, locale, key).await;
        translations.insert(key.to_string(), value);
    }

    translations
}

/// Helper function to get database pool with consistent error handling
pub async fn get_db_pool_or_error(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<crate::DbPool, Html<String>> {
    match get_current_db_pool(state, headers).await {
        Ok(pool) => Ok(pool),
        Err(e) => {
            error!("Failed to get database pool: {:?}", e);
            Err(Html("Database connection error".to_string()))
        }
    }
}

/// Helper function to get entity with consistent not-found handling
pub async fn get_entity_or_handle_error<T, F, Fut>(
    entity_fetch: F,
    state: &AppState,
    locale: &str,
    not_found_key: &str,
) -> Result<T, Html<String>>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, Error>>,
{
    match entity_fetch().await {
        Ok(entity) => Ok(entity),
        Err(_) => {
            let not_found_msg = get_translation(state, locale, not_found_key).await;
            Err(Html(not_found_msg))
        }
    }
}

/// Helper function to get database pool with consistent error handling for redirect handlers
pub async fn get_db_pool_or_redirect_error(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<crate::DbPool, (StatusCode, String)> {
    match get_current_db_pool(state, headers).await {
        Ok(pool) => Ok(pool),
        Err(e) => {
            error!("Failed to get database pool: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database connection error".to_string(),
            ))
        }
    }
}

/// Helper function to handle entity operations with consistent error handling
pub async fn handle_entity_operation<T, F, Fut>(
    operation: F,
    state: &AppState,
    locale: &str,
    entity_name: &str,
    identifier: &str,
    success_message: &str,
) -> Result<T, Html<String>>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, Error>>,
{
    match operation().await {
        Ok(result) => {
            info!("{}: {}", success_message, identifier);
            Ok(result)
        }
        Err(e) => {
            error!("Failed to {} {}: {:?}", entity_name, identifier, e);
            let error_message =
                handle_database_error(state, locale, e, entity_name, identifier).await;
            Err(Html(error_message))
        }
    }
}

/// Helper function to handle entity operations with consistent error handling for redirect handlers
pub async fn handle_entity_operation_redirect<T, F, Fut>(
    operation: F,
    state: &AppState,
    entity_name: &str,
    identifier: &str,
    success_message: &str,
) -> Result<T, (StatusCode, String)>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, Error>>,
{
    match operation().await {
        Ok(result) => {
            info!("{}: {}", success_message, identifier);
            Ok(result)
        }
        Err(e) => {
            error!("Failed to {} {}: {:?}", entity_name, identifier, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to {entity_name} {identifier}"),
            ))
        }
    }
}

/// Helper function to validate form and handle errors consistently
pub async fn validate_form_and_handle_error<F, V, E>(
    _state: &AppState,
    form: &F,
    validator: V,
    state: &AppState,
    locale: &str,
    headers: &HeaderMap,
    error_handler: E,
) -> Result<(), Html<String>>
where
    F: Clone,
    V: FnOnce(&F) -> Result<(), String>,
    E: FnOnce(
        &AppState,
        &str,
        &HeaderMap,
        F,
        &str,
        bool,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Html<String>> + Send>>,
{
    if let Err(error_key) = validator(form) {
        let form_clone = form.clone();
        return Err(error_handler(state, locale, headers, form_clone, &error_key, true).await);
    }
    Ok(())
}

/// Helper function to get entity list with pagination
pub async fn get_entity_list_with_pagination<T, F, Fut>(
    entity_fetch: F,
    pool: &crate::DbPool,
    page: i64,
    per_page: i64,
) -> Result<PaginatedResult<T>, Html<String>>
where
    F: FnOnce(&crate::DbPool) -> Fut,
    Fut: std::future::Future<Output = Result<Vec<T>, Error>>,
    T: Clone,
{
    match entity_fetch(pool).await {
        Ok(entities) => {
            let total = entities.len() as i64;
            Ok(PaginatedResult::new(entities, total, page, per_page))
        }
        Err(e) => {
            error!("Failed to retrieve entity list: {:?}", e);
            Err(Html("Failed to retrieve data".to_string()))
        }
    }
}

/// Helper function to handle entity not found errors consistently
pub async fn handle_entity_not_found(
    state: &AppState,
    headers: &HeaderMap,
    entity_name: &str,
    not_found_key: &str,
) -> Html<String> {
    let locale = get_user_locale(headers);
    let not_found_msg = get_translation(state, &locale, not_found_key).await;
    Html(not_found_msg)
}

/// Helper function to validate alias form field and return error template if validation fails
pub async fn validate_alias_form_field<F>(
    state: &AppState,
    headers: &HeaderMap,
    form: &crate::models::AliasForm,
    validator: F,
    error_key: &str,
) -> Result<(), Html<String>>
where
    F: FnOnce(&crate::models::AliasForm) -> Result<(), crate::validation::ValidationError>,
{
    match validator(form) {
        Ok(_) => Ok(()),
        Err(_) => {
            let locale = get_user_locale(headers);
            let error_msg = get_translation(state, &locale, error_key).await;

            // Get form translations
            let form_translations = get_entity_form_translations(state, &locale, "aliases").await;
            let field_translations = get_field_translations(
                state,
                &locale,
                "aliases",
                &["mail", "destination", "active"],
            )
            .await;

            let content_template = crate::templates::aliases::AliasFormTemplate {
                title: &form_translations["aliases-add-title"],
                alias: None,
                form: form.clone(),
                error: Some(error_msg),
                return_url: form.return_url.clone(),
                edit_alias: &form_translations["aliases-edit-alias"],
                new_alias: &form_translations["aliases-new-alias"],
                form_error: &form_translations["form-error"],
                mail_address: &field_translations["aliases-field-mail"],
                destination: &field_translations["aliases-field-destination"],
                placeholder_mail: &field_translations["aliases-placeholder-mail"],
                placeholder_destination: &field_translations["aliases-placeholder-destination"],
                tooltip_mail: &field_translations["aliases-field-mail-help"],
                tooltip_destination: &field_translations["aliases-field-destination-help"],
                active: &field_translations["aliases-field-active"],
                tooltip_active: &field_translations["aliases-field-active-help"],
                cancel: &form_translations["form-cancel"],
                update_alias: &form_translations["action-save"],
                create_alias: &form_translations["action-save"],
            };

            let error_html = render_form_template(
                content_template,
                state,
                &locale,
                headers,
                form_translations["aliases-add-title"].clone(),
            )
            .await;
            Err(error_html)
        }
    }
}

/// Helper function to validate user form field and return error template if validation fails
pub async fn validate_user_form_field<F>(
    state: &AppState,
    headers: &HeaderMap,
    form: &crate::models::UserForm,
    validator: F,
    error_key: &str,
) -> Result<(), Html<String>>
where
    F: FnOnce(&crate::models::UserForm) -> Result<(), crate::validation::ValidationError>,
{
    match validator(form) {
        Ok(_) => Ok(()),
        Err(_) => {
            let locale = get_user_locale(headers);
            let error_msg = get_translation(state, &locale, error_key).await;

            // Build user form template with error
            let form_template = crate::handlers::users::build_user_form_template(
                state,
                &locale,
                None,
                form.clone(),
                Some(error_msg),
            )
            .await;
            let content = form_template.render().unwrap();

            if is_htmx_request(headers) {
                Err(Html(content))
            } else {
                let (current_db_label, current_db_id) = get_current_db_info(state, headers).await;
                let template = crate::templates::layout::BaseTemplate::with_i18n(
                    get_translation(state, &locale, "users-new-user").await,
                    content,
                    state,
                    &locale,
                    current_db_label,
                    current_db_id,
                )
                .await
                .unwrap();
                Err(Html(template.render().unwrap()))
            }
        }
    }
}

/// Resource-specific helper functions for Aliases
pub async fn render_alias_list_page(
    aliases: Vec<crate::models::Alias>,
    paginated: &crate::models::PaginatedResult<crate::models::Alias>,
    state: &AppState,
    locale: &str,
    headers: &HeaderMap,
) -> Html<String> {
    // Fetch all required translations for alias list
    let title = get_translation(state, locale, "aliases-title").await;
    let description = get_translation(state, locale, "aliases-description").await;
    let add_alias = get_translation(state, locale, "aliases-add").await;
    let table_header_mail = get_translation(state, locale, "aliases-table-header-mail").await;
    let table_header_destination = get_translation(state, locale, "aliases-table-header-destination").await;
    let table_header_domain = get_translation(state, locale, "aliases-table-header-domain").await;
    let table_header_enabled = get_translation(state, locale, "aliases-table-header-enabled").await;
    let table_header_actions = get_translation(state, locale, "aliases-table-header-actions").await;
    let status_active = get_translation(state, locale, "status-active").await;
    let status_inactive = get_translation(state, locale, "status-inactive").await;
    let action_view = get_translation(state, locale, "action-view").await;
    let enable_alias = get_translation(state, locale, "aliases-enable-alias").await;
    let disable_alias = get_translation(state, locale, "aliases-disable-alias").await;
    let empty_title = get_translation(state, locale, "aliases-empty-title").await;
    let empty_description = get_translation(state, locale, "aliases-empty-description").await;

    let page_range: Vec<i64> = (1..=paginated.total_pages).collect();
    let max_item = std::cmp::min(
        paginated.current_page * paginated.per_page,
        paginated.total_count,
    );

    let content_template = crate::templates::aliases::AliasesListTemplate {
        title: &title,
        aliases: &aliases,
        pagination: paginated,
        page_range: &page_range,
        max_item,
        description: &description,
        add_alias: &add_alias,
        table_header_mail: &table_header_mail,
        table_header_domain: &table_header_domain,
        table_header_destination: &table_header_destination,
        table_header_enabled: &table_header_enabled,
        table_header_actions: &table_header_actions,
        status_active: &status_active,
        status_inactive: &status_inactive,
        action_view: &action_view,
        enable_alias: &enable_alias,
        disable_alias: &disable_alias,
        empty_title: &empty_title,
        empty_description: &empty_description,
        current_sort_by: "mail",
        current_sort_order: "asc",
    };

    render_list_template(content_template, state, locale, headers).await
}

pub async fn render_alias_show_page(
    alias: crate::models::Alias,
    domain_info: Option<crate::models::Domain>,
    state: &AppState,
    locale: &str,
    headers: &HeaderMap,
) -> Html<String> {
    // Fetch all required translations for alias show
    let title = get_translation(state, locale, "aliases-show-title").await;
    let view_edit_settings = get_translation(state, locale, "aliases-view-edit-settings").await;
    let back_to_aliases = get_translation(state, locale, "aliases-back-to-aliases").await;
    let alias_information = get_translation(state, locale, "aliases-alias-information").await;
    let alias_details = get_translation(state, locale, "aliases-alias-details").await;
    let mail = get_translation(state, locale, "aliases-mail").await;
    let forward_to = get_translation(state, locale, "aliases-forward-to").await;
    let domain = get_translation(state, locale, "aliases-domain").await;
    let status = get_translation(state, locale, "aliases-status").await;
    let status_active = get_translation(state, locale, "status-active").await;
    let status_inactive = get_translation(state, locale, "status-inactive").await;
    let created = get_translation(state, locale, "aliases-created").await;
    let modified = get_translation(state, locale, "aliases-modified").await;
    let edit_alias_button = get_translation(state, locale, "aliases-edit-alias-button").await;
    let enable_alias_button = get_translation(state, locale, "aliases-enable-alias-button").await;
    let disable_alias_button = get_translation(state, locale, "aliases-disable-alias-button").await;
    let delete_alias = get_translation(state, locale, "aliases-delete-alias").await;
    let delete_confirm = get_translation(state, locale, "aliases-delete-confirm").await;

    let content_template = crate::templates::aliases::AliasShowTemplate {
        title: &title,
        view_edit_settings: &view_edit_settings,
        back_to_aliases: &back_to_aliases,
        alias_information: &alias_information,
        alias_details: &alias_details,
        mail: &mail,
        forward_to: &forward_to,
        domain: &domain,
        domain_info,
        status: &status,
        status_active: &status_active,
        status_inactive: &status_inactive,
        created: &created,
        modified: &modified,
        edit_alias_button: &edit_alias_button,
        enable_alias_button: &enable_alias_button,
        disable_alias_button: &disable_alias_button,
        delete_alias: &delete_alias,
        delete_confirm: &delete_confirm,
        alias,
    };

    render_show_template(content_template, state, locale, headers).await
}

pub async fn render_alias_form_page(
    form: crate::models::AliasForm,
    alias: Option<crate::models::Alias>,
    title_key: &str,
    state: &AppState,
    locale: &str,
    headers: &HeaderMap,
) -> Html<String> {
    // Fetch all required translations for alias form
    let title = get_translation(state, locale, title_key).await;
    let edit_alias = get_translation(state, locale, "aliases-edit-alias").await;
    let new_alias = get_translation(state, locale, "aliases-new-alias").await;
    let form_error = get_translation(state, locale, "form-error").await;
    let mail_address = get_translation(state, locale, "aliases-form-mail").await;
    let destination = get_translation(state, locale, "aliases-form-destination").await;
    let placeholder_mail = get_translation(state, locale, "aliases-placeholder-mail").await;
    let placeholder_destination = get_translation(state, locale, "aliases-placeholder-destination").await;
    let tooltip_mail = get_translation(state, locale, "aliases-tooltip-mail").await;
    let tooltip_destination = get_translation(state, locale, "aliases-tooltip-destination").await;
    let active = get_translation(state, locale, "form-enabled").await;
    let tooltip_active = get_translation(state, locale, "aliases-tooltip-enabled").await;
    let cancel = get_translation(state, locale, "form-cancel").await;
    let update_alias = get_translation(state, locale, "aliases-update-alias").await;
    let create_alias = get_translation(state, locale, "aliases-create-alias").await;

    let content_template = crate::templates::aliases::AliasFormTemplate {
        title: &title.clone(),
        alias,
        form,
        error: None, // Will be set by validation functions if needed
        return_url: None, // Will be set by calling function if needed
        edit_alias: &edit_alias,
        new_alias: &new_alias,
        form_error: &form_error,
        mail_address: &mail_address,
        destination: &destination,
        placeholder_mail: &placeholder_mail,
        placeholder_destination: &placeholder_destination,
        tooltip_mail: &tooltip_mail,
        tooltip_destination: &tooltip_destination,
        active: &active,
        tooltip_active: &tooltip_active,
        cancel: &cancel,
        update_alias: &update_alias,
        create_alias: &create_alias,
    };

    render_form_template(content_template, state, locale, headers, title).await
}

/// Resource-specific helper functions for Domains
pub async fn render_domain_list_page(
    domains: Vec<crate::models::Domain>,
    backups: Vec<crate::models::Backup>,
    paginated: &crate::models::PaginatedResult<crate::models::Domain>,
    state: &AppState,
    locale: &str,
    headers: &HeaderMap,
) -> Html<String> {
    // Fetch all required translations for domain list
    let title = get_translation(state, locale, "domains-title").await;
    let description = get_translation(state, locale, "domains-description").await;
    let add_domain = get_translation(state, locale, "domains-add").await;
    let table_header_domain = get_translation(state, locale, "domains-table-header-domain").await;
    let table_header_transport = get_translation(state, locale, "domains-table-header-transport").await;
    let table_header_enabled = get_translation(state, locale, "domains-table-header-enabled").await;
    let table_header_actions = get_translation(state, locale, "domains-table-header-actions").await;
    let status_active = get_translation(state, locale, "status-active").await;
    let status_inactive = get_translation(state, locale, "status-inactive").await;
    let action_view = get_translation(state, locale, "action-view").await;
    let action_enable = get_translation(state, locale, "action-enable").await;
    let action_disable = get_translation(state, locale, "action-disable").await;
    let empty_title = get_translation(state, locale, "domains-empty-title").await;
    let empty_description = get_translation(state, locale, "domains-empty-description").await;
    
    // Backup translations
    let backups_title = get_translation(state, locale, "backups-title").await;
    let backups_description = get_translation(state, locale, "backups-description").await;
    let add_backup = get_translation(state, locale, "backups-add").await;
    let backups_table_header_domain = get_translation(state, locale, "backups-table-header-domain").await;
    let backups_table_header_transport = get_translation(state, locale, "backups-table-header-transport").await;
    let backups_table_header_enabled = get_translation(state, locale, "backups-table-header-enabled").await;
    let backups_table_header_actions = get_translation(state, locale, "backups-table-header-actions").await;
    let backups_view = get_translation(state, locale, "backups-view").await;
    let backups_enable = get_translation(state, locale, "backups-enable").await;
    let backups_disable = get_translation(state, locale, "backups-disable").await;
    let backups_empty_no_backup_servers = get_translation(state, locale, "backups-empty-no-backup-servers").await;
    let backups_empty_get_started = get_translation(state, locale, "backups-empty-get-started").await;

    let page_range: Vec<i64> = (1..=paginated.total_pages).collect();
    let max_item = std::cmp::min(
        paginated.current_page * paginated.per_page,
        paginated.total_count,
    );

    let content_template = crate::templates::domains::DomainsListTemplate {
        title: &title,
        description: &description,
        add_domain: &add_domain,
        table_header_domain: &table_header_domain,
        table_header_transport: &table_header_transport,
        table_header_enabled: &table_header_enabled,
        table_header_actions: &table_header_actions,
        status_active: &status_active,
        status_inactive: &status_inactive,
        action_view: &action_view,
        action_enable: &action_enable,
        action_disable: &action_disable,
        empty_title: &empty_title,
        empty_description: &empty_description,
        domains: &domains,
        pagination: paginated,
        page_range: &page_range,
        max_item,
        backups_title: &backups_title,
        backups_description: &backups_description,
        add_backup: &add_backup,
        backups_table_header_domain: &backups_table_header_domain,
        backups_table_header_transport: &backups_table_header_transport,
        backups_table_header_enabled: &backups_table_header_enabled,
        backups_table_header_actions: &backups_table_header_actions,
        backups: &backups,
        backups_view: &backups_view,
        backups_enable: &backups_enable,
        backups_disable: &backups_disable,
        backups_empty_no_backup_servers: &backups_empty_no_backup_servers,
        backups_empty_get_started: &backups_empty_get_started,
    };

    render_list_template(content_template, state, locale, headers).await
}

pub async fn render_domain_show_page(
    domain: crate::models::Domain,
    alias_report: Option<crate::models::DomainAliasReport>,
    existing_aliases: Vec<crate::models::Alias>,
    analytics_common_aliases: Vec<String>,
    state: &AppState,
    locale: &str,
    headers: &HeaderMap,
) -> Html<String> {
    // Fetch all required translations for domain show
    let title = get_translation(state, locale, "domains-title").await;
    let view_edit_settings = get_translation(state, locale, "domains-view-edit-settings").await;
    let back_to_domains = get_translation(state, locale, "domains-back-to-domains").await;
    let domain_information = get_translation(state, locale, "domains-domain-information").await;
    let domain_details = get_translation(state, locale, "domains-domain-details").await;
    let domain_name = get_translation(state, locale, "domains-domain-name").await;
    let transport = get_translation(state, locale, "domains-transport").await;
    let status = get_translation(state, locale, "domains-status").await;
    let status_active = get_translation(state, locale, "status-active").await;
    let status_inactive = get_translation(state, locale, "status-inactive").await;
    let created = get_translation(state, locale, "domains-created").await;
    let modified = get_translation(state, locale, "domains-modified").await;
    let edit_domain_button = get_translation(state, locale, "domains-edit-domain-button").await;
    let enable_domain = get_translation(state, locale, "domains-enable-domain").await;
    let disable_domain = get_translation(state, locale, "domains-disable-domain").await;
    let delete_domain = get_translation(state, locale, "domains-delete-domain").await;
    let delete_confirm = get_translation(state, locale, "domains-delete-confirm").await;
    
    // Alias report translations
    let catch_all_header = get_translation(state, locale, "reports-catch-all-header").await;
    let destination_header = get_translation(state, locale, "reports-destination-header").await;
    let required_aliases_header = get_translation(state, locale, "reports-required-aliases-header").await;
    let missing_aliases_header = get_translation(state, locale, "reports-missing-aliases-header").await;
    let missing_required_alias_header = get_translation(state, locale, "reports-missing-required-aliases-header").await;
    let missing_common_aliases_header = get_translation(state, locale, "reports-missing-common-aliases-header").await;
    let mail_header = get_translation(state, locale, "reports-mail-header").await;
    let status_header = get_translation(state, locale, "reports-status-header").await;
    let enabled_header = get_translation(state, locale, "reports-enabled-header").await;
    let actions_header = get_translation(state, locale, "reports-actions-header").await;
    let no_required_aliases = get_translation(state, locale, "reports-no-required-aliases").await;
    let no_missing_aliases = get_translation(state, locale, "reports-no-missing-aliases").await;
    let alias_report_title = get_translation(state, locale, "domains-alias-report-title").await;
    let alias_report_description = get_translation(state, locale, "domains-alias-report-description").await;
    let existing_aliases_header = get_translation(state, locale, "domains-existing-aliases-header").await;
    let add_missing_required_alias_button = get_translation(state, locale, "reports-add-missing-required-alias-button").await;
    let add_common_alias_button = get_translation(state, locale, "reports-add-common-alias-button").await;
    let add_catch_all_button = get_translation(state, locale, "reports-add-catch-all-button").await;
    let add_alias_button = get_translation(state, locale, "domains-add-alias-button").await;
    let no_catch_all_message = get_translation(state, locale, "domains-no-catch-all-message").await;
    let action_view = get_translation(state, locale, "action-view").await;
    let enable_alias = get_translation(state, locale, "aliases-enable-alias").await;
    let disable_alias = get_translation(state, locale, "aliases-disable-alias").await;
    let enable_missing_alias = get_translation(state, locale, "aliases-enable-missing-alias").await;
    let domains_mail_header = get_translation(state, locale, "domains-mail-header").await;
    let domains_destination_header = get_translation(state, locale, "domains-destination-header").await;
    let domains_enabled_header = get_translation(state, locale, "domains-enabled-header").await;
    let domains_actions_header = get_translation(state, locale, "domains-actions-header").await;
    let domains_missing_aliases_header = get_translation(state, locale, "domains-missing-aliases-header").await;
    let domains_catch_all_header = get_translation(state, locale, "domains-catch-all-header").await;
    let analytics_common_aliases_header = get_translation(state, locale, "analytics-common-aliases-header").await;
    let analytics_common_aliases_description = get_translation(state, locale, "analytics-common-aliases-description").await;

    let content_template = crate::templates::domains::DomainShowTemplate {
        title: &title,
        domain,
        view_edit_settings: &view_edit_settings,
        back_to_domains: &back_to_domains,
        domain_information: &domain_information,
        domain_details: &domain_details,
        domain_name: &domain_name,
        transport: &transport,
        status: &status,
        status_active: &status_active,
        status_inactive: &status_inactive,
        created: &created,
        modified: &modified,
        edit_domain_button: &edit_domain_button,
        enable_domain: &enable_domain,
        disable_domain: &disable_domain,
        delete_domain: &delete_domain,
        delete_confirm: &delete_confirm,
        alias_report,
        catch_all_header: &catch_all_header,
        destination_header: &destination_header,
        required_aliases_header: &required_aliases_header,
        missing_aliases_header: &missing_aliases_header,
        missing_required_alias_header: &missing_required_alias_header,
        missing_common_aliases_header: &missing_common_aliases_header,
        mail_header: &mail_header,
        status_header: &status_header,
        enabled_header: &enabled_header,
        actions_header: &actions_header,
        no_required_aliases: &no_required_aliases,
        no_missing_aliases: &no_missing_aliases,
        alias_report_title: &alias_report_title,
        alias_report_description: &alias_report_description,
        existing_aliases_header: &existing_aliases_header,
        add_missing_required_alias_button: &add_missing_required_alias_button,
        add_common_alias_button: &add_common_alias_button,
        add_catch_all_button: &add_catch_all_button,
        add_alias_button: &add_alias_button,
        no_catch_all_message: &no_catch_all_message,
        existing_aliases: &existing_aliases,
        analytics_common_aliases: &analytics_common_aliases,
        analytics_common_aliases_header: &analytics_common_aliases_header,
        analytics_common_aliases_description: &analytics_common_aliases_description,
        action_view: &action_view,
        enable_alias: &enable_alias,
        disable_alias: &disable_alias,
        enable_missing_alias: &enable_missing_alias,
        domains_mail_header: &domains_mail_header,
        domains_destination_header: &domains_destination_header,
        domains_enabled_header: &domains_enabled_header,
        domains_actions_header: &domains_actions_header,
        domains_missing_aliases_header: &domains_missing_aliases_header,
        domains_catch_all_header: &domains_catch_all_header,
    };

    render_show_template(content_template, state, locale, headers).await
}

pub async fn render_domain_form_page(
    form: crate::models::DomainForm,
    domain: Option<crate::models::Domain>,
    title_key: &str,
    state: &AppState,
    locale: &str,
    headers: &HeaderMap,
) -> Html<String> {
    // Fetch all required translations for domain form
    let title = get_translation(state, locale, title_key).await;
    let form_error = get_translation(state, locale, "form-error").await;
    let form_domain = get_translation(state, locale, "domains-form-domain").await;
    let form_transport = get_translation(state, locale, "domains-form-transport").await;
    let form_active = get_translation(state, locale, "domains-form-active").await;
    let form_cancel = get_translation(state, locale, "form-cancel").await;
    let form_create_domain = get_translation(state, locale, "domains-form-create-domain").await;
    let form_update_domain = get_translation(state, locale, "domains-form-update-domain").await;
    let form_placeholder_domain = get_translation(state, locale, "domains-form-placeholder-domain").await;
    let form_placeholder_transport = get_translation(state, locale, "domains-form-placeholder-transport").await;
    let form_tooltip_domain = get_translation(state, locale, "domains-form-tooltip-domain").await;
    let form_tooltip_transport = get_translation(state, locale, "domains-form-tooltip-transport").await;
    let form_tooltip_enable = get_translation(state, locale, "domains-form-tooltip-enable").await;
    let form_enabled = get_translation(state, locale, "form-enabled").await;
    let form_disabled = get_translation(state, locale, "form-disabled").await;

    let content_template = crate::templates::domains::DomainFormTemplate {
        title: &title.clone(),
        domain,
        form,
        error: None, // Will be set by validation functions if needed
        form_error: &form_error,
        form_domain: &form_domain,
        form_transport: &form_transport,
        form_active: &form_active,
        form_cancel: &form_cancel,
        form_create_domain: &form_create_domain,
        form_update_domain: &form_update_domain,
        form_placeholder_domain: &form_placeholder_domain,
        form_placeholder_transport: &form_placeholder_transport,
        form_tooltip_domain: &form_tooltip_domain,
        form_tooltip_transport: &form_tooltip_transport,
        form_tooltip_enable: &form_tooltip_enable,
        form_enabled: &form_enabled,
        form_disabled: &form_disabled,
    };

    render_form_template(content_template, state, locale, headers, title).await
}

/// Resource-specific helper functions for Relays
pub async fn render_relay_list_page(
    relays: Vec<crate::models::Relay>,
    state: &AppState,
    locale: &str,
    headers: &HeaderMap,
) -> Html<String> {
    // Fetch all required translations for relay list
    let title = get_translation(state, locale, "relays-title").await;
    let relays_list_description = get_translation(state, locale, "relays-list-description").await;
    let add_relay = get_translation(state, locale, "relays-add").await;
    let table_header_recipient = get_translation(state, locale, "relays-table-header-recipient").await;
    let table_header_status = get_translation(state, locale, "relays-table-header-status").await;
    let table_header_enabled = get_translation(state, locale, "relays-table-header-enabled").await;
    let table_header_actions = get_translation(state, locale, "relays-table-header-actions").await;
    let status_enabled = get_translation(state, locale, "status-enabled").await;
    let status_disabled = get_translation(state, locale, "status-disabled").await;
    let status_ok = get_translation(state, locale, "status-ok").await;
    let status_reject = get_translation(state, locale, "status-reject").await;
    let action_view = get_translation(state, locale, "action-view").await;
    let action_enable = get_translation(state, locale, "action-enable").await;
    let action_disable = get_translation(state, locale, "action-disable").await;
    let delete_confirm = get_translation(state, locale, "relays-delete-confirm").await;
    let empty_title = get_translation(state, locale, "relays-empty-title").await;
    let empty_description = get_translation(state, locale, "relays-empty-description").await;

    let content_template = crate::templates::relays::RelayListTemplate {
        title: &title,
        relays_list_description: &relays_list_description,
        add_relay: &add_relay,
        table_header_recipient: &table_header_recipient,
        table_header_status: &table_header_status,
        table_header_enabled: &table_header_enabled,
        table_header_actions: &table_header_actions,
        status_enabled: &status_enabled,
        status_disabled: &status_disabled,
        status_ok: &status_ok,
        status_reject: &status_reject,
        action_view: &action_view,
        action_enable: &action_enable,
        action_disable: &action_disable,
        delete_confirm: &delete_confirm,
        empty_title: &empty_title,
        empty_description: &empty_description,
        relays,
    };

    render_list_template(content_template, state, locale, headers).await
}

pub async fn render_relay_show_page(
    relay: crate::models::Relay,
    state: &AppState,
    locale: &str,
    headers: &HeaderMap,
) -> Html<String> {
    // Fetch all required translations for relay show
    let title = get_translation(state, locale, "relays-title").await;
    let action_edit = get_translation(state, locale, "action-edit").await;
    let action_enable = get_translation(state, locale, "action-enable").await;
    let action_disable = get_translation(state, locale, "action-disable").await;
    let action_delete = get_translation(state, locale, "action-delete").await;
    let delete_confirm = get_translation(state, locale, "relays-delete-confirm").await;
    let back_to_list = get_translation(state, locale, "relays-back-to-list").await;
    let field_id = get_translation(state, locale, "relays-field-id").await;
    let field_recipient = get_translation(state, locale, "relays-field-recipient").await;
    let field_status = get_translation(state, locale, "relays-field-status").await;
    let field_enabled = get_translation(state, locale, "relays-field-enabled").await;
    let field_created = get_translation(state, locale, "relays-field-created").await;
    let field_modified = get_translation(state, locale, "relays-field-modified").await;
    let status_enabled = get_translation(state, locale, "status-enabled").await;
    let status_disabled = get_translation(state, locale, "status-disabled").await;
    let status_ok = get_translation(state, locale, "status-ok").await;
    let status_reject = get_translation(state, locale, "status-reject").await;
    let view_edit_settings = get_translation(state, locale, "relays-view-edit-settings").await;
    let relay_show_title = get_translation(state, locale, "relays-show-title").await;
    let relay_info_title = get_translation(state, locale, "relays-info-title").await;
    let relay_info_description = get_translation(state, locale, "relays-info-description").await;

    let content_template = crate::templates::relays::RelayShowTemplate {
        title: &title,
        relay,
        action_edit: &action_edit,
        action_enable: &action_enable,
        action_disable: &action_disable,
        action_delete: &action_delete,
        delete_confirm: &delete_confirm,
        back_to_list: &back_to_list,
        field_id: &field_id,
        field_recipient: &field_recipient,
        field_status: &field_status,
        field_enabled: &field_enabled,
        field_created: &field_created,
        field_modified: &field_modified,
        status_enabled: &status_enabled,
        status_disabled: &status_disabled,
        status_ok: &status_ok,
        status_reject: &status_reject,
        view_edit_settings: &view_edit_settings,
        relay_show_title: &relay_show_title,
        relay_info_title: &relay_info_title,
        relay_info_description: &relay_info_description,
    };

    render_show_template(content_template, state, locale, headers).await
}

pub async fn render_relay_form_page(
    form: crate::models::RelayForm,
    title_key: &str,
    action_key: &str,
    state: &AppState,
    locale: &str,
    headers: &HeaderMap,
) -> Html<String> {
    // Fetch all required translations for relay form
    let title = get_translation(state, locale, title_key).await;
    let action = get_translation(state, locale, action_key).await;
    let field_recipient = get_translation(state, locale, "relays-field-recipient").await;
    let field_status = get_translation(state, locale, "relays-field-status").await;
    let field_enabled = get_translation(state, locale, "relays-field-enabled").await;
    let field_recipient_help = get_translation(state, locale, "relays-field-recipient-help").await;
    let field_status_help = get_translation(state, locale, "relays-field-status-help").await;
    let action_save = get_translation(state, locale, "action-save").await;
    let action_cancel = get_translation(state, locale, "action-cancel").await;
    let back_to_list = get_translation(state, locale, "relays-back-to-list").await;
    let placeholder_recipient = get_translation(state, locale, "relays-placeholder-recipient").await;
    let placeholder_status = get_translation(state, locale, "relays-placeholder-status").await;

    let content_template = crate::templates::relays::RelayFormTemplate {
        title: &title.clone(),
        action: &action,
        form,
        field_recipient: &field_recipient,
        field_status: &field_status,
        field_enabled: &field_enabled,
        field_recipient_help: &field_recipient_help,
        field_status_help: &field_status_help,
        action_save: &action_save,
        action_cancel: &action_cancel,
        back_to_list: &back_to_list,
        placeholder_recipient: &placeholder_recipient,
        placeholder_status: &placeholder_status,
    };

    render_form_template(content_template, state, locale, headers, title).await
}

/// Resource-specific helper functions for Users
pub async fn render_user_list_page(
    users: Vec<crate::models::User>,
    paginated: &crate::models::PaginatedResult<crate::models::User>,
    state: &AppState,
    locale: &str,
    headers: &HeaderMap,
) -> Html<String> {
    // Fetch all required translations for user list
    let title = get_translation(state, locale, "users-title").await;
    let description = get_translation(state, locale, "users-description").await;
    let add_user = get_translation(state, locale, "users-add").await;
    let table_header_username = get_translation(state, locale, "users-table-header-username").await;
    let table_header_domain = get_translation(state, locale, "users-table-header-domain").await;
    let table_header_enabled = get_translation(state, locale, "users-table-header-enabled").await;
    let table_header_actions = get_translation(state, locale, "users-table-header-actions").await;
    let status_active = get_translation(state, locale, "status-active").await;
    let status_inactive = get_translation(state, locale, "status-inactive").await;
    let action_view = get_translation(state, locale, "action-view").await;
    let enable_user = get_translation(state, locale, "users-enable-user").await;
    let disable_user = get_translation(state, locale, "users-disable-user").await;
    let empty_title = get_translation(state, locale, "users-empty-title").await;
    let empty_description = get_translation(state, locale, "users-empty-description").await;
    let pagination_previous = get_translation(state, locale, "pagination-previous").await;
    let pagination_next = get_translation(state, locale, "pagination-next").await;
    let pagination_showing = get_translation(state, locale, "pagination-showing").await;
    let pagination_to = get_translation(state, locale, "pagination-to").await;
    let pagination_of = get_translation(state, locale, "pagination-of").await;
    let pagination_results = get_translation(state, locale, "pagination-results").await;

    let page_range: Vec<i64> = (1..=paginated.total_pages).collect();
    let max_item = std::cmp::min(
        paginated.current_page * paginated.per_page,
        paginated.total_count,
    );

    let content_template = crate::templates::users::UsersListTemplate {
        title,
        description,
        add_user,
        table_header_username,
        table_header_domain,
        table_header_enabled,
        table_header_actions,
        status_active,
        status_inactive,
        action_view,
        enable_user,
        disable_user,
        empty_title,
        empty_description,
        users,
        pagination: paginated.clone(),
        page_range,
        max_item,
        pagination_previous,
        pagination_next,
        pagination_showing,
        pagination_to,
        pagination_of,
        pagination_results,
    };

    render_list_template(content_template, state, locale, headers).await
}

pub async fn render_user_show_page(
    user: crate::models::User,
    state: &AppState,
    locale: &str,
    headers: &HeaderMap,
) -> Html<String> {
    // Fetch all required translations for user show
    let title = get_translation(state, locale, "users-show-user-title").await;
    let view_edit_settings = get_translation(state, locale, "users-view-edit-settings").await;
    let back_to_users = get_translation(state, locale, "users-back-to-users").await;
    let user_information = get_translation(state, locale, "users-user-information").await;
    let user_details = get_translation(state, locale, "users-user-details").await;
    let user_id = get_translation(state, locale, "users-user-id").await;
    let full_name = get_translation(state, locale, "users-form-name").await;
    let users_maildir = get_translation(state, locale, "users-maildir").await;
    let users_home = get_translation(state, locale, "users-home").await;
    let status = get_translation(state, locale, "users-status").await;
    let created = get_translation(state, locale, "users-created").await;
    let modified = get_translation(state, locale, "users-modified").await;
    let status_active = get_translation(state, locale, "status-active").await;
    let status_inactive = get_translation(state, locale, "status-inactive").await;
    let edit_user = get_translation(state, locale, "users-edit-user").await;
    let enable_user = get_translation(state, locale, "users-enable-user").await;
    let disable_user = get_translation(state, locale, "users-disable-user").await;
    let delete_user = get_translation(state, locale, "users-delete-user").await;
    let delete_confirm = get_translation(state, locale, "users-delete-confirm").await;
    let password_change_required_label = get_translation(state, locale, "users-password-change-required-label").await;
    let password_change_required_yes = get_translation(state, locale, "users-password-change-required-yes").await;
    let password_change_required_no = get_translation(state, locale, "users-password-change-required-no").await;
    let password_management_title = get_translation(state, locale, "users-password-management-title").await;
    let change_password_button = get_translation(state, locale, "users-change-password-button").await;
    let require_password_change_button = get_translation(state, locale, "users-require-password-change-button").await;

    let content_template = crate::templates::users::UserShowTemplate {
        title,
        view_edit_settings,
        back_to_users,
        user_information,
        user_details,
        user_id,
        full_name,
        users_maildir,
        users_home,
        status,
        created,
        modified,
        status_active,
        status_inactive,
        edit_user,
        enable_user,
        disable_user,
        delete_user,
        delete_confirm,
        user,
        password_change_required_label,
        password_change_required_yes,
        password_change_required_no,
        password_management_title,
        change_password_button,
        require_password_change_button,
    };

    render_show_template(content_template, state, locale, headers).await
}

pub async fn render_user_form_page(
    form: crate::models::UserForm,
    user: Option<crate::models::User>,
    title_key: &str,
    state: &AppState,
    locale: &str,
    headers: &HeaderMap,
) -> Html<String> {
    // Fetch all required translations for user form
    let title = get_translation(state, locale, title_key).await;
    let form_user_id = get_translation(state, locale, "users-form-user-id").await;
    let form_password = get_translation(state, locale, "users-form-password").await;
    let form_name = get_translation(state, locale, "users-form-name").await;
    let form_active = get_translation(state, locale, "users-form-active").await;
    let placeholder_user_email = get_translation(state, locale, "users-placeholder-user-email").await;
    let placeholder_name = get_translation(state, locale, "users-placeholder-name").await;
    let tooltip_user_id = get_translation(state, locale, "users-tooltip-user-id").await;
    let tooltip_password = get_translation(state, locale, "users-tooltip-password").await;
    let tooltip_name = get_translation(state, locale, "users-tooltip-name").await;
    let tooltip_active = get_translation(state, locale, "users-tooltip-active").await;
    let users_change_password = get_translation(state, locale, "users-change-password").await;
    let users_change_password_tooltip = get_translation(state, locale, "users-change-password-tooltip").await;
    let users_placeholder_password = get_translation(state, locale, "users-placeholder-password").await;
    let password_management_title = get_translation(state, locale, "users-password-management-title").await;
    let change_password_button = get_translation(state, locale, "users-change-password-button").await;
    let toggle_change_password_button = get_translation(state, locale, "users-toggle-change-password-button").await;
    let cancel = get_translation(state, locale, "form-cancel").await;
    let create_user = get_translation(state, locale, "users-create-user").await;
    let update_user = get_translation(state, locale, "users-update-user").await;
    let new_user = get_translation(state, locale, "users-new-user").await;
    let edit_user_title = get_translation(state, locale, "users-edit-user-title").await;
    let users_maildir = get_translation(state, locale, "users-maildir").await;
    let users_tooltip_maildir = get_translation(state, locale, "users-tooltip-maildir").await;
    let users_placeholder_maildir = get_translation(state, locale, "users-placeholder-maildir").await;
    let users_home = get_translation(state, locale, "users-home").await;
    let users_tooltip_home = get_translation(state, locale, "users-tooltip-home").await;
    let users_placeholder_home = get_translation(state, locale, "users-placeholder-home").await;

    let content_template = crate::templates::users::UserFormTemplate {
        title: title.clone(),
        form_user_id,
        form_password,
        form_name,
        form_active,
        placeholder_user_email,
        placeholder_name,
        tooltip_user_id,
        tooltip_password,
        tooltip_name,
        tooltip_active,
        users_change_password,
        users_change_password_tooltip,
        users_placeholder_password,
        password_management_title,
        change_password_button,
        toggle_change_password_button,
        cancel,
        create_user,
        update_user,
        new_user,
        edit_user_title,
        user,
        form,
        error: None, // Will be set by validation functions if needed
        users_maildir,
        users_tooltip_maildir,
        users_placeholder_maildir,
        users_home,
        users_tooltip_home,
        users_placeholder_home,
    };

    render_form_template(content_template, state, locale, headers, title).await
}

/// Resource-specific helper functions for Clients
pub async fn render_client_list_page(
    clients: Vec<crate::models::Client>,
    paginated: &crate::models::PaginatedResult<crate::models::Client>,
    state: &AppState,
    locale: &str,
    headers: &HeaderMap,
) -> Html<String> {
    // Fetch all required translations for client list
    let title = get_translation(state, locale, "clients-title").await;
    let description = get_translation(state, locale, "clients-description").await;
    let add_client = get_translation(state, locale, "clients-add").await;
    let table_header_client = get_translation(state, locale, "clients-table-header-client").await;
    let table_header_status = get_translation(state, locale, "clients-table-header-status").await;
    let table_header_enabled = get_translation(state, locale, "clients-table-header-enabled").await;
    let table_header_actions = get_translation(state, locale, "clients-table-header-actions").await;
    let status_allowed = get_translation(state, locale, "clients-status-ok").await;
    let status_blocked = get_translation(state, locale, "clients-status-reject").await;
    let status_enabled = get_translation(state, locale, "clients-status-enabled").await;
    let status_disabled = get_translation(state, locale, "clients-status-disabled").await;
    let action_view = get_translation(state, locale, "clients-action-view").await;
    let action_enable = get_translation(state, locale, "clients-action-enable").await;
    let action_disable = get_translation(state, locale, "clients-action-disable").await;
    let action_delete = get_translation(state, locale, "clients-action-delete").await;
    let delete_confirm = get_translation(state, locale, "clients-delete-confirm").await;
    let empty_title = get_translation(state, locale, "clients-empty-title").await;
    let empty_description = get_translation(state, locale, "clients-empty-description").await;

    let page_range: Vec<i64> = (1..=paginated.total_pages).collect();
    let max_item = std::cmp::min(
        paginated.current_page * paginated.per_page,
        paginated.total_count,
    );

    let content_template = crate::templates::clients::ClientsListTemplate {
        title: &title,
        description: &description,
        add_client: &add_client,
        table_header_client: &table_header_client,
        table_header_status: &table_header_status,
        table_header_enabled: &table_header_enabled,
        table_header_actions: &table_header_actions,
        status_allowed: &status_allowed,
        status_blocked: &status_blocked,
        status_enabled: &status_enabled,
        status_disabled: &status_disabled,
        action_view: &action_view,
        action_enable: &action_enable,
        action_disable: &action_disable,
        action_delete: &action_delete,
        delete_confirm: &delete_confirm,
        empty_title: &empty_title,
        empty_description: &empty_description,
        clients: &clients,
        pagination: paginated,
        page_range: &page_range,
        max_item,
    };

    render_list_template(content_template, state, locale, headers).await
}

pub async fn render_client_show_page(
    client: crate::models::Client,
    state: &AppState,
    locale: &str,
    headers: &HeaderMap,
) -> Html<String> {
    // Fetch all required translations for client show
    let title = get_translation(state, locale, "clients-title").await;
    let view_edit_settings = get_translation(state, locale, "clients-view-edit-settings").await;
    let back_to_clients = get_translation(state, locale, "clients-back-to-clients").await;
    let client_information = get_translation(state, locale, "clients-client-information").await;
    let client_details = get_translation(state, locale, "clients-client-details").await;
    let client_name = get_translation(state, locale, "clients-client-name").await;
    let status = get_translation(state, locale, "clients-status").await;
    let status_allowed = get_translation(state, locale, "clients-status-ok").await;
    let status_blocked = get_translation(state, locale, "clients-status-reject").await;
    let status_enabled = get_translation(state, locale, "clients-status-enabled").await;
    let status_disabled = get_translation(state, locale, "clients-status-disabled").await;
    let enabled_label = get_translation(state, locale, "clients-enabled-label").await;
    let created = get_translation(state, locale, "clients-created").await;
    let updated = get_translation(state, locale, "clients-updated").await;
    let edit_client = get_translation(state, locale, "clients-edit-client").await;
    let action_enable = get_translation(state, locale, "clients-action-enable").await;
    let action_disable = get_translation(state, locale, "clients-action-disable").await;
    let delete_client = get_translation(state, locale, "clients-delete-client").await;
    let delete_confirm = get_translation(state, locale, "clients-delete-confirm").await;

    let content_template = crate::templates::clients::ClientShowTemplate {
        title: &title,
        client,
        view_edit_settings: &view_edit_settings,
        back_to_clients: &back_to_clients,
        client_information: &client_information,
        client_details: &client_details,
        client_name: &client_name,
        status: &status,
        status_allowed: &status_allowed,
        status_blocked: &status_blocked,
        status_enabled: &status_enabled,
        status_disabled: &status_disabled,
        enabled_label: &enabled_label,
        created: &created,
        updated: &updated,
        edit_client: &edit_client,
        action_enable: &action_enable,
        action_disable: &action_disable,
        delete_client: &delete_client,
        delete_confirm: &delete_confirm,
    };

    render_show_template(content_template, state, locale, headers).await
}

pub async fn render_client_form_page(
    form: crate::models::ClientForm,
    client: Option<crate::models::Client>,
    title_key: &str,
    state: &AppState,
    locale: &str,
    headers: &HeaderMap,
) -> Html<String> {
    // Fetch all required translations for client form
    let title = get_translation(state, locale, title_key).await;
    let form_error = get_translation(state, locale, "form-error").await;
    let form_client = get_translation(state, locale, "clients-form-client").await;
    let form_status = get_translation(state, locale, "clients-form-status").await;
    let form_enabled = get_translation(state, locale, "clients-form-enabled").await;
    let form_cancel = get_translation(state, locale, "form-cancel").await;
    let form_create_client = get_translation(state, locale, "clients-form-create-client").await;
    let form_update_client = get_translation(state, locale, "clients-form-update-client").await;
    let form_placeholder_client = get_translation(state, locale, "clients-form-placeholder-client").await;
    let form_tooltip_client = get_translation(state, locale, "clients-form-tooltip-client").await;
    let form_tooltip_status = get_translation(state, locale, "clients-form-tooltip-status").await;
    let form_tooltip_enabled = get_translation(state, locale, "clients-form-tooltip-enabled").await;
    let status_allowed = get_translation(state, locale, "clients-status-ok").await;
    let status_blocked = get_translation(state, locale, "clients-status-reject").await;
    let enabled_yes = get_translation(state, locale, "form-enabled").await;
    let enabled_no = get_translation(state, locale, "form-disabled").await;

    let content_template = crate::templates::clients::ClientFormTemplate {
        title: &title.clone(),
        client,
        form_error: &form_error,
        form_client: &form_client,
        form_status: &form_status,
        form_enabled: &form_enabled,
        form_cancel: &form_cancel,
        form_create_client: &form_create_client,
        form_update_client: &form_update_client,
        form_placeholder_client: &form_placeholder_client,
        form_tooltip_client: &form_tooltip_client,
        form_tooltip_status: &form_tooltip_status,
        form_tooltip_enabled: &form_tooltip_enabled,
        status_allowed: &status_allowed,
        status_blocked: &status_blocked,
        enabled_yes: &enabled_yes,
        enabled_no: &enabled_no,
    };

    render_form_template(content_template, state, locale, headers, title).await
}

/// Resource-specific helper functions for Domain Backup
pub async fn render_backup_show_page(
    backup: crate::models::Backup,
    state: &AppState,
    locale: &str,
    headers: &HeaderMap,
) -> Html<String> {
    // Fetch all required translations for backup show
    let title = get_translation(state, locale, "backups-show-title").await;
    let view_edit_settings = get_translation(state, locale, "backups-view-edit-settings").await;
    let back_to_domains = get_translation(state, locale, "domains-back-to-domains").await;
    let backup_information = get_translation(state, locale, "backups-backup-information").await;
    let backup_details = get_translation(state, locale, "backups-backup-details").await;
    let domain = get_translation(state, locale, "backups-domain").await;
    let transport = get_translation(state, locale, "backups-transport").await;
    let status = get_translation(state, locale, "backups-status").await;
    let created = get_translation(state, locale, "backups-created").await;
    let modified = get_translation(state, locale, "backups-modified").await;
    let status_active = get_translation(state, locale, "status-active").await;
    let status_inactive = get_translation(state, locale, "status-inactive").await;
    let edit_backup = get_translation(state, locale, "backups-edit-backup").await;
    let enable_backup = get_translation(state, locale, "backups-enable-backup").await;
    let disable_backup = get_translation(state, locale, "backups-disable-backup").await;
    let delete_backup = get_translation(state, locale, "backups-delete-backup").await;
    let delete_confirm = get_translation(state, locale, "backups-delete-confirm").await;

    let content_template = crate::templates::domain_backup::BackupShowTemplate {
        title,
        view_edit_settings,
        back_to_domains,
        backup_information,
        backup_details,
        domain,
        transport,
        status,
        created,
        modified,
        status_active,
        status_inactive,
        edit_backup,
        enable_backup,
        disable_backup,
        delete_backup,
        delete_confirm,
        backup,
    };

    render_show_template(content_template, state, locale, headers).await
}

pub async fn render_backup_form_page(
    form: crate::models::BackupForm,
    backup: Option<crate::models::Backup>,
    title_key: &str,
    state: &AppState,
    locale: &str,
    headers: &HeaderMap,
) -> Html<String> {
    // Fetch all required translations for backup form
    let title = get_translation(state, locale, title_key).await;
    let form_error = get_translation(state, locale, "backups-form-error").await;
    let form_domain = get_translation(state, locale, "backups-form-domain").await;
    let form_transport = get_translation(state, locale, "backups-form-transport").await;
    let form_active = get_translation(state, locale, "backups-form-active").await;
    let placeholder_domain = get_translation(state, locale, "backups-placeholder-domain").await;
    let placeholder_transport = get_translation(state, locale, "backups-placeholder-transport").await;
    let tooltip_domain = get_translation(state, locale, "backups-tooltip-domain").await;
    let tooltip_transport = get_translation(state, locale, "backups-tooltip-transport").await;
    let tooltip_active = get_translation(state, locale, "backups-tooltip-active").await;
    let cancel = get_translation(state, locale, "backups-cancel").await;
    let create_backup = get_translation(state, locale, "backups-create-backup").await;
    let update_backup = get_translation(state, locale, "backups-update-backup").await;
    let new_backup = get_translation(state, locale, "backups-new-backup").await;
    let edit_backup_title = get_translation(state, locale, "backups-edit-backup-title").await;

    let content_template = crate::templates::domain_backup::BackupFormTemplate {
        title: title.clone(),
        form_error,
        form_domain,
        form_transport,
        form_active,
        placeholder_domain,
        placeholder_transport,
        tooltip_domain,
        tooltip_transport,
        tooltip_active,
        cancel,
        create_backup,
        update_backup,
        new_backup,
        edit_backup_title,
        backup,
        form,
        error: None, // Will be set by validation functions if needed
    };

    render_form_template(content_template, state, locale, headers, title).await
}

/// Resource-specific helper functions for Relocated
pub async fn render_relocated_list_page(
    relocated: Vec<crate::models::Relocated>,
    state: &AppState,
    locale: &str,
    headers: &HeaderMap,
) -> Html<String> {
    // Fetch all required translations for relocated list
    let title = get_translation(state, locale, "relocated-title").await;
    let relocated_list_description = get_translation(state, locale, "relocated-list-description").await;
    let add_relocated = get_translation(state, locale, "relocated-add").await;
    let table_header_old_address = get_translation(state, locale, "relocated-table-header-old-address").await;
    let table_header_new_address = get_translation(state, locale, "relocated-table-header-new-address").await;
    let table_header_enabled = get_translation(state, locale, "relocated-table-header-enabled").await;
    let table_header_actions = get_translation(state, locale, "relocated-table-header-actions").await;
    let status_enabled = get_translation(state, locale, "status-enabled").await;
    let status_disabled = get_translation(state, locale, "status-disabled").await;
    let action_view = get_translation(state, locale, "action-view").await;
    let action_enable = get_translation(state, locale, "action-enable").await;
    let action_disable = get_translation(state, locale, "action-disable").await;
    let delete_confirm = get_translation(state, locale, "relocated-delete-confirm").await;
    let empty_title = get_translation(state, locale, "relocated-empty-title").await;
    let empty_description = get_translation(state, locale, "relocated-empty-description").await;

    let content_template = crate::templates::relocated::RelocatedListTemplate {
        title: &title,
        relocated_list_description: &relocated_list_description,
        add_relocated: &add_relocated,
        table_header_old_address: &table_header_old_address,
        table_header_new_address: &table_header_new_address,
        table_header_enabled: &table_header_enabled,
        table_header_actions: &table_header_actions,
        status_enabled: &status_enabled,
        status_disabled: &status_disabled,
        action_view: &action_view,
        action_enable: &action_enable,
        action_disable: &action_disable,
        delete_confirm: &delete_confirm,
        empty_title: &empty_title,
        empty_description: &empty_description,
        relocated,
    };

    render_list_template(content_template, state, locale, headers).await
}

pub async fn render_relocated_show_page(
    relocated: crate::models::Relocated,
    state: &AppState,
    locale: &str,
    headers: &HeaderMap,
) -> Html<String> {
    // Fetch all required translations for relocated show
    let title = get_translation(state, locale, "relocated-show-title").await;
    let action_edit = get_translation(state, locale, "action-edit").await;
    let action_enable = get_translation(state, locale, "action-enable").await;
    let action_disable = get_translation(state, locale, "action-disable").await;
    let action_delete = get_translation(state, locale, "action-delete").await;
    let delete_confirm = get_translation(state, locale, "relocated-delete-confirm").await;
    let back_to_list = get_translation(state, locale, "relocated-back-to-list").await;
    let field_id = get_translation(state, locale, "relocated-field-id").await;
    let field_old_address = get_translation(state, locale, "relocated-field-old-address").await;
    let field_new_address = get_translation(state, locale, "relocated-field-new-address").await;
    let field_enabled = get_translation(state, locale, "relocated-field-enabled").await;
    let field_created = get_translation(state, locale, "relocated-field-created").await;
    let field_modified = get_translation(state, locale, "relocated-field-modified").await;
    let status_enabled = get_translation(state, locale, "status-enabled").await;
    let status_disabled = get_translation(state, locale, "status-disabled").await;
    let view_edit_settings = get_translation(state, locale, "relocated-view-edit-settings").await;
    let relocated_show_title = get_translation(state, locale, "relocated-show-title").await;
    let relocated_info_title = get_translation(state, locale, "relocated-info-title").await;
    let relocated_info_description = get_translation(state, locale, "relocated-info-description").await;

    let content_template = crate::templates::relocated::RelocatedShowTemplate {
        title: &title,
        action_edit: &action_edit,
        action_enable: &action_enable,
        action_disable: &action_disable,
        action_delete: &action_delete,
        delete_confirm: &delete_confirm,
        back_to_list: &back_to_list,
        field_id: &field_id,
        field_old_address: &field_old_address,
        field_new_address: &field_new_address,
        field_enabled: &field_enabled,
        field_created: &field_created,
        field_modified: &field_modified,
        status_enabled: &status_enabled,
        status_disabled: &status_disabled,
        view_edit_settings: &view_edit_settings,
        relocated_show_title: &relocated_show_title,
        relocated_info_title: &relocated_info_title,
        relocated_info_description: &relocated_info_description,
        relocated,
    };

    render_show_template(content_template, state, locale, headers).await
}

pub async fn render_relocated_form_page(
    form: crate::models::RelocatedForm,
    title_key: &str,
    action_key: &str,
    state: &AppState,
    locale: &str,
    headers: &HeaderMap,
) -> Html<String> {
    // Fetch all required translations for relocated form
    let title = get_translation(state, locale, title_key).await;
    let action = get_translation(state, locale, action_key).await;
    let field_old_address = get_translation(state, locale, "relocated-field-old-address").await;
    let field_new_address = get_translation(state, locale, "relocated-field-new-address").await;
    let field_enabled = get_translation(state, locale, "relocated-field-enabled").await;
    let field_old_address_help = get_translation(state, locale, "relocated-field-old-address-help").await;
    let field_new_address_help = get_translation(state, locale, "relocated-field-new-address-help").await;
    let action_save = get_translation(state, locale, "action-save").await;
    let action_cancel = get_translation(state, locale, "action-cancel").await;
    let back_to_list = get_translation(state, locale, "relocated-back-to-list").await;
    let placeholder_old_address = get_translation(state, locale, "relocated-placeholder-old-address").await;
    let placeholder_new_address = get_translation(state, locale, "relocated-placeholder-new-address").await;

    let content_template = crate::templates::relocated::RelocatedFormTemplate {
        title: &title.clone(),
        action: &action,
        form,
        field_old_address: &field_old_address,
        field_new_address: &field_new_address,
        field_enabled: &field_enabled,
        field_old_address_help: &field_old_address_help,
        field_new_address_help: &field_new_address_help,
        action_save: &action_save,
        action_cancel: &action_cancel,
        back_to_list: &back_to_list,
        placeholder_old_address: &placeholder_old_address,
        placeholder_new_address: &placeholder_new_address,
    };

    render_form_template(content_template, state, locale, headers, title).await
}
