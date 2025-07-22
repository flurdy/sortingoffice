use crate::{
    db,
    handlers::utils::{get_entity_form_translations, get_field_translations},
    i18n::get_translation,
    models::{DomainForm, NewDomain, PaginatedResult, PaginationParams},
    render_template_with_title,
    templates::domains::{DomainFormTemplate, DomainShowTemplate, DomainsListTemplate},
    AppState,
};
use askama::Template;
use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    response::Html,
    Form,
};
use std::collections::HashMap;
use tracing::{error, info};

// Helper function to get domain list translations
async fn get_domain_list_translations(state: &AppState, locale: &str) -> HashMap<String, String> {
    crate::handlers::utils::get_translations_batch(
        state,
        locale,
        &[
            "domains-title",
            "domains-description",
            "domains-add",
            "domains-table-header-domain",
            "domains-table-header-enabled",
            "domains-table-header-actions",
            "domains-transport",
            "status-active",
            "status-inactive",
            "action-view",
            "action-enable",
            "action-disable",
            "domains-empty-title",
            "domains-empty-description",
            // Backups
            "backups-title",
            "backups-description",
            "backups-add",
            "backups-table-header-domain",
            "backups-table-header-transport",
            "backups-table-header-enabled",
            "backups-table-header-actions",
            "backups-view",
            "backups-enable",
            "backups-disable",
            "backups-empty-no-backup-servers",
            "backups-empty-get-started",
        ],
    )
    .await
}

// Helper function to get domain show translations
async fn get_domain_show_translations(state: &AppState, locale: &str) -> HashMap<String, String> {
    crate::handlers::utils::get_translations_batch(
        state,
        locale,
        &[
            "domains-title",
            "domains-view-edit-settings",
            "domains-back-to-domains",
            "domains-domain-information",
            "domains-domain-details",
            "domains-domain-name",
            "domains-transport",
            "domains-status",
            "status-active",
            "status-inactive",
            "domains-created",
            "domains-modified",
            "domains-edit-domain-button",
            "domains-enable-domain",
            "domains-disable-domain",
            "domains-delete-domain",
            "domains-delete-confirm",
            // Alias report/related
            "domains-alias-report-title",
            "domains-alias-report-description",
            "domains-existing-aliases-header",
            "reports-catch-all-header",
            "reports-destination-header",
            "reports-required-aliases-header",
            "reports-missing-aliases-header",
            "reports-missing-required-aliases-header",
            "reports-missing-common-aliases-header",
            "reports-mail-header",
            "reports-status-header",
            "reports-enabled-header",
            "reports-actions-header",
            "reports-no-required-aliases",
            "reports-no-missing-aliases",
            "domains-add-missing-required-aliases-button",
            "reports-add-common-alias-button",
            "domains-add-catch-all-button",
            "domains-add-alias-button",
            "domains-no-catch-all-message",
            "action-view",
            "aliases-enable-alias",
            "aliases-disable-alias",
        ],
    )
    .await
}

// Helper function to validate domain form
fn validate_domain_form(form: &DomainForm) -> Result<(), String> {
    if form.domain.trim().is_empty() {
        return Err("validation-domain-required".to_string());
    }
    Ok(())
}

// Helper function to handle domain form errors
async fn handle_domain_form_error(
    state: &AppState,
    locale: &str,
    headers: &HeaderMap,
    form: DomainForm,
    error_key: &str,
    is_edit: bool,
) -> Html<String> {
    let error_msg = get_translation(state, locale, error_key).await;

    let form_translations =
        crate::handlers::utils::get_entity_form_translations(state, locale, "domains").await;
    let field_translations = crate::handlers::utils::get_field_translations(
        state,
        locale,
        "domains",
        &["domain", "transport", "active"],
    )
    .await;

    // Create owned strings to avoid lifetime issues
    let title = if is_edit {
        form_translations
            .get("domains-edit-domain")
            .unwrap_or(&"Edit Domain".to_string())
            .clone()
    } else {
        form_translations
            .get("domains-new-domain")
            .unwrap_or(&"New Domain".to_string())
            .clone()
    };

    let form_error = form_translations
        .get("form-error")
        .unwrap_or(&"Form Error".to_string())
        .clone();
    let form_domain = field_translations
        .get("domains-field-domain")
        .unwrap_or(&"Domain".to_string())
        .clone();
    let form_transport = field_translations
        .get("domains-field-transport")
        .unwrap_or(&"Transport".to_string())
        .clone();
    let form_active = field_translations
        .get("domains-field-active")
        .unwrap_or(&"Active".to_string())
        .clone();
    let form_cancel = form_translations
        .get("form-cancel")
        .unwrap_or(&"Cancel".to_string())
        .clone();
    let form_create_domain = form_translations
        .get("action-save")
        .unwrap_or(&"Save".to_string())
        .clone();
    let form_update_domain = form_translations
        .get("action-save")
        .unwrap_or(&"Save".to_string())
        .clone();
    let form_placeholder_domain = field_translations
        .get("domains-placeholder-domain")
        .unwrap_or(&"Enter domain".to_string())
        .clone();
    let form_placeholder_transport = field_translations
        .get("domains-placeholder-transport")
        .unwrap_or(&"Enter transport".to_string())
        .clone();
    let form_tooltip_domain = field_translations
        .get("domains-field-domain-help")
        .unwrap_or(&"Domain tooltip".to_string())
        .clone();
    let form_tooltip_transport = field_translations
        .get("domains-field-transport-help")
        .unwrap_or(&"Transport tooltip".to_string())
        .clone();
    let form_tooltip_enable = field_translations
        .get("domains-field-active-help")
        .unwrap_or(&"Active tooltip".to_string())
        .clone();

    let content_template = DomainFormTemplate {
        title: &title,
        domain: None,
        form,
        error: Some(error_msg),
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
        form_enabled: &form_translations["form-enabled"],
        form_disabled: &form_translations["form-disabled"],
    };

    crate::handlers::utils::render_form_template(
        content_template,
        state,
        locale,
        headers,
        title.clone(),
    )
    .await
}

