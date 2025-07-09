use crate::templates::domains::*;
use crate::templates::layout::BaseTemplate;
use crate::{db, models::*, AppState, i18n::get_translation};
use askama::Template;
use axum::{
    extract::{Path, State, Query},
    http::HeaderMap,
    response::Html,
    Form,
};



pub async fn list(
    State(state): State<AppState>, 
    headers: HeaderMap,
    Query(params): Query<PaginationParams>,
) -> Html<String> {
    let pool = &state.pool;
    let locale = crate::handlers::language::get_user_locale(&headers);

    // Parse pagination parameters
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(20);

    tracing::debug!("Handling domains list request with pagination: page={}, per_page={}", page, per_page);
    
    let paginated_domains = match db::get_domains_paginated(pool, page, per_page) {
        Ok(domains) => {
            tracing::info!("Successfully retrieved {} domains (page {} of {})", 
                domains.items.len(), domains.current_page, domains.total_pages);
            domains
        },
        Err(e) => {
            tracing::error!("Failed to retrieve domains: {:?}", e);
            PaginatedResult::new(vec![], 0, 1, per_page)
        },
    };

    // Get backups data (keeping non-paginated for now as it's a smaller dataset)
    let backups = match db::get_backups(pool) {
        Ok(backups) => {
            tracing::info!("Successfully retrieved {} backups", backups.len());
            backups
        },
        Err(e) => {
            tracing::error!("Failed to retrieve backups: {:?}", e);
            vec![]
        },
    };

    tracing::debug!("Rendering template with {} domains and {} backups", 
        paginated_domains.items.len(), backups.len());
    
    // Get all translations
    let title = get_translation(&state, &locale, "domains-title").await;
    let description = get_translation(&state, &locale, "domains-description").await;
    let add_domain = get_translation(&state, &locale, "domains-add").await;
    let table_header_domain = get_translation(&state, &locale, "domains-table-header-domain").await;
    let table_header_enabled = get_translation(&state, &locale, "domains-table-header-enabled").await;
    let table_header_actions = get_translation(&state, &locale, "domains-table-header-actions").await;
    let table_header_transport = get_translation(&state, &locale, "domains-transport").await;
    let status_active = get_translation(&state, &locale, "status-active").await;
    let status_inactive = get_translation(&state, &locale, "status-inactive").await;
    let action_view = get_translation(&state, &locale, "action-view").await;
    let action_enable = get_translation(&state, &locale, "action-enable").await;
    let action_disable = get_translation(&state, &locale, "action-disable").await;
    let empty_title = get_translation(&state, &locale, "domains-empty-title").await;
    let empty_description = get_translation(&state, &locale, "domains-empty-description").await;
    
    // Backup translations
    let backups_title = get_translation(&state, &locale, "backups-title").await;
    let backups_description = get_translation(&state, &locale, "backups-description").await;
    let add_backup = get_translation(&state, &locale, "backups-add").await;
    let backups_table_header_domain = get_translation(&state, &locale, "backups-table-header-domain").await;
    let backups_table_header_transport = get_translation(&state, &locale, "backups-table-header-transport").await;
    let backups_table_header_enabled = get_translation(&state, &locale, "backups-table-header-enabled").await;
    let backups_table_header_actions = get_translation(&state, &locale, "backups-table-header-actions").await;
    let backups_view = get_translation(&state, &locale, "backups-view").await;
    let backups_enable = get_translation(&state, &locale, "backups-enable").await;
    let backups_disable = get_translation(&state, &locale, "backups-disable").await;
    let backups_empty_no_backup_servers = get_translation(&state, &locale, "backups-empty-no-backup-servers").await;
    let backups_empty_get_started = get_translation(&state, &locale, "backups-empty-get-started").await;

    let paginated = PaginatedResult::new(paginated_domains.items.clone(), paginated_domains.total_count, paginated_domains.current_page, paginated_domains.per_page);
    let page_range: Vec<i64> = (1..=paginated.total_pages).collect();
    let max_item = std::cmp::min(paginated.current_page * paginated.per_page, paginated.total_count);
    let content_template = DomainsListTemplate {
        title: &title,
        description: &description,
        add_domain: &add_domain,
        table_header_domain: &table_header_domain,
        table_header_enabled: &table_header_enabled,
        table_header_actions: &table_header_actions,
        table_header_transport: &table_header_transport,
        status_active: &status_active,
        status_inactive: &status_inactive,
        action_view: &action_view,
        action_enable: &action_enable,
        action_disable: &action_disable,
        empty_title: &empty_title,
        empty_description: &empty_description,
        domains: &paginated_domains.items,
        pagination: &paginated,
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
    let content = content_template.render().unwrap();

    let template = BaseTemplate::with_i18n(
        title,
        content,
        &state,
        &locale,
    ).await.unwrap();
    
    Html(template.render().unwrap())
}

pub async fn new(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    let locale = crate::handlers::language::get_user_locale(&headers);
    
    let form = DomainForm {
        domain: "".to_string(),
        transport: "virtual".to_string(),
        enabled: true,
    };

    let title = get_translation(&state, &locale, "domains-new-domain").await;
    let form_error = get_translation(&state, &locale, "form-error").await;
    let form_domain = get_translation(&state, &locale, "form-domain").await;
    let form_transport = get_translation(&state, &locale, "form-transport").await;
    let form_active = get_translation(&state, &locale, "form-active").await;
    let form_cancel = get_translation(&state, &locale, "form-cancel").await;
    let form_create_domain = get_translation(&state, &locale, "form-create-domain").await;
    let form_update_domain = get_translation(&state, &locale, "form-update-domain").await;
    let form_placeholder_domain = get_translation(&state, &locale, "form-placeholder-domain").await;
    let form_placeholder_transport = get_translation(&state, &locale, "form-placeholder-transport").await;
    let form_tooltip_domain = get_translation(&state, &locale, "form-tooltip-domain").await;
    let form_tooltip_transport = get_translation(&state, &locale, "form-tooltip-transport").await;
    let form_tooltip_enable = get_translation(&state, &locale, "form-tooltip-enable").await;

    let content_template = DomainFormTemplate {
        title: &title,
        domain: None,
        form,
        error: None,
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
    };
    Html(content_template.render().unwrap())
}

pub async fn show(State(state): State<AppState>, Path(id): Path<i32>, headers: HeaderMap) -> Html<String> {
    let pool = &state.pool;
    let locale = crate::handlers::language::get_user_locale(&headers);

    let domain = match db::get_domain(pool, id) {
        Ok(domain) => domain,
        Err(_) => {
            let not_found_msg = get_translation(&state, &locale, "domains-not-found").await;
            return Html(not_found_msg);
        }
    };

    // Get alias report for this domain
    let alias_report = match db::get_domain_alias_report(pool, &domain.domain) {
        Ok(report) => Some(report),
        Err(e) => {
            tracing::warn!("Failed to get alias report for domain {}: {:?}", domain.domain, e);
            None
        }
    };

    // Get all aliases for this domain
    let existing_aliases = match db::get_aliases_for_domain(pool, &domain.domain) {
        Ok(aliases) => aliases,
        Err(_) => vec![],
    };

    let title = get_translation(&state, &locale, "domains-title").await;
    let view_edit_settings = get_translation(&state, &locale, "domains-view-edit-settings").await;
    let back_to_domains = get_translation(&state, &locale, "domains-back-to-domains").await;
    let domain_information = get_translation(&state, &locale, "domains-domain-information").await;
    let domain_details = get_translation(&state, &locale, "domains-domain-details").await;
    let domain_name = get_translation(&state, &locale, "domains-domain-name").await;
    let transport = get_translation(&state, &locale, "domains-transport").await;
    let status = get_translation(&state, &locale, "domains-status").await;
    let status_active = get_translation(&state, &locale, "status-active").await;
    let status_inactive = get_translation(&state, &locale, "status-inactive").await;
    let created = get_translation(&state, &locale, "domains-created").await;
    let modified = get_translation(&state, &locale, "domains-modified").await;
    let edit_domain_button = get_translation(&state, &locale, "domains-edit-domain-button").await;
    let enable_domain = get_translation(&state, &locale, "domains-enable-domain").await;
    let disable_domain = get_translation(&state, &locale, "domains-disable-domain").await;
    let delete_domain = get_translation(&state, &locale, "domains-delete-domain").await;
    let delete_confirm = get_translation(&state, &locale, "domains-delete-confirm").await;
    
    // Alias report translations
    let alias_report_title = get_translation(&state, &locale, "domains-alias-report-title").await;
    let alias_report_description = get_translation(&state, &locale, "domains-alias-report-description").await;
    let existing_aliases_header = get_translation(&state, &locale, "domains-existing-aliases-header").await;
    let catch_all_header = get_translation(&state, &locale, "reports-catch-all-header").await;
    let destination_header = get_translation(&state, &locale, "reports-destination-header").await;
    let required_aliases_header = get_translation(&state, &locale, "reports-required-aliases-header").await;
    let missing_aliases_header = get_translation(&state, &locale, "reports-missing-aliases-header").await;
    let missing_required_aliases_header = get_translation(&state, &locale, "reports-missing-required-aliases-header").await;
    let missing_common_aliases_header = get_translation(&state, &locale, "reports-missing-common-aliases-header").await;
    let mail_header = get_translation(&state, &locale, "reports-mail-header").await;
                let status_header = get_translation(&state, &locale, "reports-status-header").await;
            let enabled_header = get_translation(&state, &locale, "reports-enabled-header").await;
            let actions_header = get_translation(&state, &locale, "reports-actions-header").await;
            let no_required_aliases = get_translation(&state, &locale, "reports-no-required-aliases").await;
    let no_missing_aliases = get_translation(&state, &locale, "reports-no-missing-aliases").await;
    
    // New button translations
    let add_missing_required_alias_button = get_translation(&state, &locale, "domains-add-missing-required-aliases-button").await;
    let add_catch_all_button = get_translation(&state, &locale, "domains-add-catch-all-button").await;
    let add_alias_button = get_translation(&state, &locale, "domains-add-alias-button").await;
    let no_catch_all_message = get_translation(&state, &locale, "domains-no-catch-all-message").await;
    
    let add_common_alias_button = get_translation(&state, &locale, "reports-add-common-alias-button").await;
    
    let content_template = DomainShowTemplate {
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
        alias_report: alias_report,
        catch_all_header: &catch_all_header,
        destination_header: &destination_header,
        required_aliases_header: &required_aliases_header,
        missing_aliases_header: &missing_aliases_header,
        missing_required_alias_header: &missing_required_aliases_header,
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
        action_view: "",
        enable_alias: "",
        disable_alias: "",
    };
    let content = content_template.render().unwrap();

    let template = BaseTemplate::with_i18n(
        get_translation(&state, &locale, "domains-title").await,
        content,
        &state,
        &locale,
    ).await.unwrap();
    
    Html(template.render().unwrap())
}

pub async fn edit(State(state): State<AppState>, Path(id): Path<i32>, headers: HeaderMap) -> Html<String> {
    let pool = &state.pool;
    let locale = crate::handlers::language::get_user_locale(&headers);

    let domain = match db::get_domain(pool, id) {
        Ok(domain) => domain,
        Err(_) => return Html("Domain not found".to_string()),
    };

    let form = DomainForm {
        domain: domain.domain.clone(),
        transport: domain.transport.clone().unwrap_or_default(),
        enabled: domain.enabled,
    };

    let title = get_translation(&state, &locale, "domains-edit-domain").await;
    let form_error = get_translation(&state, &locale, "form-error").await;
    let form_domain = get_translation(&state, &locale, "form-domain").await;
    let form_transport = get_translation(&state, &locale, "form-transport").await;
    let form_active = get_translation(&state, &locale, "form-active").await;
    let form_cancel = get_translation(&state, &locale, "form-cancel").await;
    let form_create_domain = get_translation(&state, &locale, "form-create-domain").await;
    let form_update_domain = get_translation(&state, &locale, "form-update-domain").await;
    let form_placeholder_domain = get_translation(&state, &locale, "form-placeholder-domain").await;
    let form_placeholder_transport = get_translation(&state, &locale, "form-placeholder-transport").await;
    let form_tooltip_domain = get_translation(&state, &locale, "form-tooltip-domain").await;
    let form_tooltip_transport = get_translation(&state, &locale, "form-tooltip-transport").await;
    let form_tooltip_enable = get_translation(&state, &locale, "form-tooltip-enable").await;

    let content_template = DomainFormTemplate {
        title: &title,
        domain: Some(domain),
        form,
        error: None,
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
    };
    Html(content_template.render().unwrap())
}

pub async fn create(State(state): State<AppState>, headers: HeaderMap, Form(form): Form<DomainForm>) -> Html<String> {
    let pool = &state.pool;

    // Validate form data
    if form.domain.trim().is_empty() {
        let locale = crate::handlers::language::get_user_locale(&headers);
        let error_msg = get_translation(&state, &locale, "validation-domain-required").await;
        let title = get_translation(&state, &locale, "domains-new-domain").await;
        let form_error = get_translation(&state, &locale, "form-error").await;
        let form_domain = get_translation(&state, &locale, "form-domain").await;
        let form_transport = get_translation(&state, &locale, "form-transport").await;
        let form_active = get_translation(&state, &locale, "form-active").await;
        let form_cancel = get_translation(&state, &locale, "form-cancel").await;
        let form_create_domain = get_translation(&state, &locale, "form-create-domain").await;
        let form_update_domain = get_translation(&state, &locale, "form-update-domain").await;
        let form_placeholder_domain = get_translation(&state, &locale, "form-placeholder-domain").await;
        let form_placeholder_transport = get_translation(&state, &locale, "form-placeholder-transport").await;
        let form_tooltip_domain = get_translation(&state, &locale, "form-tooltip-domain").await;
        let form_tooltip_transport = get_translation(&state, &locale, "form-tooltip-transport").await;
        let form_tooltip_enable = get_translation(&state, &locale, "form-tooltip-enable").await;
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
        };
        return Html(content_template.render().unwrap());
    }

    let new_domain = NewDomain {
        domain: form.domain.trim().to_string(),
        transport: Some(form.transport.clone()),
        enabled: form.enabled,
    };

    match db::create_domain(pool, new_domain) {
        Ok(_) => {
            let domains = match db::get_domains(pool) {
                Ok(domains) => domains,
                Err(_) => vec![],
            };
            
            let locale = crate::handlers::language::get_user_locale(&headers);
            let title = get_translation(&state, &locale, "domains-title").await;
            let description = get_translation(&state, &locale, "domains-description").await;
            let add_domain = get_translation(&state, &locale, "domains-add").await;
            let table_header_domain = get_translation(&state, &locale, "domains-table-header-domain").await;
            let table_header_enabled = get_translation(&state, &locale, "domains-table-header-enabled").await;
            let table_header_actions = get_translation(&state, &locale, "domains-table-header-actions").await;
            let table_header_transport = get_translation(&state, &locale, "domains-transport").await;
            let status_active = get_translation(&state, &locale, "status-active").await;
            let status_inactive = get_translation(&state, &locale, "status-inactive").await;
            let action_enable = get_translation(&state, &locale, "action-enable").await;
            let action_disable = get_translation(&state, &locale, "action-disable").await;
            let empty_title = get_translation(&state, &locale, "domains-empty-title").await;
            let empty_description = get_translation(&state, &locale, "domains-empty-description").await;
            
            // Get backups data
            let backups = match db::get_backups(pool) {
                Ok(backups) => backups,
                Err(e) => {
                    tracing::error!("Failed to retrieve backups: {:?}", e);
                    vec![]
                },
            };
            
            // Backup translations
            let backups_title = get_translation(&state, &locale, "backups-title").await;
            let backups_description = get_translation(&state, &locale, "backups-description").await;
            let add_backup = get_translation(&state, &locale, "backups-add").await;
            let backups_table_header_domain = get_translation(&state, &locale, "backups-table-header-domain").await;
            let backups_table_header_transport = get_translation(&state, &locale, "backups-table-header-transport").await;
            let backups_table_header_enabled = get_translation(&state, &locale, "backups-table-header-enabled").await;
            let backups_table_header_actions = get_translation(&state, &locale, "backups-table-header-actions").await;
            let backups_view = get_translation(&state, &locale, "backups-view").await;
            let backups_disable = get_translation(&state, &locale, "backups-disable").await;
            let backups_enable = get_translation(&state, &locale, "backups-enable").await;
            let backups_empty_no_backup_servers = get_translation(&state, &locale, "backups-empty-no-backup-servers").await;
            let backups_empty_get_started = get_translation(&state, &locale, "backups-empty-get-started").await;
            
            let paginated = PaginatedResult::new(domains.clone(), 0, 1, 20);
            let page_range: Vec<i64> = (1..=paginated.total_pages).collect();
            let max_item = std::cmp::min(paginated.current_page * paginated.per_page, paginated.total_count);
            let template = DomainsListTemplate {
                title: &title,
                description: &description,
                add_domain: &add_domain,
                table_header_domain: &table_header_domain,
                table_header_transport: &table_header_transport,
                table_header_enabled: &table_header_enabled,
                table_header_actions: &table_header_actions,
                status_active: &status_active,
                status_inactive: &status_inactive,
                action_view: "",
                action_enable: &action_enable,
                action_disable: &action_disable,
                empty_title: &empty_title,
                empty_description: &empty_description,
                domains: &domains,
                pagination: &paginated,
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
                backups_disable: &backups_disable,
                backups_enable: &backups_enable,
                backups_empty_no_backup_servers: &backups_empty_no_backup_servers,
                backups_empty_get_started: &backups_empty_get_started,
            };
            Html(template.render().unwrap())
        }
        Err(e) => {
            let locale = crate::handlers::language::get_user_locale(&headers);
            let error_message = match e {
                diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::UniqueViolation,
                    _,
                ) => get_translation(&state, &locale, "error-duplicate-domain").await.replace("{domain}", &form.domain),
                diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::CheckViolation,
                    _,
                ) => get_translation(&state, &locale, "error-constraint-violation").await,
                _ => get_translation(&state, &locale, "error-unexpected").await,
            };

            let title = get_translation(&state, &locale, "domains-new-domain").await;
            let form_error = get_translation(&state, &locale, "form-error").await;
            let form_domain = get_translation(&state, &locale, "form-domain").await;
            let form_transport = get_translation(&state, &locale, "form-transport").await;
            let form_active = get_translation(&state, &locale, "form-active").await;
            let form_cancel = get_translation(&state, &locale, "form-cancel").await;
            let form_create_domain = get_translation(&state, &locale, "form-create-domain").await;
            let form_update_domain = get_translation(&state, &locale, "form-update-domain").await;
            let form_placeholder_domain = get_translation(&state, &locale, "form-placeholder-domain").await;
            let form_placeholder_transport = get_translation(&state, &locale, "form-placeholder-transport").await;
            let form_tooltip_domain = get_translation(&state, &locale, "form-tooltip-domain").await;
            let form_tooltip_transport = get_translation(&state, &locale, "form-tooltip-transport").await;
            let form_tooltip_enable = get_translation(&state, &locale, "form-tooltip-enable").await;
            let content_template = DomainFormTemplate {
                title: &title,
                domain: None,
                form,
                error: Some(error_message),
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
            };
            Html(content_template.render().unwrap())
        }
    }
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
    Form(form): Form<DomainForm>,
) -> Html<String> {
    let pool = &state.pool;

    // Validate form data
    if form.domain.trim().is_empty() {
        let locale = crate::handlers::language::get_user_locale(&headers);
        let error_msg = get_translation(&state, &locale, "validation-domain-required").await;
        let title = get_translation(&state, &locale, "domains-edit-domain").await;
        let form_error = get_translation(&state, &locale, "form-error").await;
        let form_domain = get_translation(&state, &locale, "form-domain").await;
        let form_transport = get_translation(&state, &locale, "form-transport").await;
        let form_active = get_translation(&state, &locale, "form-active").await;
        let form_cancel = get_translation(&state, &locale, "form-cancel").await;
        let form_create_domain = get_translation(&state, &locale, "form-create-domain").await;
        let form_update_domain = get_translation(&state, &locale, "form-update-domain").await;
        let form_placeholder_domain = get_translation(&state, &locale, "form-placeholder-domain").await;
        let form_placeholder_transport = get_translation(&state, &locale, "form-placeholder-transport").await;
        let form_tooltip_domain = get_translation(&state, &locale, "form-tooltip-domain").await;
        let form_tooltip_transport = get_translation(&state, &locale, "form-tooltip-transport").await;
        let form_tooltip_enable = get_translation(&state, &locale, "form-tooltip-enable").await;
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
        };
        return Html(content_template.render().unwrap());
    }

    let domain_name = form.domain.clone();
    match db::update_domain(pool, id, form) {
        Ok(_) => {
            let domain = match db::get_domain(pool, id) {
                Ok(domain) => domain,
                Err(_) => return Html("Domain not found".to_string()),
            };
            let locale = crate::handlers::language::get_user_locale(&headers);
            let title = get_translation(&state, &locale, "domains-title").await;
            let view_edit_settings = get_translation(&state, &locale, "domains-view-edit-settings").await;
            let back_to_domains = get_translation(&state, &locale, "domains-back-to-domains").await;
            let domain_information = get_translation(&state, &locale, "domains-domain-information").await;
            let domain_details = get_translation(&state, &locale, "domains-domain-details").await;
            let domain_name = get_translation(&state, &locale, "domains-domain-name").await;
            let transport = get_translation(&state, &locale, "domains-transport").await;
            let status = get_translation(&state, &locale, "domains-status").await;
            let status_active = get_translation(&state, &locale, "status-active").await;
            let status_inactive = get_translation(&state, &locale, "status-inactive").await;
            let created = get_translation(&state, &locale, "domains-created").await;
            let modified = get_translation(&state, &locale, "domains-modified").await;
            let edit_domain_button = get_translation(&state, &locale, "domains-edit-domain-button").await;
            let enable_domain = get_translation(&state, &locale, "domains-enable-domain").await;
            let disable_domain = get_translation(&state, &locale, "domains-disable-domain").await;
            let delete_domain = get_translation(&state, &locale, "domains-delete-domain").await;
            let delete_confirm = get_translation(&state, &locale, "domains-delete-confirm").await;
            let content_template = DomainShowTemplate {
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
                alias_report: None,
                catch_all_header: "",
                destination_header: "",
                required_aliases_header: "",
                missing_aliases_header: "",
                missing_required_alias_header: "",
                missing_common_aliases_header: "",
                mail_header: "",
                status_header: "",
                enabled_header: "",
                actions_header: "",
                no_required_aliases: "",
                no_missing_aliases: "",
                alias_report_title: "",
                alias_report_description: "",
                existing_aliases_header: "",
                add_missing_required_alias_button: "",
                add_common_alias_button: "",
                add_catch_all_button: "",
                add_alias_button: "",
                no_catch_all_message: "",
                existing_aliases: &[],
                action_view: "",
                enable_alias: "",
                disable_alias: "",
            };
            Html(content_template.render().unwrap())
        }
        Err(e) => {
            let error_message = match e {
                diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::UniqueViolation,
                    _,
                ) => format!("A domain with the name '{}' already exists.", domain_name),
                diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::CheckViolation,
                    _,
                ) => "The domain data does not meet the required constraints. Please check your input.".to_string(),
                _ => "An unexpected error occurred while updating the domain. Please try again.".to_string(),
            };

            // Recreate the form for error display
            let error_form = DomainForm {
                domain: domain_name,
                transport: "virtual".to_string(),
                enabled: true,
            };

            let locale = crate::handlers::language::get_user_locale(&headers);
            let title = get_translation(&state, &locale, "domains-edit-domain").await;
            let form_error = get_translation(&state, &locale, "form-error").await;
            let form_domain = get_translation(&state, &locale, "form-domain").await;
            let form_transport = get_translation(&state, &locale, "form-transport").await;
            let form_active = get_translation(&state, &locale, "form-active").await;
            let form_cancel = get_translation(&state, &locale, "form-cancel").await;
            let form_create_domain = get_translation(&state, &locale, "form-create-domain").await;
            let form_update_domain = get_translation(&state, &locale, "form-update-domain").await;
            let form_placeholder_domain = get_translation(&state, &locale, "form-placeholder-domain").await;
            let form_placeholder_transport = get_translation(&state, &locale, "form-placeholder-transport").await;
            let form_tooltip_domain = get_translation(&state, &locale, "form-tooltip-domain").await;
            let form_tooltip_transport = get_translation(&state, &locale, "form-tooltip-transport").await;
            let form_tooltip_enable = get_translation(&state, &locale, "form-tooltip-enable").await;
            let content_template = DomainFormTemplate {
                title: &title,
                domain: None,
                form: error_form,
                error: Some(error_message),
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
            };
            Html(content_template.render().unwrap())
        }
    }
}

