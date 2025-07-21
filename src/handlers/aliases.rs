use axum::{
    extract::{Form, Path, Query, State},
    http::HeaderMap,
    response::Html,
};
use serde::Deserialize;
use askama::Template;

use crate::{
    db,
    handlers::{
        auth::get_selected_database,
        language::get_user_locale,
        utils::{
            get_field_translations, get_translations_batch,
            get_entity_form_translations, get_current_db_pool,
            is_htmx_request, render_form_template, handle_database_error,
            render_list_template, render_show_template,
        },
    },
    i18n::get_translation,
    models::{PaginatedResult, PaginationParams},
    templates::{
        aliases::{
            AliasFormTemplate, AliasSearchResultsTemplate, AliasShowTemplate, AliasesListTemplate,
            DomainSearchResultsTemplate,
        },
        domains::DomainShowTemplate,
        layout::BaseTemplate,
    },
    AppState,
};

#[derive(Deserialize)]
pub struct AliasPrefill {
    pub domain: Option<String>,
    pub alias: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct AliasSearchQuery {
    pub destination: Option<String>,
    pub alias: Option<String>,
    pub limit: Option<i64>,
}

#[derive(serde::Deserialize)]
pub struct DomainSearchQuery {
    pub domain: Option<String>,
    pub limit: Option<i64>,
}

pub async fn list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<PaginationParams>,
) -> Html<String> {
    let pool = get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(20);
    let paginated_aliases = match db::get_aliases_paginated(&pool, page, per_page) {
        Ok(aliases) => aliases,
        Err(_) => PaginatedResult::new(vec![], 0, 1, per_page),
    };
    let locale = get_user_locale(&headers);
    let translations = get_translations_batch(
        &state,
        &locale,
        &[
            "aliases-title",
            "aliases-description",
            "aliases-add",
            "aliases-table-header-mail",
            "aliases-table-header-destination",
            "aliases-table-header-enabled",
            "aliases-table-header-actions",
            "status-active",
            "status-inactive",
            "action-view",
            "aliases-enable-alias",
            "aliases-disable-alias",
            "aliases-empty-title",
            "aliases-empty-description",
        ],
    )
    .await;
    let paginated = PaginatedResult::new(
        paginated_aliases.items.clone(),
        paginated_aliases.total_count,
        paginated_aliases.current_page,
        paginated_aliases.per_page,
    );
    let page_range: Vec<i64> = (1..=paginated.total_pages).collect();
    let max_item = std::cmp::min(
        paginated.current_page * paginated.per_page,
        paginated.total_count,
    );
    let content_template = AliasesListTemplate {
        title: &translations["aliases-title"],
        aliases: &paginated_aliases.items,
        pagination: &paginated,
        page_range: &page_range,  // Pass as reference
        max_item,
        description: &translations["aliases-description"],
        add_alias: &translations["aliases-add"],
        table_header_mail: &translations["aliases-table-header-mail"],
        table_header_destination: &translations["aliases-table-header-destination"],
        table_header_enabled: &translations["aliases-table-header-enabled"],
        table_header_actions: &translations["aliases-table-header-actions"],
        status_active: &translations["status-active"],
        status_inactive: &translations["status-inactive"],
        action_view: &translations["action-view"],
        enable_alias: &translations["aliases-enable-alias"],
        disable_alias: &translations["aliases-disable-alias"],
        empty_title: &translations["aliases-empty-title"],
        empty_description: &translations["aliases-empty-description"],
    };
    render_list_template(
        content_template,
        &state,
        &locale,
        &headers
    ).await
}

