use crate::{i18n::get_translation, AppState};
use askama::Template;
use axum::http::HeaderMap;
use axum::http::StatusCode;
use axum::response::Html;
use tracing::error;
use tracing::info;

// Import template functions from the new templates module
use crate::handlers::templates::render_template_safely;

// Import HTTP helper functions from the new http_helpers module
use crate::handlers::http_helpers::is_htmx_request;

// Import specific translation functions that are still used in utils.rs

/// Helper function to get current database info without unnecessary cloning
pub fn get_current_db_info_optimized(state: &AppState, headers: &HeaderMap) -> (String, String) {
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

/// Helper function to fetch field-related translations

/// Helper function to fetch status-related translations

/// Helper function to fetch action-related translations

/// Common translation keys for table headers

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

/// Helper function to fetch entity-specific translations for show pages

/// Helper function to fetch entity-specific error translations

/// Helper function to fetch all common translations for an entity

/// Helper function to fetch login-related translations

/// Helper function to fetch reports-related translations

/// Helper function to fetch not-found related translations

/// Helper function to get database pool with consistent error handling
pub async fn get_db_pool_or_handle_error(
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

/// Functional helper for simple database operations with standard error handling
/// Separates concerns using functional pattern: operation execution vs error handling
pub async fn execute_db_operation_with_standard_error_handling<T>(
    pool: &crate::DbPool,
    operation: impl FnOnce(&crate::DbPool) -> Result<T, diesel::result::Error>,
    success_response: Html<String>,
    error_context: &str,
    identifier: &str,
) -> Html<String> {
    match operation(pool) {
        Ok(_result) => {
            info!("Successfully completed {}: {}", error_context, identifier);
            success_response
        }
        Err(e) => {
            error!(
                "Database operation failed for {} {}: {:?}",
                error_context, identifier, e
            );
            Html(format!("Failed to {}", error_context))
        }
    }
}

/// Helper function to fetch pagination-related translations

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

/// Resource-specific helper functions for Aliases

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
    let table_header_transport =
        get_translation(state, locale, "domains-table-header-transport").await;
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
    let backups_table_header_domain =
        get_translation(state, locale, "backups-table-header-domain").await;
    let backups_table_header_transport =
        get_translation(state, locale, "backups-table-header-transport").await;
    let backups_table_header_enabled =
        get_translation(state, locale, "backups-table-header-enabled").await;
    let backups_table_header_actions =
        get_translation(state, locale, "backups-table-header-actions").await;
    let backups_view = get_translation(state, locale, "backups-view").await;
    let backups_enable = get_translation(state, locale, "backups-enable").await;
    let backups_disable = get_translation(state, locale, "backups-disable").await;
    let backups_empty_no_backup_servers =
        get_translation(state, locale, "backups-empty-no-backup-servers").await;
    let backups_empty_get_started =
        get_translation(state, locale, "backups-empty-get-started").await;

    // Pagination translations
    let pagination_showing = get_translation(state, locale, "pagination-showing").await;
    let pagination_to = get_translation(state, locale, "pagination-to").await;
    let pagination_of = get_translation(state, locale, "pagination-of").await;
    let pagination_results = get_translation(state, locale, "pagination-results").await;
    let pagination_previous = get_translation(state, locale, "pagination-previous").await;
    let pagination_next = get_translation(state, locale, "pagination-next").await;

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
        pagination_showing: &pagination_showing,
        pagination_to: &pagination_to,
        pagination_of: &pagination_of,
        pagination_results: &pagination_results,
        pagination_previous: &pagination_previous,
        pagination_next: &pagination_next,
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
    let required_aliases_header =
        get_translation(state, locale, "reports-required-aliases-header").await;
    let missing_aliases_header =
        get_translation(state, locale, "reports-missing-aliases-header").await;
    let missing_required_alias_header =
        get_translation(state, locale, "reports-missing-required-aliases-header").await;
    let missing_common_aliases_header =
        get_translation(state, locale, "reports-missing-common-aliases-header").await;
    let mail_header = get_translation(state, locale, "reports-mail-header").await;
    let status_header = get_translation(state, locale, "reports-status-header").await;
    let enabled_header = get_translation(state, locale, "reports-enabled-header").await;
    let actions_header = get_translation(state, locale, "reports-actions-header").await;
    let no_required_aliases = get_translation(state, locale, "reports-no-required-aliases").await;
    let no_missing_aliases = get_translation(state, locale, "reports-no-missing-aliases").await;
    let alias_report_title = get_translation(state, locale, "domains-alias-report-title").await;
    let alias_report_description =
        get_translation(state, locale, "domains-alias-report-description").await;
    let existing_aliases_header =
        get_translation(state, locale, "domains-existing-aliases-header").await;
    let add_missing_required_alias_button =
        get_translation(state, locale, "reports-add-missing-required-alias-button").await;
    let add_common_alias_button =
        get_translation(state, locale, "reports-add-common-alias-button").await;
    let add_catch_all_button = get_translation(state, locale, "reports-add-catch-all-button").await;
    let add_alias_button = get_translation(state, locale, "domains-add-alias-button").await;
    let no_catch_all_message = get_translation(state, locale, "domains-no-catch-all-message").await;
    let action_view = get_translation(state, locale, "action-view").await;
    let enable_alias = get_translation(state, locale, "aliases-enable-alias").await;
    let disable_alias = get_translation(state, locale, "aliases-disable-alias").await;
    let enable_missing_alias = get_translation(state, locale, "aliases-enable-missing-alias").await;
    let domains_mail_header = get_translation(state, locale, "domains-mail-header").await;
    let domains_destination_header =
        get_translation(state, locale, "domains-destination-header").await;
    let domains_enabled_header = get_translation(state, locale, "domains-enabled-header").await;
    let domains_actions_header = get_translation(state, locale, "domains-actions-header").await;
    let domains_missing_aliases_header =
        get_translation(state, locale, "domains-missing-aliases-header").await;
    let domains_catch_all_header = get_translation(state, locale, "domains-catch-all-header").await;
    let analytics_common_aliases_header =
        get_translation(state, locale, "analytics-common-aliases-header").await;
    let analytics_common_aliases_description =
        get_translation(state, locale, "analytics-common-aliases-description").await;

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
    let form_placeholder_domain =
        get_translation(state, locale, "domains-form-placeholder-domain").await;
    let form_placeholder_transport =
        get_translation(state, locale, "domains-form-placeholder-transport").await;
    let form_tooltip_domain = get_translation(state, locale, "domains-form-tooltip-domain").await;
    let form_tooltip_transport =
        get_translation(state, locale, "domains-form-tooltip-transport").await;
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
    let password_change_required_label =
        get_translation(state, locale, "users-password-change-required-label").await;
    let password_change_required_yes =
        get_translation(state, locale, "users-password-change-required-yes").await;
    let password_change_required_no =
        get_translation(state, locale, "users-password-change-required-no").await;
    let password_management_title =
        get_translation(state, locale, "users-password-management-title").await;
    let change_password_button =
        get_translation(state, locale, "users-change-password-button").await;
    let require_password_change_button =
        get_translation(state, locale, "users-require-password-change-button").await;

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
    let placeholder_user_email =
        get_translation(state, locale, "users-placeholder-user-email").await;
    let placeholder_name = get_translation(state, locale, "users-placeholder-name").await;
    let tooltip_user_id = get_translation(state, locale, "users-tooltip-user-id").await;
    let tooltip_password = get_translation(state, locale, "users-tooltip-password").await;
    let tooltip_name = get_translation(state, locale, "users-tooltip-name").await;
    let tooltip_active = get_translation(state, locale, "users-tooltip-active").await;
    let users_change_password = get_translation(state, locale, "users-change-password").await;
    let users_change_password_tooltip =
        get_translation(state, locale, "users-change-password-tooltip").await;
    let users_placeholder_password =
        get_translation(state, locale, "users-placeholder-password").await;
    let password_management_title =
        get_translation(state, locale, "users-password-management-title").await;
    let change_password_button =
        get_translation(state, locale, "users-change-password-button").await;
    let toggle_change_password_button =
        get_translation(state, locale, "users-toggle-change-password-button").await;
    let cancel = get_translation(state, locale, "form-cancel").await;
    let create_user = get_translation(state, locale, "users-create-user").await;
    let update_user = get_translation(state, locale, "users-update-user").await;
    let new_user = get_translation(state, locale, "users-new-user").await;
    let edit_user_title = get_translation(state, locale, "users-edit-user-title").await;
    let users_maildir = get_translation(state, locale, "users-maildir").await;
    let users_tooltip_maildir = get_translation(state, locale, "users-tooltip-maildir").await;
    let users_placeholder_maildir =
        get_translation(state, locale, "users-placeholder-maildir").await;
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

    // Pagination translations
    let pagination_showing = get_translation(state, locale, "pagination-showing").await;
    let pagination_to = get_translation(state, locale, "pagination-to").await;
    let pagination_of = get_translation(state, locale, "pagination-of").await;
    let pagination_results = get_translation(state, locale, "pagination-results").await;
    let pagination_previous = get_translation(state, locale, "pagination-previous").await;
    let pagination_next = get_translation(state, locale, "pagination-next").await;

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
        pagination_showing: &pagination_showing,
        pagination_to: &pagination_to,
        pagination_of: &pagination_of,
        pagination_results: &pagination_results,
        pagination_previous: &pagination_previous,
        pagination_next: &pagination_next,
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
    _form: crate::models::ClientForm,
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
    let form_placeholder_client =
        get_translation(state, locale, "clients-form-placeholder-client").await;
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



/// Resource-specific helper functions for Relocated
pub async fn render_relocated_list_page(
    relocated: Vec<crate::models::Relocated>,
    state: &AppState,
    locale: &str,
    headers: &HeaderMap,
) -> Html<String> {
    // Fetch all required translations for relocated list
    let title = get_translation(state, locale, "relocated-title").await;
    let relocated_list_description =
        get_translation(state, locale, "relocated-list-description").await;
    let add_relocated = get_translation(state, locale, "relocated-add").await;
    let table_header_old_address =
        get_translation(state, locale, "relocated-table-header-old-address").await;
    let table_header_new_address =
        get_translation(state, locale, "relocated-table-header-new-address").await;
    let table_header_enabled =
        get_translation(state, locale, "relocated-table-header-enabled").await;
    let table_header_actions =
        get_translation(state, locale, "relocated-table-header-actions").await;
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
    let relocated_info_description =
        get_translation(state, locale, "relocated-info-description").await;

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
    let field_old_address_help =
        get_translation(state, locale, "relocated-field-old-address-help").await;
    let field_new_address_help =
        get_translation(state, locale, "relocated-field-new-address-help").await;
    let action_save = get_translation(state, locale, "action-save").await;
    let action_cancel = get_translation(state, locale, "action-cancel").await;
    let back_to_list = get_translation(state, locale, "relocated-back-to-list").await;
    let placeholder_old_address =
        get_translation(state, locale, "relocated-placeholder-old-address").await;
    let placeholder_new_address =
        get_translation(state, locale, "relocated-placeholder-new-address").await;

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