pub async fn delete(State(state): State<AppState>, Path(id): Path<i32>, headers: HeaderMap) -> Html<String> {
    let pool = &state.pool;

    match db::delete_domain(pool, id) {
        Ok(_) => {
            let domains = match db::get_domains(pool) {
                Ok(domains) => domains,
                Err(_) => vec![],
            };
            
            let locale = crate::handlers::language::get_user_locale(&headers);
            let title = get_translation(&state, &locale, "domains-title").await;
            let description = get_translation(&state, &locale, "domains-description").await;
            let add_domain = get_translation(&state, &locale, "domains-add").await;
            let table_header_domain = get_translation(&state, &locale, "domains-table-header-domain").await;
            let table_header_enabled = get_translation(&state, &locale, "domains-table-header-enabled").await;
            let table_header_actions = get_translation(&state, &locale, "domains-table-header-actions").await;
            let table_header_transport = get_translation(&state, &locale, "domains-transport").await;
            let status_active = get_translation(&state, &locale, "status-active").await;
            let status_inactive = get_translation(&state, &locale, "status-inactive").await;
            let action_enable = get_translation(&state, &locale, "action-enable").await;
            let action_disable = get_translation(&state, &locale, "action-disable").await;
            let empty_title = get_translation(&state, &locale, "domains-empty-title").await;
            let empty_description = get_translation(&state, &locale, "domains-empty-description").await;
            
            // Get backups data
            let backups = match db::get_backups(pool) {
                Ok(backups) => backups,
                Err(e) => {
                    tracing::error!("Failed to retrieve backups: {:?}", e);
                    vec![]
                },
            };
            
            // Backup translations
            let backups_title = get_translation(&state, &locale, "backups-title").await;
            let backups_description = get_translation(&state, &locale, "backups-description").await;
            let add_backup = get_translation(&state, &locale, "backups-add").await;
            let backups_table_header_domain = get_translation(&state, &locale, "backups-table-header-domain").await;
            let backups_table_header_transport = get_translation(&state, &locale, "backups-table-header-transport").await;
            let backups_table_header_enabled = get_translation(&state, &locale, "backups-table-header-enabled").await;
            let backups_table_header_actions = get_translation(&state, &locale, "backups-table-header-actions").await;
            let backups_view = get_translation(&state, &locale, "backups-view").await;
            let backups_disable = get_translation(&state, &locale, "backups-disable").await;
            let backups_enable = get_translation(&state, &locale, "backups-enable").await;
            let backups_empty_no_backup_servers = get_translation(&state, &locale, "backups-empty-no-backup-servers").await;
            let backups_empty_get_started = get_translation(&state, &locale, "backups-empty-get-started").await;
            
            let paginated = PaginatedResult::new(domains.clone(), 0, 1, 20);
            let page_range: Vec<i64> = (1..=paginated.total_pages).collect();
            let max_item = std::cmp::min(paginated.current_page * paginated.per_page, paginated.total_count);
            let template = DomainsListTemplate {
                title: &title,
                description: &description,
                add_domain: &add_domain,
                table_header_domain: &table_header_domain,
                table_header_transport: &table_header_transport,
                table_header_enabled: &table_header_enabled,
                table_header_actions: &table_header_actions,
                status_active: &status_active,
                status_inactive: &status_inactive,
                action_view: "",
                action_enable: &action_enable,
                action_disable: &action_disable,
                empty_title: &empty_title,
                empty_description: &empty_description,
                domains: &domains,
                pagination: &paginated,
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
                backups_disable: &backups_disable,
                backups_enable: &backups_enable,
                backups_empty_no_backup_servers: &backups_empty_no_backup_servers,
                backups_empty_get_started: &backups_empty_get_started,
            };
            Html(template.render().unwrap())
        }
        Err(_) => Html("Error deleting domain".to_string()),
    }
}

