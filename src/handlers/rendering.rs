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
    let table_header_domain =
        crate::i18n::get_translation(state, locale, "domains-table-header-domain").await;
    let table_header_transport =
        crate::i18n::get_translation(state, locale, "domains-table-header-transport").await;
    let table_header_enabled =
        crate::i18n::get_translation(state, locale, "domains-table-header-enabled").await;
    let table_header_actions =
        crate::i18n::get_translation(state, locale, "domains-table-header-actions").await;
    let status_active = crate::i18n::get_translation(state, locale, "status-active").await;
    let status_inactive = crate::i18n::get_translation(state, locale, "status-inactive").await;
    let action_view = crate::i18n::get_translation(state, locale, "action-view").await;
    let action_enable = crate::i18n::get_translation(state, locale, "action-enable").await;
    let action_disable = crate::i18n::get_translation(state, locale, "action-disable").await;
    let empty_title = crate::i18n::get_translation(state, locale, "domains-empty-title").await;
    let empty_description =
        crate::i18n::get_translation(state, locale, "domains-empty-description").await;

    // Backup translations
    let backups_title = crate::i18n::get_translation(state, locale, "backups-title").await;
    let backups_description =
        crate::i18n::get_translation(state, locale, "backups-description").await;
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
    let pagination_showing =
        crate::i18n::get_translation(state, locale, "pagination-showing").await;
    let pagination_to = crate::i18n::get_translation(state, locale, "pagination-to").await;
    let pagination_of = crate::i18n::get_translation(state, locale, "pagination-of").await;
    let pagination_results =
        crate::i18n::get_translation(state, locale, "pagination-results").await;
    let pagination_previous =
        crate::i18n::get_translation(state, locale, "pagination-previous").await;
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

