use axum::{
    extract::State,
    http::StatusCode,
    response::Response,
    Form,
};
use serde::Deserialize;
use crate::AppState;

#[derive(Deserialize)]
pub struct ThemeForm {
    theme: String,
}

pub async fn toggle_theme(
    State(_state): State<AppState>,
    Form(form): Form<ThemeForm>,
) -> Response {
    let new_theme = if form.theme == "dark" { "light" } else { "dark" };
    
    // Return a simple response that will be handled by JavaScript
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/plain")
        .body(new_theme.into())
        .unwrap()
} 
