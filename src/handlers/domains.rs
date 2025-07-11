use crate::templates::domains::*;
use crate::templates::layout::BaseTemplate;
use crate::{
    db, get_entity_or_not_found, i18n::get_translation, models::*, render_template_with_title,
    AppState,
};
use askama::Template;
use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    response::Html,
    Form,
};

pub async fn list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<PaginationParams>,
) -> Html<String> {
    let pool = crate::handlers::utils::get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");
    let locale = crate::handlers::utils::get_user_locale(&headers);
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(20);
    let paginated_domains = match db::get_domains_paginated(&pool, page, per_page) {
        Ok(domains) => domains,
        Err(e) => {
            tracing::error!("Failed to retrieve domains: {:?}", e);
            PaginatedResult::new(vec![], 0, 1, per_page)
        }
    };
    let backups = match db::get_backups(&pool) {
        Ok(backups) => backups,
        Err(e) => {
            tracing::error!("Failed to retrieve backups: {:?}", e);
            vec![]
        }
    };
    let translations = crate::handlers::utils::get_translations_batch(
        &state,
        &locale,
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
    .await;
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
    let translations = crate::handlers::utils::get_translations_batch(
        &state,
        &locale,
        &[
            "domains-new-domain",
            "form-error",
            "form-domain",
            "form-transport",
            "form-active",
            "form-cancel",
            "form-create-domain",
            "form-update-domain",
            "form-placeholder-domain",
            "form-placeholder-transport",
            "form-tooltip-domain",
            "form-tooltip-transport",
            "form-tooltip-enable",
        ],
    )
    .await;
    let content_template = DomainFormTemplate {
        title: &translations["domains-new-domain"],
        domain: None,
        form,
        error: None,
        form_error: &translations["form-error"],
        form_domain: &translations["form-domain"],
        form_transport: &translations["form-transport"],
        form_active: &translations["form-active"],
        form_cancel: &translations["form-cancel"],
        form_create_domain: &translations["form-create-domain"],
        form_update_domain: &translations["form-update-domain"],
        form_placeholder_domain: &translations["form-placeholder-domain"],
        form_placeholder_transport: &translations["form-placeholder-transport"],
        form_tooltip_domain: &translations["form-tooltip-domain"],
        form_tooltip_transport: &translations["form-tooltip-transport"],
        form_tooltip_enable: &translations["form-tooltip-enable"],
    };
    render_template_with_title!(
        content_template,
        content_template.title,
        &state,
        &locale,
        &headers
    )
}

pub async fn show(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = crate::handlers::utils::get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");
    let domain = get_entity_or_not_found!(
        db::get_domain(&pool, id),
        &state,
        &crate::handlers::utils::get_user_locale(&headers),
        "domains-not-found"
    );
    let locale = crate::handlers::utils::get_user_locale(&headers);
    let alias_report = db::get_domain_alias_report(&pool, &domain.domain).ok();
    let existing_aliases = db::get_aliases_for_domain(&pool, &domain.domain).unwrap_or_default();
    let translations = crate::handlers::utils::get_translations_batch(
        &state,
        &locale,
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
    .await;
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
    let pool = crate::handlers::utils::get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");
    let locale = crate::handlers::language::get_user_locale(&headers);

    let domain = match db::get_domain(&pool, id) {
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
    let form_placeholder_transport =
        get_translation(&state, &locale, "form-placeholder-transport").await;
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

pub async fn create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<DomainForm>,
) -> Html<String> {
    let pool = crate::handlers::utils::get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");

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
        let form_placeholder_domain =
            get_translation(&state, &locale, "form-placeholder-domain").await;
        let form_placeholder_transport =
            get_translation(&state, &locale, "form-placeholder-transport").await;
        let form_tooltip_domain = get_translation(&state, &locale, "form-tooltip-domain").await;
        let form_tooltip_transport =
            get_translation(&state, &locale, "form-tooltip-transport").await;
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

    match db::create_domain(&pool, new_domain) {
        Ok(_) => {
            let domains = db::get_domains(&pool).unwrap_or_default();

            let locale = crate::handlers::language::get_user_locale(&headers);
            let title = get_translation(&state, &locale, "domains-title").await;
            let description = get_translation(&state, &locale, "domains-description").await;
            let add_domain = get_translation(&state, &locale, "domains-add").await;
            let table_header_domain =
                get_translation(&state, &locale, "domains-table-header-domain").await;
            let table_header_enabled =
                get_translation(&state, &locale, "domains-table-header-enabled").await;
            let table_header_actions =
                get_translation(&state, &locale, "domains-table-header-actions").await;
            let table_header_transport =
                get_translation(&state, &locale, "domains-transport").await;
            let status_active = get_translation(&state, &locale, "status-active").await;
            let status_inactive = get_translation(&state, &locale, "status-inactive").await;
            let action_enable = get_translation(&state, &locale, "action-enable").await;
            let action_disable = get_translation(&state, &locale, "action-disable").await;
            let empty_title = get_translation(&state, &locale, "domains-empty-title").await;
            let empty_description =
                get_translation(&state, &locale, "domains-empty-description").await;

            // Get backups data
            let backups = match db::get_backups(&pool) {
                Ok(backups) => backups,
                Err(e) => {
                    tracing::error!("Failed to retrieve backups: {:?}", e);
                    vec![]
                }
            };

            // Backup translations
            let backups_title = get_translation(&state, &locale, "backups-title").await;
            let backups_description = get_translation(&state, &locale, "backups-description").await;
            let add_backup = get_translation(&state, &locale, "backups-add").await;
            let backups_table_header_domain =
                get_translation(&state, &locale, "backups-table-header-domain").await;
            let backups_table_header_transport =
                get_translation(&state, &locale, "backups-table-header-transport").await;
            let backups_table_header_enabled =
                get_translation(&state, &locale, "backups-table-header-enabled").await;
            let backups_table_header_actions =
                get_translation(&state, &locale, "backups-table-header-actions").await;
            let backups_view = get_translation(&state, &locale, "backups-view").await;
            let backups_disable = get_translation(&state, &locale, "backups-disable").await;
            let backups_enable = get_translation(&state, &locale, "backups-enable").await;
            let backups_empty_no_backup_servers =
                get_translation(&state, &locale, "backups-empty-no-backup-servers").await;
            let backups_empty_get_started =
                get_translation(&state, &locale, "backups-empty-get-started").await;

            let paginated = PaginatedResult::new(domains.clone(), 0, 1, 20);
            let page_range: Vec<i64> = (1..=paginated.total_pages).collect();
            let max_item = std::cmp::min(
                paginated.current_page * paginated.per_page,
                paginated.total_count,
            );
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
                ) => get_translation(&state, &locale, "error-duplicate-domain")
                    .await
                    .replace("{domain}", &form.domain),
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
            let form_placeholder_domain =
                get_translation(&state, &locale, "form-placeholder-domain").await;
            let form_placeholder_transport =
                get_translation(&state, &locale, "form-placeholder-transport").await;
            let form_tooltip_domain = get_translation(&state, &locale, "form-tooltip-domain").await;
            let form_tooltip_transport =
                get_translation(&state, &locale, "form-tooltip-transport").await;
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
    let pool = crate::handlers::utils::get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");

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
        let form_placeholder_domain =
            get_translation(&state, &locale, "form-placeholder-domain").await;
        let form_placeholder_transport =
            get_translation(&state, &locale, "form-placeholder-transport").await;
        let form_tooltip_domain = get_translation(&state, &locale, "form-tooltip-domain").await;
        let form_tooltip_transport =
            get_translation(&state, &locale, "form-tooltip-transport").await;
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
    match db::update_domain(&pool, id, form) {
        Ok(_) => {
            let domain = match db::get_domain(&pool, id) {
                Ok(domain) => domain,
                Err(_) => return Html("Domain not found".to_string()),
            };
            let locale = crate::handlers::language::get_user_locale(&headers);
            let title = get_translation(&state, &locale, "domains-title").await;
            let view_edit_settings =
                get_translation(&state, &locale, "domains-view-edit-settings").await;
            let back_to_domains = get_translation(&state, &locale, "domains-back-to-domains").await;
            let domain_information =
                get_translation(&state, &locale, "domains-domain-information").await;
            let domain_details = get_translation(&state, &locale, "domains-domain-details").await;
            let domain_name = get_translation(&state, &locale, "domains-domain-name").await;
            let transport = get_translation(&state, &locale, "domains-transport").await;
            let status = get_translation(&state, &locale, "domains-status").await;
            let status_active = get_translation(&state, &locale, "status-active").await;
            let status_inactive = get_translation(&state, &locale, "status-inactive").await;
            let created = get_translation(&state, &locale, "domains-created").await;
            let modified = get_translation(&state, &locale, "domains-modified").await;
            let edit_domain_button =
                get_translation(&state, &locale, "domains-edit-domain-button").await;
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
                ) => format!("A domain with the name '{domain_name}' already exists."),
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
            let form_placeholder_domain =
                get_translation(&state, &locale, "form-placeholder-domain").await;
            let form_placeholder_transport =
                get_translation(&state, &locale, "form-placeholder-transport").await;
            let form_tooltip_domain = get_translation(&state, &locale, "form-tooltip-domain").await;
            let form_tooltip_transport =
                get_translation(&state, &locale, "form-tooltip-transport").await;
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

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = crate::handlers::utils::get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");

    match db::delete_domain(&pool, id) {
        Ok(_) => {
            let domains = db::get_domains(&pool).unwrap_or_default();

            let locale = crate::handlers::language::get_user_locale(&headers);
            let title = get_translation(&state, &locale, "domains-title").await;
            let description = get_translation(&state, &locale, "domains-description").await;
            let add_domain = get_translation(&state, &locale, "domains-add").await;
            let table_header_domain =
                get_translation(&state, &locale, "domains-table-header-domain").await;
            let table_header_enabled =
                get_translation(&state, &locale, "domains-table-header-enabled").await;
            let table_header_actions =
                get_translation(&state, &locale, "domains-table-header-actions").await;
            let table_header_transport =
                get_translation(&state, &locale, "domains-transport").await;
            let status_active = get_translation(&state, &locale, "status-active").await;
            let status_inactive = get_translation(&state, &locale, "status-inactive").await;
            let action_enable = get_translation(&state, &locale, "action-enable").await;
            let action_disable = get_translation(&state, &locale, "action-disable").await;
            let empty_title = get_translation(&state, &locale, "domains-empty-title").await;
            let empty_description =
                get_translation(&state, &locale, "domains-empty-description").await;

            // Get backups data
            let backups = match db::get_backups(&pool) {
                Ok(backups) => backups,
                Err(e) => {
                    tracing::error!("Failed to retrieve backups: {:?}", e);
                    vec![]
                }
            };

            // Backup translations
            let backups_title = get_translation(&state, &locale, "backups-title").await;
            let backups_description = get_translation(&state, &locale, "backups-description").await;
            let add_backup = get_translation(&state, &locale, "backups-add").await;
            let backups_table_header_domain =
                get_translation(&state, &locale, "backups-table-header-domain").await;
            let backups_table_header_transport =
                get_translation(&state, &locale, "backups-table-header-transport").await;
            let backups_table_header_enabled =
                get_translation(&state, &locale, "backups-table-header-enabled").await;
            let backups_table_header_actions =
                get_translation(&state, &locale, "backups-table-header-actions").await;
            let backups_view = get_translation(&state, &locale, "backups-view").await;
            let backups_disable = get_translation(&state, &locale, "backups-disable").await;
            let backups_enable = get_translation(&state, &locale, "backups-enable").await;
            let backups_empty_no_backup_servers =
                get_translation(&state, &locale, "backups-empty-no-backup-servers").await;
            let backups_empty_get_started =
                get_translation(&state, &locale, "backups-empty-get-started").await;

            let paginated = PaginatedResult::new(domains.clone(), 0, 1, 20);
            let page_range: Vec<i64> = (1..=paginated.total_pages).collect();
            let max_item = std::cmp::min(
                paginated.current_page * paginated.per_page,
                paginated.total_count,
            );
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

pub async fn toggle_enabled(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = crate::handlers::utils::get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");
    let locale = crate::handlers::language::get_user_locale(&headers);

    match db::toggle_domain_enabled(&pool, id) {
        Ok(_) => {
            let domain = match db::get_domain(&pool, id) {
                Ok(domain) => domain,
                Err(_) => return Html("Domain not found".to_string()),
            };

            let title = get_translation(&state, &locale, "domains-title").await;
            let view_edit_settings =
                get_translation(&state, &locale, "domains-view-edit-settings").await;
            let back_to_domains = get_translation(&state, &locale, "domains-back-to-domains").await;
            let domain_information =
                get_translation(&state, &locale, "domains-domain-information").await;
            let domain_details = get_translation(&state, &locale, "domains-domain-details").await;
            let domain_name = get_translation(&state, &locale, "domains-domain-name").await;
            let transport = get_translation(&state, &locale, "domains-transport").await;
            let status = get_translation(&state, &locale, "domains-status").await;
            let status_active = get_translation(&state, &locale, "status-active").await;
            let status_inactive = get_translation(&state, &locale, "status-inactive").await;
            let created = get_translation(&state, &locale, "domains-created").await;
            let modified = get_translation(&state, &locale, "domains-modified").await;
            let edit_domain_button =
                get_translation(&state, &locale, "domains-edit-domain-button").await;
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

            // Get current database id from session/cookie or default
            let current_db_id = crate::handlers::auth::get_selected_database(&headers)
                .unwrap_or_else(|| state.db_manager.get_default_db_id().to_string());
            // Get current database label from db_manager
            let current_db_label = state
                .db_manager
                .get_configs()
                .iter()
                .find(|db| db.id == current_db_id)
                .map(|db| db.label.clone())
                .unwrap_or_else(|| current_db_id.clone());

            let template = BaseTemplate::with_i18n(
                get_translation(&state, &locale, "domains-title").await,
                content,
                &state,
                &locale,
                current_db_label,
                current_db_id,
            )
            .await
            .unwrap();

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
    let pool = crate::handlers::utils::get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");
    match db::toggle_domain_enabled(&pool, id) {
        Ok(_) => {
            let domains = db::get_domains(&pool).unwrap_or_default();

            let locale = crate::handlers::language::get_user_locale(&headers);
            let title = get_translation(&state, &locale, "domains-title").await;
            let description = get_translation(&state, &locale, "domains-description").await;
            let add_domain = get_translation(&state, &locale, "domains-add").await;
            let table_header_domain =
                get_translation(&state, &locale, "domains-table-header-domain").await;
            let table_header_enabled =
                get_translation(&state, &locale, "domains-table-header-enabled").await;
            let table_header_actions =
                get_translation(&state, &locale, "domains-table-header-actions").await;
            let table_header_transport =
                get_translation(&state, &locale, "domains-transport").await;
            let status_active = get_translation(&state, &locale, "status-active").await;
            let status_inactive = get_translation(&state, &locale, "status-inactive").await;
            let action_enable = get_translation(&state, &locale, "action-enable").await;
            let action_disable = get_translation(&state, &locale, "action-disable").await;
            let empty_title = get_translation(&state, &locale, "domains-empty-title").await;
            let empty_description =
                get_translation(&state, &locale, "domains-empty-description").await;

            // Get backups data
            let backups = match db::get_backups(&pool) {
                Ok(backups) => backups,
                Err(e) => {
                    tracing::error!("Failed to retrieve backups: {:?}", e);
                    vec![]
                }
            };

            // Backup translations
            let backups_title = get_translation(&state, &locale, "backups-title").await;
            let backups_description = get_translation(&state, &locale, "backups-description").await;
            let add_backup = get_translation(&state, &locale, "backups-add").await;
            let backups_table_header_domain =
                get_translation(&state, &locale, "backups-table-header-domain").await;
            let backups_table_header_transport =
                get_translation(&state, &locale, "backups-table-header-transport").await;
            let backups_table_header_enabled =
                get_translation(&state, &locale, "backups-table-header-enabled").await;
            let backups_table_header_actions =
                get_translation(&state, &locale, "backups-table-header-actions").await;
            let backups_view = get_translation(&state, &locale, "backups-view").await;
            let backups_disable = get_translation(&state, &locale, "backups-disable").await;
            let backups_enable = get_translation(&state, &locale, "backups-enable").await;
            let backups_empty_no_backup_servers =
                get_translation(&state, &locale, "backups-empty-no-backup-servers").await;
            let backups_empty_get_started =
                get_translation(&state, &locale, "backups-empty-get-started").await;

            let paginated = PaginatedResult::new(domains.clone(), 0, 1, 20);
            let page_range: Vec<i64> = (1..=paginated.total_pages).collect();
            let max_item = std::cmp::min(
                paginated.current_page * paginated.per_page,
                paginated.total_count,
            );
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
    let pool = crate::handlers::utils::get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");
    match db::toggle_domain_enabled(&pool, id) {
        Ok(_) => {
            let domain = match db::get_domain(&pool, id) {
                Ok(domain) => domain,
                Err(_) => return Html("Domain not found".to_string()),
            };
            let locale = crate::handlers::language::get_user_locale(&headers);
            let title = get_translation(&state, &locale, "domains-title").await;
            let view_edit_settings =
                get_translation(&state, &locale, "domains-view-edit-settings").await;
            let back_to_domains = get_translation(&state, &locale, "domains-back-to-domains").await;
            let domain_information =
                get_translation(&state, &locale, "domains-domain-information").await;
            let domain_details = get_translation(&state, &locale, "domains-domain-details").await;
            let domain_name = get_translation(&state, &locale, "domains-domain-name").await;
            let transport = get_translation(&state, &locale, "domains-transport").await;
            let status = get_translation(&state, &locale, "domains-status").await;
            let status_active = get_translation(&state, &locale, "status-active").await;
            let status_inactive = get_translation(&state, &locale, "status-inactive").await;
            let created = get_translation(&state, &locale, "domains-created").await;
            let modified = get_translation(&state, &locale, "domains-modified").await;
            let edit_domain_button =
                get_translation(&state, &locale, "domains-edit-domain-button").await;
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
    let pool = crate::handlers::utils::get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");
    let locale = crate::handlers::language::get_user_locale(&headers);

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
            tracing::warn!("Failed to load config, using defaults: {:?}", e);
            crate::config::Config::default()
        }
    };

    // Get current alias report to see what's missing
    let alias_report = match db::get_domain_alias_report(&pool, &domain.domain) {
        Ok(report) => report,
        Err(e) => {
            tracing::error!(
                "Failed to get alias report for domain {}: {:?}",
                domain.domain,
                e
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
                tracing::info!(
                    "Created {} missing required aliases for domain {}",
                    created_aliases.len(),
                    domain.domain
                );
            }
            Err(e) => {
                tracing::error!(
                    "Failed to create missing required aliases for domain {}: {:?}",
                    domain.domain,
                    e
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
    let pool = crate::handlers::utils::get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");
    let locale = crate::handlers::language::get_user_locale(&headers);

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
            tracing::info!(
                "Created missing required alias {} for domain {}",
                alias,
                domain.domain
            );
        }
        Err(e) => {
            tracing::error!(
                "Failed to create missing required alias {} for domain {}: {:?}",
                alias,
                domain.domain,
                e
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