pub async fn new(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(prefill): Query<AliasPrefill>,
) -> Html<String> {
    let return_url = headers
        .get("referer")
        .and_then(|r| r.to_str().ok())
        .filter(|r| r.contains("/domains/"))
        .map(|r| r.to_string());
    let mail = match (&prefill.alias, &prefill.domain) {
        (Some(alias), Some(domain)) => format!("{alias}@{domain}"),
        (Some(alias), None) => alias.clone(),
        (None, Some(domain)) => domain.to_string(),
        (None, None) => "".to_string(),
    };
    let form = crate::models::AliasForm {
        mail,
        destination: "".to_string(),
        enabled: true,
        return_url: None,
    };
    let locale = get_user_locale(&headers);

    // Use helper functions to fetch translations in batches
    let form_translations =
        get_entity_form_translations(&state, &locale, "aliases").await;
    let field_translations = get_field_translations(
        &state,
        &locale,
        "aliases",
        &["mail", "destination", "active"],
    )
    .await;

    let content_template = AliasFormTemplate {
        title: &form_translations["aliases-add-title"],
        alias: None,
        form,
        error: None,
        return_url,
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

    // Use helper function for template rendering
    render_form_template(
        content_template,
        &state,
        &locale,
        &headers,
        form_translations["aliases-add-title"].clone(),
    )
    .await
}

pub async fn show(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");
    let alias = match db::get_alias(&pool, id) {
        Ok(alias) => alias,
        Err(_) => {
            let locale = get_user_locale(&headers);
            let not_found_msg = get_translation(&state, &locale, "aliases-not-found").await;
            return Html(not_found_msg);
        }
    };
    let locale = get_user_locale(&headers);
    let translations = get_translations_batch(
        &state,
        &locale,
        &[
            "aliases-show-title",
            "aliases-view-edit-settings",
            "aliases-back-to-aliases",
            "aliases-alias-information",
            "aliases-alias-details",
            "aliases-mail",
            "aliases-forward-to",
            "aliases-status",
            "status-active",
            "status-inactive",
            "aliases-created",
            "aliases-modified",
            "aliases-edit-alias-button",
            "aliases-enable-alias-button",
            "aliases-disable-alias-button",
            "aliases-delete-alias",
            "aliases-delete-confirm",
        ],
    )
    .await;
    let content_template = AliasShowTemplate {
        title: &translations["aliases-show-title"],
        view_edit_settings: &translations["aliases-view-edit-settings"],
        back_to_aliases: &translations["aliases-back-to-aliases"],
        alias_information: &translations["aliases-alias-information"],
        alias_details: &translations["aliases-alias-details"],
        mail: &translations["aliases-mail"],
        forward_to: &translations["aliases-forward-to"],
        status: &translations["aliases-status"],
        status_active: &translations["status-active"],
        status_inactive: &translations["status-inactive"],
        created: &translations["aliases-created"],
        modified: &translations["aliases-modified"],
        edit_alias_button: &translations["aliases-edit-alias-button"],
        enable_alias_button: &translations["aliases-enable-alias-button"],
        disable_alias_button: &translations["aliases-disable-alias-button"],
        delete_alias: &translations["aliases-delete-alias"],
        delete_confirm: &translations["aliases-delete-confirm"],
        alias,
    };
    render_show_template(
        content_template,
        &state,
        &locale,
        &headers
    ).await
}

pub async fn edit(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");

    let alias = match db::get_alias(&pool, id) {
        Ok(alias) => alias,
        Err(_) => return Html("Alias not found".to_string()),
    };

    let form = crate::models::AliasForm {
        mail: alias.mail.clone(),
        destination: alias.destination.clone(),
        enabled: alias.enabled,
        return_url: None,
    };

    let locale = get_user_locale(&headers);

    // Use helper functions to fetch translations in batches
    let form_translations =
        get_entity_form_translations(&state, &locale, "aliases").await;
    let field_translations = get_field_translations(
        &state,
        &locale,
        "aliases",
        &["mail", "destination", "active"],
    )
    .await;

    let content_template = AliasFormTemplate {
        title: &form_translations["aliases-edit-title"],
        alias: Some(alias),
        form,
        error: None,
        return_url: None,
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

    // Use helper function for template rendering
    render_form_template(
        content_template,
        &state,
        &locale,
        &headers,
        form_translations["aliases-edit-title"].clone(),
    )
    .await
}

pub async fn create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<crate::models::AliasForm>,
) -> Html<String> {
    let pool = get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");

    match db::create_alias(&pool, form.clone()) {
        Ok(created_alias) => {
            // Extract domain from the created alias and redirect to domain show page
            let domain_name = created_alias.domain();

            // Try to find the domain by name
            match db::get_domain_by_name(&pool, &domain_name) {
                Ok(domain) => {
                    // Redirect to the domain show page
                    let locale = get_user_locale(&headers);
                    let title = get_translation(&state, &locale, "domains-show-title").await;
                    let status = get_translation(&state, &locale, "domains-status").await;
                    let status_active = get_translation(&state, &locale, "status-active").await;
                    let status_inactive = get_translation(&state, &locale, "status-inactive").await;
                    let created = get_translation(&state, &locale, "domains-created").await;
                    let modified = get_translation(&state, &locale, "domains-modified").await;
                    let edit_domain_button =
                        get_translation(&state, &locale, "domains-edit-domain-button").await;
                    let enable_domain =
                        get_translation(&state, &locale, "domains-enable-domain").await;
                    let disable_domain =
                        get_translation(&state, &locale, "domains-disable-domain").await;
                    let delete_domain =
                        get_translation(&state, &locale, "domains-delete-domain").await;
                    let delete_confirm =
                        get_translation(&state, &locale, "domains-delete-confirm").await;
                    let alias_report_title =
                        get_translation(&state, &locale, "reports-alias-report-title").await;
                    let alias_report_description =
                        get_translation(&state, &locale, "reports-alias-report-description").await;
                    let add_alias_button =
                        get_translation(&state, &locale, "domains-add-alias-button").await;
                    let missing_aliases_header =
                        get_translation(&state, &locale, "reports-missing-aliases-header").await;
                    let missing_required_alias_header =
                        get_translation(&state, &locale, "reports-missing-required-alias-header")
                            .await;
                    let missing_common_aliases_header =
                        get_translation(&state, &locale, "reports-missing-common-aliases-header")
                            .await;
                    let add_missing_required_alias_button = get_translation(
                        &state,
                        &locale,
                        "reports-add-missing-required-alias-button",
                    )
                    .await;
                    let add_common_alias_button =
                        get_translation(&state, &locale, "reports-add-common-alias-button").await;
                    let catch_all_header =
                        get_translation(&state, &locale, "reports-catch-all-header").await;
                    let add_catch_all_button =
                        get_translation(&state, &locale, "reports-add-catch-all-button").await;
                    let no_catch_all_message =
                        get_translation(&state, &locale, "reports-no-catch-all-message").await;
                    let destination_header =
                        get_translation(&state, &locale, "reports-destination-header").await;
                    let existing_aliases_header =
                        get_translation(&state, &locale, "reports-existing-aliases-header").await;
                    let no_required_aliases =
                        get_translation(&state, &locale, "reports-no-required-aliases").await;
                    let mail_header = get_translation(&state, &locale, "reports-mail-header").await;
                    let enabled_header =
                        get_translation(&state, &locale, "reports-enabled-header").await;
                    let actions_header =
                        get_translation(&state, &locale, "reports-actions-header").await;
                    let no_missing_aliases =
                        get_translation(&state, &locale, "reports-no-missing-aliases").await;

                    // Get alias report for the domain
                    let alias_report = match db::get_domain_alias_report(&pool, &domain.domain) {
                        Ok(report) => Some(report),
                        Err(e) => {
                            eprintln!("Error getting domain alias report: {e:?}");
                            None
                        }
                    };

                    // Get existing aliases for the domain
                    let existing_aliases = match db::get_aliases_for_domain(&pool, &domain.domain) {
                        Ok(aliases) => aliases,
                        Err(e) => {
                            eprintln!("Error getting aliases for domain: {e:?}");
                            vec![]
                        }
                    };

                    let view_edit_settings =
                        get_translation(&state, &locale, "domains-view-edit-settings").await;
                    let back_to_domains =
                        get_translation(&state, &locale, "domains-back-to-domains").await;
                    let domain_information =
                        get_translation(&state, &locale, "domains-domain-information").await;
                    let domain_details =
                        get_translation(&state, &locale, "domains-domain-details").await;
                    let domain_name = get_translation(&state, &locale, "domains-domain-name").await;
                    let transport = get_translation(&state, &locale, "domains-transport").await;
                    let required_aliases_header =
                        get_translation(&state, &locale, "reports-required-aliases-header").await;
                    let status_header =
                        get_translation(&state, &locale, "reports-status-header").await;

                    let action_view = get_translation(&state, &locale, "action-view").await;
                    let enable_alias =
                        get_translation(&state, &locale, "aliases-enable-alias").await;
                    let disable_alias =
                        get_translation(&state, &locale, "aliases-disable-alias").await;

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
                        action_view: &action_view,
                        enable_alias: &enable_alias,
                        disable_alias: &disable_alias,
                    };
                    let content = content_template.render().unwrap();

                    if is_htmx_request(&headers) {
                        Html(content)
                    } else {
                        let locale = get_user_locale(&headers);
                        let current_db_id = get_selected_database(&headers)
                            .unwrap_or_else(|| state.db_manager.get_default_db_id().to_string());
                        let current_db_label = state
                            .db_manager
                            .get_configs()
                            .iter()
                            .find(|db| db.id == current_db_id)
                            .map(|db| db.label.clone())
                            .unwrap_or_else(|| current_db_id.clone());
                        let template = BaseTemplate::with_i18n(
                            get_translation(&state, &locale, "domains-show-title").await,
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
                }
                Err(e) => {
                    eprintln!("Error finding domain by name: {e:?}");
                    // Fallback to aliases list if domain not found
                    let aliases = match db::get_aliases(&pool) {
                        Ok(aliases) => aliases,
                        Err(e) => {
                            eprintln!("Error getting aliases: {e:?}");
                            vec![]
                        }
                    };
                    let locale = get_user_locale(&headers);
                    let title = get_translation(&state, &locale, "aliases-title").await;
                    let description = get_translation(&state, &locale, "aliases-description").await;
                    let add_alias = get_translation(&state, &locale, "aliases-add").await;
                    let table_header_mail =
                        get_translation(&state, &locale, "aliases-table-header-mail").await;
                    let table_header_destination =
                        get_translation(&state, &locale, "aliases-table-header-destination").await;
                    let table_header_enabled =
                        get_translation(&state, &locale, "aliases-table-header-enabled").await;
                    let table_header_actions =
                        get_translation(&state, &locale, "aliases-table-header-actions").await;
                    let status_active = get_translation(&state, &locale, "status-active").await;
                    let status_inactive = get_translation(&state, &locale, "status-inactive").await;
                    let action_view = get_translation(&state, &locale, "action-view").await;
                    let enable_alias =
                        get_translation(&state, &locale, "aliases-enable-alias").await;
                    let disable_alias =
                        get_translation(&state, &locale, "aliases-disable-alias").await;
                    let empty_title = get_translation(&state, &locale, "aliases-empty-title").await;
                    let empty_description =
                        get_translation(&state, &locale, "aliases-empty-description").await;
                    let paginated = PaginatedResult::new(aliases.clone(), 0, 1, 20);
                    let page_range: Vec<i64> = (1..=paginated.total_pages).collect();
                    let max_item = std::cmp::min(
                        paginated.current_page * paginated.per_page,
                        paginated.total_count,
                    );
                    let content_template = AliasesListTemplate {
                        title: &title,
                        aliases: &aliases,
                        pagination: &paginated,
                        page_range: &page_range,  // Pass as reference
                        max_item,
                        description: &description,
                        add_alias: &add_alias,
                        table_header_mail: &table_header_mail,
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
                    };
                    let content = content_template.render().unwrap();

                    if is_htmx_request(&headers) {
                        Html(content)
                    } else {
                        let locale = get_user_locale(&headers);
                        let current_db_id = get_selected_database(&headers)
                            .unwrap_or_else(|| state.db_manager.get_default_db_id().to_string());
                        let current_db_label = state
                            .db_manager
                            .get_configs()
                            .iter()
                            .find(|db| db.id == current_db_id)
                            .map(|db| db.label.clone())
                            .unwrap_or_else(|| current_db_id.clone());
                        let template = BaseTemplate::with_i18n(
                            get_translation(&state, &locale, "aliases-title").await,
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
                }
            }
        }
        Err(e) => {
            eprintln!("Error creating alias: {e:?}");

            // Handle specific database errors with user-friendly messages
            let locale = get_user_locale(&headers);
            let error_message = handle_database_error(
                &state, &locale, e, "alias", &form.mail,
            )
            .await;

            // Use helper functions to fetch translations in batches
            let form_translations =
                get_entity_form_translations(&state, &locale, "aliases")
                    .await;
            let field_translations = get_field_translations(
                &state,
                &locale,
                "aliases",
                &["mail", "destination", "active"],
            )
            .await;

            let content_template = AliasFormTemplate {
                title: &form_translations["aliases-add-title"],
                alias: None,
                form: form.clone(),
                error: Some(error_message),
                return_url: None,
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

            // Use helper function for template rendering
            render_form_template(
                content_template,
                &state,
                &locale,
                &headers,
                form_translations["aliases-add-title"].clone(),
            )
            .await
        }
    }
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
    Form(form): Form<crate::models::AliasForm>,
) -> Html<String> {
    let pool = get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");

    match db::update_alias(&pool, id, form.clone()) {
        Ok(_) => {
            let alias = match db::get_alias(&pool, id) {
                Ok(alias) => alias,
                Err(_) => return Html("Alias not found".to_string()),
            };

            let locale = get_user_locale(&headers);
            let title = get_translation(&state, &locale, "aliases-show-title").await;
            let view_edit_settings =
                get_translation(&state, &locale, "aliases-view-edit-settings").await;
            let back_to_aliases = get_translation(&state, &locale, "aliases-back-to-aliases").await;
            let alias_information =
                get_translation(&state, &locale, "aliases-alias-information").await;
            let alias_details = get_translation(&state, &locale, "aliases-alias-details").await;
            let mail = get_translation(&state, &locale, "aliases-mail").await;
            let forward_to = get_translation(&state, &locale, "aliases-forward-to").await;
            let status = get_translation(&state, &locale, "aliases-status").await;
            let status_active = get_translation(&state, &locale, "status-active").await;
            let status_inactive = get_translation(&state, &locale, "status-inactive").await;
            let created = get_translation(&state, &locale, "aliases-created").await;
            let modified = get_translation(&state, &locale, "aliases-modified").await;
            let edit_alias_button =
                get_translation(&state, &locale, "aliases-edit-alias-button").await;
            let enable_alias_button =
                get_translation(&state, &locale, "aliases-enable-alias-button").await;
            let disable_alias_button =
                get_translation(&state, &locale, "aliases-disable-alias-button").await;
            let delete_alias = get_translation(&state, &locale, "aliases-delete-alias").await;
            let delete_confirm = get_translation(&state, &locale, "aliases-delete-confirm").await;

            let content_template = AliasShowTemplate {
                title: &title,
                alias,
                view_edit_settings: &view_edit_settings,
                back_to_aliases: &back_to_aliases,
                alias_information: &alias_information,
                alias_details: &alias_details,
                mail: &mail,
                forward_to: &forward_to,
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
            };
            let content = content_template.render().unwrap();

            if is_htmx_request(&headers) {
                Html(content)
            } else {
                let locale = get_user_locale(&headers);
                let current_db_id = get_selected_database(&headers)
                    .unwrap_or_else(|| state.db_manager.get_default_db_id().to_string());
                let current_db_label = state
                    .db_manager
                    .get_configs()
                    .iter()
                    .find(|db| db.id == current_db_id)
                    .map(|db| db.label.clone())
                    .unwrap_or_else(|| current_db_id.clone());
                let template = BaseTemplate::with_i18n(
                    get_translation(&state, &locale, "aliases-show-title").await,
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
        }
        Err(e) => {
            eprintln!("Error updating alias: {e:?}");

            // Handle specific database errors with user-friendly messages
            // Get the original alias for the form
            let original_alias = db::get_alias(&pool, id).ok();

            let locale = get_user_locale(&headers);
            let error_message = handle_database_error(
                &state, &locale, e, "alias", &form.mail,
            )
            .await;

            // Use helper functions to fetch translations in batches
            let form_translations =
                get_entity_form_translations(&state, &locale, "aliases")
                    .await;
            let field_translations = get_field_translations(
                &state,
                &locale,
                "aliases",
                &["mail", "destination", "active"],
            )
            .await;

            let content_template = AliasFormTemplate {
                title: &form_translations["aliases-edit-title"],
                alias: original_alias,
                form: form.clone(),
                error: Some(error_message),
                return_url: None,
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

            // Use helper function for template rendering
            render_form_template(
                content_template,
                &state,
                &locale,
                &headers,
                form_translations["aliases-edit-title"].clone(),
            )
            .await
        }
    }
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");

    match db::delete_alias(&pool, id) {
        Ok(_) => {
            let aliases = match db::get_aliases(&pool) {
                Ok(aliases) => aliases,
                Err(e) => {
                    eprintln!("Error getting aliases: {e:?}");
                    vec![]
                }
            };
            let locale = get_user_locale(&headers);
            let title = get_translation(&state, &locale, "aliases-title").await;
            let description = get_translation(&state, &locale, "aliases-description").await;
            let add_alias = get_translation(&state, &locale, "aliases-add").await;
            let table_header_mail =
                get_translation(&state, &locale, "aliases-table-header-mail").await;
            let table_header_destination =
                get_translation(&state, &locale, "aliases-table-header-destination").await;
            let table_header_enabled =
                get_translation(&state, &locale, "aliases-table-header-enabled").await;
            let table_header_actions =
                get_translation(&state, &locale, "aliases-table-header-actions").await;
            let status_active = get_translation(&state, &locale, "status-active").await;
            let status_inactive = get_translation(&state, &locale, "status-inactive").await;
            let action_view = get_translation(&state, &locale, "action-view").await;
            let enable_alias = get_translation(&state, &locale, "aliases-enable-alias").await;
            let disable_alias = get_translation(&state, &locale, "aliases-disable-alias").await;
            let empty_title = get_translation(&state, &locale, "aliases-empty-title").await;
            let empty_description =
                get_translation(&state, &locale, "aliases-empty-description").await;
            let paginated = PaginatedResult::new(aliases.clone(), 0, 1, 20);
            let page_range: Vec<i64> = (1..=paginated.total_pages).collect();
            let max_item = std::cmp::min(
                paginated.current_page * paginated.per_page,
                paginated.total_count,
            );
            let content_template = AliasesListTemplate {
                title: &title,
                aliases: &aliases,
                pagination: &paginated,
                page_range: &page_range,  // Pass as reference
                max_item,
                description: &description,
                add_alias: &add_alias,
                table_header_mail: &table_header_mail,
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
            };
            let content = content_template.render().unwrap();

            if is_htmx_request(&headers) {
                Html(content)
            } else {
                let locale = get_user_locale(&headers);
                let current_db_id = get_selected_database(&headers)
                    .unwrap_or_else(|| state.db_manager.get_default_db_id().to_string());
                let current_db_label = state
                    .db_manager
                    .get_configs()
                    .iter()
                    .find(|db| db.id == current_db_id)
                    .map(|db| db.label.clone())
                    .unwrap_or_else(|| current_db_id.clone());
                let template = BaseTemplate::with_i18n(
                    get_translation(&state, &locale, "aliases-title").await,
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
        }
        Err(_) => Html("Error deleting alias".to_string()),
    }
}

pub async fn toggle_enabled(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");

    match db::toggle_alias_enabled(&pool, id) {
        Ok(_) => {
            let alias = match db::get_alias(&pool, id) {
                Ok(alias) => alias,
                Err(_) => return Html("Alias not found".to_string()),
            };

            let locale = get_user_locale(&headers);
            let title = get_translation(&state, &locale, "aliases-show-title").await;
            let view_edit_settings =
                get_translation(&state, &locale, "aliases-view-edit-settings").await;
            let back_to_aliases = get_translation(&state, &locale, "aliases-back-to-aliases").await;
            let alias_information =
                get_translation(&state, &locale, "aliases-alias-information").await;
            let alias_details = get_translation(&state, &locale, "aliases-alias-details").await;
            let mail = get_translation(&state, &locale, "aliases-mail").await;
            let forward_to = get_translation(&state, &locale, "aliases-forward-to").await;
            let status = get_translation(&state, &locale, "aliases-status").await;
            let status_active = get_translation(&state, &locale, "status-active").await;
            let status_inactive = get_translation(&state, &locale, "status-inactive").await;
            let created = get_translation(&state, &locale, "aliases-created").await;
            let modified = get_translation(&state, &locale, "aliases-modified").await;
            let edit_alias_button =
                get_translation(&state, &locale, "aliases-edit-alias-button").await;
            let enable_alias_button =
                get_translation(&state, &locale, "aliases-enable-alias-button").await;
            let disable_alias_button =
                get_translation(&state, &locale, "aliases-disable-alias-button").await;
            let delete_alias = get_translation(&state, &locale, "aliases-delete-alias").await;
            let delete_confirm = get_translation(&state, &locale, "aliases-delete-confirm").await;

            let content_template = AliasShowTemplate {
                title: &title,
                alias,
                view_edit_settings: &view_edit_settings,
                back_to_aliases: &back_to_aliases,
                alias_information: &alias_information,
                alias_details: &alias_details,
                mail: &mail,
                forward_to: &forward_to,
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
            };
            let content = content_template.render().unwrap();
            if is_htmx_request(&headers) {
                Html(content)
            } else {
                let locale = get_user_locale(&headers);
                let current_db_id = get_selected_database(&headers)
                    .unwrap_or_else(|| state.db_manager.get_default_db_id().to_string());
                let current_db_label = state
                    .db_manager
                    .get_configs()
                    .iter()
                    .find(|db| db.id == current_db_id)
                    .map(|db| db.label.clone())
                    .unwrap_or_else(|| current_db_id.clone());
                let template = BaseTemplate::with_i18n(
                    get_translation(&state, &locale, "aliases-show-title").await,
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
        }
        Err(_) => Html("Error toggling alias status".to_string()),
    }
}

pub async fn toggle_enabled_list(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");
    match db::toggle_alias_enabled(&pool, id) {
        Ok(_) => {
            let aliases = match db::get_aliases(&pool) {
                Ok(aliases) => aliases,
                Err(e) => {
                    eprintln!("Error getting aliases: {e:?}");
                    vec![]
                }
            };
            let locale = get_user_locale(&headers);
            let title = get_translation(&state, &locale, "aliases-title").await;
            let description = get_translation(&state, &locale, "aliases-description").await;
            let add_alias = get_translation(&state, &locale, "aliases-add").await;
            let table_header_mail =
                get_translation(&state, &locale, "aliases-table-header-mail").await;
            let table_header_destination =
                get_translation(&state, &locale, "aliases-table-header-destination").await;
            let table_header_enabled =
                get_translation(&state, &locale, "aliases-table-header-enabled").await;
            let table_header_actions =
                get_translation(&state, &locale, "aliases-table-header-actions").await;
            let status_active = get_translation(&state, &locale, "status-active").await;
            let status_inactive = get_translation(&state, &locale, "status-inactive").await;
            let action_view = get_translation(&state, &locale, "action-view").await;
            let enable_alias = get_translation(&state, &locale, "aliases-enable-alias").await;
            let disable_alias = get_translation(&state, &locale, "aliases-disable-alias").await;
            let empty_title = get_translation(&state, &locale, "aliases-empty-title").await;
            let empty_description =
                get_translation(&state, &locale, "aliases-empty-description").await;
            let paginated = PaginatedResult::new(aliases.clone(), 0, 1, 20);
            let page_range: Vec<i64> = (1..=paginated.total_pages).collect();
            let max_item = std::cmp::min(
                paginated.current_page * paginated.per_page,
                paginated.total_count,
            );
            let content_template = AliasesListTemplate {
                title: &title,
                aliases: &aliases,
                pagination: &paginated,
                page_range: &page_range,  // Pass as reference
                max_item,
                description: &description,
                add_alias: &add_alias,
                table_header_mail: &table_header_mail,
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
            };
            let content = content_template.render().unwrap();

            if is_htmx_request(&headers) {
                Html(content)
            } else {
                let locale = get_user_locale(&headers);
                let current_db_id = get_selected_database(&headers)
                    .unwrap_or_else(|| state.db_manager.get_default_db_id().to_string());
                let current_db_label = state
                    .db_manager
                    .get_configs()
                    .iter()
                    .find(|db| db.id == current_db_id)
                    .map(|db| db.label.clone())
                    .unwrap_or_else(|| current_db_id.clone());
                let template = BaseTemplate::with_i18n(
                    get_translation(&state, &locale, "aliases-title").await,
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
        }
        Err(_) => Html("Error toggling alias status".to_string()),
    }
}

pub async fn toggle_enabled_show(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");
    match db::toggle_alias_enabled(&pool, id) {
        Ok(_) => {
            let alias = match db::get_alias(&pool, id) {
                Ok(alias) => alias,
                Err(_) => return Html("Alias not found".to_string()),
            };

            let locale = get_user_locale(&headers);
            let title = get_translation(&state, &locale, "aliases-show-title").await;
            let view_edit_settings =
                get_translation(&state, &locale, "aliases-view-edit-settings").await;
            let back_to_aliases = get_translation(&state, &locale, "aliases-back-to-aliases").await;
            let alias_information =
                get_translation(&state, &locale, "aliases-alias-information").await;
            let alias_details = get_translation(&state, &locale, "aliases-alias-details").await;
            let mail = get_translation(&state, &locale, "aliases-mail").await;
            let forward_to = get_translation(&state, &locale, "aliases-forward-to").await;
            let status = get_translation(&state, &locale, "aliases-status").await;
            let status_active = get_translation(&state, &locale, "status-active").await;
            let status_inactive = get_translation(&state, &locale, "status-inactive").await;
            let created = get_translation(&state, &locale, "aliases-created").await;
            let modified = get_translation(&state, &locale, "aliases-modified").await;
            let edit_alias_button =
                get_translation(&state, &locale, "aliases-edit-alias-button").await;
            let enable_alias_button =
                get_translation(&state, &locale, "aliases-enable-alias-button").await;
            let disable_alias_button =
                get_translation(&state, &locale, "aliases-disable-alias-button").await;
            let delete_alias = get_translation(&state, &locale, "aliases-delete-alias").await;
            let delete_confirm = get_translation(&state, &locale, "aliases-delete-confirm").await;

            let content_template = AliasShowTemplate {
                title: &title,
                alias,
                view_edit_settings: &view_edit_settings,
                back_to_aliases: &back_to_aliases,
                alias_information: &alias_information,
                alias_details: &alias_details,
                mail: &mail,
                forward_to: &forward_to,
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
            };
            let content = content_template.render().unwrap();

            if is_htmx_request(&headers) {
                Html(content)
            } else {
                let locale = get_user_locale(&headers);
                let current_db_id = get_selected_database(&headers)
                    .unwrap_or_else(|| state.db_manager.get_default_db_id().to_string());
                let current_db_label = state
                    .db_manager
                    .get_configs()
                    .iter()
                    .find(|db| db.id == current_db_id)
                    .map(|db| db.label.clone())
                    .unwrap_or_else(|| current_db_id.clone());
                let template = BaseTemplate::with_i18n(
                    get_translation(&state, &locale, "aliases-show-title").await,
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
        }
        Err(_) => Html("Error toggling alias status".to_string()),
    }
}

pub async fn toggle_enabled_domain_show(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");

    // First toggle the alias
    match db::toggle_alias_enabled(&pool, id) {
        Ok(_) => {
            // Get the alias to find its domain
            let alias = match db::get_alias(&pool, id) {
                Ok(alias) => alias,
                Err(_) => return Html("Alias not found".to_string()),
            };

            // Extract domain from alias mail (e.g., "user@domain.com" -> "domain.com")
            let domain_name = alias.mail.split('@').next_back().unwrap_or("");

            // Find the domain by name
            let domain = match db::get_domain_by_name(&pool, domain_name) {
                Ok(domain) => domain,
                Err(_) => return Html("Domain not found".to_string()),
            };

            // Now render the domain show page with updated alias data
            let locale = get_user_locale(&headers);
            let title = get_translation(&state, &locale, "domains-title").await;
            let view_edit_settings =
                get_translation(&state, &locale, "domains-view-edit-settings").await;
            let back_to_domains = get_translation(&state, &locale, "domains-back-to-domains").await;
            let domain_information =
                get_translation(&state, &locale, "domains-domain-information").await;
            let domain_details = get_translation(&state, &locale, "domains-domain-details").await;
            let domain_name_label = get_translation(&state, &locale, "domains-domain-name").await;
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

            // Get alias report for the domain
            let alias_report = db::get_domain_alias_report(&pool, &domain.domain).ok();

            // Get existing aliases for the domain
            let existing_aliases =
                db::get_aliases_for_domain(&pool, &domain.domain).unwrap_or_default();

            let catch_all_header =
                get_translation(&state, &locale, "reports-catch-all-header").await;
            let destination_header =
                get_translation(&state, &locale, "reports-destination-header").await;
            let required_aliases_header =
                get_translation(&state, &locale, "reports-required-aliases-header").await;
            let missing_aliases_header =
                get_translation(&state, &locale, "reports-missing-aliases-header").await;
            let missing_required_alias_header =
                get_translation(&state, &locale, "reports-missing-required-aliases-header").await;
            let missing_common_aliases_header =
                get_translation(&state, &locale, "reports-missing-common-aliases-header").await;
            let mail_header = get_translation(&state, &locale, "reports-mail-header").await;
            let status_header = get_translation(&state, &locale, "reports-status-header").await;
            let enabled_header = get_translation(&state, &locale, "reports-enabled-header").await;
            let actions_header = get_translation(&state, &locale, "reports-actions-header").await;
            let no_required_aliases =
                get_translation(&state, &locale, "reports-no-required-aliases").await;
            let no_missing_aliases =
                get_translation(&state, &locale, "reports-no-missing-aliases").await;
            let alias_report_title =
                get_translation(&state, &locale, "domains-alias-report-title").await;
            let alias_report_description =
                get_translation(&state, &locale, "domains-alias-report-description").await;
            let existing_aliases_header =
                get_translation(&state, &locale, "domains-existing-aliases-header").await;
            let add_missing_required_alias_button =
                get_translation(&state, &locale, "reports-add-missing-required-alias-button").await;
            let add_common_alias_button =
                get_translation(&state, &locale, "reports-add-common-alias-button").await;
            let add_catch_all_button =
                get_translation(&state, &locale, "reports-add-catch-all-button").await;
            let add_alias_button =
                get_translation(&state, &locale, "domains-add-alias-button").await;
            let no_catch_all_message =
                get_translation(&state, &locale, "domains-no-catch-all-message").await;

            let action_view = get_translation(&state, &locale, "action-view").await;
            let enable_alias = get_translation(&state, &locale, "aliases-enable-alias").await;
            let disable_alias = get_translation(&state, &locale, "aliases-disable-alias").await;

            let content_template = DomainShowTemplate {
                title: &title,
                domain,
                view_edit_settings: &view_edit_settings,
                back_to_domains: &back_to_domains,
                domain_information: &domain_information,
                domain_details: &domain_details,
                domain_name: &domain_name_label,
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
                action_view: &action_view,
                enable_alias: &enable_alias,
                disable_alias: &disable_alias,
            };
            let content = content_template.render().unwrap();

            if is_htmx_request(&headers) {
                Html(content)
            } else {
                let locale = get_user_locale(&headers);
                let current_db_id = get_selected_database(&headers)
                    .unwrap_or_else(|| state.db_manager.get_default_db_id().to_string());
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
        }
        Err(_) => Html("Error toggling alias status".to_string()),
    }
}

pub async fn search(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<AliasSearchQuery>,
) -> Html<String> {
    let pool = get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");

    // Get the query string
    let query_string = if let Some(alias_query) = &query.alias {
        alias_query.clone()
    } else if let Some(dest_query) = &query.destination {
        dest_query.clone()
    } else {
        String::new()
    };

    // Handle empty or missing query
    if query_string.len() < 2 {
        let locale = get_user_locale(&headers);
        let translations = get_translations_batch(
            &state,
            &locale,
            &["aliases-search-no-results", "aliases-search-select"],
        )
        .await;
        let content_template = AliasSearchResultsTemplate {
            aliases: &[],
            no_results: &translations["aliases-search-no-results"],
            select_text: &translations["aliases-search-select"],
        };
        return Html(content_template.render().unwrap());
    }

    let limit = query.limit.unwrap_or(10);

    // --- Collect all matching values from aliases and users ---
    let mut values = std::collections::HashSet::new();

    // 1. Alias mail and destination
    if let Ok(aliases) = db::search_aliases(&pool, &query_string, limit * 2) {
        for alias in aliases {
            if alias.mail.contains(&query_string) {
                values.insert(alias.mail);
            }
            if alias.destination.contains(&query_string) {
                values.insert(alias.destination);
            }
        }
    }

    // 2. User ids
    use diesel::prelude::*;
    if let Ok(mut conn) = pool.get() {
        let search_pattern = format!("%{query_string}%");
        let user_ids: Vec<String> = crate::schema::users::dsl::users
            .filter(crate::schema::users::dsl::id.like(&search_pattern))
            .select(crate::schema::users::dsl::id)
            .limit(limit * 2)
            .load::<String>(&mut conn)
            .unwrap_or_default();
        for user_id in user_ids {
            values.insert(user_id);
        }
    }

    // 3. Sort and limit
    let mut values: Vec<String> = values.into_iter().collect();
    values.sort_by_key(|a| a.to_lowercase());
    values.truncate(limit as usize);

    // 4. Render as a flat list of suggestions
    let html = if values.is_empty() {
        let locale = get_user_locale(&headers);
        let translations = get_translations_batch(
            &state,
            &locale,
            &["aliases-search-no-results", "aliases-search-select"],
        )
        .await;
        format!(
            "<ul><li class=\"text-gray-400\">{}</li></ul>",
            translations["aliases-search-no-results"]
        )
    } else {
        let items: String = values
            .into_iter()
            .map(|v| format!("<li class=\"cursor-pointer\">{v}</li>"))
            .collect();
        format!("<ul>{items}</ul>")
    };

    Html(html)
}

pub async fn domain_search(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<DomainSearchQuery>,
) -> Html<String> {
    let pool = get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");

    let query_string = query.domain.unwrap_or_default();

    // Handle empty or missing query
    if query_string.len() < 2 {
        let locale = get_user_locale(&headers);
        let translations = get_translations_batch(
            &state,
            &locale,
            &[
                "domains-search-no-results",
                "domains-search-select",
                "status-active",
                "status-inactive",
            ],
        )
        .await;
        let content_template = DomainSearchResultsTemplate {
            domains: &[],
            no_results: &translations["domains-search-no-results"],
            select_text: &translations["domains-search-select"],
            status_active: &translations["status-active"],
            status_inactive: &translations["status-inactive"],
        };
        return Html(content_template.render().unwrap());
    }

    let limit = query.limit.unwrap_or(10);
    let search_results = db::search_domains(&pool, &query_string, limit);

    let domains = search_results.unwrap_or_default();

    let locale = get_user_locale(&headers);
    let translations = get_translations_batch(
        &state,
        &locale,
        &[
            "domains-search-no-results",
            "domains-search-select",
            "status-active",
            "status-inactive",
        ],
    )
    .await;
    let content_template = DomainSearchResultsTemplate {
        domains: &domains,
        no_results: &translations["domains-search-no-results"],
        select_text: &translations["domains-search-select"],
        status_active: &translations["status-active"],
        status_inactive: &translations["status-inactive"],
    };
    Html(content_template.render().unwrap())
}