/// Alias-specific rendering functions
pub async fn render_alias_list_page(
    aliases: Vec<crate::models::Alias>,
    paginated: &crate::models::PaginatedResult<crate::models::Alias>,
    state: &AppState,
    locale: &str,
    headers: &HeaderMap,
) -> Html<String> {
    // Fetch all required translations for alias list
    let title = crate::i18n::get_translation(state, locale, "aliases-title").await;
    let description = crate::i18n::get_translation(state, locale, "aliases-description").await;
    let add_alias = crate::i18n::get_translation(state, locale, "aliases-add").await;
    let table_header_mail =
        crate::i18n::get_translation(state, locale, "aliases-table-header-mail").await;
    let table_header_destination =
        crate::i18n::get_translation(state, locale, "aliases-table-header-destination").await;
    let table_header_domain =
        crate::i18n::get_translation(state, locale, "aliases-table-header-domain").await;
    let table_header_enabled =
        crate::i18n::get_translation(state, locale, "aliases-table-header-enabled").await;
    let table_header_actions =
        crate::i18n::get_translation(state, locale, "aliases-table-header-actions").await;
    let status_active = crate::i18n::get_translation(state, locale, "status-active").await;
    let status_inactive = crate::i18n::get_translation(state, locale, "status-inactive").await;
    let action_view = crate::i18n::get_translation(state, locale, "action-view").await;
    let enable_alias = crate::i18n::get_translation(state, locale, "aliases-enable-alias").await;
    let disable_alias = crate::i18n::get_translation(state, locale, "aliases-disable-alias").await;
    let empty_title = crate::i18n::get_translation(state, locale, "aliases-empty-title").await;
    let empty_description =
        crate::i18n::get_translation(state, locale, "aliases-empty-description").await;

    // Pagination translations
    let pagination_showing =
        crate::i18n::get_translation(state, locale, "pagination-showing").await;
    let pagination_to = crate::i18n::get_translation(state, locale, "pagination-to").await;
    let pagination_of = crate::i18n::get_translation(state, locale, "pagination-of").await;
    let pagination_results =
        crate::i18n::get_translation(state, locale, "pagination-results").await;
    let pagination_previous =
        crate::i18n::get_translation(state, locale, "pagination-previous").await;
    let pagination_next = crate::i18n::get_translation(state, locale, "pagination-next").await;

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
        pagination_showing: &pagination_showing,
        pagination_to: &pagination_to,
        pagination_of: &pagination_of,
        pagination_results: &pagination_results,
        pagination_previous: &pagination_previous,
        pagination_next: &pagination_next,
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
    let title = crate::i18n::get_translation(state, locale, "aliases-show-title").await;
    let view_edit_settings =
        crate::i18n::get_translation(state, locale, "aliases-view-edit-settings").await;
    let back_to_aliases =
        crate::i18n::get_translation(state, locale, "aliases-back-to-aliases").await;
    let alias_information =
        crate::i18n::get_translation(state, locale, "aliases-alias-information").await;
    let alias_details = crate::i18n::get_translation(state, locale, "aliases-alias-details").await;
    let mail = crate::i18n::get_translation(state, locale, "aliases-mail").await;
    let forward_to = crate::i18n::get_translation(state, locale, "aliases-forward-to").await;
    let domain = crate::i18n::get_translation(state, locale, "aliases-domain").await;
    let status = crate::i18n::get_translation(state, locale, "aliases-status").await;
    let status_active = crate::i18n::get_translation(state, locale, "status-active").await;
    let status_inactive = crate::i18n::get_translation(state, locale, "status-inactive").await;
    let created = crate::i18n::get_translation(state, locale, "aliases-created").await;
    let modified = crate::i18n::get_translation(state, locale, "aliases-modified").await;
    let edit_alias_button =
        crate::i18n::get_translation(state, locale, "aliases-edit-alias-button").await;
    let enable_alias_button =
        crate::i18n::get_translation(state, locale, "aliases-enable-alias-button").await;
    let disable_alias_button =
        crate::i18n::get_translation(state, locale, "aliases-disable-alias-button").await;
    let delete_alias = crate::i18n::get_translation(state, locale, "aliases-delete-alias").await;
    let delete_confirm =
        crate::i18n::get_translation(state, locale, "aliases-delete-confirm").await;

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
    let title = crate::i18n::get_translation(state, locale, title_key).await;
    let edit_alias = crate::i18n::get_translation(state, locale, "aliases-edit-alias").await;
    let new_alias = crate::i18n::get_translation(state, locale, "aliases-new-alias").await;
    let form_error = crate::i18n::get_translation(state, locale, "form-error").await;
    let mail_address = crate::i18n::get_translation(state, locale, "aliases-form-mail").await;
    let destination = crate::i18n::get_translation(state, locale, "aliases-form-destination").await;
    let placeholder_mail =
        crate::i18n::get_translation(state, locale, "aliases-placeholder-mail").await;
    let placeholder_destination =
        crate::i18n::get_translation(state, locale, "aliases-placeholder-destination").await;
    let tooltip_mail = crate::i18n::get_translation(state, locale, "aliases-tooltip-mail").await;
    let tooltip_destination =
        crate::i18n::get_translation(state, locale, "aliases-tooltip-destination").await;
    let active = crate::i18n::get_translation(state, locale, "form-enabled").await;
    let tooltip_active =
        crate::i18n::get_translation(state, locale, "aliases-tooltip-enabled").await;
    let cancel = crate::i18n::get_translation(state, locale, "form-cancel").await;
    let update_alias = crate::i18n::get_translation(state, locale, "aliases-update-alias").await;
    let create_alias = crate::i18n::get_translation(state, locale, "aliases-create-alias").await;

    let content_template = crate::templates::aliases::AliasFormTemplate {
        title: &title.clone(),
        alias,
        form,
        error: None,      // Will be set by validation functions if needed
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

    render_form_template(content_template, state, locale, headers, title.clone()).await
}

/// Relay-specific rendering functions
pub async fn render_relay_list_page(
    relays: Vec<crate::models::Relay>,
    state: &AppState,
    locale: &str,
    headers: &HeaderMap,
) -> Html<String> {
    // Fetch all required translations for relay list
    let title = crate::i18n::get_translation(state, locale, "relays-title").await;
    let relays_list_description =
        crate::i18n::get_translation(state, locale, "relays-list-description").await;
    let add_relay = crate::i18n::get_translation(state, locale, "relays-add").await;
    let table_header_recipient =
        crate::i18n::get_translation(state, locale, "relays-table-header-recipient").await;
    let table_header_status =
        crate::i18n::get_translation(state, locale, "relays-table-header-status").await;
    let table_header_enabled =
        crate::i18n::get_translation(state, locale, "relays-table-header-enabled").await;
    let table_header_actions =
        crate::i18n::get_translation(state, locale, "relays-table-header-actions").await;
    let status_enabled = crate::i18n::get_translation(state, locale, "status-enabled").await;
    let status_disabled = crate::i18n::get_translation(state, locale, "status-disabled").await;
    let status_ok = crate::i18n::get_translation(state, locale, "status-ok").await;
    let status_reject = crate::i18n::get_translation(state, locale, "status-reject").await;
    let action_view = crate::i18n::get_translation(state, locale, "action-view").await;
    let action_enable = crate::i18n::get_translation(state, locale, "action-enable").await;
    let action_disable = crate::i18n::get_translation(state, locale, "action-disable").await;
    let delete_confirm = crate::i18n::get_translation(state, locale, "relays-delete-confirm").await;
    let empty_title = crate::i18n::get_translation(state, locale, "relays-empty-title").await;
    let empty_description =
        crate::i18n::get_translation(state, locale, "relays-empty-description").await;

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
    let title = crate::i18n::get_translation(state, locale, "relays-title").await;
    let action_edit = crate::i18n::get_translation(state, locale, "action-edit").await;
    let action_enable = crate::i18n::get_translation(state, locale, "action-enable").await;
    let action_disable = crate::i18n::get_translation(state, locale, "action-disable").await;
    let action_delete = crate::i18n::get_translation(state, locale, "action-delete").await;
    let delete_confirm = crate::i18n::get_translation(state, locale, "relays-delete-confirm").await;
    let back_to_list = crate::i18n::get_translation(state, locale, "relays-back-to-list").await;
    let field_id = crate::i18n::get_translation(state, locale, "relays-field-id").await;
    let field_recipient =
        crate::i18n::get_translation(state, locale, "relays-field-recipient").await;
    let field_status = crate::i18n::get_translation(state, locale, "relays-field-status").await;
    let field_enabled = crate::i18n::get_translation(state, locale, "relays-field-enabled").await;
    let field_created = crate::i18n::get_translation(state, locale, "relays-field-created").await;
    let field_modified = crate::i18n::get_translation(state, locale, "relays-field-modified").await;
    let status_enabled = crate::i18n::get_translation(state, locale, "status-enabled").await;
    let status_disabled = crate::i18n::get_translation(state, locale, "status-disabled").await;
    let status_ok = crate::i18n::get_translation(state, locale, "status-ok").await;
    let status_reject = crate::i18n::get_translation(state, locale, "status-reject").await;
    let view_edit_settings =
        crate::i18n::get_translation(state, locale, "relays-view-edit-settings").await;
    let relay_show_title = crate::i18n::get_translation(state, locale, "relays-show-title").await;
    let relay_info_title = crate::i18n::get_translation(state, locale, "relays-info-title").await;
    let relay_info_description =
        crate::i18n::get_translation(state, locale, "relays-info-description").await;

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
    let title = crate::i18n::get_translation(state, locale, title_key).await;
    let action = crate::i18n::get_translation(state, locale, action_key).await;
    let field_recipient =
        crate::i18n::get_translation(state, locale, "relays-field-recipient").await;
    let field_status = crate::i18n::get_translation(state, locale, "relays-field-status").await;
    let field_enabled = crate::i18n::get_translation(state, locale, "relays-field-enabled").await;
    let field_recipient_help =
        crate::i18n::get_translation(state, locale, "relays-field-recipient-help").await;
    let field_status_help =
        crate::i18n::get_translation(state, locale, "relays-field-status-help").await;
    let action_save = crate::i18n::get_translation(state, locale, "action-save").await;
    let action_cancel = crate::i18n::get_translation(state, locale, "action-cancel").await;
    let back_to_list = crate::i18n::get_translation(state, locale, "relays-back-to-list").await;
    let placeholder_recipient =
        crate::i18n::get_translation(state, locale, "relays-placeholder-recipient").await;
    let placeholder_status =
        crate::i18n::get_translation(state, locale, "relays-placeholder-status").await;

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

    render_form_template(content_template, state, locale, headers, title.clone()).await
}

/// Backup-specific rendering functions
pub async fn render_backup_show_page(
    backup: crate::models::Backup,
    state: &AppState,
    locale: &str,
    headers: &HeaderMap,
) -> Html<String> {
    // Fetch all required translations for backup show
    let title = crate::i18n::get_translation(state, locale, "backups-show-title").await;
    let view_edit_settings =
        crate::i18n::get_translation(state, locale, "backups-view-edit-settings").await;
    let back_to_domains =
        crate::i18n::get_translation(state, locale, "domains-back-to-domains").await;
    let backup_information =
        crate::i18n::get_translation(state, locale, "backups-backup-information").await;
    let backup_details =
        crate::i18n::get_translation(state, locale, "backups-backup-details").await;
    let domain = crate::i18n::get_translation(state, locale, "backups-domain").await;
    let transport = crate::i18n::get_translation(state, locale, "backups-transport").await;
    let status = crate::i18n::get_translation(state, locale, "backups-status").await;
    let created = crate::i18n::get_translation(state, locale, "backups-created").await;
    let modified = crate::i18n::get_translation(state, locale, "backups-modified").await;
    let status_active = crate::i18n::get_translation(state, locale, "status-active").await;
    let status_inactive = crate::i18n::get_translation(state, locale, "status-inactive").await;
    let edit_backup = crate::i18n::get_translation(state, locale, "backups-edit-backup").await;
    let enable_backup = crate::i18n::get_translation(state, locale, "backups-enable-backup").await;
    let disable_backup =
        crate::i18n::get_translation(state, locale, "backups-disable-backup").await;
    let delete_backup = crate::i18n::get_translation(state, locale, "backups-delete-backup").await;
    let delete_confirm =
        crate::i18n::get_translation(state, locale, "backups-delete-confirm").await;

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
    let title = crate::i18n::get_translation(state, locale, title_key).await;
    let form_error = crate::i18n::get_translation(state, locale, "backups-form-error").await;
    let form_domain = crate::i18n::get_translation(state, locale, "backups-form-domain").await;
    let form_transport =
        crate::i18n::get_translation(state, locale, "backups-form-transport").await;
    let form_active = crate::i18n::get_translation(state, locale, "backups-form-active").await;
    let placeholder_domain =
        crate::i18n::get_translation(state, locale, "backups-placeholder-domain").await;
    let placeholder_transport =
        crate::i18n::get_translation(state, locale, "backups-placeholder-transport").await;
    let tooltip_domain =
        crate::i18n::get_translation(state, locale, "backups-tooltip-domain").await;
    let tooltip_transport =
        crate::i18n::get_translation(state, locale, "backups-tooltip-transport").await;
    let tooltip_active =
        crate::i18n::get_translation(state, locale, "backups-tooltip-active").await;
    let cancel = crate::i18n::get_translation(state, locale, "backups-cancel").await;
    let create_backup = crate::i18n::get_translation(state, locale, "backups-create-backup").await;
    let update_backup = crate::i18n::get_translation(state, locale, "backups-update-backup").await;
    let new_backup = crate::i18n::get_translation(state, locale, "backups-new-backup").await;
    let edit_backup_title =
        crate::i18n::get_translation(state, locale, "backups-edit-backup-title").await;

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

    render_form_template(content_template, state, locale, headers, title.clone()).await
}

pub async fn render_backup_form_page_with_error(
    form: crate::models::BackupForm,
    backup: Option<crate::models::Backup>,
    title_key: &str,
    error_key: &str,
    state: &AppState,
    locale: &str,
    headers: &HeaderMap,
) -> Html<String> {
    // Fetch all required translations for backup form
    let title = crate::i18n::get_translation(state, locale, title_key).await;
    let form_error = crate::i18n::get_translation(state, locale, "backups-form-error").await;
    let form_domain = crate::i18n::get_translation(state, locale, "backups-form-domain").await;
    let form_transport =
        crate::i18n::get_translation(state, locale, "backups-form-transport").await;
    let form_active = crate::i18n::get_translation(state, locale, "backups-form-active").await;
    let placeholder_domain =
        crate::i18n::get_translation(state, locale, "backups-placeholder-domain").await;
    let placeholder_transport =
        crate::i18n::get_translation(state, locale, "backups-placeholder-transport").await;
    let tooltip_domain =
        crate::i18n::get_translation(state, locale, "backups-tooltip-domain").await;
    let tooltip_transport =
        crate::i18n::get_translation(state, locale, "backups-tooltip-transport").await;
    let tooltip_active =
        crate::i18n::get_translation(state, locale, "backups-tooltip-active").await;
    let cancel = crate::i18n::get_translation(state, locale, "backups-cancel").await;
    let create_backup = crate::i18n::get_translation(state, locale, "backups-create-backup").await;
    let update_backup = crate::i18n::get_translation(state, locale, "backups-update-backup").await;
    let new_backup = crate::i18n::get_translation(state, locale, "backups-new-backup").await;
    let edit_backup_title =
        crate::i18n::get_translation(state, locale, "backups-edit-backup-title").await;
    let error_message = crate::i18n::get_translation(state, locale, error_key).await;

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
        error: Some(error_message),
    };

    render_form_template(content_template, state, locale, headers, title.clone()).await
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
    let title = crate::i18n::get_translation(state, locale, "domains-title").await;
    let view_edit_settings =
        crate::i18n::get_translation(state, locale, "domains-view-edit-settings").await;
    let back_to_domains =
        crate::i18n::get_translation(state, locale, "domains-back-to-domains").await;
    let domain_information =
        crate::i18n::get_translation(state, locale, "domains-domain-information").await;
    let domain_details =
        crate::i18n::get_translation(state, locale, "domains-domain-details").await;
    let domain_name = crate::i18n::get_translation(state, locale, "domains-domain-name").await;
    let transport = crate::i18n::get_translation(state, locale, "domains-transport").await;
    let status = crate::i18n::get_translation(state, locale, "domains-status").await;
    let status_active = crate::i18n::get_translation(state, locale, "status-active").await;
    let status_inactive = crate::i18n::get_translation(state, locale, "status-inactive").await;
    let created = crate::i18n::get_translation(state, locale, "domains-created").await;
    let modified = crate::i18n::get_translation(state, locale, "domains-modified").await;
    let edit_domain_button =
        crate::i18n::get_translation(state, locale, "domains-edit-domain-button").await;
    let enable_domain = crate::i18n::get_translation(state, locale, "domains-enable-domain").await;
    let disable_domain =
        crate::i18n::get_translation(state, locale, "domains-disable-domain").await;
    let delete_domain = crate::i18n::get_translation(state, locale, "domains-delete-domain").await;
    let delete_confirm =
        crate::i18n::get_translation(state, locale, "domains-delete-confirm").await;

    // Alias report translations
    let catch_all_header =
        crate::i18n::get_translation(state, locale, "reports-catch-all-header").await;
    let destination_header =
        crate::i18n::get_translation(state, locale, "reports-destination-header").await;
    let required_aliases_header =
        crate::i18n::get_translation(state, locale, "reports-required-aliases-header").await;
    let missing_aliases_header =
        crate::i18n::get_translation(state, locale, "reports-missing-aliases-header").await;
    let missing_required_alias_header =
        crate::i18n::get_translation(state, locale, "reports-missing-required-aliases-header")
            .await;
    let missing_common_aliases_header =
        crate::i18n::get_translation(state, locale, "reports-missing-common-aliases-header").await;
    let mail_header = crate::i18n::get_translation(state, locale, "reports-mail-header").await;
    let status_header = crate::i18n::get_translation(state, locale, "reports-status-header").await;
    let enabled_header =
        crate::i18n::get_translation(state, locale, "reports-enabled-header").await;
    let actions_header =
        crate::i18n::get_translation(state, locale, "reports-actions-header").await;
    let no_required_aliases =
        crate::i18n::get_translation(state, locale, "reports-no-required-aliases").await;
    let no_missing_aliases =
        crate::i18n::get_translation(state, locale, "reports-no-missing-aliases").await;
    let alias_report_title =
        crate::i18n::get_translation(state, locale, "domains-alias-report-title").await;
    let alias_report_description =
        crate::i18n::get_translation(state, locale, "domains-alias-report-description").await;
    let existing_aliases_header =
        crate::i18n::get_translation(state, locale, "domains-existing-aliases-header").await;
    let add_missing_required_alias_button =
        crate::i18n::get_translation(state, locale, "reports-add-missing-required-alias-button")
            .await;
    let add_common_alias_button =
        crate::i18n::get_translation(state, locale, "reports-add-common-alias-button").await;
    let add_catch_all_button =
        crate::i18n::get_translation(state, locale, "reports-add-catch-all-button").await;
    let add_alias_button =
        crate::i18n::get_translation(state, locale, "domains-add-alias-button").await;
    let no_catch_all_message =
        crate::i18n::get_translation(state, locale, "domains-no-catch-all-message").await;
    let action_view = crate::i18n::get_translation(state, locale, "action-view").await;
    let enable_alias = crate::i18n::get_translation(state, locale, "aliases-enable-alias").await;
    let disable_alias = crate::i18n::get_translation(state, locale, "aliases-disable-alias").await;
    let enable_missing_alias =
        crate::i18n::get_translation(state, locale, "aliases-enable-missing-alias").await;
    let domains_mail_header =
        crate::i18n::get_translation(state, locale, "domains-mail-header").await;
    let domains_destination_header =
        crate::i18n::get_translation(state, locale, "domains-destination-header").await;
    let domains_enabled_header =
        crate::i18n::get_translation(state, locale, "domains-enabled-header").await;
    let domains_actions_header =
        crate::i18n::get_translation(state, locale, "domains-actions-header").await;
    let domains_missing_aliases_header =
        crate::i18n::get_translation(state, locale, "domains-missing-aliases-header").await;
    let domains_catch_all_header =
        crate::i18n::get_translation(state, locale, "domains-catch-all-header").await;
    let analytics_common_aliases_header =
        crate::i18n::get_translation(state, locale, "analytics-common-aliases-header").await;
    let analytics_common_aliases_description =
        crate::i18n::get_translation(state, locale, "analytics-common-aliases-description").await;

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
    let title = crate::i18n::get_translation(state, locale, title_key).await;
    let form_error = crate::i18n::get_translation(state, locale, "form-error").await;
    let form_domain = crate::i18n::get_translation(state, locale, "domains-form-domain").await;
    let form_transport =
        crate::i18n::get_translation(state, locale, "domains-form-transport").await;
    let form_active = crate::i18n::get_translation(state, locale, "domains-form-active").await;
    let form_cancel = crate::i18n::get_translation(state, locale, "form-cancel").await;
    let form_create_domain =
        crate::i18n::get_translation(state, locale, "domains-form-create-domain").await;
    let form_update_domain =
        crate::i18n::get_translation(state, locale, "domains-form-update-domain").await;
    let form_placeholder_domain =
        crate::i18n::get_translation(state, locale, "domains-form-placeholder-domain").await;
    let form_placeholder_transport =
        crate::i18n::get_translation(state, locale, "domains-form-placeholder-transport").await;
    let form_tooltip_domain =
        crate::i18n::get_translation(state, locale, "domains-form-tooltip-domain").await;
    let form_tooltip_transport =
        crate::i18n::get_translation(state, locale, "domains-form-tooltip-transport").await;
    let form_tooltip_enable =
        crate::i18n::get_translation(state, locale, "domains-form-tooltip-enable").await;
    let form_enabled = crate::i18n::get_translation(state, locale, "form-enabled").await;
    let form_disabled = crate::i18n::get_translation(state, locale, "form-disabled").await;

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

    render_form_template(content_template, state, locale, headers, title.clone()).await
}

