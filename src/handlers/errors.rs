use crate::AppState;
use axum::http::HeaderMap;
use axum::response::Html;

/// Custom error type for database operations
/// Based on structured error handling patterns from Rust error handling guide
#[derive(Debug, thiserror::Error)]
pub enum DatabaseOperationError {
    #[error("Database connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Database query failed: {0}")]
    QueryFailed(String),

    #[error("Entity not found: {entity_type} with id {identifier}")]
    NotFound {
        entity_type: String,
        identifier: String,
    },

    #[error("Database constraint violation: {0}")]
    ConstraintViolation(String),

    #[error("Database operation not allowed: {operation} on {entity}")]
    OperationNotAllowed { operation: String, entity: String },
}

/// Custom error type for form validation operations
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Required field missing: {field}")]
    RequiredFieldMissing { field: String },

    #[error("Invalid format: {field} - {reason}")]
    InvalidFormat { field: String, reason: String },

    #[error("Validation failed: {message}")]
    Custom { message: String },
}

/// Helper function to render error pages consistently with proper theming and translations
pub async fn render_error_page(
    title_key: &str,
    message_key: &str,
    state: &AppState,
    headers: &HeaderMap,
) -> Html<String> {
    let locale = crate::handlers::utils::get_user_locale(headers);
    let (current_db_label, current_db_id) =
        crate::handlers::utils::get_current_db_info_optimized(state, headers);

    let title = crate::i18n::get_translation(state, &locale, title_key).await;
    let message = crate::i18n::get_translation(state, &locale, message_key).await;

    println!(
        "[DEBUG] Rendering error page with title: {}, message: {}",
        title, message
    );

    match crate::templates::error::ErrorTemplate::new(
        &title,
        &message,
        state,
        &locale,
        &current_db_label,
        &current_db_id,
    )
    .await
    {
        Ok(template) => match crate::handlers::utils::render_template_safely(template) {
            Ok(html) => {
                println!(
                    "[DEBUG] Error template rendered successfully, length: {}",
                    html.len()
                );
                Html(html)
            }
            Err(e) => {
                println!("[DEBUG] Error rendering template: {}", e);
                Html("Error rendering error page".to_string())
            }
        },
        Err(e) => {
            println!("[DEBUG] Error creating template: {:?}", e);
            Html("Error creating error page".to_string())
        }
    }
}

/// Render a 404 Not Found error page
pub async fn render_404_page(state: &AppState, headers: &HeaderMap) -> Html<String> {
    render_error_page(
        "error-page-not-found-title",
        "error-page-not-found-message",
        state,
        headers,
    )
    .await
}

/// Render a 500 Internal Server Error page
pub async fn render_500_page(state: &AppState, headers: &HeaderMap) -> Html<String> {
    render_error_page(
        "error-page-operation-failed-title",
        "error-page-operation-failed-message",
        state,
        headers,
    )
    .await
}

/// Render a 403 Forbidden error page
pub async fn render_403_page(state: &AppState, headers: &HeaderMap) -> Html<String> {
    render_error_page(
        "error-page-validation-error-title",
        "error-page-validation-error-message",
        state,
        headers,
    )
    .await
}

/// Render a 401 Unauthorized error page
pub async fn render_401_page(state: &AppState, headers: &HeaderMap) -> Html<String> {
    render_error_page(
        "error-page-validation-error-title",
        "error-page-validation-error-message",
        state,
        headers,
    )
    .await
}

/// Render a database error page
pub async fn render_database_error_page(state: &AppState, headers: &HeaderMap) -> Html<String> {
    render_error_page(
        "error-page-database-error-title",
        "error-page-database-error-message",
        state,
        headers,
    )
    .await
}

/// Render a domain not found page
pub async fn render_domain_not_found_page(state: &AppState, headers: &HeaderMap) -> Html<String> {
    render_error_page(
        "domains-not-found-title",
        "domains-not-found-message",
        state,
        headers,
    )
    .await
}

/// Render a user not found page
pub async fn render_user_not_found_page(state: &AppState, headers: &HeaderMap) -> Html<String> {
    render_error_page(
        "users-not-found-title",
        "users-not-found-message",
        state,
        headers,
    )
    .await
}

/// Render an alias not found page
pub async fn render_alias_not_found_page(state: &AppState, headers: &HeaderMap) -> Html<String> {
    render_error_page(
        "aliases-not-found-title",
        "aliases-not-found-message",
        state,
        headers,
    )
    .await
}

/// Render a client not found page
pub async fn render_client_not_found_page(state: &AppState, headers: &HeaderMap) -> Html<String> {
    render_error_page(
        "clients-not-found-title",
        "clients-not-found-message",
        state,
        headers,
    )
    .await
}

/// Render a relay not found page
pub async fn render_relay_not_found_page(state: &AppState, headers: &HeaderMap) -> Html<String> {
    render_error_page(
        "relays-not-found-title",
        "relays-not-found-message",
        state,
        headers,
    )
    .await
}

/// Render a backup not found page
pub async fn render_backup_not_found_page(state: &AppState, headers: &HeaderMap) -> Html<String> {
    render_error_page(
        "backups-not-found-title",
        "backups-not-found-message",
        state,
        headers,
    )
    .await
}

/// Render a relocated not found page
pub async fn render_relocated_not_found_page(
    state: &AppState,
    headers: &HeaderMap,
) -> Html<String> {
    render_error_page(
        "relocated-not-found-title",
        "relocated-not-found-message",
        state,
        headers,
    )
    .await
}

/// Handle entity not found with consistent error handling
pub async fn handle_entity_not_found(
    state: &AppState,
    headers: &HeaderMap,
    entity_type: &str,
    _error_key: &str,
) -> Html<String> {
    match entity_type {
        "domains" => render_domain_not_found_page(state, headers).await,
        "users" => render_user_not_found_page(state, headers).await,
        "aliases" => render_alias_not_found_page(state, headers).await,
        "clients" => render_client_not_found_page(state, headers).await,
        "relays" => render_relay_not_found_page(state, headers).await,
        "backups" => render_backup_not_found_page(state, headers).await,
        "relocated" => render_relocated_not_found_page(state, headers).await,
        _ => render_404_page(state, headers).await,
    }
}
