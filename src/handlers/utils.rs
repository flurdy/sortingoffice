use crate::{AppState, i18n::get_translation, default_system_stats};
use axum::http::HeaderMap;
use axum::response::Html;
use askama::Template;
use std::collections::HashMap;
use tracing::error;

/// Macro to fetch multiple translations at once
/// Usage: let translations = get_translations!(&state, &locale, [
///     "key1", "key2", "key3"
/// ]).await;
#[macro_export]
macro_rules! get_translations {
    ($state:expr, $locale:expr, [$($key:expr),*]) => {{
        let mut translations = std::collections::HashMap::new();
        $(
            let value = crate::i18n::get_translation($state, $locale, $key).await;
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

        if crate::handlers::utils::is_htmx_request($headers) {
            Html(content)
        } else {
            let base_template = match crate::templates::layout::BaseTemplate::with_i18n(
                "".to_string(), // Title will be set by the template
                content,
                $state,
                $locale,
            ).await {
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
/// Usage: let entity = get_entity_or_not_found!(db::get_entity(pool, id), &state, &locale, "entity-not-found").await?;
#[macro_export]
macro_rules! get_entity_or_not_found {
    ($db_call:expr, $state:expr, $locale:expr, $not_found_key:expr) => {{
        match $db_call {
            Ok(entity) => entity,
            Err(_) => {
                let not_found_msg = crate::i18n::get_translation($state, $locale, $not_found_key).await;
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
        crate::models::SystemStats {
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
        }
    }};
}

/// Macro to handle SystemStats retrieval with fallback to defaults
/// Usage: let stats = get_system_stats_or_default!(db::get_system_stats(pool));
#[macro_export]
macro_rules! get_system_stats_or_default {
    ($db_call:expr) => {{
        match $db_call {
            Ok(stats) => stats,
            Err(_) => crate::default_system_stats!(),
        }
    }};
}

/// Check if the request is an HTMX request
pub fn is_htmx_request(headers: &HeaderMap) -> bool {
    headers.get("HX-Request").map_or(false, |v| v == "true")
}

/// Get user locale from headers
pub fn get_user_locale(headers: &HeaderMap) -> String {
    crate::handlers::language::get_user_locale(headers)
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
pub async fn get_form_translations(
    state: &AppState,
    locale: &str,
) -> HashMap<String, String> {
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
pub async fn get_table_translations(
    state: &AppState,
    locale: &str,
) -> HashMap<String, String> {
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
        let base_template = match crate::templates::layout::BaseTemplate::with_i18n(
            "".to_string(), // Title will be set by the template
            content,
            state,
            locale,
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
pub async fn handle_not_found(
    state: &AppState,
    locale: &str,
    not_found_key: &str,
) -> Html<String> {
    let not_found_msg = get_translation(state, locale, not_found_key).await;
    Html(not_found_msg)
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