/// User-specific rendering functions
pub async fn render_user_list_page(
    users: Vec<crate::models::User>,
    paginated: &crate::models::PaginatedResult<crate::models::User>,
    state: &AppState,
    locale: &str,
    headers: &HeaderMap,
) -> Html<String> {
    // Fetch all required translations for user list
    let title = crate::i18n::get_translation(state, locale, "users-title").await;
    let description = crate::i18n::get_translation(state, locale, "users-description").await;
    let add_user = crate::i18n::get_translation(state, locale, "users-add").await;
    let table_header_username =
        crate::i18n::get_translation(state, locale, "users-table-header-username").await;
    let table_header_domain =
        crate::i18n::get_translation(state, locale, "users-table-header-domain").await;
    let table_header_enabled =
        crate::i18n::get_translation(state, locale, "users-table-header-enabled").await;
    let table_header_actions =
        crate::i18n::get_translation(state, locale, "users-table-header-actions").await;
    let status_active = crate::i18n::get_translation(state, locale, "status-active").await;
    let status_inactive = crate::i18n::get_translation(state, locale, "status-inactive").await;
    let action_view = crate::i18n::get_translation(state, locale, "action-view").await;
    let enable_user = crate::i18n::get_translation(state, locale, "users-enable-user").await;
    let disable_user = crate::i18n::get_translation(state, locale, "users-disable-user").await;
    let empty_title = crate::i18n::get_translation(state, locale, "users-empty-title").await;
    let empty_description =
        crate::i18n::get_translation(state, locale, "users-empty-description").await;
    let pagination_previous =
        crate::i18n::get_translation(state, locale, "pagination-previous").await;
    let pagination_next = crate::i18n::get_translation(state, locale, "pagination-next").await;
    let pagination_showing =
        crate::i18n::get_translation(state, locale, "pagination-showing").await;
    let pagination_to = crate::i18n::get_translation(state, locale, "pagination-to").await;
    let pagination_of = crate::i18n::get_translation(state, locale, "pagination-of").await;
    let pagination_results =
        crate::i18n::get_translation(state, locale, "pagination-results").await;

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
    let title = crate::i18n::get_translation(state, locale, "users-show-user-title").await;
    let view_edit_settings =
        crate::i18n::get_translation(state, locale, "users-view-edit-settings").await;
    let back_to_users = crate::i18n::get_translation(state, locale, "users-back-to-users").await;
    let user_information =
        crate::i18n::get_translation(state, locale, "users-user-information").await;
    let user_details = crate::i18n::get_translation(state, locale, "users-user-details").await;
    let user_id = crate::i18n::get_translation(state, locale, "users-user-id").await;
    let full_name = crate::i18n::get_translation(state, locale, "users-form-name").await;
    let users_maildir = crate::i18n::get_translation(state, locale, "users-maildir").await;
    let users_home = crate::i18n::get_translation(state, locale, "users-home").await;
    let status = crate::i18n::get_translation(state, locale, "users-status").await;
    let created = crate::i18n::get_translation(state, locale, "users-created").await;
    let modified = crate::i18n::get_translation(state, locale, "users-modified").await;
    let status_active = crate::i18n::get_translation(state, locale, "status-active").await;
    let status_inactive = crate::i18n::get_translation(state, locale, "status-inactive").await;
    let edit_user = crate::i18n::get_translation(state, locale, "users-edit-user").await;
    let enable_user = crate::i18n::get_translation(state, locale, "users-enable-user").await;
    let disable_user = crate::i18n::get_translation(state, locale, "users-disable-user").await;
    let delete_user = crate::i18n::get_translation(state, locale, "users-delete-user").await;
    let delete_confirm = crate::i18n::get_translation(state, locale, "users-delete-confirm").await;
    let password_change_required_label =
        crate::i18n::get_translation(state, locale, "users-password-change-required-label").await;
    let password_change_required_yes =
        crate::i18n::get_translation(state, locale, "users-password-change-required-yes").await;
    let password_change_required_no =
        crate::i18n::get_translation(state, locale, "users-password-change-required-no").await;
    let password_management_title =
        crate::i18n::get_translation(state, locale, "users-password-management-title").await;
    let change_password_button =
        crate::i18n::get_translation(state, locale, "users-change-password-button").await;
    let require_password_change_button =
        crate::i18n::get_translation(state, locale, "users-require-password-change-button").await;

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
    let title = crate::i18n::get_translation(state, locale, title_key).await;
    let form_user_id = crate::i18n::get_translation(state, locale, "users-form-user-id").await;
    let form_password = crate::i18n::get_translation(state, locale, "users-form-password").await;
    let form_name = crate::i18n::get_translation(state, locale, "users-form-name").await;
    let form_active = crate::i18n::get_translation(state, locale, "users-form-active").await;
    let placeholder_user_email =
        crate::i18n::get_translation(state, locale, "users-placeholder-user-email").await;
    let placeholder_name =
        crate::i18n::get_translation(state, locale, "users-placeholder-name").await;
    let tooltip_user_id =
        crate::i18n::get_translation(state, locale, "users-tooltip-user-id").await;
    let tooltip_password =
        crate::i18n::get_translation(state, locale, "users-tooltip-password").await;
    let tooltip_name = crate::i18n::get_translation(state, locale, "users-tooltip-name").await;
    let tooltip_active = crate::i18n::get_translation(state, locale, "users-tooltip-active").await;
    let users_change_password =
        crate::i18n::get_translation(state, locale, "users-change-password").await;
    let users_change_password_tooltip =
        crate::i18n::get_translation(state, locale, "users-change-password-tooltip").await;
    let users_placeholder_password =
        crate::i18n::get_translation(state, locale, "users-placeholder-password").await;
    let password_management_title =
        crate::i18n::get_translation(state, locale, "users-password-management-title").await;
    let change_password_button =
        crate::i18n::get_translation(state, locale, "users-change-password-button").await;
    let toggle_change_password_button =
        crate::i18n::get_translation(state, locale, "users-toggle-change-password-button").await;
    let cancel = crate::i18n::get_translation(state, locale, "form-cancel").await;
    let create_user = crate::i18n::get_translation(state, locale, "users-create-user").await;
    let update_user = crate::i18n::get_translation(state, locale, "users-update-user").await;
    let new_user = crate::i18n::get_translation(state, locale, "users-new-user").await;
    let edit_user_title =
        crate::i18n::get_translation(state, locale, "users-edit-user-title").await;
    let users_maildir = crate::i18n::get_translation(state, locale, "users-maildir").await;
    let users_tooltip_maildir =
        crate::i18n::get_translation(state, locale, "users-tooltip-maildir").await;
    let users_placeholder_maildir =
        crate::i18n::get_translation(state, locale, "users-placeholder-maildir").await;
    let users_home = crate::i18n::get_translation(state, locale, "users-home").await;
    let users_tooltip_home =
        crate::i18n::get_translation(state, locale, "users-tooltip-home").await;
    let users_placeholder_home =
        crate::i18n::get_translation(state, locale, "users-placeholder-home").await;

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

    render_form_template(content_template, state, locale, headers, title.clone()).await
}

