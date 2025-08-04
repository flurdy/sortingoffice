use crate::models::*;
use crate::{db, AppState};
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{Html, Redirect},
    Form,
};
use serde::Deserialize;
use tracing::{info, warn};

use crate::handlers::database_ops::handle_entity_operation_redirect;
use crate::handlers::utils::{
    render_client_form_page, render_client_list_page, render_client_show_page,
};

#[derive(Deserialize)]
pub struct ToggleClientRedirectQuery {
    pub redirect: Option<String>,
}

pub async fn list_clients(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<PaginationParams>,
) -> Html<String> {
    let pool = match crate::handlers::utils::get_db_pool_or_handle_error(&state, &headers).await {
        Ok(pool) => pool,
        Err(error_html) => return error_html,
    };
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

    render_client_list_page(
        paginated_clients.items.clone(),
        &paginated_clients,
        &state,
        &locale,
        &headers,
    )
    .await
}

pub async fn show_client(
    State(state): State<AppState>,
    Path(client_id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = match crate::handlers::utils::get_db_pool_or_handle_error(&state, &headers).await {
        Ok(pool) => pool,
        Err(error_html) => return error_html,
    };
    let locale = crate::handlers::language::get_user_locale(&headers);

    let client = match db::get_client(&pool, client_id) {
        Ok(client) => client,
        Err(_) => {
            return crate::handlers::utils::handle_entity_not_found(
                &state,
                &headers,
                "clients",
                "clients-not-found",
            )
            .await;
        }
    };

    render_client_show_page(client, &state, &locale, &headers).await
}

pub async fn create_client_form(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    let locale = crate::handlers::language::get_user_locale(&headers);
    let form = ClientForm {
        client: String::new(),
        status: "OK".to_string(),
        enabled: true,
    };

    render_client_form_page(
        form,
        None,
        "clients-form-create-title",
        &state,
        &locale,
        &headers,
    )
    .await
}

pub async fn edit_client_form(
    State(state): State<AppState>,
    Path(client_id): Path<i32>,
    headers: HeaderMap,
) -> Html<String> {
    let pool = match crate::handlers::utils::get_db_pool_or_handle_error(&state, &headers).await {
        Ok(pool) => pool,
        Err(error_html) => return error_html,
    };
    let locale = crate::handlers::language::get_user_locale(&headers);

    let client = match db::get_client(&pool, client_id) {
        Ok(client) => client,
        Err(_) => {
            return crate::handlers::utils::handle_entity_not_found(
                &state,
                &headers,
                "clients",
                "clients-not-found",
            )
            .await;
        }
    };

    let form = ClientForm {
        client: client.client.clone(),
        status: client.status.clone(),
        enabled: client.enabled,
    };

    render_client_form_page(
        form,
        Some(client),
        "clients-form-edit-title",
        &state,
        &locale,
        &headers,
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
    let client = match handle_entity_operation_redirect(
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
    let client = match handle_entity_operation_redirect(
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
    match handle_entity_operation_redirect(
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
    let _client = match handle_entity_operation_redirect(
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