pub async fn list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<PaginationParams>,
) -> Html<String> {
    let pool = match crate::handlers::utils::get_current_db_pool(&state, &headers).await {
        Ok(pool) => pool,
        Err(e) => {
            error!("Failed to get database pool: {:?}", e);
            return Html("Database connection error".to_string());
        }
    };

    let locale = crate::handlers::utils::get_user_locale(&headers);
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(20);

    // Get domains with error handling
    let paginated_domains = match db::get_domains_paginated(&pool, page, per_page) {
        Ok(domains) => domains,
        Err(e) => {
            error!("Failed to retrieve domains: {:?}", e);
            PaginatedResult::new(vec![], 0, 1, per_page)
        }
    };

    // Get backups with error handling
    let backups = match db::get_backups(&pool) {
        Ok(backups) => backups,
        Err(e) => {
            error!("Failed to retrieve backups: {:?}", e);
            vec![]
        }
    };

    // Create template directly
    let translations = get_domain_list_translations(&state, &locale).await;

    let paginated = PaginatedResult::new(
        paginated_domains.items.clone(),
        paginated_domains.total_count,
        paginated_domains.current_page,
        paginated_domains.per_page,
    );
    let page_range: Vec<i64> = (1..=paginated.total_pages).collect();
    let max_item = std::cmp::min(
        paginated.current_page * paginated.per_page,
        paginated.total_count,
    );

    let content_template = DomainsListTemplate {
        title: &translations["domains-title"],
        description: &translations["domains-description"],
        add_domain: &translations["domains-add"],
        table_header_domain: &translations["domains-table-header-domain"],
        table_header_enabled: &translations["domains-table-header-enabled"],
        table_header_actions: &translations["domains-table-header-actions"],
        table_header_transport: &translations["domains-transport"],
        status_active: &translations["status-active"],
        status_inactive: &translations["status-inactive"],
        action_view: &translations["action-view"],
        action_enable: &translations["action-enable"],
        action_disable: &translations["action-disable"],
        empty_title: &translations["domains-empty-title"],
        empty_description: &translations["domains-empty-description"],
        domains: &paginated_domains.items,
        pagination: &paginated,
        page_range: &page_range,
        max_item,
        backups_title: &translations["backups-title"],
        backups_description: &translations["backups-description"],
        add_backup: &translations["backups-add"],
        backups_table_header_domain: &translations["backups-table-header-domain"],
        backups_table_header_transport: &translations["backups-table-header-transport"],
        backups_table_header_enabled: &translations["backups-table-header-enabled"],
        backups_table_header_actions: &translations["backups-table-header-actions"],
        backups: &backups,
        backups_view: &translations["backups-view"],
        backups_enable: &translations["backups-enable"],
        backups_disable: &translations["backups-disable"],
        backups_empty_no_backup_servers: &translations["backups-empty-no-backup-servers"],
        backups_empty_get_started: &translations["backups-empty-get-started"],
    };

    render_template_with_title!(
        content_template,
        content_template.title,
        &state,
        &locale,
        &headers
    )
}

pub async fn new(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    let locale = crate::handlers::utils::get_user_locale(&headers);
    let form = DomainForm {
        domain: "".to_string(),
        transport: "virtual".to_string(),
        enabled: true,
    };

    // Use helper functions to fetch translations in batches
    let mut form_translations = get_entity_form_translations(&state, &locale, "domains").await;
    let field_translations = get_field_translations(
        &state,
        &locale,
        "domains",
        &["domain", "transport", "active"],
    )
    .await;

    // Merge field translations into form translations
    form_translations.extend(field_translations);

    let content_template = DomainFormTemplate {
        title: &form_translations["domains-add-title"],
        domain: None,
        form,
        error: None,
        form_error: &form_translations["form-error"],
        form_domain: &form_translations["domains-field-domain"],
        form_transport: &form_translations["domains-field-transport"],
        form_active: &form_translations["domains-field-active"],
        form_cancel: &form_translations["form-cancel"],
        form_create_domain: &form_translations["form-create-domain"],
        form_update_domain: &form_translations["form-update-domain"],
        form_placeholder_domain: &form_translations["form-placeholder-domain"],
        form_placeholder_transport: &form_translations["form-placeholder-transport"],
        form_tooltip_domain: &form_translations["domains-field-domain-help"],
        form_tooltip_transport: &form_translations["domains-field-transport-help"],
        form_tooltip_enable: &form_translations["domains-field-active-help"],
        form_enabled: &form_translations["form-enabled"],
        form_disabled: &form_translations["form-disabled"],
    };

    // Use helper function for template rendering
    crate::handlers::utils::render_form_template(
        content_template,
        &state,
        &locale,
        &headers,
        form_translations["domains-add-title"].clone(),
    )
    .await
}