/// Client-specific rendering functions
pub async fn render_client_list_page(
    clients: Vec<crate::models::Client>,
    paginated: &crate::models::PaginatedResult<crate::models::Client>,
    state: &AppState,
    locale: &str,
    headers: &HeaderMap,
) -> Html<String> {
    // Fetch all required translations for client list
    let title = crate::i18n::get_translation(state, locale, "clients-title").await;
    let description = crate::i18n::get_translation(state, locale, "clients-description").await;
    let add_client = crate::i18n::get_translation(state, locale, "clients-add").await;
    let table_header_client =
        crate::i18n::get_translation(state, locale, "clients-table-header-client").await;
    let table_header_status =
        crate::i18n::get_translation(state, locale, "clients-table-header-status").await;
    let table_header_enabled =
        crate::i18n::get_translation(state, locale, "clients-table-header-enabled").await;
    let table_header_actions =
        crate::i18n::get_translation(state, locale, "clients-table-header-actions").await;
    let status_allowed = crate::i18n::get_translation(state, locale, "clients-status-ok").await;
    let status_blocked = crate::i18n::get_translation(state, locale, "clients-status-reject").await;
    let status_enabled =
        crate::i18n::get_translation(state, locale, "clients-status-enabled").await;
    let status_disabled =
        crate::i18n::get_translation(state, locale, "clients-status-disabled").await;
    let action_view = crate::i18n::get_translation(state, locale, "clients-action-view").await;
    let action_enable = crate::i18n::get_translation(state, locale, "clients-action-enable").await;
    let action_disable =
        crate::i18n::get_translation(state, locale, "clients-action-disable").await;
    let action_delete = crate::i18n::get_translation(state, locale, "clients-action-delete").await;
    let delete_confirm =
        crate::i18n::get_translation(state, locale, "clients-delete-confirm").await;
    let empty_title = crate::i18n::get_translation(state, locale, "clients-empty-title").await;
    let empty_description =
        crate::i18n::get_translation(state, locale, "clients-empty-description").await;

    // Pagination translations
    let pagination_showing =
        crate::i18n::get_translation(state, locale, "pagination-showing").await;
    let pagination_to = crate::i18n::get_translation(state, locale, "pagination-to").await;
    let pagination_of = crate::i18n::get_translation(state, locale, "pagination-of").await;
    let pagination_results =
        crate::i18n::get_translation(state, locale, "pagination-results").await;
    let pagination_previous =
        crate::i18n::get_translation(state, locale, "pagination-previous").await;
    let pagination_next = crate::i18n::get_translation(state, locale, "pagination-next").await;

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
    let title = crate::i18n::get_translation(state, locale, "clients-title").await;
    let view_edit_settings =
        crate::i18n::get_translation(state, locale, "clients-view-edit-settings").await;
    let back_to_clients =
        crate::i18n::get_translation(state, locale, "clients-back-to-clients").await;
    let client_information =
        crate::i18n::get_translation(state, locale, "clients-client-information").await;
    let client_details =
        crate::i18n::get_translation(state, locale, "clients-client-details").await;
    let client_name = crate::i18n::get_translation(state, locale, "clients-client-name").await;
    let status = crate::i18n::get_translation(state, locale, "clients-status").await;
    let status_allowed = crate::i18n::get_translation(state, locale, "clients-status-ok").await;
    let status_blocked = crate::i18n::get_translation(state, locale, "clients-status-reject").await;
    let status_enabled =
        crate::i18n::get_translation(state, locale, "clients-status-enabled").await;
    let status_disabled =
        crate::i18n::get_translation(state, locale, "clients-status-disabled").await;
    let enabled_label = crate::i18n::get_translation(state, locale, "clients-enabled-label").await;
    let created = crate::i18n::get_translation(state, locale, "clients-created").await;
    let updated = crate::i18n::get_translation(state, locale, "clients-updated").await;
    let edit_client = crate::i18n::get_translation(state, locale, "clients-edit-client").await;
    let action_enable = crate::i18n::get_translation(state, locale, "clients-action-enable").await;
    let action_disable =
        crate::i18n::get_translation(state, locale, "clients-action-disable").await;
    let delete_client = crate::i18n::get_translation(state, locale, "clients-delete-client").await;
    let delete_confirm =
        crate::i18n::get_translation(state, locale, "clients-delete-confirm").await;

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
    let title = crate::i18n::get_translation(state, locale, title_key).await;
    let form_error = crate::i18n::get_translation(state, locale, "form-error").await;
    let form_client = crate::i18n::get_translation(state, locale, "clients-form-client").await;
    let form_status = crate::i18n::get_translation(state, locale, "clients-form-status").await;
    let form_enabled = crate::i18n::get_translation(state, locale, "clients-form-enabled").await;
    let form_cancel = crate::i18n::get_translation(state, locale, "form-cancel").await;
    let form_create_client =
        crate::i18n::get_translation(state, locale, "clients-form-create-client").await;
    let form_update_client =
        crate::i18n::get_translation(state, locale, "clients-form-update-client").await;
    let form_placeholder_client =
        crate::i18n::get_translation(state, locale, "clients-form-placeholder-client").await;
    let form_tooltip_client =
        crate::i18n::get_translation(state, locale, "clients-form-tooltip-client").await;
    let form_tooltip_status =
        crate::i18n::get_translation(state, locale, "clients-form-tooltip-status").await;
    let form_tooltip_enabled =
        crate::i18n::get_translation(state, locale, "clients-form-tooltip-enabled").await;
    let status_allowed = crate::i18n::get_translation(state, locale, "clients-status-ok").await;
    let status_blocked = crate::i18n::get_translation(state, locale, "clients-status-reject").await;
    let enabled_yes = crate::i18n::get_translation(state, locale, "form-enabled").await;
    let enabled_no = crate::i18n::get_translation(state, locale, "form-disabled").await;

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

    render_form_template(content_template, state, locale, headers, title.clone()).await
}

