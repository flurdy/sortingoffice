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
    let pool = crate::handlers::utils::get_current_db_pool(&state, &headers)
        .await
        .expect("Failed to get database pool");
    let locale = crate::handlers::language::get_user_locale(&headers);
    let current_db_id = crate::handlers::auth::get_selected_database(&headers)
        .unwrap_or_else(|| state.db_manager.get_default_db_id().to_string());

    // Check if clients table is available for this database
    if !state.config.is_clients_available(&current_db_id) {
        let not_available_msg =
            crate::i18n::get_translation(&state, &locale, "clients-not-available").await;
        return Html(not_available_msg);
    }

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

    // Get all translations using consolidated helper functions
    let translations =
        crate::handlers::utils::get_entity_all_translations(&state, &locale, "clients").await;

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
        title: &translations["clients-title"],
        description: &translations["clients-description"],
        add_client: &translations["clients-add"],
        table_header_client: &translations["clients-table-header-client"],
        table_header_status: &translations["clients-table-header-status"],
        table_header_enabled: &translations["clients-table-header-enabled"],
        table_header_actions: &translations["clients-table-header-actions"],
        status_allowed: &translations["clients-status-ok"],
        status_blocked: &translations["clients-status-reject"],
        status_enabled: &translations["clients-status-enabled"],
        status_disabled: &translations["clients-status-disabled"],
        action_view: &translations["clients-action-view"],
        action_enable: &translations["clients-action-enable"],
        action_disable: &translations["clients-action-disable"],
        action_delete: &translations["clients-action-delete"],
        delete_confirm: &translations["clients-delete-confirm"],
        empty_title: &translations["clients-empty-title"],
        empty_description: &translations["clients-empty-description"],
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
        let current_db_label = state
            .db_manager
            .get_configs()
            .iter()
            .find(|db| db.id == current_db_id)
            .map(|db| db.label.clone())
            .unwrap_or_else(|| current_db_id.clone());
        let template = BaseTemplate::with_i18n(
            translations["clients-title"].clone(),
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
    let pool = crate::handlers::utils::get_current_db_pool(&state, &headers)
        .await
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

    // Get all translations using consolidated helper functions
    let translations =
        crate::handlers::utils::get_entity_all_translations(&state, &locale, "clients").await;

    let content_template = ClientShowTemplate {
        title: &translations["clients-show-title"],
        client,
        view_edit_settings: &translations["clients-view-edit-settings"],
        back_to_clients: &translations["clients-back-to-clients"],
        client_information: &translations["clients-info-title"],
        client_details: &translations["clients-info-description"],
        client_name: &translations["clients-field-client"],
        status: &translations["clients-field-status"],
        status_allowed: &translations["clients-status-ok"],
        status_blocked: &translations["clients-status-reject"],
        status_enabled: &translations["clients-status-enabled"],
        status_disabled: &translations["clients-status-disabled"],
        created: &translations["clients-field-created"],
        updated: &translations["clients-field-updated"],
        edit_client: &translations["clients-action-edit"],
        action_enable: &translations["clients-action-enable"],
        action_disable: &translations["clients-action-disable"],
        delete_client: &translations["clients-action-delete"],
        delete_confirm: &translations["clients-delete-confirm"],
        enabled_label: &translations["clients-field-enabled"],
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
        let current_db_label = state
            .db_manager
            .get_configs()
            .iter()
            .find(|db| db.id == current_db_id)
            .map(|db| db.label.clone())
            .unwrap_or_else(|| current_db_id.clone());
        let template = BaseTemplate::with_i18n(
            translations["clients-show-title"].clone(),
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

    // Use helper functions to fetch translations in batches
    let form_translations =
        crate::handlers::utils::get_entity_form_translations(&state, &locale, "clients").await;
    let field_translations = crate::handlers::utils::get_field_translations(
        &state,
        &locale,
        "clients",
        &["client", "status", "enabled"],
    )
    .await;
    let status_translations =
        crate::handlers::utils::get_status_translations(&state, &locale, "clients").await;

    let content_template = ClientFormTemplate {
        title: &form_translations["clients-add-title"],
        client: None,
        form_error: &form_translations["form-error"],
        form_client: &field_translations["clients-field-client"],
        form_status: &field_translations["clients-field-status"],
        form_cancel: &form_translations["form-cancel"],
        form_create_client: &form_translations["action-save"],
        form_update_client: &form_translations["action-save"],
        form_placeholder_client: &field_translations["clients-placeholder-client"],
        form_tooltip_client: &field_translations["clients-field-client-help"],
        form_tooltip_status: &field_translations["clients-field-status-help"],
        form_enabled: &field_translations["clients-field-enabled"],
        form_tooltip_enabled: &field_translations["clients-field-enabled-help"],
        enabled_yes: &status_translations["clients-enabled-yes"],
        enabled_no: &status_translations["clients-enabled-no"],
        status_allowed: &status_translations["clients-status-ok"],
        status_blocked: &status_translations["clients-status-reject"],
    };

    // Use helper function for template rendering
    crate::handlers::utils::render_form_template(
        content_template,
        &state,
        &locale,
        &headers,
        form_translations["clients-add-title"].clone(),
    )
    .await
}

pub async fn edit_client_form(
    State(state): State<AppState>,
    Path(client_id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = crate::handlers::utils::get_current_db_pool(&state, &headers)
        .await
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

    // Use helper functions to fetch translations in batches
    let form_translations =
        crate::handlers::utils::get_entity_form_translations(&state, &locale, "clients").await;
    let field_translations = crate::handlers::utils::get_field_translations(
        &state,
        &locale,
        "clients",
        &["client", "status", "enabled"],
    )
    .await;
    let status_translations =
        crate::handlers::utils::get_status_translations(&state, &locale, "clients").await;

    let content_template = ClientFormTemplate {
        title: &form_translations["clients-edit-title"],
        client: Some(client),
        form_error: &form_translations["form-error"],
        form_client: &field_translations["clients-field-client"],
        form_status: &field_translations["clients-field-status"],
        form_cancel: &form_translations["form-cancel"],
        form_create_client: &form_translations["action-save"],
        form_update_client: &form_translations["action-save"],
        form_placeholder_client: &field_translations["clients-placeholder-client"],
        form_tooltip_client: &field_translations["clients-field-client-help"],
        form_tooltip_status: &field_translations["clients-field-status-help"],
        form_enabled: &field_translations["clients-field-enabled"],
        form_tooltip_enabled: &field_translations["clients-field-enabled-help"],
        enabled_yes: &status_translations["clients-enabled-yes"],
        enabled_no: &status_translations["clients-enabled-no"],
        status_allowed: &status_translations["clients-status-ok"],
        status_blocked: &status_translations["clients-status-reject"],
    };

    // Use helper function for template rendering
    crate::handlers::utils::render_form_template(
        content_template,
        &state,
        &locale,
        &headers,
        form_translations["clients-edit-title"].clone(),
    )
    .await
}

pub async fn create_client(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(client_data): Form<ClientForm>,
) -> Result<Redirect, (StatusCode, String)> {
    info!("Handling client creation request");

    // Get database pool using helper function
    let pool = match crate::handlers::utils::get_db_pool_or_redirect_error(&state, &headers).await {
        Ok(pool) => pool,
        Err(error) => return Err(error),
    };

    // Use helper function for entity operation
    let client = match crate::handlers::utils::handle_entity_operation_redirect(
        || async { db::create_client(&pool, client_data) },
        &state,
        "create client",
        "new client",
        "Successfully created client",
    )
    .await
    {
        Ok(client) => client,
        Err(error) => return Err(error),
    };

    Ok(Redirect::to(&format!("/clients/{}", client.id)))
}

pub async fn update_client(
    State(state): State<AppState>,
    Path(client_id): Path<i32>,
    headers: HeaderMap,
    Form(client_data): Form<ClientForm>,
) -> Result<Redirect, (StatusCode, String)> {
    info!("Handling client update request for ID: {}", client_id);

    // Get database pool using helper function
    let pool = match crate::handlers::utils::get_db_pool_or_redirect_error(&state, &headers).await {
        Ok(pool) => pool,
        Err(error) => return Err(error),
    };

    // Use helper function for entity operation
    let client = match crate::handlers::utils::handle_entity_operation_redirect(
        || async { db::update_client(&pool, client_id, client_data) },
        &state,
        "update client",
        &client_id.to_string(),
        "Successfully updated client",
    )
    .await
    {
        Ok(client) => client,
        Err(error) => return Err(error),
    };

    Ok(Redirect::to(&format!("/clients/{}", client.id)))
}

pub async fn delete_client(
    State(state): State<AppState>,
    Path(client_id): Path<i32>,
    headers: HeaderMap,
) -> Result<Redirect, (StatusCode, String)> {
    info!("Handling client deletion request for ID: {}", client_id);

    // Get database pool using helper function
    let pool = match crate::handlers::utils::get_db_pool_or_redirect_error(&state, &headers).await {
        Ok(pool) => pool,
        Err(error) => return Err(error),
    };

    // Use helper function for entity operation
    match crate::handlers::utils::handle_entity_operation_redirect(
        || async { db::delete_client(&pool, client_id) },
        &state,
        "delete client",
        &client_id.to_string(),
        "Successfully deleted client",
    )
    .await
    {
        Ok(_) => Ok(Redirect::to("/clients")),
        Err(error) => Err(error),
    }
}

pub async fn toggle_client(
    State(state): State<AppState>,
    Path(client_id): Path<i32>,
    headers: HeaderMap,
    Query(redirect_query): Query<ToggleClientRedirectQuery>,
) -> Result<Redirect, (StatusCode, String)> {
    info!("Handling client toggle request for ID: {}", client_id);

    // Get database pool using helper function
    let pool = match crate::handlers::utils::get_db_pool_or_redirect_error(&state, &headers).await {
        Ok(pool) => pool,
        Err(error) => return Err(error),
    };

    // Use helper function for entity operation
    let _client = match crate::handlers::utils::handle_entity_operation_redirect(
        || async { db::toggle_client_enabled(&pool, client_id) },
        &state,
        "toggle client",
        &client_id.to_string(),
        "Successfully toggled client",
    )
    .await
    {
        Ok(client) => client,
        Err(error) => return Err(error),
    };

    let redirect_url = match redirect_query.redirect.as_deref() {
        Some("list") => "/clients".to_string(),
        Some("show") | None => format!("/clients/{client_id}"),
        Some(_) => format!("/clients/{client_id}"),
    };

    Ok(Redirect::to(&redirect_url))
}