pub async fn toggle_enabled(State(state): State<AppState>, Path(id): Path<i32>, headers: HeaderMap) -> Html<String> {
    let pool = &state.pool;
    let locale = crate::handlers::language::get_user_locale(&headers);

    match db::toggle_domain_enabled(pool, id) {
        Ok(_) => {
            let domain = match db::get_domain(pool, id) {
                Ok(domain) => domain,
                Err(_) => return Html("Domain not found".to_string()),
            };

            let title = get_translation(&state, &locale, "domains-title").await;
            let view_edit_settings = get_translation(&state, &locale, "domains-view-edit-settings").await;
            let back_to_domains = get_translation(&state, &locale, "domains-back-to-domains").await;
            let domain_information = get_translation(&state, &locale, "domains-domain-information").await;
            let domain_details = get_translation(&state, &locale, "domains-domain-details").await;
            let domain_name = get_translation(&state, &locale, "domains-domain-name").await;
            let transport = get_translation(&state, &locale, "domains-transport").await;
            let status = get_translation(&state, &locale, "domains-status").await;
            let status_active = get_translation(&state, &locale, "status-active").await;
            let status_inactive = get_translation(&state, &locale, "status-inactive").await;
            let created = get_translation(&state, &locale, "domains-created").await;
            let modified = get_translation(&state, &locale, "domains-modified").await;
            let edit_domain_button = get_translation(&state, &locale, "domains-edit-domain-button").await;
            let enable_domain = get_translation(&state, &locale, "domains-enable-domain").await;
            let disable_domain = get_translation(&state, &locale, "domains-disable-domain").await;
            let delete_domain = get_translation(&state, &locale, "domains-delete-domain").await;
            let delete_confirm = get_translation(&state, &locale, "domains-delete-confirm").await;
            let content_template = DomainShowTemplate {
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
                alias_report: None,
                catch_all_header: "",
                destination_header: "",
                required_aliases_header: "",
                missing_aliases_header: "",
                missing_required_alias_header: "",
                missing_common_aliases_header: "",
                mail_header: "",
                status_header: "",
                enabled_header: "",
                actions_header: "",
                no_required_aliases: "",
                no_missing_aliases: "",
                alias_report_title: "",
                alias_report_description: "",
                existing_aliases_header: "",
                add_missing_required_alias_button: "",
                add_common_alias_button: "",
                add_catch_all_button: "",
                add_alias_button: "",
                no_catch_all_message: "",
                existing_aliases: &[],
                action_view: "",
                enable_alias: "",
                disable_alias: "",
            };
            let content = content_template.render().unwrap();

            let template = BaseTemplate::with_i18n(
                get_translation(&state, &locale, "domains-title").await,
                content,
                &state,
                &locale,
            ).await.unwrap();
            
            Html(template.render().unwrap())
        }
        Err(_) => Html("Error toggling domain status".to_string()),
    }
}

