use crate::templates::aliases::*;
use crate::templates::layout::BaseTemplate;
use crate::{db, models::*, AppState, i18n::get_translation};
use askama::Template;
use axum::{
    extract::{Path, State, Query},
    http::HeaderMap,
    response::Html,
    Form,
};
use serde::Deserialize;

fn is_htmx_request(headers: &HeaderMap) -> bool {
    headers.get("HX-Request").map_or(false, |v| v == "true")
}

#[derive(Deserialize)]
pub struct AliasPrefill {
    pub domain: Option<String>,
    pub alias: Option<String>,
}

pub async fn list(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    let pool = &state.pool;

    let aliases = match db::get_aliases(pool) {
        Ok(aliases) => aliases,
        Err(_) => vec![],
    };

    let locale = crate::handlers::language::get_user_locale(&headers);
    let title = get_translation(&state, &locale, "aliases-title").await;
    let description = get_translation(&state, &locale, "aliases-description").await;
    let add_alias = get_translation(&state, &locale, "aliases-add").await;
    let table_header_mail = get_translation(&state, &locale, "aliases-table-header-mail").await;
    let table_header_destination = get_translation(&state, &locale, "aliases-table-header-destination").await;
    let table_header_enabled = get_translation(&state, &locale, "aliases-table-header-enabled").await;
    let table_header_actions = get_translation(&state, &locale, "aliases-table-header-actions").await;
    let status_active = get_translation(&state, &locale, "status-active").await;
    let status_inactive = get_translation(&state, &locale, "status-inactive").await;
    let action_view = get_translation(&state, &locale, "action-view").await;
    let enable_alias = get_translation(&state, &locale, "aliases-enable-alias").await;
    let disable_alias = get_translation(&state, &locale, "aliases-disable-alias").await;
    let empty_title = get_translation(&state, &locale, "aliases-empty-title").await;
    let empty_description = get_translation(&state, &locale, "aliases-empty-description").await;

    let content_template = AliasListTemplate {
        title: &title,
        aliases,
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
        let locale = crate::handlers::language::get_user_locale(&headers);
        let template = BaseTemplate::with_i18n(
            get_translation(&state, &locale, "aliases-title").await,
            content,
            &state,
            &locale,
        ).await.unwrap();
        
        Html(template.render().unwrap())
    }
}