pub async fn show(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = match crate::handlers::utils::get_current_db_pool(&state, &headers).await {
        Ok(pool) => pool,
        Err(e) => {
            error!("Failed to get database pool: {:?}", e);
            return Html("Database connection error".to_string());
        }
    };

    let locale = crate::handlers::utils::get_user_locale(&headers);

    // Get domain with proper error handling
    let domain = match db::get_domain(&pool, id) {
        Ok(domain) => domain,
        Err(_) => {
            let not_found_msg = get_translation(&state, &locale, "domains-not-found").await;
            return Html(not_found_msg);
        }
    };

    // Get alias report and existing aliases
    let alias_report = db::get_domain_alias_report(&pool, &domain.domain).ok();
    let existing_aliases = db::get_aliases_for_domain(&pool, &domain.domain).unwrap_or_default();

    // Create template directly
    let translations = get_domain_show_translations(&state, &locale).await;

    let content_template = DomainShowTemplate {
        title: &translations["domains-title"],
        domain,
        view_edit_settings: &translations["domains-view-edit-settings"],
        back_to_domains: &translations["domains-back-to-domains"],
        domain_information: &translations["domains-domain-information"],
        domain_details: &translations["domains-domain-details"],
        domain_name: &translations["domains-domain-name"],
        transport: &translations["domains-transport"],
        status: &translations["domains-status"],
        status_active: &translations["status-active"],
        status_inactive: &translations["status-inactive"],
        created: &translations["domains-created"],
        modified: &translations["domains-modified"],
        edit_domain_button: &translations["domains-edit-domain-button"],
        enable_domain: &translations["domains-enable-domain"],
        disable_domain: &translations["domains-disable-domain"],
        delete_domain: &translations["domains-delete-domain"],
        delete_confirm: &translations["domains-delete-confirm"],
        alias_report,
        catch_all_header: &translations["reports-catch-all-header"],
        destination_header: &translations["reports-destination-header"],
        required_aliases_header: &translations["reports-required-aliases-header"],
        missing_aliases_header: &translations["reports-missing-aliases-header"],
        missing_required_alias_header: &translations["reports-missing-required-aliases-header"],
        missing_common_aliases_header: &translations["reports-missing-common-aliases-header"],
        mail_header: &translations["reports-mail-header"],
        status_header: &translations["reports-status-header"],
        enabled_header: &translations["reports-enabled-header"],
        actions_header: &translations["reports-actions-header"],
        no_required_aliases: &translations["reports-no-required-aliases"],
        no_missing_aliases: &translations["reports-no-missing-aliases"],
        alias_report_title: &translations["domains-alias-report-title"],
        alias_report_description: &translations["domains-alias-report-description"],
        existing_aliases_header: &translations["domains-existing-aliases-header"],
        add_missing_required_alias_button: &translations
            ["domains-add-missing-required-aliases-button"],
        add_common_alias_button: &translations["reports-add-common-alias-button"],
        add_catch_all_button: &translations["domains-add-catch-all-button"],
        add_alias_button: &translations["domains-add-alias-button"],
        no_catch_all_message: &translations["domains-no-catch-all-message"],
        existing_aliases: &existing_aliases,
        action_view: &translations["action-view"],
        enable_alias: &translations["aliases-enable-alias"],
        disable_alias: &translations["aliases-disable-alias"],
    };

    render_template_with_title!(
        content_template,
        content_template.title,
        &state,
        &locale,
        &headers
    )
}

pub async fn edit(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = match crate::handlers::utils::get_current_db_pool(&state, &headers).await {
        Ok(pool) => pool,
        Err(e) => {
            error!("Failed to get database pool: {:?}", e);
            return Html("Database connection error".to_string());
        }
    };

    let locale = crate::handlers::utils::get_user_locale(&headers);

    // Get domain with proper error handling
    let domain = match db::get_domain(&pool, id) {
        Ok(domain) => domain,
        Err(_) => {
            let not_found_msg = get_translation(&state, &locale, "domains-not-found").await;
            return Html(not_found_msg);
        }
    };

    let form = DomainForm {
        domain: domain.domain.clone(),
        transport: domain.transport.clone().unwrap_or_default(),
        enabled: domain.enabled,
    };

    // Use helper functions to fetch translations in batches
    let form_translations = crate::handlers::utils::get_translations_batch(
        &state,
        &locale,
        &[
            "domains-edit-domain",
            "form-error",
            "form-cancel",
            "action-save",
        ],
    )
    .await;
    let field_translations = crate::handlers::utils::get_field_translations(
        &state,
        &locale,
        "domains",
        &["domain", "transport", "active"],
    )
    .await;

    let content_template = DomainFormTemplate {
        title: form_translations
            .get("domains-edit-domain")
            .map(|s| s.as_str())
            .unwrap_or("Edit Domain"),
        domain: Some(domain),
        form,
        error: None,
        form_error: form_translations
            .get("form-error")
            .map(|s| s.as_str())
            .unwrap_or("Form Error"),
        form_domain: field_translations
            .get("domains-field-domain")
            .map(|s| s.as_str())
            .unwrap_or("Domain"),
        form_transport: field_translations
            .get("domains-field-transport")
            .map(|s| s.as_str())
            .unwrap_or("Transport"),
        form_active: field_translations
            .get("domains-field-active")
            .map(|s| s.as_str())
            .unwrap_or("Active"),
        form_cancel: form_translations
            .get("form-cancel")
            .map(|s| s.as_str())
            .unwrap_or("Cancel"),
        form_create_domain: form_translations
            .get("action-save")
            .map(|s| s.as_str())
            .unwrap_or("Save"),
        form_update_domain: form_translations
            .get("action-save")
            .map(|s| s.as_str())
            .unwrap_or("Save"),
        form_placeholder_domain: field_translations
            .get("domains-placeholder-domain")
            .map(|s| s.as_str())
            .unwrap_or("example.com"),
        form_placeholder_transport: field_translations
            .get("domains-placeholder-transport")
            .map(|s| s.as_str())
            .unwrap_or("virtual"),
        form_tooltip_domain: field_translations
            .get("domains-field-domain-help")
            .map(|s| s.as_str())
            .unwrap_or("Domain tooltip"),
        form_tooltip_transport: field_translations
            .get("domains-field-transport-help")
            .map(|s| s.as_str())
            .unwrap_or("Transport tooltip"),
        form_tooltip_enable: field_translations
            .get("domains-field-active-help")
            .map(|s| s.as_str())
            .unwrap_or("Active tooltip"),
        form_enabled: form_translations
            .get("form-enabled")
            .map(|s| s.as_str())
            .unwrap_or("Enabled"),
        form_disabled: form_translations
            .get("form-disabled")
            .map(|s| s.as_str())
            .unwrap_or("Disabled"),
    };

    // Use helper function for template rendering
    crate::handlers::utils::render_form_template(
        content_template,
        &state,
        &locale,
        &headers,
        form_translations["domains-edit-domain"].clone(),
    )
    .await
}

