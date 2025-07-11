use axum::{
    extract::State,
    http::StatusCode,
    response::Html,
    Form,
};
use serde::Deserialize;
use askama::Template;

use crate::{AppState, render_template_with_title};

#[derive(Deserialize)]
pub struct DatabaseSelectionForm {
    database_id: String,
}

/// Show the database selection page
pub async fn index(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Html<String> {
    let databases = state.db_manager.get_configs();

    // Get the currently selected database from the session, or fall back to default
    let current_db = crate::handlers::auth::get_selected_database(&headers)
        .unwrap_or_else(|| state.db_manager.get_default_db_id().to_string());

    let content_template = crate::templates::database::DatabaseSelectionTemplate {
        databases,
        current_db: &current_db,
    };

    render_template_with_title!(content_template, "Database Selection".to_string(), &state, &"en-US", &headers)
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

    // Update the session with the new database selection
    if let Some(new_cookie) = crate::handlers::auth::update_session_database(&headers, &form.database_id) {
        // Redirect back to the dashboard with the updated cookie
        Ok(axum::response::Response::builder()
            .status(axum::http::StatusCode::FOUND)
            .header("Location", "/")
            .header("Set-Cookie", new_cookie)
            .body("".into())
            .unwrap())
    } else {
        // If we can't update the session, just redirect
        Ok(axum::response::Response::builder()
            .status(axum::http::StatusCode::FOUND)
            .header("Location", "/")
            .body("".into())
            .unwrap())
    }
}

/// Get available databases as JSON (for API use)
pub async fn list_databases(
    State(state): State<AppState>,
) -> Result<axum::Json<Vec<crate::config::DatabaseConfig>>, StatusCode> {
    let configs = state.db_manager.get_configs().to_vec();
    Ok(axum::Json(configs))
}