pub async fn new(State(state): State<AppState>, headers: HeaderMap, Query(prefill): Query<AliasPrefill>) -> Html<String> {
    // Check if we have a return URL from the referer header
    let return_url = headers
        .get("referer")
        .and_then(|r| r.to_str().ok())
        .filter(|r| r.contains("/domains/"))
        .map(|r| r.to_string());
    let mail = match (&prefill.alias, &prefill.domain) {
        (Some(alias), Some(domain)) => format!("{}@{}", alias, domain),
        (Some(alias), None) => alias.clone(),
        (None, Some(domain)) => format!("@{}", domain),
        (None, None) => "".to_string(),
    };
    let form = AliasForm {
        mail,
        destination: "".to_string(),
        enabled: true,
        return_url: None,
    };

    let locale = crate::handlers::language::get_user_locale(&headers);
    let title = get_translation(&state, &locale, "aliases-add-title").await;
    let edit_alias = get_translation(&state, &locale, "aliases-edit-alias").await;
    let new_alias = get_translation(&state, &locale, "aliases-new-alias").await;
    let form_error = get_translation(&state, &locale, "aliases-form-error").await;
    let mail_address = get_translation(&state, &locale, "aliases-mail-address").await;
    let destination = get_translation(&state, &locale, "aliases-destination").await;
    let placeholder_mail = get_translation(&state, &locale, "aliases-placeholder-mail").await;
    let placeholder_destination = get_translation(&state, &locale, "aliases-placeholder-destination").await;
    let tooltip_mail = get_translation(&state, &locale, "aliases-tooltip-mail").await;
    let tooltip_destination = get_translation(&state, &locale, "aliases-tooltip-destination").await;
    let active = get_translation(&state, &locale, "aliases-active").await;
    let tooltip_active = get_translation(&state, &locale, "aliases-tooltip-active").await;
    let cancel = get_translation(&state, &locale, "aliases-cancel").await;
    let update_alias = get_translation(&state, &locale, "aliases-update-alias").await;
    let create_alias = get_translation(&state, &locale, "aliases-create-alias").await;

    let content_template = AliasFormTemplate {
        title: &title,
        alias: None,
        form,
        error: None,
        return_url: return_url.as_deref(),
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
    let content = content_template.render().unwrap();

    if is_htmx_request(&headers) {
        Html(content)
    } else {
        let locale = crate::handlers::language::get_user_locale(&headers);
        let template = BaseTemplate::with_i18n(
            get_translation(&state, &locale, "aliases-add-title").await,
            content,
            &state,
            &locale,
        ).await.unwrap();
        
        Html(template.render().unwrap())
    }
}

pub async fn show(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = &state.pool;

    let alias = match db::get_alias(pool, id) {
        Ok(alias) => alias,
        Err(_) => return Html("Alias not found".to_string()),
    };

    let locale = crate::handlers::language::get_user_locale(&headers);
    let title = get_translation(&state, &locale, "aliases-show-title").await;
    let view_edit_settings = get_translation(&state, &locale, "aliases-view-edit-settings").await;
    let back_to_aliases = get_translation(&state, &locale, "aliases-back-to-aliases").await;
    let alias_information = get_translation(&state, &locale, "aliases-alias-information").await;
    let alias_details = get_translation(&state, &locale, "aliases-alias-details").await;
    let mail = get_translation(&state, &locale, "aliases-mail").await;
    let forward_to = get_translation(&state, &locale, "aliases-forward-to").await;
    let status = get_translation(&state, &locale, "aliases-status").await;
    let status_active = get_translation(&state, &locale, "status-active").await;
    let status_inactive = get_translation(&state, &locale, "status-inactive").await;
    let created = get_translation(&state, &locale, "aliases-created").await;
    let modified = get_translation(&state, &locale, "aliases-modified").await;
    let edit_alias_button = get_translation(&state, &locale, "aliases-edit-alias-button").await;
    let enable_alias_button = get_translation(&state, &locale, "aliases-enable-alias-button").await;
    let disable_alias_button = get_translation(&state, &locale, "aliases-disable-alias-button").await;
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
        let locale = crate::handlers::language::get_user_locale(&headers);
        let template = BaseTemplate::with_i18n(
            get_translation(&state, &locale, "aliases-title").await,
            content,
            &state,
            &locale,
        ).await.unwrap();
        
        Html(template.render().unwrap())
    }
}