pub async fn create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<DomainForm>,
) -> Html<String> {
    let locale = crate::handlers::utils::get_user_locale(&headers);

    // Get current database ID for restriction checks
    let current_db_id = crate::handlers::auth::get_selected_database(&headers)
        .unwrap_or_else(|| state.db_manager.get_default_db_id().to_string());

    // Check database restrictions
    if let Err(_status_code) =
        crate::handlers::utils::check_database_restrictions(&state, &current_db_id, "create_domain")
    {
        return handle_domain_form_error(
            &state,
            &locale,
            &headers,
            form,
            "error-operation-not-allowed",
            false,
        )
        .await;
    }

    // Validate form data
    if let Err(error_key) = validate_domain_form(&form) {
        return handle_domain_form_error(&state, &locale, &headers, form, &error_key, false).await;
    }

    let pool = match crate::handlers::utils::get_current_db_pool(&state, &headers).await {
        Ok(pool) => pool,
        Err(e) => {
            error!("Failed to get database pool: {:?}", e);
            return Html("Database connection error".to_string());
        }
    };

    let new_domain = NewDomain {
        domain: form.domain.trim().to_string(),
        transport: Some(form.transport.clone()),
        enabled: form.enabled,
    };

    match db::create_domain(&pool, new_domain) {
        Ok(_) => {
            info!("Successfully created domain: {}", form.domain);

            // Redirect to domains list
            let domains = match db::get_domains(&pool) {
                Ok(domains) => domains,
                Err(e) => {
                    error!("Failed to retrieve domains after creation: {:?}", e);
                    vec![]
                }
            };

            let backups = match db::get_backups(&pool) {
                Ok(backups) => backups,
                Err(e) => {
                    error!("Failed to retrieve backups: {:?}", e);
                    vec![]
                }
            };

            let paginated_domains =
                PaginatedResult::new(domains.clone(), domains.len() as i64, 1, 20);

            let translations = get_domain_list_translations(&state, &locale).await;

            let page_range: Vec<i64> = (1..=paginated_domains.total_pages).collect();
            let max_item = std::cmp::min(
                paginated_domains.current_page * paginated_domains.per_page,
                paginated_domains.total_count,
            );

            let content_template = DomainsListTemplate {
                title: &translations["domains-title"],
                description: &translations["domains-description"],
                add_domain: &translations["domains-add"],
                table_header_domain: &translations["domains-table-header-domain"],
                table_header_enabled: &translations["domains-table-header-enabled"],
                table_header_actions: &translations["domains-table-header-actions"],
                table_header_transport: &translations["domains-transport"],
                status_active: &translations["status-active"],
                status_inactive: &translations["status-inactive"],
                action_view: &translations["action-view"],
                action_enable: &translations["action-enable"],
                action_disable: &translations["action-disable"],
                empty_title: &translations["domains-empty-title"],
                empty_description: &translations["domains-empty-description"],
                domains: &paginated_domains.items,
                pagination: &paginated_domains,
                page_range: &page_range,
                max_item,
                backups_title: &translations["backups-title"],
                backups_description: &translations["backups-description"],
                add_backup: &translations["backups-add"],
                backups_table_header_domain: &translations["backups-table-header-domain"],
                backups_table_header_transport: &translations["backups-table-header-transport"],
                backups_table_header_enabled: &translations["backups-table-header-enabled"],
                backups_table_header_actions: &translations["backups-table-header-actions"],
                backups: &backups,
                backups_view: &translations["backups-view"],
                backups_enable: &translations["backups-enable"],
                backups_disable: &translations["backups-disable"],
                backups_empty_no_backup_servers: &translations["backups-empty-no-backup-servers"],
                backups_empty_get_started: &translations["backups-empty-get-started"],
            };

            render_template_with_title!(
                content_template,
                content_template.title,
                &state,
                &locale,
                &headers
            )
        }
        Err(e) => {
            error!("Failed to create domain: {:?}", e);
            let error_message = crate::handlers::utils::handle_database_error(
                &state,
                &locale,
                e,
                "domain",
                &form.domain,
            )
            .await;

            return handle_domain_form_error(
                &state,
                &locale,
                &headers,
                form,
                &error_message,
                false,
            )
            .await;
        }
    }
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
    Form(form): Form<DomainForm>,
) -> Html<String> {
    let locale = crate::handlers::utils::get_user_locale(&headers);

    // Get current database ID for restriction checks
    let current_db_id = crate::handlers::auth::get_selected_database(&headers)
        .unwrap_or_else(|| state.db_manager.get_default_db_id().to_string());

    // Check database restrictions
    if let Err(_status_code) =
        crate::handlers::utils::check_database_restrictions(&state, &current_db_id, "update_domain")
    {
        return handle_domain_form_error(
            &state,
            &locale,
            &headers,
            form,
            "error-operation-not-allowed",
            true,
        )
        .await;
    }

    // Validate form data
    if let Err(error_key) = validate_domain_form(&form) {
        return handle_domain_form_error(&state, &locale, &headers, form, &error_key, true).await;
    }

    let pool = match crate::handlers::utils::get_current_db_pool(&state, &headers).await {
        Ok(pool) => pool,
        Err(e) => {
            error!("Failed to get database pool: {:?}", e);
            return Html("Database connection error".to_string());
        }
    };

    let domain_name = form.domain.clone();
    match db::update_domain(&pool, id, form) {
        Ok(_) => {
            info!("Successfully updated domain: {}", domain_name);

            // Get updated domain
            let domain = match db::get_domain(&pool, id) {
                Ok(domain) => domain,
                Err(_) => {
                    let not_found_msg = get_translation(&state, &locale, "domains-not-found").await;
                    return Html(not_found_msg);
                }
            };

            // Get alias report and existing aliases
            let alias_report = db::get_domain_alias_report(&pool, &domain.domain).ok();
            let existing_aliases =
                db::get_aliases_for_domain(&pool, &domain.domain).unwrap_or_default();

            // Create template directly
            let translations = get_domain_show_translations(&state, &locale).await;

            let content_template = DomainShowTemplate {
                title: &translations["domains-title"],
                domain,
                view_edit_settings: &translations["domains-view-edit-settings"],
                back_to_domains: &translations["domains-back-to-domains"],
                domain_information: &translations["domains-domain-information"],
                domain_details: &translations["domains-domain-details"],
                domain_name: &translations["domains-domain-name"],
                transport: &translations["domains-transport"],
                status: &translations["domains-status"],
                status_active: &translations["status-active"],
                status_inactive: &translations["status-inactive"],
                created: &translations["domains-created"],
                modified: &translations["domains-modified"],
                edit_domain_button: &translations["domains-edit-domain-button"],
                enable_domain: &translations["domains-enable-domain"],
                disable_domain: &translations["domains-disable-domain"],
                delete_domain: &translations["domains-delete-domain"],
                delete_confirm: &translations["domains-delete-confirm"],
                alias_report,
                catch_all_header: &translations["reports-catch-all-header"],
                destination_header: &translations["reports-destination-header"],
                required_aliases_header: &translations["reports-required-aliases-header"],
                missing_aliases_header: &translations["reports-missing-aliases-header"],
                missing_required_alias_header: &translations
                    ["reports-missing-required-aliases-header"],
                missing_common_aliases_header: &translations
                    ["reports-missing-common-aliases-header"],
                mail_header: &translations["reports-mail-header"],
                status_header: &translations["reports-status-header"],
                enabled_header: &translations["reports-enabled-header"],
                actions_header: &translations["reports-actions-header"],
                no_required_aliases: &translations["reports-no-required-aliases"],
                no_missing_aliases: &translations["reports-no-missing-aliases"],
                alias_report_title: &translations["domains-alias-report-title"],
                alias_report_description: &translations["domains-alias-report-description"],
                existing_aliases_header: &translations["domains-existing-aliases-header"],
                add_missing_required_alias_button: &translations
                    ["domains-add-missing-required-aliases-button"],
                add_common_alias_button: &translations["reports-add-common-alias-button"],
                add_catch_all_button: &translations["domains-add-catch-all-button"],
                add_alias_button: &translations["domains-add-alias-button"],
                no_catch_all_message: &translations["domains-no-catch-all-message"],
                existing_aliases: &existing_aliases,
                action_view: &translations["action-view"],
                enable_alias: &translations["aliases-enable-alias"],
                disable_alias: &translations["aliases-disable-alias"],
            };

            render_template_with_title!(
                content_template,
                content_template.title,
                &state,
                &locale,
                &headers
            )
        }
        Err(e) => {
            error!("Failed to update domain: {:?}", e);
            let error_message = crate::handlers::utils::handle_database_error(
                &state,
                &locale,
                e,
                "domain",
                &domain_name,
            )
            .await;

            // Recreate the form for error display
            let error_form = DomainForm {
                domain: domain_name,
                transport: "virtual".to_string(),
                enabled: true,
            };

            return handle_domain_form_error(
                &state,
                &locale,
                &headers,
                error_form,
                &error_message,
                true,
            )
            .await;
        }
    }
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = match crate::handlers::utils::get_current_db_pool(&state, &headers).await {
        Ok(pool) => pool,
        Err(e) => {
            error!("Failed to get database pool: {:?}", e);
            return Html("Database connection error".to_string());
        }
    };

    match db::delete_domain(&pool, id) {
        Ok(_) => {
            info!("Successfully deleted domain with ID: {}", id);

            let locale = crate::handlers::utils::get_user_locale(&headers);

            // Get updated domains list
            let domains = match db::get_domains(&pool) {
                Ok(domains) => domains,
                Err(e) => {
                    error!("Failed to retrieve domains after deletion: {:?}", e);
                    vec![]
                }
            };

            // Get backups data
            let backups = match db::get_backups(&pool) {
                Ok(backups) => backups,
                Err(e) => {
                    error!("Failed to retrieve backups: {:?}", e);
                    vec![]
                }
            };

            let paginated_domains =
                PaginatedResult::new(domains.clone(), domains.len() as i64, 1, 20);

            let translations = get_domain_list_translations(&state, &locale).await;

            let page_range: Vec<i64> = (1..=paginated_domains.total_pages).collect();
            let max_item = std::cmp::min(
                paginated_domains.current_page * paginated_domains.per_page,
                paginated_domains.total_count,
            );

            let content_template = DomainsListTemplate {
                title: &translations["domains-title"],
                description: &translations["domains-description"],
                add_domain: &translations["domains-add"],
                table_header_domain: &translations["domains-table-header-domain"],
                table_header_enabled: &translations["domains-table-header-enabled"],
                table_header_actions: &translations["domains-table-header-actions"],
                table_header_transport: &translations["domains-transport"],
                status_active: &translations["status-active"],
                status_inactive: &translations["status-inactive"],
                action_view: &translations["action-view"],
                action_enable: &translations["action-enable"],
                action_disable: &translations["action-disable"],
                empty_title: &translations["domains-empty-title"],
                empty_description: &translations["domains-empty-description"],
                domains: &paginated_domains.items,
                pagination: &paginated_domains,
                page_range: &page_range,
                max_item,
                backups_title: &translations["backups-title"],
                backups_description: &translations["backups-description"],
                add_backup: &translations["backups-add"],
                backups_table_header_domain: &translations["backups-table-header-domain"],
                backups_table_header_transport: &translations["backups-table-header-transport"],
                backups_table_header_enabled: &translations["backups-table-header-enabled"],
                backups_table_header_actions: &translations["backups-table-header-actions"],
                backups: &backups,
                backups_view: &translations["backups-view"],
                backups_enable: &translations["backups-enable"],
                backups_disable: &translations["backups-disable"],
                backups_empty_no_backup_servers: &translations["backups-empty-no-backup-servers"],
                backups_empty_get_started: &translations["backups-empty-get-started"],
            };

            render_template_with_title!(
                content_template,
                content_template.title,
                &state,
                &locale,
                &headers
            )
        }
        Err(e) => {
            error!("Failed to delete domain: {:?}", e);
            Html("Error deleting domain".to_string())
        }
    }
}