pub async fn toggle_enabled_list(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = &state.pool;
    match db::toggle_domain_enabled(pool, id) {
        Ok(_) => {
            let domains = match db::get_domains(pool) {
                Ok(domains) => domains,
                Err(_) => vec![],
            };
            
            let locale = crate::handlers::language::get_user_locale(&headers);
            let title = get_translation(&state, &locale, "domains-title").await;
            let description = get_translation(&state, &locale, "domains-description").await;
            let add_domain = get_translation(&state, &locale, "domains-add").await;
            let table_header_domain = get_translation(&state, &locale, "domains-table-header-domain").await;
            let table_header_enabled = get_translation(&state, &locale, "domains-table-header-enabled").await;
            let table_header_actions = get_translation(&state, &locale, "domains-table-header-actions").await;
            let table_header_transport = get_translation(&state, &locale, "domains-transport").await;
            let status_active = get_translation(&state, &locale, "status-active").await;
            let status_inactive = get_translation(&state, &locale, "status-inactive").await;
            let action_enable = get_translation(&state, &locale, "action-enable").await;
            let action_disable = get_translation(&state, &locale, "action-disable").await;
            let empty_title = get_translation(&state, &locale, "domains-empty-title").await;
            let empty_description = get_translation(&state, &locale, "domains-empty-description").await;
            
            // Get backups data
            let backups = match db::get_backups(pool) {
                Ok(backups) => backups,
                Err(e) => {
                    tracing::error!("Failed to retrieve backups: {:?}", e);
                    vec![]
                },
            };
            
            // Backup translations
            let backups_title = get_translation(&state, &locale, "backups-title").await;
            let backups_description = get_translation(&state, &locale, "backups-description").await;
            let add_backup = get_translation(&state, &locale, "backups-add").await;
            let backups_table_header_domain = get_translation(&state, &locale, "backups-table-header-domain").await;
            let backups_table_header_transport = get_translation(&state, &locale, "backups-table-header-transport").await;
            let backups_table_header_enabled = get_translation(&state, &locale, "backups-table-header-enabled").await;
            let backups_table_header_actions = get_translation(&state, &locale, "backups-table-header-actions").await;
            let backups_view = get_translation(&state, &locale, "backups-view").await;
            let backups_disable = get_translation(&state, &locale, "backups-disable").await;
            let backups_enable = get_translation(&state, &locale, "backups-enable").await;
            let backups_empty_no_backup_servers = get_translation(&state, &locale, "backups-empty-no-backup-servers").await;
            let backups_empty_get_started = get_translation(&state, &locale, "backups-empty-get-started").await;
            
            let paginated = PaginatedResult::new(domains.clone(), 0, 1, 20);
            let page_range: Vec<i64> = (1..=paginated.total_pages).collect();
            let max_item = std::cmp::min(paginated.current_page * paginated.per_page, paginated.total_count);
            let template = DomainsListTemplate {
                title: &title,
                description: &description,
                add_domain: &add_domain,
                table_header_domain: &table_header_domain,
                table_header_transport: &table_header_transport,
                table_header_enabled: &table_header_enabled,
                table_header_actions: &table_header_actions,
                status_active: &status_active,
                status_inactive: &status_inactive,
                action_view: "",
                action_enable: &action_enable,
                action_disable: &action_disable,
                empty_title: &empty_title,
                empty_description: &empty_description,
                domains: &domains,
                pagination: &paginated,
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
                backups_disable: &backups_disable,
                backups_enable: &backups_enable,
                backups_empty_no_backup_servers: &backups_empty_no_backup_servers,
                backups_empty_get_started: &backups_empty_get_started,
            };
            Html(template.render().unwrap())
        }
        Err(_) => Html("Error toggling domain status".to_string()),
    }
}