/// Relocated-specific rendering functions
pub async fn render_relocated_list_page(
    relocated: Vec<crate::models::Relocated>,
    state: &AppState,
    locale: &str,
    headers: &HeaderMap,
) -> Html<String> {
    // Fetch all required translations for relocated list
    let title = crate::i18n::get_translation(state, locale, "relocated-title").await;
    let relocated_list_description =
        crate::i18n::get_translation(state, locale, "relocated-list-description").await;
    let add_relocated = crate::i18n::get_translation(state, locale, "relocated-add").await;
    let table_header_old_address =
        crate::i18n::get_translation(state, locale, "relocated-table-header-old-address").await;
    let table_header_new_address =
        crate::i18n::get_translation(state, locale, "relocated-table-header-new-address").await;
    let table_header_enabled =
        crate::i18n::get_translation(state, locale, "relocated-table-header-enabled").await;
    let table_header_actions =
        crate::i18n::get_translation(state, locale, "relocated-table-header-actions").await;
    let status_enabled = crate::i18n::get_translation(state, locale, "status-enabled").await;
    let status_disabled = crate::i18n::get_translation(state, locale, "status-disabled").await;
    let action_view = crate::i18n::get_translation(state, locale, "action-view").await;
    let action_enable = crate::i18n::get_translation(state, locale, "action-enable").await;
    let action_disable = crate::i18n::get_translation(state, locale, "action-disable").await;
    let delete_confirm =
        crate::i18n::get_translation(state, locale, "relocated-delete-confirm").await;
    let empty_title = crate::i18n::get_translation(state, locale, "relocated-empty-title").await;
    let empty_description =
        crate::i18n::get_translation(state, locale, "relocated-empty-description").await;

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
    let title = crate::i18n::get_translation(state, locale, "relocated-show-title").await;
    let action_edit = crate::i18n::get_translation(state, locale, "action-edit").await;
    let action_enable = crate::i18n::get_translation(state, locale, "action-enable").await;
    let action_disable = crate::i18n::get_translation(state, locale, "action-disable").await;
    let action_delete = crate::i18n::get_translation(state, locale, "action-delete").await;
    let delete_confirm =
        crate::i18n::get_translation(state, locale, "relocated-delete-confirm").await;
    let back_to_list = crate::i18n::get_translation(state, locale, "relocated-back-to-list").await;
    let field_id = crate::i18n::get_translation(state, locale, "relocated-field-id").await;
    let field_old_address =
        crate::i18n::get_translation(state, locale, "relocated-field-old-address").await;
    let field_new_address =
        crate::i18n::get_translation(state, locale, "relocated-field-new-address").await;
    let field_enabled =
        crate::i18n::get_translation(state, locale, "relocated-field-enabled").await;
    let field_created =
        crate::i18n::get_translation(state, locale, "relocated-field-created").await;
    let field_modified =
        crate::i18n::get_translation(state, locale, "relocated-field-modified").await;
    let status_enabled = crate::i18n::get_translation(state, locale, "status-enabled").await;
    let status_disabled = crate::i18n::get_translation(state, locale, "status-disabled").await;
    let view_edit_settings =
        crate::i18n::get_translation(state, locale, "relocated-view-edit-settings").await;
    let relocated_show_title =
        crate::i18n::get_translation(state, locale, "relocated-show-title").await;
    let relocated_info_title =
        crate::i18n::get_translation(state, locale, "relocated-info-title").await;
    let relocated_info_description =
        crate::i18n::get_translation(state, locale, "relocated-info-description").await;

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
    let title = crate::i18n::get_translation(state, locale, title_key).await;
    let action = crate::i18n::get_translation(state, locale, action_key).await;
    let field_old_address =
        crate::i18n::get_translation(state, locale, "relocated-field-old-address").await;
    let field_new_address =
        crate::i18n::get_translation(state, locale, "relocated-field-new-address").await;
    let field_enabled =
        crate::i18n::get_translation(state, locale, "relocated-field-enabled").await;
    let field_old_address_help =
        crate::i18n::get_translation(state, locale, "relocated-field-old-address-help").await;
    let field_new_address_help =
        crate::i18n::get_translation(state, locale, "relocated-field-new-address-help").await;
    let action_save = crate::i18n::get_translation(state, locale, "action-save").await;
    let action_cancel = crate::i18n::get_translation(state, locale, "action-cancel").await;
    let back_to_list = crate::i18n::get_translation(state, locale, "relocated-back-to-list").await;
    let placeholder_old_address =
        crate::i18n::get_translation(state, locale, "relocated-placeholder-old-address").await;
    let placeholder_new_address =
        crate::i18n::get_translation(state, locale, "relocated-placeholder-new-address").await;

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

    render_form_template(content_template, state, locale, headers, title.clone()).await
}