pub async fn toggle_enabled(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = match crate::handlers::utils::get_current_db_pool(&state, &headers).await {
        Ok(pool) => pool,
        Err(e) => {
            error!("Failed to get database pool: {:?}", e);
            return Html("Database connection error".to_string());
        }
    };

    let locale = crate::handlers::utils::get_user_locale(&headers);

    match db::toggle_domain_enabled(&pool, id) {
        Ok(_) => {
            info!("Successfully toggled domain enabled status for ID: {}", id);

            // Get updated domain
            let domain = match db::get_domain(&pool, id) {
                Ok(domain) => domain,
                Err(_) => {
                    let not_found_msg = get_translation(&state, &locale, "domains-not-found").await;
                    return Html(not_found_msg);
                }
            };

            // Get alias report and existing aliases
            let alias_report = db::get_domain_alias_report(&pool, &domain.domain).ok();
            let existing_aliases =
                db::get_aliases_for_domain(&pool, &domain.domain).unwrap_or_default();

            // Create template directly
            let translations = get_domain_show_translations(&state, &locale).await;

            let content_template = DomainShowTemplate {
                title: &translations["domains-title"],
                domain,
                view_edit_settings: &translations["domains-view-edit-settings"],
                back_to_domains: &translations["domains-back-to-domains"],
                domain_information: &translations["domains-domain-information"],
                domain_details: &translations["domains-domain-details"],
                domain_name: &translations["domains-domain-name"],
                transport: &translations["domains-transport"],
                status: &translations["domains-status"],
                status_active: &translations["status-active"],
                status_inactive: &translations["status-inactive"],
                created: &translations["domains-created"],
                modified: &translations["domains-modified"],
                edit_domain_button: &translations["domains-edit-domain-button"],
                enable_domain: &translations["domains-enable-domain"],
                disable_domain: &translations["domains-disable-domain"],
                delete_domain: &translations["domains-delete-domain"],
                delete_confirm: &translations["domains-delete-confirm"],
                alias_report,
                catch_all_header: &translations["reports-catch-all-header"],
                destination_header: &translations["reports-destination-header"],
                required_aliases_header: &translations["reports-required-aliases-header"],
                missing_aliases_header: &translations["reports-missing-aliases-header"],
                missing_required_alias_header: &translations
                    ["reports-missing-required-aliases-header"],
                missing_common_aliases_header: &translations
                    ["reports-missing-common-aliases-header"],
                mail_header: &translations["reports-mail-header"],
                status_header: &translations["reports-status-header"],
                enabled_header: &translations["reports-enabled-header"],
                actions_header: &translations["reports-actions-header"],
                no_required_aliases: &translations["reports-no-required-aliases"],
                no_missing_aliases: &translations["reports-no-missing-aliases"],
                alias_report_title: &translations["domains-alias-report-title"],
                alias_report_description: &translations["domains-alias-report-description"],
                existing_aliases_header: &translations["domains-existing-aliases-header"],
                add_missing_required_alias_button: &translations
                    ["domains-add-missing-required-aliases-button"],
                add_common_alias_button: &translations["reports-add-common-alias-button"],
                add_catch_all_button: &translations["domains-add-catch-all-button"],
                add_alias_button: &translations["domains-add-alias-button"],
                no_catch_all_message: &translations["domains-no-catch-all-message"],
                existing_aliases: &existing_aliases,
                action_view: &translations["action-view"],
                enable_alias: &translations["aliases-enable-alias"],
                disable_alias: &translations["aliases-disable-alias"],
            };

            render_template_with_title!(
                content_template,
                content_template.title,
                &state,
                &locale,
                &headers
            )
        }
        Err(e) => {
            error!("Failed to toggle domain enabled status: {:?}", e);
            Html("Error toggling domain status".to_string())
        }
    }
}

