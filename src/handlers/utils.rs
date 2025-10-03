use crate::{i18n::get_translation, AppState};
use askama::Template;
use axum::http::HeaderMap;
use axum::http::StatusCode;
use axum::response::Html;
use tracing::error;

// Import template functions from the new templates module
use crate::handlers::templates::render_template_safely;

// Import HTTP helper functions from the new http_helpers module
use crate::handlers::http_helpers::is_htmx_request;

// Import specific translation functions that are still used in utils.rs

/// Helper function to get current database info without unnecessary cloning
pub fn get_current_db_info_optimized(state: &AppState, headers: &HeaderMap) -> (String, String) {
    let current_db_id = get_current_db_id(state, headers);
    let current_db_label = state
        .db_manager
        .get_configs()
        .iter()
        .find(|db| db.id == current_db_id)
        .map(|db| db.label.clone())
        .unwrap_or_else(|| current_db_id.clone());

    (current_db_label, current_db_id)
}

/// Helper function to get current database ID with fallback to default
pub fn get_current_db_id(state: &AppState, headers: &HeaderMap) -> String {
    crate::handlers::auth::get_selected_database(headers)
        .unwrap_or_else(|| state.db_manager.get_default_db_id().to_string())
}

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

        if $crate::handlers::http_helpers::is_htmx_request($headers) {
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

        if $crate::handlers::http_helpers::is_htmx_request($headers) {
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
/// Usage: let entity = get_entity_or_not_found!(db::get_entity(&pool, id), &state, &headers, "entity-not-found").await?;
#[macro_export]
macro_rules! get_entity_or_not_found {
    ($db_call:expr, $state:expr, $headers:expr, $not_found_key:expr) => {{
        match $db_call {
            Ok(entity) => entity,
            Err(_) => {
                return $crate::handlers::errors::render_404_page($state, $headers).await;
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

/// Get the current database pool from the state
/// This gets the database pool based on the user's session selection
pub async fn get_current_db_pool(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<crate::DbPool, Box<dyn std::error::Error + Send + Sync>> {
    crate::handlers::database_ops::get_current_db_pool(state, headers).await
}

/// Helper function to fetch field-related translations
///
/// Helper function to fetch status-related translations
///
/// Helper function to fetch action-related translations
///
/// Common translation keys for table headers
///
/// Helper function to handle "not found" errors consistently
pub async fn handle_not_found<T>(
    result: Result<T, Box<dyn std::error::Error>>,
) -> Result<T, StatusCode> {
    match result {
        Ok(value) => Ok(value),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
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
    let content = match render_template_safely(template) {
        Ok(content) => content,
        Err(_) => return crate::handlers::errors::render_500_page(state, headers).await,
    };

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
        match render_template_safely(template) {
            Ok(content) => Html(content),
            Err(_) => crate::handlers::errors::render_500_page(state, headers).await,
        }
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
    let content = match render_template_safely(template) {
        Ok(content) => content,
        Err(_) => return crate::handlers::errors::render_500_page(state, headers).await,
    };

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
        match render_template_safely(template) {
            Ok(content) => Html(content),
            Err(_) => crate::handlers::errors::render_500_page(state, headers).await,
        }
    }
}

/// Helper function to fetch entity-specific translations for list pages
///
/// Helper function to get database pool with consistent error handling
pub async fn get_db_pool_or_handle_error(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<crate::DbPool, Html<String>> {
    crate::handlers::database_ops::get_db_pool_or_handle_error(state, headers).await
}

/// Helper function to get database pool with consistent error handling
pub async fn get_db_pool_or_error(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<crate::DbPool, Html<String>> {
    crate::handlers::database_ops::get_db_pool_or_error(state, headers).await
}

/// Helper function to get database pool with consistent error handling for redirect handlers
pub async fn get_db_pool_or_redirect_error(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<crate::DbPool, (StatusCode, String)> {
    crate::handlers::database_ops::get_db_pool_or_redirect_error(state, headers).await
}

/// Functional helper for simple database operations with standard error handling
/// Separates concerns using functional pattern: operation execution vs error handling
pub async fn execute_db_operation_with_standard_error_handling<T>(
    pool: &crate::DbPool,
    operation: impl FnOnce(&crate::DbPool) -> Result<T, diesel::result::Error>,
    success_response: Html<String>,
    error_context: &str,
    identifier: &str,
) -> Html<String> {
    crate::handlers::database_ops::execute_db_operation_with_standard_error_handling(
        pool,
        operation,
        success_response,
        error_context,
        identifier,
    )
    .await
}

/// Helper function to handle entity not found errors consistently
pub async fn handle_entity_not_found(
    state: &AppState,
    headers: &HeaderMap,
    entity_type: &str,
    _error_key: &str,
) -> Html<String> {
    match entity_type {
        "domain" => crate::handlers::errors::render_domain_not_found_page(state, headers).await,
        "user" => crate::handlers::errors::render_user_not_found_page(state, headers).await,
        "alias" => crate::handlers::errors::render_alias_not_found_page(state, headers).await,
        "client" => crate::handlers::errors::render_client_not_found_page(state, headers).await,
        "relay" => crate::handlers::errors::render_relay_not_found_page(state, headers).await,
        "backup" => crate::handlers::errors::render_backup_not_found_page(state, headers).await,
        "relocated" => {
            crate::handlers::errors::render_relocated_not_found_page(state, headers).await
        }
        _ => crate::handlers::errors::render_404_page(state, headers).await,
    }
}