// Duplicate Wizard rendering functions

/// Render duplicate domain selection page
pub async fn render_duplicate_domain_selection_page(
    domains: Vec<crate::models::Domain>,
    state: &AppState,
    locale: &str,
    headers: &HeaderMap,
) -> Html<String> {
    // Fetch all required translations for duplicate domain selection
    let title = crate::i18n::get_translation(state, locale, "duplicate-wizard-title").await;
    let description =
        crate::i18n::get_translation(state, locale, "duplicate-wizard-description").await;
    let source_domain_label =
        crate::i18n::get_translation(state, locale, "duplicate-wizard-source-domain-label").await;
    let source_domain_placeholder =
        crate::i18n::get_translation(state, locale, "duplicate-wizard-source-domain-placeholder").await;
    let source_domain_description =
        crate::i18n::get_translation(state, locale, "duplicate-wizard-source-domain-description").await;
    let new_domain_section_title =
        crate::i18n::get_translation(state, locale, "duplicate-wizard-new-domain-section-title").await;
    let new_domain_label = crate::i18n::get_translation(state, locale, "duplicate-wizard-new-domain-label").await;
    let new_domain_placeholder =
        crate::i18n::get_translation(state, locale, "duplicate-wizard-new-domain-placeholder").await;
    let new_domain_description =
        crate::i18n::get_translation(state, locale, "duplicate-wizard-new-domain-description").await;
    let transport_label = crate::i18n::get_translation(state, locale, "form-transport").await;
    let transport_description =
        crate::i18n::get_translation(state, locale, "duplicate-wizard-transport-description").await;
    let transport_virtual = crate::i18n::get_translation(state, locale, "duplicate-wizard-transport-virtual").await;
    let transport_smtp = crate::i18n::get_translation(state, locale, "duplicate-wizard-transport-smtp").await;
    let enabled_label = crate::i18n::get_translation(state, locale, "form-enabled").await;
    let duplicate_section_title =
        crate::i18n::get_translation(state, locale, "duplicate-wizard-section-title").await;
    let duplicate_aliases_label =
        crate::i18n::get_translation(state, locale, "duplicate-wizard-aliases-label").await;
    let duplicate_relays_label =
        crate::i18n::get_translation(state, locale, "duplicate-wizard-relays-label").await;
    let next_button = crate::i18n::get_translation(state, locale, "wizard-next").await;
    let cancel_button = crate::i18n::get_translation(state, locale, "form-cancel").await;

    let content_template = crate::templates::wizard::DuplicateDomainSelectionTemplate {
        title: &title,
        description: &description,
        error: "",
        domains: &domains,
        source_domain_label: &source_domain_label,
        source_domain_placeholder: &source_domain_placeholder,
        source_domain_description: &source_domain_description,
        new_domain_section_title: &new_domain_section_title,
        new_domain_label: &new_domain_label,
        new_domain_placeholder: &new_domain_placeholder,
        new_domain_description: &new_domain_description,
        transport_label: &transport_label,
        transport_description: &transport_description,
        transport_virtual: &transport_virtual,
        transport_smtp: &transport_smtp,
        enabled_label: &enabled_label,
        duplicate_section_title: &duplicate_section_title,
        duplicate_aliases_label: &duplicate_aliases_label,
        duplicate_relays_label: &duplicate_relays_label,
        next_button: &next_button,
        cancel_button: &cancel_button,
    };

    render_form_template(content_template, state, locale, headers, title.clone()).await
}