pub async fn toggle_enabled_list(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = match crate::handlers::utils::get_current_db_pool(&state, &headers).await {
        Ok(pool) => pool,
        Err(e) => {
            error!("Failed to get database pool: {:?}", e);
            return Html("Database connection error".to_string());
        }
    };

    match db::toggle_domain_enabled(&pool, id) {
        Ok(_) => {
            info!(
                "Successfully toggled domain enabled status for ID: {} (list view)",
                id
            );

            let locale = crate::handlers::utils::get_user_locale(&headers);

            // Get updated domains list
            let domains = match db::get_domains(&pool) {
                Ok(domains) => domains,
                Err(e) => {
                    error!("Failed to retrieve domains after toggle: {:?}", e);
                    vec![]
                }
            };

            // Get backups data
            let backups = match db::get_backups(&pool) {
                Ok(backups) => backups,
                Err(e) => {
                    error!("Failed to retrieve backups: {:?}", e);
                    vec![]
                }
            };

            let paginated_domains =
                PaginatedResult::new(domains.clone(), domains.len() as i64, 1, 20);

            let translations = get_domain_list_translations(&state, &locale).await;

            let page_range: Vec<i64> = (1..=paginated_domains.total_pages).collect();
            let max_item = std::cmp::min(
                paginated_domains.current_page * paginated_domains.per_page,
                paginated_domains.total_count,
            );

            let content_template = DomainsListTemplate {
                title: &translations["domains-title"],
                description: &translations["domains-description"],
                add_domain: &translations["domains-add"],
                table_header_domain: &translations["domains-table-header-domain"],
                table_header_enabled: &translations["domains-table-header-enabled"],
                table_header_actions: &translations["domains-table-header-actions"],
                table_header_transport: &translations["domains-transport"],
                status_active: &translations["status-active"],
                status_inactive: &translations["status-inactive"],
                action_view: &translations["action-view"],
                action_enable: &translations["action-enable"],
                action_disable: &translations["action-disable"],
                empty_title: &translations["domains-empty-title"],
                empty_description: &translations["domains-empty-description"],
                domains: &paginated_domains.items,
                pagination: &paginated_domains,
                page_range: &page_range,
                max_item,
                backups_title: &translations["backups-title"],
                backups_description: &translations["backups-description"],
                add_backup: &translations["backups-add"],
                backups_table_header_domain: &translations["backups-table-header-domain"],
                backups_table_header_transport: &translations["backups-table-header-transport"],
                backups_table_header_enabled: &translations["backups-table-header-enabled"],
                backups_table_header_actions: &translations["backups-table-header-actions"],
                backups: &backups,
                backups_view: &translations["backups-view"],
                backups_enable: &translations["backups-enable"],
                backups_disable: &translations["backups-disable"],
                backups_empty_no_backup_servers: &translations["backups-empty-no-backup-servers"],
                backups_empty_get_started: &translations["backups-empty-get-started"],
            };

            render_template_with_title!(
                content_template,
                content_template.title,
                &state,
                &locale,
                &headers
            )
        }
        Err(e) => {
            error!("Failed to toggle domain enabled status: {:?}", e);
            Html("Error toggling domain status".to_string())
        }
    }
}

