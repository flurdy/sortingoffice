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