/// Render duplicate domain review page
pub async fn render_duplicate_domain_review_page(
    session: &crate::models::DuplicateDomainSession,
    state: &AppState,
    locale: &str,
    headers: &HeaderMap,
) -> Html<String> {
    // Fetch all required translations for duplicate domain review
    let title = crate::i18n::get_translation(state, locale, "duplicate-wizard-review-title").await;
    let description = crate::i18n::get_translation(state, locale, "duplicate-wizard-review-description").await;
    let source_domain_title =
        crate::i18n::get_translation(state, locale, "duplicate-wizard-source-domain-title").await;
    let new_domain_title = crate::i18n::get_translation(state, locale, "duplicate-wizard-new-domain-title").await;
    let items_to_duplicate_title =
        crate::i18n::get_translation(state, locale, "duplicate-wizard-items-to-duplicate-title").await;
    let domain_label = crate::i18n::get_translation(state, locale, "form-domain").await;
    let transport_label = crate::i18n::get_translation(state, locale, "form-transport").await;
    let enabled_label = crate::i18n::get_translation(state, locale, "form-enabled").await;
    let enabled_status = crate::i18n::get_translation(state, locale, "status-enabled").await;
    let disabled_status = crate::i18n::get_translation(state, locale, "status-disabled").await;
    let aliases_label = crate::i18n::get_translation(state, locale, "duplicate-wizard-aliases-label").await;
    let relays_label = crate::i18n::get_translation(state, locale, "duplicate-wizard-relays-label").await;
    let yes_status = crate::i18n::get_translation(state, locale, "duplicate-wizard-yes-status").await;
    let no_status = crate::i18n::get_translation(state, locale, "duplicate-wizard-no-status").await;
    let items_label = crate::i18n::get_translation(state, locale, "duplicate-wizard-items-label").await;
    let back_button = crate::i18n::get_translation(state, locale, "wizard-back").await;
    let cancel_button = crate::i18n::get_translation(state, locale, "form-cancel").await;
    let confirm_button = crate::i18n::get_translation(state, locale, "wizard-confirm").await;

    // Get source domain info
    let source_domain = session
        .source_domain
        .as_ref()
        .map(|d| d.domain.as_str())
        .unwrap_or("None");
    let source_transport = session
        .source_domain
        .as_ref()
        .map(|d| d.transport_display())
        .unwrap_or("None".to_string());

    let content_template = crate::templates::wizard::DuplicateReviewTemplate {
        title: &title,
        description: &description,
        source_domain_title: &source_domain_title,
        source_domain,
        source_transport: &source_transport,
        source_enabled: session
            .source_domain
            .as_ref()
            .map(|d| d.enabled)
            .unwrap_or(false),
        new_domain_title: &new_domain_title,
        new_domain: &session.new_domain,
        new_transport: &session.transport,
        new_enabled: session.enabled,
        items_to_duplicate_title: &items_to_duplicate_title,
        duplicate_aliases: session.duplicate_aliases,
        aliases_count: session.aliases_to_duplicate.len(),
        duplicate_relays: session.duplicate_relays,
        relays_count: session.relays_to_duplicate.len(),
        domain_label: &domain_label,
        transport_label: &transport_label,
        enabled_label: &enabled_label,
        enabled_status: &enabled_status,
        disabled_status: &disabled_status,
        aliases_label: &aliases_label,
        relays_label: &relays_label,
        yes_status: &yes_status,
        no_status: &no_status,
        items_label: &items_label,
        back_button: &back_button,
        cancel_button: &cancel_button,
        confirm_button: &confirm_button,
    };

    render_form_template(content_template, state, locale, headers, title.clone()).await
}