pub async fn toggle_enabled_show(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = match crate::handlers::utils::get_current_db_pool(&state, &headers).await {
        Ok(pool) => pool,
        Err(e) => {
            error!("Failed to get database pool: {:?}", e);
            return Html("Database connection error".to_string());
        }
    };

    match db::toggle_domain_enabled(&pool, id) {
        Ok(_) => {
            info!(
                "Successfully toggled domain enabled status for ID: {} (show view)",
                id
            );

            let locale = crate::handlers::utils::get_user_locale(&headers);

            // Get updated domain
            let domain = match db::get_domain(&pool, id) {
                Ok(domain) => domain,
                Err(_) => {
                    let not_found_msg = get_translation(&state, &locale, "domains-not-found").await;
                    return Html(not_found_msg);
                }
            };

            // Get alias report and existing aliases
            let alias_report = db::get_domain_alias_report(&pool, &domain.domain).ok();
            let existing_aliases =
                db::get_aliases_for_domain(&pool, &domain.domain).unwrap_or_default();

            // Create template directly
            let translations = get_domain_show_translations(&state, &locale).await;

            let content_template = DomainShowTemplate {
                title: &translations["domains-title"],
                domain,
                view_edit_settings: &translations["domains-view-edit-settings"],
                back_to_domains: &translations["domains-back-to-domains"],
                domain_information: &translations["domains-domain-information"],
                domain_details: &translations["domains-domain-details"],
                domain_name: &translations["domains-domain-name"],
                transport: &translations["domains-transport"],
                status: &translations["domains-status"],
                status_active: &translations["status-active"],
                status_inactive: &translations["status-inactive"],
                created: &translations["domains-created"],
                modified: &translations["domains-modified"],
                edit_domain_button: &translations["domains-edit-domain-button"],
                enable_domain: &translations["domains-enable-domain"],
                disable_domain: &translations["domains-disable-domain"],
                delete_domain: &translations["domains-delete-domain"],
                delete_confirm: &translations["domains-delete-confirm"],
                alias_report,
                catch_all_header: &translations["reports-catch-all-header"],
                destination_header: &translations["reports-destination-header"],
                required_aliases_header: &translations["reports-required-aliases-header"],
                missing_aliases_header: &translations["reports-missing-aliases-header"],
                missing_required_alias_header: &translations
                    ["reports-missing-required-aliases-header"],
                missing_common_aliases_header: &translations
                    ["reports-missing-common-aliases-header"],
                mail_header: &translations["reports-mail-header"],
                status_header: &translations["reports-status-header"],
                enabled_header: &translations["reports-enabled-header"],
                actions_header: &translations["reports-actions-header"],
                no_required_aliases: &translations["reports-no-required-aliases"],
                no_missing_aliases: &translations["reports-no-missing-aliases"],
                alias_report_title: &translations["domains-alias-report-title"],
                alias_report_description: &translations["domains-alias-report-description"],
                existing_aliases_header: &translations["domains-existing-aliases-header"],
                add_missing_required_alias_button: &translations
                    ["domains-add-missing-required-aliases-button"],
                add_common_alias_button: &translations["reports-add-common-alias-button"],
                add_catch_all_button: &translations["domains-add-catch-all-button"],
                add_alias_button: &translations["domains-add-alias-button"],
                no_catch_all_message: &translations["domains-no-catch-all-message"],
                existing_aliases: &existing_aliases,
                action_view: &translations["action-view"],
                enable_alias: &translations["aliases-enable-alias"],
                disable_alias: &translations["aliases-disable-alias"],
            };

            render_template_with_title!(
                content_template,
                content_template.title,
                &state,
                &locale,
                &headers
            )
        }
        Err(e) => {
            error!("Failed to toggle domain enabled status: {:?}", e);
            Html("Error toggling domain status".to_string())
        }
    }
}

