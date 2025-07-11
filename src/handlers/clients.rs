use crate::templates::clients::*;
use crate::templates::layout::BaseTemplate;
use crate::{db, i18n::get_translation, models::*, AppState};
use askama::Template;
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{Html, Redirect},
    Form,
};
use serde::Deserialize;
use tracing::{info, warn};

#[derive(Deserialize)]
pub struct ToggleClientRedirectQuery {
    pub redirect: Option<String>,
}

fn is_htmx_request(headers: &HeaderMap) -> bool {
    headers.get("HX-Request").is_some_and(|v| v == "true")
}

pub async fn list_clients(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<PaginationParams>,
) -> Html<String> {
    let pool = crate::handlers::utils::get_current_db_pool(&state, &headers).await
        .expect("Failed to get database pool");
    let locale = crate::handlers::language::get_user_locale(&headers);

    // Parse pagination parameters
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(20);

    info!(
        "Handling clients list request with pagination: page={}, per_page={}",
        page, per_page
    );

    let paginated_clients = match db::get_clients_paginated(&pool, page, per_page) {
        Ok(clients) => {
            info!(
                "Successfully retrieved {} clients (page {} of {})",
                clients.items.len(),
                clients.current_page,
                clients.total_pages
            );
            clients
        }
        Err(e) => {
            warn!("Failed to retrieve clients: {:?}", e);
            PaginatedResult::new(vec![], 0, 1, per_page)
        }
    };

    // Get all translations
    let title = get_translation(&state, &locale, "clients-title").await;
    let description = get_translation(&state, &locale, "clients-description").await;
    let add_client = get_translation(&state, &locale, "clients-add").await;
    let table_header_client = get_translation(&state, &locale, "clients-table-header-client").await;
    let table_header_status = get_translation(&state, &locale, "clients-table-header-status").await;
    let table_header_actions =
        get_translation(&state, &locale, "clients-table-header-actions").await;
    let table_header_enabled =
        get_translation(&state, &locale, "clients-table-header-enabled").await;
    let status_allowed = get_translation(&state, &locale, "clients-status-allowed").await;
    let status_blocked = get_translation(&state, &locale, "clients-status-blocked").await;
    let status_enabled = get_translation(&state, &locale, "clients-status-enabled").await;
    let status_disabled = get_translation(&state, &locale, "clients-status-disabled").await;
    let action_view = get_translation(&state, &locale, "clients-action-view").await;
    let action_enable = get_translation(&state, &locale, "clients-action-enable").await;
    let action_disable = get_translation(&state, &locale, "clients-action-disable").await;
    let action_delete = get_translation(&state, &locale, "clients-action-delete").await;
    let delete_confirm = get_translation(&state, &locale, "clients-delete-confirm").await;
    let empty_title = get_translation(&state, &locale, "clients-empty-title").await;
    let empty_description = get_translation(&state, &locale, "clients-empty-description").await;

    let paginated = PaginatedResult::new(
        paginated_clients.items.clone(),
        paginated_clients.total_count,
        paginated_clients.current_page,
        paginated_clients.per_page,
    );
    let page_range: Vec<i64> = (1..=paginated.total_pages).collect();
    let max_item = std::cmp::min(
        paginated.current_page * paginated.per_page,
        paginated.total_count,
    );
    let content_template = ClientsListTemplate {
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
        clients: &paginated_clients.items,
        pagination: &paginated,
        page_range: &page_range,
        max_item,
    };

    let content = content_template.render().unwrap();

    if is_htmx_request(&headers) {
        Html(content)
    } else {
        let current_db_id = crate::handlers::auth::get_selected_database(&headers)
            .unwrap_or_else(|| state.db_manager.get_default_db_id().to_string());
        let current_db_label = state.db_manager.get_configs()
            .iter()
            .find(|db| db.id == current_db_id)
            .map(|db| db.label.clone())
            .unwrap_or_else(|| current_db_id.clone());
        let template = BaseTemplate::with_i18n(
            title,
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

pub async fn show_client(
    State(state): State<AppState>,
    Path(client_id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = crate::handlers::utils::get_current_db_pool(&state, &headers).await
        .expect("Failed to get database pool");
    let locale = crate::handlers::language::get_user_locale(&headers);

    info!("Handling client show request for ID: {}", client_id);

    let client = match db::get_client(&pool, client_id) {
        Ok(client) => client,
        Err(_) => {
            let not_found_msg = get_translation(&state, &locale, "clients-not-found").await;
            return Html(not_found_msg);
        }
    };

    info!("Successfully retrieved client: {}", client.client);

    let title = get_translation(&state, &locale, "clients-show-title").await;
    let view_edit_settings = get_translation(&state, &locale, "clients-view-edit-settings").await;
    let back_to_clients = get_translation(&state, &locale, "clients-back-to-clients").await;
    let client_information = get_translation(&state, &locale, "clients-info-title").await;
    let client_details = get_translation(&state, &locale, "clients-info-description").await;
    let client_name = get_translation(&state, &locale, "clients-field-client").await;
    let status = get_translation(&state, &locale, "clients-field-status").await;
    let status_allowed = get_translation(&state, &locale, "clients-status-allowed").await;
    let status_blocked = get_translation(&state, &locale, "clients-status-blocked").await;
    let status_enabled = get_translation(&state, &locale, "clients-status-enabled").await;
    let status_disabled = get_translation(&state, &locale, "clients-status-disabled").await;
    let created = get_translation(&state, &locale, "clients-field-created").await;
    let updated = get_translation(&state, &locale, "clients-field-updated").await;
    let edit_client = get_translation(&state, &locale, "clients-action-edit").await;
    let action_enable = get_translation(&state, &locale, "clients-action-enable").await;
    let action_disable = get_translation(&state, &locale, "clients-action-disable").await;
    let delete_client = get_translation(&state, &locale, "clients-action-delete").await;
    let delete_confirm = get_translation(&state, &locale, "clients-delete-confirm").await;
    let enabled_label = get_translation(&state, &locale, "clients-field-enabled").await;

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
        status_enabled: &status_enabled,
        status_disabled: &status_disabled,
        created: &created,
        updated: &updated,
        edit_client: &edit_client,
        action_enable: &action_enable,
        action_disable: &action_disable,
        delete_client: &delete_client,
        delete_confirm: &delete_confirm,
        enabled_label: &enabled_label,
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
        let current_db_id = crate::handlers::auth::get_selected_database(&headers)
            .unwrap_or_else(|| state.db_manager.get_default_db_id().to_string());
        let current_db_label = state.db_manager.get_configs()
            .iter()
            .find(|db| db.id == current_db_id)
            .map(|db| db.label.clone())
            .unwrap_or_else(|| current_db_id.clone());
        let template = BaseTemplate::with_i18n(
            title,
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

pub async fn create_client_form(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    let locale = crate::handlers::language::get_user_locale(&headers);

    info!("Handling client create form request");

    let title = get_translation(&state, &locale, "clients-add-title").await;
    let form_error = get_translation(&state, &locale, "form-error").await;
    let form_client = get_translation(&state, &locale, "clients-field-client").await;
    let form_status = get_translation(&state, &locale, "clients-field-status").await;
    let form_cancel = get_translation(&state, &locale, "clients-action-cancel").await;
    let form_create_client = get_translation(&state, &locale, "clients-action-save").await;
    let form_update_client = get_translation(&state, &locale, "clients-action-save").await;
    let form_placeholder_client =
        get_translation(&state, &locale, "clients-placeholder-client").await;
    let form_tooltip_client = get_translation(&state, &locale, "clients-field-client-help").await;
    let form_tooltip_status = get_translation(&state, &locale, "clients-field-status-help").await;
    let form_enabled = get_translation(&state, &locale, "clients-field-enabled").await;
    let form_tooltip_enabled = get_translation(&state, &locale, "clients-field-enabled-help").await;
    let enabled_yes = get_translation(&state, &locale, "clients-enabled-yes").await;
    let enabled_no = get_translation(&state, &locale, "clients-enabled-no").await;
    let status_allowed = get_translation(&state, &locale, "clients-status-allowed").await;
    let status_blocked = get_translation(&state, &locale, "clients-status-blocked").await;

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
        form_enabled: &form_enabled,
        form_tooltip_enabled: &form_tooltip_enabled,
        enabled_yes: &enabled_yes,
        enabled_no: &enabled_no,
        status_allowed: &status_allowed,
        status_blocked: &status_blocked,
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
        let current_db_id = crate::handlers::auth::get_selected_database(&headers)
            .unwrap_or_else(|| state.db_manager.get_default_db_id().to_string());
        let current_db_label = state.db_manager.get_configs()
            .iter()
            .find(|db| db.id == current_db_id)
            .map(|db| db.label.clone())
            .unwrap_or_else(|| current_db_id.clone());
        let template = BaseTemplate::with_i18n(
            title,
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

pub async fn edit_client_form(
    State(state): State<AppState>,
    Path(client_id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = crate::handlers::utils::get_current_db_pool(&state, &headers).await
        .expect("Failed to get database pool");
    let locale = crate::handlers::language::get_user_locale(&headers);

    info!("Handling client edit form request for ID: {}", client_id);

    let client = match db::get_client(&pool, client_id) {
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
    let form_cancel = get_translation(&state, &locale, "clients-action-cancel").await;
    let form_create_client = get_translation(&state, &locale, "clients-action-save").await;
    let form_update_client = get_translation(&state, &locale, "clients-action-save").await;
    let form_placeholder_client =
        get_translation(&state, &locale, "clients-placeholder-client").await;
    let form_tooltip_client = get_translation(&state, &locale, "clients-field-client-help").await;
    let form_tooltip_status = get_translation(&state, &locale, "clients-field-status-help").await;
    let form_enabled = get_translation(&state, &locale, "clients-field-enabled").await;
    let form_tooltip_enabled = get_translation(&state, &locale, "clients-field-enabled-help").await;
    let enabled_yes = get_translation(&state, &locale, "clients-enabled-yes").await;
    let enabled_no = get_translation(&state, &locale, "clients-enabled-no").await;
    let status_allowed = get_translation(&state, &locale, "clients-status-allowed").await;
    let status_blocked = get_translation(&state, &locale, "clients-status-blocked").await;

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
        form_enabled: &form_enabled,
        form_tooltip_enabled: &form_tooltip_enabled,
        enabled_yes: &enabled_yes,
        enabled_no: &enabled_no,
        status_allowed: &status_allowed,
        status_blocked: &status_blocked,
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
        let current_db_id = crate::handlers::auth::get_selected_database(&headers)
            .unwrap_or_else(|| state.db_manager.get_default_db_id().to_string());
        let current_db_label = state.db_manager.get_configs()
            .iter()
            .find(|db| db.id == current_db_id)
            .map(|db| db.label.clone())
            .unwrap_or_else(|| current_db_id.clone());
        let template = BaseTemplate::with_i18n(
            title,
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

pub async fn create_client(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(client_data): Form<ClientForm>,
) -> Result<Redirect, (StatusCode, String)> {
    info!("Handling client creation request");

    let pool = crate::handlers::utils::get_current_db_pool(&state, &headers).await
        .expect("Failed to get database pool");
    let client = db::create_client(&pool, client_data).map_err(|e| {
        warn!("Failed to create client: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to create client".to_string(),
        )
    })?;

    info!("Successfully created client: {}", client.client);

    Ok(Redirect::to(&format!("/clients/{}", client.id)))
}

pub async fn update_client(
    State(state): State<AppState>,
    Path(client_id): Path<i32>,
    headers: HeaderMap,
    Form(client_data): Form<ClientForm>,
) -> Result<Redirect, (StatusCode, String)> {
    info!("Handling client update request for ID: {}", client_id);

    let pool = crate::handlers::utils::get_current_db_pool(&state, &headers).await
        .expect("Failed to get database pool");
    let client = db::update_client(&pool, client_id, client_data).map_err(|e| {
        warn!("Failed to update client {}: {:?}", client_id, e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to update client".to_string(),
        )
    })?;

    info!("Successfully updated client: {}", client.client);

    Ok(Redirect::to(&format!("/clients/{}", client.id)))
}

pub async fn delete_client(
    State(state): State<AppState>,
    Path(client_id): Path<i32>,
    headers: HeaderMap,
) -> Result<Redirect, (StatusCode, String)> {
    info!("Handling client deletion request for ID: {}", client_id);

    let pool = crate::handlers::utils::get_current_db_pool(&state, &headers).await
        .expect("Failed to get database pool");
    db::delete_client(&pool, client_id).map_err(|e| {
        warn!("Failed to delete client {}: {:?}", client_id, e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to delete client".to_string(),
        )
    })?;

    info!("Successfully deleted client with ID: {}", client_id);

    Ok(Redirect::to("/clients"))
}

pub async fn toggle_client(
    State(state): State<AppState>,
    Path(client_id): Path<i32>,
    headers: HeaderMap,
    Query(redirect_query): Query<ToggleClientRedirectQuery>,
) -> Result<Redirect, (StatusCode, String)> {
    info!("Handling client toggle request for ID: {}", client_id);

    let pool = crate::handlers::utils::get_current_db_pool(&state, &headers).await
        .expect("Failed to get database pool");
    let client = db::toggle_client_enabled(&pool, client_id).map_err(|e| {
        warn!("Failed to toggle client {}: {:?}", client_id, e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to toggle client".to_string(),
        )
    })?;

    info!("Successfully toggled client: {}", client.client);

    let redirect_url = match redirect_query.redirect.as_deref() {
        Some("list") => "/clients".to_string(),
        Some("show") | None => format!("/clients/{client_id}"),
        Some(_) => format!("/clients/{client_id}"),
    };

    Ok(Redirect::to(&redirect_url))
}