pub async fn toggle_enabled_show(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = &state.pool;
    match db::toggle_domain_enabled(pool, id) {
        Ok(_) => {
            let domain = match db::get_domain(pool, id) {
                Ok(domain) => domain,
                Err(_) => return Html("Domain not found".to_string()),
            };
            let locale = crate::handlers::language::get_user_locale(&headers);
            let title = get_translation(&state, &locale, "domains-title").await;
            let view_edit_settings = get_translation(&state, &locale, "domains-view-edit-settings").await;
            let back_to_domains = get_translation(&state, &locale, "domains-back-to-domains").await;
            let domain_information = get_translation(&state, &locale, "domains-domain-information").await;
            let domain_details = get_translation(&state, &locale, "domains-domain-details").await;
            let domain_name = get_translation(&state, &locale, "domains-domain-name").await;
            let transport = get_translation(&state, &locale, "domains-transport").await;
            let status = get_translation(&state, &locale, "domains-status").await;
            let status_active = get_translation(&state, &locale, "status-active").await;
            let status_inactive = get_translation(&state, &locale, "status-inactive").await;
            let created = get_translation(&state, &locale, "domains-created").await;
            let modified = get_translation(&state, &locale, "domains-modified").await;
            let edit_domain_button = get_translation(&state, &locale, "domains-edit-domain-button").await;
            let enable_domain = get_translation(&state, &locale, "domains-enable-domain").await;
            let disable_domain = get_translation(&state, &locale, "domains-disable-domain").await;
            let delete_domain = get_translation(&state, &locale, "domains-delete-domain").await;
            let delete_confirm = get_translation(&state, &locale, "domains-delete-confirm").await;
            let content_template = DomainShowTemplate {
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
                alias_report: None,
                catch_all_header: "",
                destination_header: "",
                required_aliases_header: "",
                missing_aliases_header: "",
                missing_required_alias_header: "",
                missing_common_aliases_header: "",
                mail_header: "",
                status_header: "",
                enabled_header: "",
                actions_header: "",
                no_required_aliases: "",
                no_missing_aliases: "",
                alias_report_title: "",
                alias_report_description: "",
                existing_aliases_header: "",
                add_missing_required_alias_button: "",
                add_common_alias_button: "",
                add_catch_all_button: "",
                add_alias_button: "",
                no_catch_all_message: "",
                existing_aliases: &[],
                action_view: "",
                enable_alias: "",
                disable_alias: "",
            };
            Html(content_template.render().unwrap())
        }
        Err(_) => Html("Error toggling domain status".to_string()),
    }
}

