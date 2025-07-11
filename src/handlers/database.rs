use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, Redirect},
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
    let current_db = state.db_manager.get_default_db_id();



    let content_template = crate::templates::database::DatabaseSelectionTemplate {
        databases,
        current_db: &current_db,
    };

    render_template_with_title!(content_template, "Database Selection".to_string(), &state, &"en-US", &headers)
}

/// Handle database selection
pub async fn select(
    State(state): State<AppState>,
    Form(form): Form<DatabaseSelectionForm>,
) -> Result<Redirect, StatusCode> {
    // Validate that the selected database exists
    if !state.db_manager.has_database(&form.database_id).await {
        return Err(StatusCode::BAD_REQUEST);
    }

    // For now, we'll redirect back to the dashboard
    // In a full implementation, you might want to store the selection in a session
    Ok(Redirect::to("/"))
}

/// Get available databases as JSON (for API use)
pub async fn list_databases(
    State(state): State<AppState>,
) -> Result<axum::Json<Vec<crate::config::DatabaseConfig>>, StatusCode> {
    let configs = state.db_manager.get_configs().to_vec();
    Ok(axum::Json(configs))
}