pub async fn edit(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = &state.pool;

    let alias = match db::get_alias(pool, id) {
        Ok(alias) => alias,
        Err(_) => return Html("Alias not found".to_string()),
    };

    let form = AliasForm {
        mail: alias.mail.clone(),
        destination: alias.destination.clone(),
        enabled: alias.enabled,
        return_url: None,
    };

    let locale = crate::handlers::language::get_user_locale(&headers);
    let title = get_translation(&state, &locale, "aliases-edit-title").await;
    let edit_alias = get_translation(&state, &locale, "aliases-edit-alias").await;
    let new_alias = get_translation(&state, &locale, "aliases-new-alias").await;
    let form_error = get_translation(&state, &locale, "aliases-form-error").await;
    let mail_address = get_translation(&state, &locale, "aliases-mail-address").await;
    let destination = get_translation(&state, &locale, "aliases-destination").await;
    let placeholder_mail = get_translation(&state, &locale, "aliases-placeholder-mail").await;
    let placeholder_destination = get_translation(&state, &locale, "aliases-placeholder-destination").await;
    let tooltip_mail = get_translation(&state, &locale, "aliases-tooltip-mail").await;
    let tooltip_destination = get_translation(&state, &locale, "aliases-tooltip-destination").await;
    let active = get_translation(&state, &locale, "aliases-active").await;
    let tooltip_active = get_translation(&state, &locale, "aliases-tooltip-active").await;
    let cancel = get_translation(&state, &locale, "aliases-cancel").await;
    let update_alias = get_translation(&state, &locale, "aliases-update-alias").await;
    let create_alias = get_translation(&state, &locale, "aliases-create-alias").await;

    let content_template = AliasFormTemplate {
        title: &title,
        alias: Some(alias),
        form,
        error: None,
        return_url: None,
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
    let content = content_template.render().unwrap();

    if is_htmx_request(&headers) {
        Html(content)
    } else {
        let locale = crate::handlers::language::get_user_locale(&headers);
        let template = BaseTemplate::with_i18n(
            get_translation(&state, &locale, "aliases-title").await,
            content,
            &state,
            &locale,
        ).await.unwrap();
        
        Html(template.render().unwrap())
    }
}

pub async fn create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<AliasForm>,
) -> Html<String> {
    let pool = &state.pool;

    match db::create_alias(pool, form.clone()) {
        Ok(created_alias) => {
            // Extract domain from the created alias and redirect to domain show page
            let domain_name = created_alias.domain();
            
            // Try to find the domain by name
            match db::get_domain_by_name(pool, &domain_name) {
                Ok(domain) => {
                    // Redirect to the domain show page
                    let locale = crate::handlers::language::get_user_locale(&headers);
                    let title = get_translation(&state, &locale, "domains-show-title").await;
                    let description = get_translation(&state, &locale, "domains-description").await;
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
                    let alias_report_title = get_translation(&state, &locale, "reports-alias-report-title").await;
                    let alias_report_description = get_translation(&state, &locale, "reports-alias-report-description").await;
                    let add_alias_button = get_translation(&state, &locale, "domains-add-alias-button").await;
                    let missing_aliases_header = get_translation(&state, &locale, "reports-missing-aliases-header").await;
                    let missing_required_alias_header = get_translation(&state, &locale, "reports-missing-required-alias-header").await;
                    let missing_common_aliases_header = get_translation(&state, &locale, "reports-missing-common-aliases-header").await;
                    let add_missing_required_alias_button = get_translation(&state, &locale, "reports-add-missing-required-alias-button").await;
                    let add_common_alias_button = get_translation(&state, &locale, "reports-add-common-alias-button").await;
                    let catch_all_header = get_translation(&state, &locale, "reports-catch-all-header").await;
                    let add_catch_all_button = get_translation(&state, &locale, "reports-add-catch-all-button").await;
                    let no_catch_all_message = get_translation(&state, &locale, "reports-no-catch-all-message").await;
                    let destination_header = get_translation(&state, &locale, "reports-destination-header").await;
                    let existing_aliases_header = get_translation(&state, &locale, "reports-existing-aliases-header").await;
                    let no_required_aliases = get_translation(&state, &locale, "reports-no-required-aliases").await;
                    let mail_header = get_translation(&state, &locale, "reports-mail-header").await;
                    let enabled_header = get_translation(&state, &locale, "reports-enabled-header").await;
                    let actions_header = get_translation(&state, &locale, "reports-actions-header").await;
                    let no_missing_aliases = get_translation(&state, &locale, "reports-no-missing-aliases").await;

                    // Get alias report for the domain
                    let alias_report = match db::get_domain_alias_report(pool, &domain.domain) {
                        Ok(report) => Some(report),
                        Err(e) => {
                            eprintln!("Error getting domain alias report: {:?}", e);
                            None
                        }
                    };

                    // Get existing aliases for the domain
                    let existing_aliases = match db::get_aliases_for_domain(pool, &domain.domain) {
                        Ok(aliases) => aliases,
                        Err(e) => {
                            eprintln!("Error getting aliases for domain: {:?}", e);
                            vec![]
                        }
                    };

                    let view_edit_settings = get_translation(&state, &locale, "domains-view-edit-settings").await;
                    let back_to_domains = get_translation(&state, &locale, "domains-back-to-domains").await;
                    let domain_information = get_translation(&state, &locale, "domains-domain-information").await;
                    let domain_details = get_translation(&state, &locale, "domains-domain-details").await;
                    let domain_name = get_translation(&state, &locale, "domains-domain-name").await;
                    let transport = get_translation(&state, &locale, "domains-transport").await;
                    let required_aliases_header = get_translation(&state, &locale, "reports-required-aliases-header").await;
                    let status_header = get_translation(&state, &locale, "reports-status-header").await;

                    let action_view = get_translation(&state, &locale, "action-view").await;
                    let enable_alias = get_translation(&state, &locale, "aliases-enable-alias").await;
                    let disable_alias = get_translation(&state, &locale, "aliases-disable-alias").await;

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
                        alias_report: alias_report.as_ref(),
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
                        let locale = crate::handlers::language::get_user_locale(&headers);
                        let template = BaseTemplate::with_i18n(
                            get_translation(&state, &locale, "domains-show-title").await,
                            content,
                            &state,
                            &locale,
                        ).await.unwrap();
                        Html(template.render().unwrap())
                    }
                }
                Err(e) => {
                    eprintln!("Error finding domain by name: {:?}", e);
                    // Fallback to aliases list if domain not found
                    let aliases = match db::get_aliases(pool) {
                        Ok(aliases) => aliases,
                        Err(e) => {
                            eprintln!("Error getting aliases: {:?}", e);
                            vec![]
                        }
                    };
                    let locale = crate::handlers::language::get_user_locale(&headers);
                    let title = get_translation(&state, &locale, "aliases-title").await;
                    let description = get_translation(&state, &locale, "aliases-description").await;
                    let add_alias = get_translation(&state, &locale, "aliases-add").await;
                    let table_header_mail = get_translation(&state, &locale, "aliases-table-header-mail").await;
                    let table_header_destination = get_translation(&state, &locale, "aliases-table-header-destination").await;
                    let table_header_enabled = get_translation(&state, &locale, "aliases-table-header-enabled").await;
                    let table_header_actions = get_translation(&state, &locale, "aliases-table-header-actions").await;
                    let status_active = get_translation(&state, &locale, "status-active").await;
                    let status_inactive = get_translation(&state, &locale, "status-inactive").await;
                    let action_view = get_translation(&state, &locale, "action-view").await;
                    let enable_alias = get_translation(&state, &locale, "aliases-enable-alias").await;
                    let disable_alias = get_translation(&state, &locale, "aliases-disable-alias").await;
                    let empty_title = get_translation(&state, &locale, "aliases-empty-title").await;
                    let empty_description = get_translation(&state, &locale, "aliases-empty-description").await;
                    let content_template = AliasListTemplate {
                        title: &title,
                        aliases,
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
                        let locale = crate::handlers::language::get_user_locale(&headers);
                        let template = BaseTemplate::with_i18n(
                            get_translation(&state, &locale, "aliases-title").await,
                            content,
                            &state,
                            &locale,
                        ).await.unwrap();
                        Html(template.render().unwrap())
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Error creating alias: {:?}", e);
            
            // Handle specific database errors with user-friendly messages
            let error_message = match e {
                diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::UniqueViolation,
                    _,
                ) => format!("An alias with the email '{}' already exists.", form.mail),
                diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::CheckViolation,
                    _,
                ) => "The alias data does not meet the required constraints. Please check your input.".to_string(),
                _ => "An unexpected error occurred while creating the alias. Please try again.".to_string(),
            };

            // Return to form with error message
            let locale = crate::handlers::language::get_user_locale(&headers);
            let title = get_translation(&state, &locale, "aliases-add-title").await;
            let edit_alias = get_translation(&state, &locale, "aliases-edit-alias").await;
            let new_alias = get_translation(&state, &locale, "aliases-new-alias").await;
            let form_error = get_translation(&state, &locale, "aliases-form-error").await;
            let mail_address = get_translation(&state, &locale, "aliases-mail-address").await;
            let destination = get_translation(&state, &locale, "aliases-destination").await;
            let placeholder_mail = get_translation(&state, &locale, "aliases-placeholder-mail").await;
            let placeholder_destination = get_translation(&state, &locale, "aliases-placeholder-destination").await;
            let tooltip_mail = get_translation(&state, &locale, "aliases-tooltip-mail").await;
            let tooltip_destination = get_translation(&state, &locale, "aliases-tooltip-destination").await;
            let tooltip_active = get_translation(&state, &locale, "aliases-tooltip-active").await;
            let cancel = get_translation(&state, &locale, "aliases-cancel").await;
            let update_alias = get_translation(&state, &locale, "aliases-update-alias").await;
            let create_alias = get_translation(&state, &locale, "aliases-create-alias").await;
            
            let active = get_translation(&state, &locale, "aliases-active").await;
            let content_template = AliasFormTemplate {
                title: &title,
                alias: None,
                form: form.clone(),
                error: Some(error_message),
                return_url: None,
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
            let content = content_template.render().unwrap();

            if is_htmx_request(&headers) {
                Html(content)
            } else {
                let locale = crate::handlers::language::get_user_locale(&headers);
                let template = BaseTemplate::with_i18n(
                    get_translation(&state, &locale, "aliases-add-title").await,
                    content,
                    &state,
                    &locale,
                ).await.unwrap();
                Html(template.render().unwrap())
            }
        }
    }
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
    Form(form): Form<AliasForm>,
) -> Html<String> {
    let pool = &state.pool;

    match db::update_alias(pool, id, form.clone()) {
        Ok(_) => {
            let alias = match db::get_alias(pool, id) {
                Ok(alias) => alias,
                Err(_) => return Html("Alias not found".to_string()),
            };
            
            let locale = crate::handlers::language::get_user_locale(&headers);
            let title = get_translation(&state, &locale, "aliases-show-title").await;
            let view_edit_settings = get_translation(&state, &locale, "aliases-view-edit-settings").await;
            let back_to_aliases = get_translation(&state, &locale, "aliases-back-to-aliases").await;
            let alias_information = get_translation(&state, &locale, "aliases-alias-information").await;
            let alias_details = get_translation(&state, &locale, "aliases-alias-details").await;
            let mail = get_translation(&state, &locale, "aliases-mail").await;
            let forward_to = get_translation(&state, &locale, "aliases-forward-to").await;
            let status = get_translation(&state, &locale, "aliases-status").await;
            let status_active = get_translation(&state, &locale, "status-active").await;
            let status_inactive = get_translation(&state, &locale, "status-inactive").await;
            let created = get_translation(&state, &locale, "aliases-created").await;
            let modified = get_translation(&state, &locale, "aliases-modified").await;
            let edit_alias_button = get_translation(&state, &locale, "aliases-edit-alias-button").await;
            let enable_alias_button = get_translation(&state, &locale, "aliases-enable-alias-button").await;
            let disable_alias_button = get_translation(&state, &locale, "aliases-disable-alias-button").await;
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
                let locale = crate::handlers::language::get_user_locale(&headers);
                let template = BaseTemplate::with_i18n(
                    get_translation(&state, &locale, "aliases-show-title").await,
                    content,
                    &state,
                    &locale,
                ).await.unwrap();
                Html(template.render().unwrap())
            }
        }
        Err(e) => {
            eprintln!("Error updating alias: {:?}", e);
            
            // Handle specific database errors with user-friendly messages
            let error_message = match e {
                diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::ForeignKeyViolation,
                    _,
                ) => "The domain does not exist. Please create the domain first before adding aliases.".to_string(),
                diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::UniqueViolation,
                    _,
                ) => format!("An alias with the email '{}' already exists.", form.mail),
                diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::CheckViolation,
                    _,
                ) => "The alias data does not meet the required constraints. Please check your input.".to_string(),
                _ => "An unexpected error occurred while updating the alias. Please try again.".to_string(),
            };

            // Get the original alias for the form
            let original_alias = match db::get_alias(pool, id) {
                Ok(alias) => Some(alias),
                Err(_) => None,
            };

            // Return to form with error message
            let locale = crate::handlers::language::get_user_locale(&headers);
            let title = get_translation(&state, &locale, "aliases-edit-title").await;
            let edit_alias = get_translation(&state, &locale, "aliases-edit-alias").await;
            let new_alias = get_translation(&state, &locale, "aliases-new-alias").await;
            let form_error = get_translation(&state, &locale, "aliases-form-error").await;
            let mail_address = get_translation(&state, &locale, "aliases-mail-address").await;
            let destination = get_translation(&state, &locale, "aliases-destination").await;
            let placeholder_mail = get_translation(&state, &locale, "aliases-placeholder-mail").await;
            let placeholder_destination = get_translation(&state, &locale, "aliases-placeholder-destination").await;
            let tooltip_mail = get_translation(&state, &locale, "aliases-tooltip-mail").await;
            let tooltip_destination = get_translation(&state, &locale, "aliases-tooltip-destination").await;
            let tooltip_active = get_translation(&state, &locale, "aliases-tooltip-active").await;
            let cancel = get_translation(&state, &locale, "aliases-cancel").await;
            let update_alias = get_translation(&state, &locale, "aliases-update-alias").await;
            let create_alias = get_translation(&state, &locale, "aliases-create-alias").await;
            
            let active = get_translation(&state, &locale, "aliases-active").await;
            let content_template = AliasFormTemplate {
                title: &title,
                alias: original_alias,
                form: form.clone(),
                error: Some(error_message),
                return_url: None,
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
            let content = content_template.render().unwrap();

            if is_htmx_request(&headers) {
                Html(content)
            } else {
                let locale = crate::handlers::language::get_user_locale(&headers);
                let template = BaseTemplate::with_i18n(
                    get_translation(&state, &locale, "aliases-edit-title").await,
                    content,
                    &state,
                    &locale,
                ).await.unwrap();
                Html(template.render().unwrap())
            }
        }
    }
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = &state.pool;

    match db::delete_alias(pool, id) {
        Ok(_) => {
            let aliases = match db::get_aliases(pool) {
                Ok(aliases) => aliases,
                Err(e) => {
                    eprintln!("Error getting aliases: {:?}", e);
                    vec![]
                }
            };
            let locale = crate::handlers::language::get_user_locale(&headers);
            let title = get_translation(&state, &locale, "aliases-title").await;
            let description = get_translation(&state, &locale, "aliases-description").await;
            let add_alias = get_translation(&state, &locale, "aliases-add").await;
            let table_header_mail = get_translation(&state, &locale, "aliases-table-header-mail").await;
            let table_header_destination = get_translation(&state, &locale, "aliases-table-header-destination").await;
            let table_header_enabled = get_translation(&state, &locale, "aliases-table-header-enabled").await;
            let table_header_actions = get_translation(&state, &locale, "aliases-table-header-actions").await;
            let status_active = get_translation(&state, &locale, "status-active").await;
            let status_inactive = get_translation(&state, &locale, "status-inactive").await;
            let action_view = get_translation(&state, &locale, "action-view").await;
            let enable_alias = get_translation(&state, &locale, "aliases-enable-alias").await;
            let disable_alias = get_translation(&state, &locale, "aliases-disable-alias").await;
            let empty_title = get_translation(&state, &locale, "aliases-empty-title").await;
            let empty_description = get_translation(&state, &locale, "aliases-empty-description").await;
            let content_template = AliasListTemplate {
                title: &title,
                aliases,
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
                let locale = crate::handlers::language::get_user_locale(&headers);
                let template = BaseTemplate::with_i18n(
                    get_translation(&state, &locale, "aliases-title").await,
                    content,
                    &state,
                    &locale,
                ).await.unwrap();
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
    let pool = &state.pool;

    match db::toggle_alias_enabled(pool, id) {
        Ok(_) => {
            let alias = match db::get_alias(pool, id) {
                Ok(alias) => alias,
                Err(_) => return Html("Alias not found".to_string()),
            };
            
            let locale = crate::handlers::language::get_user_locale(&headers);
            let title = get_translation(&state, &locale, "aliases-show-title").await;
            let view_edit_settings = get_translation(&state, &locale, "aliases-view-edit-settings").await;
            let back_to_aliases = get_translation(&state, &locale, "aliases-back-to-aliases").await;
            let alias_information = get_translation(&state, &locale, "aliases-alias-information").await;
            let alias_details = get_translation(&state, &locale, "aliases-alias-details").await;
            let mail = get_translation(&state, &locale, "aliases-mail").await;
            let forward_to = get_translation(&state, &locale, "aliases-forward-to").await;
            let status = get_translation(&state, &locale, "aliases-status").await;
            let status_active = get_translation(&state, &locale, "status-active").await;
            let status_inactive = get_translation(&state, &locale, "status-inactive").await;
            let created = get_translation(&state, &locale, "aliases-created").await;
            let modified = get_translation(&state, &locale, "aliases-modified").await;
            let edit_alias_button = get_translation(&state, &locale, "aliases-edit-alias-button").await;
            let enable_alias_button = get_translation(&state, &locale, "aliases-enable-alias-button").await;
            let disable_alias_button = get_translation(&state, &locale, "aliases-disable-alias-button").await;
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
                let locale = crate::handlers::language::get_user_locale(&headers);
                let template = BaseTemplate::with_i18n(
                    get_translation(&state, &locale, "aliases-show-title").await,
                    content,
                    &state,
                    &locale,
                ).await.unwrap();
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
    let pool = &state.pool;
    match db::toggle_alias_enabled(pool, id) {
        Ok(_) => {
            let aliases = match db::get_aliases(pool) {
                Ok(aliases) => aliases,
                Err(e) => {
                    eprintln!("Error getting aliases: {:?}", e);
                    vec![]
                }
            };
            let locale = crate::handlers::language::get_user_locale(&headers);
            let title = get_translation(&state, &locale, "aliases-title").await;
            let description = get_translation(&state, &locale, "aliases-description").await;
            let add_alias = get_translation(&state, &locale, "aliases-add").await;
            let table_header_mail = get_translation(&state, &locale, "aliases-table-header-mail").await;
            let table_header_destination = get_translation(&state, &locale, "aliases-table-header-destination").await;
            let table_header_enabled = get_translation(&state, &locale, "aliases-table-header-enabled").await;
            let table_header_actions = get_translation(&state, &locale, "aliases-table-header-actions").await;
            let status_active = get_translation(&state, &locale, "status-active").await;
            let status_inactive = get_translation(&state, &locale, "status-inactive").await;
            let action_view = get_translation(&state, &locale, "action-view").await;
            let enable_alias = get_translation(&state, &locale, "aliases-enable-alias").await;
            let disable_alias = get_translation(&state, &locale, "aliases-disable-alias").await;
            let empty_title = get_translation(&state, &locale, "aliases-empty-title").await;
            let empty_description = get_translation(&state, &locale, "aliases-empty-description").await;
            let content_template = AliasListTemplate {
                title: &title,
                aliases,
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
                let locale = crate::handlers::language::get_user_locale(&headers);
                let template = BaseTemplate::with_i18n(
                    get_translation(&state, &locale, "aliases-title").await,
                    content,
                    &state,
                    &locale,
                ).await.unwrap();
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
    let pool = &state.pool;
    match db::toggle_alias_enabled(pool, id) {
        Ok(_) => {
            let alias = match db::get_alias(pool, id) {
                Ok(alias) => alias,
                Err(_) => return Html("Alias not found".to_string()),
            };
            
            let locale = crate::handlers::language::get_user_locale(&headers);
            let title = get_translation(&state, &locale, "aliases-show-title").await;
            let view_edit_settings = get_translation(&state, &locale, "aliases-view-edit-settings").await;
            let back_to_aliases = get_translation(&state, &locale, "aliases-back-to-aliases").await;
            let alias_information = get_translation(&state, &locale, "aliases-alias-information").await;
            let alias_details = get_translation(&state, &locale, "aliases-alias-details").await;
            let mail = get_translation(&state, &locale, "aliases-mail").await;
            let forward_to = get_translation(&state, &locale, "aliases-forward-to").await;
            let status = get_translation(&state, &locale, "aliases-status").await;
            let status_active = get_translation(&state, &locale, "status-active").await;
            let status_inactive = get_translation(&state, &locale, "status-inactive").await;
            let created = get_translation(&state, &locale, "aliases-created").await;
            let modified = get_translation(&state, &locale, "aliases-modified").await;
            let edit_alias_button = get_translation(&state, &locale, "aliases-edit-alias-button").await;
            let enable_alias_button = get_translation(&state, &locale, "aliases-enable-alias-button").await;
            let disable_alias_button = get_translation(&state, &locale, "aliases-disable-alias-button").await;
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
                let locale = crate::handlers::language::get_user_locale(&headers);
                let template = BaseTemplate::with_i18n(
                    get_translation(&state, &locale, "aliases-show-title").await,
                    content,
                    &state,
                    &locale,
                ).await.unwrap();
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
    let pool = &state.pool;
    
    // First toggle the alias
    match db::toggle_alias_enabled(pool, id) {
        Ok(_) => {
            // Get the alias to find its domain
            let alias = match db::get_alias(pool, id) {
                Ok(alias) => alias,
                Err(_) => return Html("Alias not found".to_string()),
            };
            
            // Extract domain from alias mail (e.g., "user@domain.com" -> "domain.com")
            let domain_name = alias.mail.split('@').last().unwrap_or("");
            
            // Find the domain by name
            let domain = match db::get_domain_by_name(pool, domain_name) {
                Ok(domain) => domain,
                Err(_) => return Html("Domain not found".to_string()),
            };
            
            // Now render the domain show page with updated alias data
            let locale = crate::handlers::language::get_user_locale(&headers);
            let title = get_translation(&state, &locale, "domains-title").await;
            let view_edit_settings = get_translation(&state, &locale, "domains-view-edit-settings").await;
            let back_to_domains = get_translation(&state, &locale, "domains-back-to-domains").await;
            let domain_information = get_translation(&state, &locale, "domains-domain-information").await;
            let domain_details = get_translation(&state, &locale, "domains-domain-details").await;
            let domain_name_label = get_translation(&state, &locale, "domains-domain-name").await;
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
            
            // Get alias report for the domain
            let alias_report = match db::get_domain_alias_report(pool, &domain.domain) {
                Ok(report) => Some(report),
                Err(_) => None,
            };
            
            // Get existing aliases for the domain
            let existing_aliases = match db::get_aliases_for_domain(pool, &domain.domain) {
                Ok(aliases) => aliases,
                Err(_) => vec![],
            };
            
            let catch_all_header = get_translation(&state, &locale, "reports-catch-all-header").await;
            let destination_header = get_translation(&state, &locale, "reports-destination-header").await;
            let required_aliases_header = get_translation(&state, &locale, "reports-required-aliases-header").await;
            let missing_aliases_header = get_translation(&state, &locale, "reports-missing-aliases-header").await;
            let missing_required_alias_header = get_translation(&state, &locale, "reports-missing-required-aliases-header").await;
            let missing_common_aliases_header = get_translation(&state, &locale, "reports-missing-common-aliases-header").await;
            let mail_header = get_translation(&state, &locale, "reports-mail-header").await;
            let status_header = get_translation(&state, &locale, "reports-status-header").await;
            let enabled_header = get_translation(&state, &locale, "reports-enabled-header").await;
            let actions_header = get_translation(&state, &locale, "reports-actions-header").await;
            let no_required_aliases = get_translation(&state, &locale, "reports-no-required-aliases").await;
            let no_missing_aliases = get_translation(&state, &locale, "reports-no-missing-aliases").await;
            let alias_report_title = get_translation(&state, &locale, "domains-alias-report-title").await;
            let alias_report_description = get_translation(&state, &locale, "domains-alias-report-description").await;
            let existing_aliases_header = get_translation(&state, &locale, "domains-existing-aliases-header").await;
            let add_missing_required_alias_button = get_translation(&state, &locale, "reports-add-missing-required-alias-button").await;
            let add_common_alias_button = get_translation(&state, &locale, "reports-add-common-alias-button").await;
            let add_catch_all_button = get_translation(&state, &locale, "reports-add-catch-all-button").await;
            let add_alias_button = get_translation(&state, &locale, "domains-add-alias-button").await;
            let no_catch_all_message = get_translation(&state, &locale, "domains-no-catch-all-message").await;
            
            let action_view = get_translation(&state, &locale, "action-view").await;
            let enable_alias = get_translation(&state, &locale, "aliases-enable-alias").await;
            let disable_alias = get_translation(&state, &locale, "aliases-disable-alias").await;

            let content_template = crate::templates::domains::DomainShowTemplate {
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
                alias_report: alias_report.as_ref(),
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
                let locale = crate::handlers::language::get_user_locale(&headers);
                let template = BaseTemplate::with_i18n(
                    get_translation(&state, &locale, "domains-title").await,
                    content,
                    &state,
                    &locale,
                ).await.unwrap();
                Html(template.render().unwrap())
            }
        }
        Err(_) => Html("Error toggling alias status".to_string()),
    }
}
