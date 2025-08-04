use crate::AppState;
use askama::Template;
use axum::http::HeaderMap;
use axum::response::Html;

// Import HTTP helper functions
use crate::handlers::http_helpers::is_htmx_request;

/// Generic template rendering with common error handling
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
            tracing::error!("Failed to render form template: {:?}", e);
            return Html("Error rendering template".to_string());
        }
    };

    if is_htmx_request(headers) {
        Html(content)
    } else {
        // Get current database info
        let current_db_id = crate::handlers::auth::get_selected_database(headers)
            .unwrap_or_else(|| state.db_manager.get_default_db_id().to_string());
        let current_db_label = state
            .db_manager
            .get_configs()
            .iter()
            .find(|db| db.id == current_db_id)
            .map(|db| db.label.clone())
            .unwrap_or_else(|| current_db_id.clone());

        let base_template = match crate::templates::layout::BaseTemplate::with_i18n(
            title,
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
}

/// Generic template rendering for list pages
pub async fn render_list_template<T>(
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
            tracing::error!("Failed to render list template: {:?}", e);
            return Html("Error rendering template".to_string());
        }
    };

    if is_htmx_request(headers) {
        Html(content)
    } else {
        // Get current database info
        let current_db_id = crate::handlers::auth::get_selected_database(headers)
            .unwrap_or_else(|| state.db_manager.get_default_db_id().to_string());
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
}

/// Generic template rendering for show pages
pub async fn render_show_template<T>(
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
            tracing::error!("Failed to render show template: {:?}", e);
            return Html("Error rendering template".to_string());
        }
    };

    if is_htmx_request(headers) {
        Html(content)
    } else {
        // Get current database info
        let current_db_id = crate::handlers::auth::get_selected_database(headers)
            .unwrap_or_else(|| state.db_manager.get_default_db_id().to_string());
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
}

/// Domain-specific rendering functions
pub async fn render_domain_list_page(
    domains: Vec<crate::models::Domain>,
    backups: Vec<crate::models::Backup>,
    paginated: &crate::models::PaginatedResult<crate::models::Domain>,
    state: &AppState,
    locale: &str,
    headers: &HeaderMap,
) -> Html<String> {
    // Fetch all required translations for domain list
    let title = crate::i18n::get_translation(state, locale, "domains-title").await;
    let description = crate::i18n::get_translation(state, locale, "domains-description").await;
    let add_domain = crate::i18n::get_translation(state, locale, "domains-add").await;
    let table_header_domain = crate::i18n::get_translation(state, locale, "domains-table-header-domain").await;
    let table_header_transport =
        crate::i18n::get_translation(state, locale, "domains-table-header-transport").await;
    let table_header_enabled = crate::i18n::get_translation(state, locale, "domains-table-header-enabled").await;
    let table_header_actions = crate::i18n::get_translation(state, locale, "domains-table-header-actions").await;
    let status_active = crate::i18n::get_translation(state, locale, "status-active").await;
    let status_inactive = crate::i18n::get_translation(state, locale, "status-inactive").await;
    let action_view = crate::i18n::get_translation(state, locale, "action-view").await;
    let action_enable = crate::i18n::get_translation(state, locale, "action-enable").await;
    let action_disable = crate::i18n::get_translation(state, locale, "action-disable").await;
    let empty_title = crate::i18n::get_translation(state, locale, "domains-empty-title").await;
    let empty_description = crate::i18n::get_translation(state, locale, "domains-empty-description").await;

    // Backup translations
    let backups_title = crate::i18n::get_translation(state, locale, "backups-title").await;
    let backups_description = crate::i18n::get_translation(state, locale, "backups-description").await;
    let add_backup = crate::i18n::get_translation(state, locale, "backups-add").await;
    let backups_table_header_domain =
        crate::i18n::get_translation(state, locale, "backups-table-header-domain").await;
    let backups_table_header_transport =
        crate::i18n::get_translation(state, locale, "backups-table-header-transport").await;
    let backups_table_header_enabled =
        crate::i18n::get_translation(state, locale, "backups-table-header-enabled").await;
    let backups_table_header_actions =
        crate::i18n::get_translation(state, locale, "backups-table-header-actions").await;
    let backups_view = crate::i18n::get_translation(state, locale, "backups-view").await;
    let backups_enable = crate::i18n::get_translation(state, locale, "backups-enable").await;
    let backups_disable = crate::i18n::get_translation(state, locale, "backups-disable").await;
    let backups_empty_no_backup_servers =
        crate::i18n::get_translation(state, locale, "backups-empty-no-backup-servers").await;
    let backups_empty_get_started =
        crate::i18n::get_translation(state, locale, "backups-empty-get-started").await;

    // Pagination translations
    let pagination_showing = crate::i18n::get_translation(state, locale, "pagination-showing").await;
    let pagination_to = crate::i18n::get_translation(state, locale, "pagination-to").await;
    let pagination_of = crate::i18n::get_translation(state, locale, "pagination-of").await;
    let pagination_results = crate::i18n::get_translation(state, locale, "pagination-results").await;
    let pagination_previous = crate::i18n::get_translation(state, locale, "pagination-previous").await;
    let pagination_next = crate::i18n::get_translation(state, locale, "pagination-next").await;

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
