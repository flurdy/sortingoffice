use askama::Template;
use axum::{extract::State, http::StatusCode, response::Html, Form};
use serde::Deserialize;

use crate::{render_template_with_title, AppState};

/// Helper function to redirect resource-specific pages to their list pages when switching databases
fn redirect_to_list_page_if_resource_specific(url: &str) -> &str {
    // Parse the URL to extract the path
    let path = if let Some(query_start) = url.find('?') {
        &url[..query_start]
    } else {
        url
    };
    
    // Extract just the path part (remove protocol and domain)
    let path_only = if let Some(path_start) = path.find("://") {
        if let Some(slash_pos) = path[path_start + 3..].find('/') {
            &path[path_start + 3 + slash_pos..]
        } else {
            "/"
        }
    } else {
        path
    };
    
    
    // Check for resource-specific patterns and redirect to list pages
    if path_only.starts_with("/domains/") && path_only.len() > "/domains/".len() {
        // /domains/123 -> /domains
        tracing::info!("Redirecting resource-specific page {} to /domains", path_only);
        return "/domains";
    } else if path_only.starts_with("/aliases/") && path_only.len() > "/aliases/".len() {
        // /aliases/123 -> /aliases
        tracing::info!("Redirecting resource-specific page {} to /aliases", path_only);
        return "/aliases";
    } else if path_only.starts_with("/users/") && path_only.len() > "/users/".len() {
        // /users/123 -> /users
        tracing::info!("Redirecting resource-specific page {} to /users", path_only);
        return "/users";
    } else if path_only.starts_with("/clients/") && path_only.len() > "/clients/".len() {
        // /clients/123 -> /clients
        tracing::info!("Redirecting resource-specific page {} to /clients", path_only);
        return "/clients";
    } else if path_only.starts_with("/relays/") && path_only.len() > "/relays/".len() {
        // /relays/123 -> /relays
        tracing::info!("Redirecting resource-specific page {} to /relays", path_only);
        return "/relays";
    } else if path_only.starts_with("/relocated/") && path_only.len() > "/relocated/".len() {
        // /relocated/123 -> /relocated
        tracing::info!("Redirecting resource-specific page {} to /relocated", path_only);
        return "/relocated";
    } else if path_only.starts_with("/domain_backup/") && path_only.len() > "/domain_backup/".len() {
        // /domain_backup/123 -> /domains (backup domains are shown in domains list)
        tracing::info!("Redirecting resource-specific page {} to /domains", path_only);
        return "/domains";
    }
    
    // Return the original URL if it's not a resource-specific page
    tracing::info!("No redirect needed for URL: {}", url);
    url
}

#[derive(Deserialize)]
pub struct DatabaseSelectionForm {
    database_id: String,
    redirect: Option<String>,
}

/// Show the database selection page
pub async fn index(State(state): State<AppState>, headers: axum::http::HeaderMap) -> Html<String> {
    let databases = state.db_manager.get_enabled_configs();

    // Get the currently selected database from the session, or fall back to default
    let current_db = crate::handlers::auth::get_selected_database(&headers)
        .unwrap_or_else(|| state.db_manager.get_default_db_id().to_string());

    // Check if the currently selected database is disabled
    let current_db = if state.config.is_database_disabled(&current_db) {
        // If current database is disabled, fall back to default or first available
        state.db_manager.get_default_db_id().to_string()
    } else {
        current_db
    };

    let locale = crate::handlers::language::get_user_locale(&headers);

    // Get translations for the database selection page
    let translations = crate::handlers::translations::get_translations_batch(
        &state,
        &locale,
        &[
            "database-selection-title",
            "database-selection-description",
            "database-switch-button",
        ],
    )
    .await;

    let content_template = crate::templates::database::DatabaseSelectionTemplate {
        databases: &databases,
        current_db: &current_db,
        title: &translations["database-selection-title"],
        description: &translations["database-selection-description"],
        switch_button: &translations["database-switch-button"],
    };

    render_template_with_title!(
        content_template,
        translations["database-selection-title"].clone(),
        &state,
        &locale,
        &headers
    )
}

/// Handle database selection
pub async fn select(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Form(form): Form<DatabaseSelectionForm>,
) -> Result<axum::response::Response, StatusCode> {
    // Validate that the selected database exists
    if !state.db_manager.has_database(&form.database_id).await {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Determine redirect target
    let redirect_url = form.redirect.as_deref().unwrap_or("/");
    let redirect_url = if redirect_url.is_empty() {
        "/"
    } else {
        // Check if we're on a resource-specific page and redirect to the list page
        redirect_to_list_page_if_resource_specific(redirect_url)
    };

    // Update the session with the new database selection
    let new_cookie = crate::handlers::auth::update_session_database(&headers, &form.database_id);

    // Clear all caches when database changes to ensure fresh data
    state.db_manager.clear_all_caches().await;

    // Check if this is an HTMX request
    let is_htmx = headers
        .get("HX-Request")
        .and_then(|v| v.to_str().ok())
        .map(|v| v == "true")
        .unwrap_or(false);

    if is_htmx {
        // For HTMX requests, return a response with HX-Redirect header
        let mut response_builder = axum::response::Response::builder()
            .status(axum::http::StatusCode::OK)
            .header("HX-Redirect", redirect_url);

        if let Some(cookie) = new_cookie {
            response_builder = response_builder.header("Set-Cookie", cookie);
        }

        Ok(response_builder.body("".into()).unwrap())
    } else {
        // For regular requests, return a standard redirect
        let mut response_builder = axum::response::Response::builder()
            .status(axum::http::StatusCode::FOUND)
            .header("Location", redirect_url);

        if let Some(cookie) = new_cookie {
            response_builder = response_builder.header("Set-Cookie", cookie);
        }

        Ok(response_builder.body("".into()).unwrap())
    }
}

/// Get available databases as JSON (for API use)
pub async fn list_databases(
    State(state): State<AppState>,
) -> Result<axum::Json<Vec<crate::config::DatabaseConfig>>, StatusCode> {
    let configs = state.db_manager.get_enabled_configs();
    Ok(axum::Json(configs))
}

/// HTMX endpoint to render the database dropdown
pub async fn dropdown(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Html<String> {
    let databases = state.db_manager.get_enabled_configs();
    let current_db = crate::handlers::auth::get_selected_database(&headers)
        .unwrap_or_else(|| state.db_manager.get_default_db_id().to_string());

    // Check if the currently selected database is disabled
    let current_db = if state.config.is_database_disabled(&current_db) {
        // If current database is disabled, fall back to default or first available
        state.db_manager.get_default_db_id().to_string()
    } else {
        current_db
    };

    // Try to get the current URL from HTMX headers first, then Referer header, fallback to "/"
    let current_url = headers
        .get("HX-Current-URL")
        .and_then(|v| v.to_str().ok())
        .or_else(|| headers.get("Referer").and_then(|v| v.to_str().ok()))
        .unwrap_or("/");
    
    let content_template = crate::templates::database::DatabaseDropdownTemplate {
        databases: &databases,
        current_db: &current_db,
        current_url,
    };
    match crate::handlers::templates::render_template_safely(content_template) {
        Ok(content) => Html(content),
        Err(_) => crate::handlers::errors::render_500_page(&state, &headers).await,
    }
}
