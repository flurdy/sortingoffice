use crate::templates::domains::*;
use crate::templates::layout::BaseTemplate;
use crate::{db, models::*, AppState, i18n::get_translation};
use askama::Template;
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::Html,
    Form,
};

pub async fn list(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    let pool = &state.pool;
    let locale = crate::handlers::language::get_user_locale(&headers);

    tracing::debug!("Handling domains list request");
    let domains = match db::get_domains(pool) {
        Ok(domains) => {
            tracing::info!("Successfully retrieved {} domains", domains.len());
            domains
        },
        Err(e) => {
            tracing::error!("Failed to retrieve domains: {:?}", e);
            vec![]
        },
    };

    tracing::debug!("Rendering template with {} domains", domains.len());
    
    // Get all translations
    let title = get_translation(&state, &locale, "domains-title").await;
    let description = get_translation(&state, &locale, "domains-description").await;
    let add_domain = get_translation(&state, &locale, "domains-add").await;
    let table_header_domain = get_translation(&state, &locale, "domains-table-header-domain").await;
    let table_header_status = get_translation(&state, &locale, "domains-table-header-status").await;
    let table_header_actions = get_translation(&state, &locale, "domains-table-header-actions").await;
    let table_header_transport = get_translation(&state, &locale, "domains-transport").await;
    let status_active = get_translation(&state, &locale, "status-active").await;
    let status_inactive = get_translation(&state, &locale, "status-inactive").await;
    let action_view = get_translation(&state, &locale, "action-view").await;
    let action_enable = get_translation(&state, &locale, "action-enable").await;
    let action_disable = get_translation(&state, &locale, "action-disable").await;
    let empty_title = get_translation(&state, &locale, "domains-empty-title").await;
    let empty_description = get_translation(&state, &locale, "domains-empty-description").await;
    
    let content_template = DomainListTemplate {
        title: &title,
        description: &description,
        add_domain: &add_domain,
        table_header_domain: &table_header_domain,
        table_header_transport: &table_header_transport,
        table_header_status: &table_header_status,
        table_header_actions: &table_header_actions,
        status_active: &status_active,
        status_inactive: &status_inactive,
        action_view: &action_view,
        action_enable: &action_enable,
        action_disable: &action_disable,
        empty_title: &empty_title,
        empty_description: &empty_description,
        domains,
    };
    let content = match content_template.render() {
        Ok(content) => {
            tracing::debug!("Template rendered successfully, content length: {}", content.len());
            content
        },
        Err(e) => {
            tracing::error!("Failed to render template: {:?}", e);
            return Html("Error rendering template".to_string());
        }
    };

    let template = BaseTemplate::with_i18n(
        get_translation(&state, &locale, "domains-title").await,
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
    
    let content = content_template.render().unwrap();
    let template = BaseTemplate::with_i18n(
        get_translation(&state, &locale, "domains-add-title").await,
        content,
        &state,
        &locale,
    ).await.unwrap();
    
    Html(template.render().unwrap())
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
            let table_header_status = get_translation(&state, &locale, "domains-table-header-status").await;
            let table_header_actions = get_translation(&state, &locale, "domains-table-header-actions").await;
            let table_header_transport = get_translation(&state, &locale, "domains-transport").await;
            let status_active = get_translation(&state, &locale, "status-active").await;
            let status_inactive = get_translation(&state, &locale, "status-inactive").await;
            let action_view = get_translation(&state, &locale, "action-view").await;
            let action_enable = get_translation(&state, &locale, "action-enable").await;
            let action_disable = get_translation(&state, &locale, "action-disable").await;
            let empty_title = get_translation(&state, &locale, "domains-empty-title").await;
            let empty_description = get_translation(&state, &locale, "domains-empty-description").await;
            
            let template = DomainListTemplate {
                title: &title,
                description: &description,
                add_domain: &add_domain,
                table_header_domain: &table_header_domain,
                table_header_transport: &table_header_transport,
                table_header_status: &table_header_status,
                table_header_actions: &table_header_actions,
                status_active: &status_active,
                status_inactive: &status_inactive,
                action_view: &action_view,
                action_enable: &action_enable,
                action_disable: &action_disable,
                empty_title: &empty_title,
                empty_description: &empty_description,
                domains,
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
            let table_header_status = get_translation(&state, &locale, "domains-table-header-status").await;
            let table_header_actions = get_translation(&state, &locale, "domains-table-header-actions").await;
            let table_header_transport = get_translation(&state, &locale, "domains-transport").await;
            let status_active = get_translation(&state, &locale, "status-active").await;
            let status_inactive = get_translation(&state, &locale, "status-inactive").await;
            let action_view = get_translation(&state, &locale, "action-view").await;
            let action_enable = get_translation(&state, &locale, "action-enable").await;
            let action_disable = get_translation(&state, &locale, "action-disable").await;
            let empty_title = get_translation(&state, &locale, "domains-empty-title").await;
            let empty_description = get_translation(&state, &locale, "domains-empty-description").await;
            
            let template = DomainListTemplate {
                title: &title,
                description: &description,
                add_domain: &add_domain,
                table_header_domain: &table_header_domain,
                table_header_transport: &table_header_transport,
                table_header_status: &table_header_status,
                table_header_actions: &table_header_actions,
                status_active: &status_active,
                status_inactive: &status_inactive,
                action_view: &action_view,
                action_enable: &action_enable,
                action_disable: &action_disable,
                empty_title: &empty_title,
                empty_description: &empty_description,
                domains,
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
            let table_header_status = get_translation(&state, &locale, "domains-table-header-status").await;
            let table_header_actions = get_translation(&state, &locale, "domains-table-header-actions").await;
            let table_header_transport = get_translation(&state, &locale, "domains-transport").await;
            let status_active = get_translation(&state, &locale, "status-active").await;
            let status_inactive = get_translation(&state, &locale, "status-inactive").await;
            let action_view = get_translation(&state, &locale, "action-view").await;
            let action_enable = get_translation(&state, &locale, "action-enable").await;
            let action_disable = get_translation(&state, &locale, "action-disable").await;
            let empty_title = get_translation(&state, &locale, "domains-empty-title").await;
            let empty_description = get_translation(&state, &locale, "domains-empty-description").await;
            
            let template = DomainListTemplate {
                title: &title,
                description: &description,
                add_domain: &add_domain,
                table_header_domain: &table_header_domain,
                table_header_transport: &table_header_transport,
                table_header_status: &table_header_status,
                table_header_actions: &table_header_actions,
                status_active: &status_active,
                status_inactive: &status_inactive,
                action_view: &action_view,
                action_enable: &action_enable,
                action_disable: &action_disable,
                empty_title: &empty_title,
                empty_description: &empty_description,
                domains,
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
            };
            Html(content_template.render().unwrap())
        }
        Err(_) => Html("Error toggling domain status".to_string()),
    }
}