/// Render duplicate domain complete page
pub async fn render_duplicate_domain_complete_page(
    source_domain: &str,
    new_domain: &str,
    new_domain_id: i32,
    state: &AppState,
    locale: &str,
    headers: &HeaderMap,
) -> Html<String> {
    // Fetch all required translations for duplicate domain complete
    let title = crate::i18n::get_translation(state, locale, "duplicate-wizard-complete-title").await;
    let description = crate::i18n::get_translation(state, locale, "duplicate-wizard-complete-description").await;
    let success_message = crate::i18n::get_translation(state, locale, "duplicate-wizard-success-message").await;
    let source_label = crate::i18n::get_translation(state, locale, "duplicate-wizard-source-label").await;
    let destination_label = crate::i18n::get_translation(state, locale, "duplicate-wizard-destination-label").await;
    let view_domain_button =
        crate::i18n::get_translation(state, locale, "duplicate-wizard-view-domain-button").await;
    let back_to_domains_button =
        crate::i18n::get_translation(state, locale, "duplicate-wizard-back-to-domains-button").await;
    let duplicate_another_button =
        crate::i18n::get_translation(state, locale, "duplicate-wizard-duplicate-another-button").await;

    let content_template = crate::templates::wizard::DuplicateCompleteTemplate {
        title: &title,
        description: &description,
        success_message: &success_message,
        source_domain,
        new_domain,
        new_domain_id,
        source_label: &source_label,
        destination_label: &destination_label,
        view_domain_button: &view_domain_button,
        back_to_domains_button: &back_to_domains_button,
        duplicate_another_button: &duplicate_another_button,
    };

    render_form_template(content_template, state, locale, headers, title.clone()).await
}
