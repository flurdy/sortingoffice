use crate::templates::clients::*;
use crate::templates::layout::BaseTemplate;
use crate::{db, models::*, AppState, i18n::get_translation};
use askama::Template;
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{Html, Redirect},
    Form,
};
use tracing::{info, warn};

fn is_htmx_request(headers: &HeaderMap) -> bool {
    headers.get("HX-Request").map_or(false, |v| v == "true")
}

pub async fn list_clients(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = &state.pool;
    let locale = crate::handlers::language::get_user_locale(&headers);

    info!("Handling clients list request");

    let clients = match db::get_clients(pool) {
        Ok(clients) => {
            info!("Successfully retrieved {} clients", clients.len());
            clients
        },
        Err(e) => {
            warn!("Failed to retrieve clients: {:?}", e);
            vec![]
        },
    };

    // Get all translations
    let title = get_translation(&state, &locale, "clients-title").await;
    let description = get_translation(&state, &locale, "clients-description").await;
    let add_client = get_translation(&state, &locale, "clients-add").await;
    let table_header_client = get_translation(&state, &locale, "clients-table-header-client").await;
    let table_header_status = get_translation(&state, &locale, "clients-table-header-status").await;
    let table_header_created = get_translation(&state, &locale, "clients-table-header-created").await;
    let table_header_actions = get_translation(&state, &locale, "clients-table-header-actions").await;
    let status_allowed = get_translation(&state, &locale, "clients-status-allowed").await;
    let status_blocked = get_translation(&state, &locale, "clients-status-blocked").await;
    let action_view = get_translation(&state, &locale, "action-view").await;
    let action_edit = get_translation(&state, &locale, "action-edit").await;
    let action_delete = get_translation(&state, &locale, "action-delete").await;
    let delete_confirm = get_translation(&state, &locale, "clients-delete-confirm").await;
    let empty_title = get_translation(&state, &locale, "clients-empty-title").await;
    let empty_description = get_translation(&state, &locale, "clients-empty-description").await;

    let content_template = ClientsListTemplate {
        title: &title,
        description: &description,
        add_client: &add_client,
        table_header_client: &table_header_client,
        table_header_status: &table_header_status,
        table_header_created: &table_header_created,
        table_header_actions: &table_header_actions,
        status_allowed: &status_allowed,
        status_blocked: &status_blocked,
        action_view: &action_view,
        action_edit: &action_edit,
        action_delete: &action_delete,
        delete_confirm: &delete_confirm,
        empty_title: &empty_title,
        empty_description: &empty_description,
        clients,
    };

    let content = match content_template.render() {
        Ok(content) => {
            info!("Template rendered successfully, content length: {}", content.len());
            content
        },
        Err(e) => {
            warn!("Failed to render template: {:?}", e);
            return Html("Error rendering template".to_string());
        }
    };

    if is_htmx_request(&headers) {
        Html(content)
    } else {
        let template = BaseTemplate::with_i18n(
            get_translation(&state, &locale, "clients-title").await,
            content,
            &state,
            &locale,
        ).await.unwrap();
        
        Html(template.render().unwrap())
    }
}