// Add missing required aliases for a domain
pub async fn add_missing_required_aliases(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = &state.pool;
    let locale = crate::handlers::language::get_user_locale(&headers);

    let domain = match db::get_domain(pool, id) {
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
            tracing::warn!("Failed to load config, using defaults: {:?}", e);
            crate::config::Config::default()
        }
    };

    // Get current alias report to see what's missing
    let alias_report = match db::get_domain_alias_report(pool, &domain.domain) {
        Ok(report) => report,
        Err(e) => {
            tracing::error!("Failed to get alias report for domain {}: {:?}", domain.domain, e);
            let error_msg = get_translation(&state, &locale, "domains-error-loading-report").await;
            return Html(error_msg);
        }
    };

    // Create aliases for missing required aliases
    let aliases_to_create: Vec<(String, String)> = alias_report.missing_required_aliases
        .iter()
        .map(|alias| (alias.clone(), format!("admin@{}", domain.domain)))
        .collect();

    if !aliases_to_create.is_empty() {
        match db::create_domain_aliases(pool, &domain.domain, aliases_to_create) {
            Ok(created_aliases) => {
                tracing::info!("Created {} missing required aliases for domain {}", created_aliases.len(), domain.domain);
            }
            Err(e) => {
                tracing::error!("Failed to create missing required aliases for domain {}: {:?}", domain.domain, e);
                let error_msg = get_translation(&state, &locale, "domains-error-creating-aliases").await;
                return Html(error_msg);
            }
        }
    }

    // Redirect back to the domain show page
    let redirect_url = format!("/domains/{}", domain.pkid);
    Html(format!("<script>window.location.href = '{}';</script>", redirect_url))
}