// Add missing required aliases for a domain
pub async fn add_missing_required_aliases(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = match crate::handlers::utils::get_current_db_pool(&state, &headers).await {
        Ok(pool) => pool,
        Err(e) => {
            error!("Failed to get database pool: {:?}", e);
            return Html("Database connection error".to_string());
        }
    };

    let locale = crate::handlers::utils::get_user_locale(&headers);

    // Get domain with proper error handling
    let domain = match db::get_domain(&pool, id) {
        Ok(domain) => domain,
        Err(_) => {
            let not_found_msg = get_translation(&state, &locale, "domains-not-found").await;
            return Html(not_found_msg);
        }
    };

    // Load configuration to get required aliases
    let _config = match crate::config::Config::load() {
        Ok(config) => config,
        Err(e) => {
            error!("Failed to load config, using defaults: {:?}", e);
            crate::config::Config::default()
        }
    };

    // Get current alias report to see what's missing
    let alias_report = match db::get_domain_alias_report(&pool, &domain.domain) {
        Ok(report) => report,
        Err(e) => {
            error!(
                "Failed to get alias report for domain {}: {:?}",
                domain.domain, e
            );
            let error_msg = get_translation(&state, &locale, "domains-error-loading-report").await;
            return Html(error_msg);
        }
    };

    // Create aliases for missing required aliases
    let aliases_to_create: Vec<(String, String)> = alias_report
        .missing_required_aliases
        .iter()
        .map(|alias| (alias.clone(), format!("admin@{}", domain.domain)))
        .collect();

    if !aliases_to_create.is_empty() {
        match db::create_domain_aliases(&pool, &domain.domain, aliases_to_create) {
            Ok(created_aliases) => {
                info!(
                    "Created {} missing required aliases for domain {}",
                    created_aliases.len(),
                    domain.domain
                );
            }
            Err(e) => {
                error!(
                    "Failed to create missing required aliases for domain {}: {:?}",
                    domain.domain, e
                );
                let error_msg =
                    get_translation(&state, &locale, "domains-error-creating-aliases").await;
                return Html(error_msg);
            }
        }
    }

    // Redirect back to the domain show page
    let redirect_url = format!("/domains/{}", domain.pkid);
    Html(format!(
        "<script>window.location.href = '{redirect_url}';</script>"
    ))
}

// Add a single missing required alias for a domain
pub async fn add_missing_required_alias(
    State(state): State<AppState>,
    Path((id, alias)): Path<(i32, String)>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = match crate::handlers::utils::get_current_db_pool(&state, &headers).await {
        Ok(pool) => pool,
        Err(e) => {
            error!("Failed to get database pool: {:?}", e);
            return Html("Database connection error".to_string());
        }
    };

    let locale = crate::handlers::utils::get_user_locale(&headers);

    // Get domain with proper error handling
    let domain = match db::get_domain(&pool, id) {
        Ok(domain) => domain,
        Err(_) => {
            let not_found_msg = get_translation(&state, &locale, "domains-not-found").await;
            return Html(not_found_msg);
        }
    };

    // Create the alias (destination defaults to admin@domain)
    let destination = format!("admin@{}", domain.domain);
    let aliases_to_create = vec![(alias.clone(), destination)];

    match db::create_domain_aliases(&pool, &domain.domain, aliases_to_create) {
        Ok(_created_aliases) => {
            info!(
                "Created missing required alias {} for domain {}",
                alias, domain.domain
            );
        }
        Err(e) => {
            error!(
                "Failed to create missing required alias {} for domain {}: {:?}",
                alias, domain.domain, e
            );
            let error_msg =
                get_translation(&state, &locale, "domains-error-creating-aliases").await;
            return Html(error_msg);
        }
    }

    // Redirect back to the domain show page
    let redirect_url = format!("/domains/{}", domain.pkid);
    Html(format!(
        "<script>window.location.href = '{redirect_url}';</script>"
    ))
}