pub async fn show_client(
    State(state): State<AppState>,
    Path(client_id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = &state.pool;
    let locale = crate::handlers::language::get_user_locale(&headers);

    info!("Handling client show request for ID: {}", client_id);

    let client = match db::get_client(pool, client_id) {
        Ok(client) => client,
        Err(_) => {
            let not_found_msg = get_translation(&state, &locale, "clients-not-found").await;
            return Html(not_found_msg);
        }
    };

    info!("Successfully retrieved client: {}", client.client);

    let title = get_translation(&state, &locale, "clients-title").await;
    let view_edit_settings = get_translation(&state, &locale, "clients-view-edit-settings").await;
    let back_to_clients = get_translation(&state, &locale, "clients-back-to-clients").await;
    let client_information = get_translation(&state, &locale, "clients-info-title").await;
    let client_details = get_translation(&state, &locale, "clients-info-description").await;
    let client_name = get_translation(&state, &locale, "clients-field-client").await;
    let status = get_translation(&state, &locale, "clients-field-status").await;
    let status_allowed = get_translation(&state, &locale, "clients-status-allowed").await;
    let status_blocked = get_translation(&state, &locale, "clients-status-blocked").await;
    let created = get_translation(&state, &locale, "clients-field-created").await;
    let updated = get_translation(&state, &locale, "clients-field-updated").await;
    let edit_client = get_translation(&state, &locale, "clients-action-edit").await;
    let delete_client = get_translation(&state, &locale, "clients-action-delete").await;
    let delete_confirm = get_translation(&state, &locale, "clients-delete-confirm").await;

    let content_template = ClientShowTemplate {
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
        created: &created,
        updated: &updated,
        edit_client: &edit_client,
        delete_client: &delete_client,
        delete_confirm: &delete_confirm,
    };

    let content = match content_template.render() {
        Ok(content) => content,
        Err(e) => {
            warn!("Failed to render template: {:?}", e);
            return Html("Error rendering template".to_string());
        }
    };

    if is_htmx_request(&headers) {
        Html(content)
    } else {
        let template = BaseTemplate::with_i18n(
            get_translation(&state, &locale, "clients-title").await,
            content,
            &state,
            &locale,
        ).await.unwrap();
        
        Html(template.render().unwrap())
    }
}

pub async fn create_client_form(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Html<String> {
    let locale = crate::handlers::language::get_user_locale(&headers);

    info!("Handling client create form request");

    let title = get_translation(&state, &locale, "clients-add-title").await;
    let form_error = get_translation(&state, &locale, "form-error").await;
    let form_client = get_translation(&state, &locale, "clients-field-client").await;
    let form_status = get_translation(&state, &locale, "clients-field-status").await;
    let form_cancel = get_translation(&state, &locale, "form-cancel").await;
    let form_create_client = get_translation(&state, &locale, "clients-action-save").await;
    let form_update_client = get_translation(&state, &locale, "clients-action-save").await;
    let form_placeholder_client = get_translation(&state, &locale, "clients-placeholder-client").await;
    let form_tooltip_client = get_translation(&state, &locale, "clients-field-client-help").await;
    let form_tooltip_status = get_translation(&state, &locale, "clients-field-status-help").await;

    let content_template = ClientFormTemplate {
        title: &title,
        client: None,
        form_error: &form_error,
        form_client: &form_client,
        form_status: &form_status,
        form_cancel: &form_cancel,
        form_create_client: &form_create_client,
        form_update_client: &form_update_client,
        form_placeholder_client: &form_placeholder_client,
        form_tooltip_client: &form_tooltip_client,
        form_tooltip_status: &form_tooltip_status,
    };

    let content = match content_template.render() {
        Ok(content) => content,
        Err(e) => {
            warn!("Failed to render template: {:?}", e);
            return Html("Error rendering template".to_string());
        }
    };

    if is_htmx_request(&headers) {
        Html(content)
    } else {
        let template = BaseTemplate::with_i18n(
            get_translation(&state, &locale, "clients-add-title").await,
            content,
            &state,
            &locale,
        ).await.unwrap();
        
        Html(template.render().unwrap())
    }
}

pub async fn edit_client_form(
    State(state): State<AppState>,
    Path(client_id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = &state.pool;
    let locale = crate::handlers::language::get_user_locale(&headers);

    info!("Handling client edit form request for ID: {}", client_id);

    let client = match db::get_client(pool, client_id) {
        Ok(client) => client,
        Err(_) => {
            let not_found_msg = get_translation(&state, &locale, "clients-not-found").await;
            return Html(not_found_msg);
        }
    };

    let title = get_translation(&state, &locale, "clients-edit-title").await;
    let form_error = get_translation(&state, &locale, "form-error").await;
    let form_client = get_translation(&state, &locale, "clients-field-client").await;
    let form_status = get_translation(&state, &locale, "clients-field-status").await;
    let form_cancel = get_translation(&state, &locale, "form-cancel").await;
    let form_create_client = get_translation(&state, &locale, "clients-action-save").await;
    let form_update_client = get_translation(&state, &locale, "clients-action-save").await;
    let form_placeholder_client = get_translation(&state, &locale, "clients-placeholder-client").await;
    let form_tooltip_client = get_translation(&state, &locale, "clients-field-client-help").await;
    let form_tooltip_status = get_translation(&state, &locale, "clients-field-status-help").await;

    let content_template = ClientFormTemplate {
        title: &title,
        client: Some(client),
        form_error: &form_error,
        form_client: &form_client,
        form_status: &form_status,
        form_cancel: &form_cancel,
        form_create_client: &form_create_client,
        form_update_client: &form_update_client,
        form_placeholder_client: &form_placeholder_client,
        form_tooltip_client: &form_tooltip_client,
        form_tooltip_status: &form_tooltip_status,
    };

    let content = match content_template.render() {
        Ok(content) => content,
        Err(e) => {
            warn!("Failed to render template: {:?}", e);
            return Html("Error rendering template".to_string());
        }
    };

    if is_htmx_request(&headers) {
        Html(content)
    } else {
        let template = BaseTemplate::with_i18n(
            get_translation(&state, &locale, "clients-edit-title").await,
            content,
            &state,
            &locale,
        ).await.unwrap();
        
        Html(template.render().unwrap())
    }
}

pub async fn create_client(
    State(state): State<AppState>,
    Form(client_data): Form<ClientForm>,
) -> Result<Redirect, (StatusCode, String)> {
    info!("Handling client creation request");

    let client = db::create_client(&state.pool, client_data).map_err(|e| {
        warn!("Failed to create client: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create client".to_string())
    })?;

    info!("Successfully created client: {}", client.client);

    Ok(Redirect::to(&format!("/clients/{}", client.id)))
}

pub async fn update_client(
    State(state): State<AppState>,
    Path(client_id): Path<i32>,
    Form(client_data): Form<ClientForm>,
) -> Result<Redirect, (StatusCode, String)> {
    info!("Handling client update request for ID: {}", client_id);

    let client = db::update_client(&state.pool, client_id, client_data).map_err(|e| {
        warn!("Failed to update client {}: {:?}", client_id, e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update client".to_string())
    })?;

    info!("Successfully updated client: {}", client.client);

    Ok(Redirect::to(&format!("/clients/{}", client.id)))
}

pub async fn delete_client(
    State(state): State<AppState>,
    Path(client_id): Path<i32>,
) -> Result<Redirect, (StatusCode, String)> {
    info!("Handling client deletion request for ID: {}", client_id);

    db::delete_client(&state.pool, client_id).map_err(|e| {
        warn!("Failed to delete client {}: {:?}", client_id, e);
        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete client".to_string())
    })?;

    info!("Successfully deleted client with ID: {}", client_id);

    Ok(Redirect::to("/clients"))
} 