// Add a single missing required alias for a domain
pub async fn add_missing_required_alias(
    State(state): State<AppState>,
    Path((id, alias)): Path<(i32, String)>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = &state.pool;
    let locale = crate::handlers::language::get_user_locale(&headers);

    let domain = match db::get_domain(pool, id) {
        Ok(domain) => domain,
        Err(_) => {
            let not_found_msg = get_translation(&state, &locale, "domains-not-found").await;
            return Html(not_found_msg);
        }
    };

    // Create the alias (destination defaults to admin@domain)
    let destination = format!("admin@{}", domain.domain);
    let aliases_to_create = vec![(alias.clone(), destination)];

    match db::create_domain_aliases(pool, &domain.domain, aliases_to_create) {
        Ok(_created_aliases) => {
            tracing::info!("Created missing required alias {} for domain {}", alias, domain.domain);
        }
        Err(e) => {
            tracing::error!("Failed to create missing required alias {} for domain {}: {:?}", alias, domain.domain, e);
            let error_msg = get_translation(&state, &locale, "domains-error-creating-aliases").await;
            return Html(error_msg);
        }
    }

    // Redirect back to the domain show page
    let redirect_url = format!("/domains/{}", domain.pkid);
    Html(format!("<script>window.location.href = '{}';</script>", redirect_url))
}
