use crate::handlers::auth::get_selected_database;
use crate::handlers::language::get_user_locale;

use crate::templates::layout::BaseTemplate;
use crate::AppState;

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};

pub async fn not_found(headers: HeaderMap, State(state): State<AppState>) -> Response {
    // Get locale and translations
    let locale = get_user_locale(&headers);
    let translations = crate::handlers::translations::get_not_found_translations(&state, &locale).await;
    let current_db_id = get_selected_database(&headers)
        .unwrap_or_else(|| state.db_manager.get_default_db_id().to_string());
    let current_db_label = state
        .db_manager
        .get_configs()
        .iter()
        .find(|db| db.id == current_db_id)
        .map(|db| db.label.clone())
        .unwrap_or_else(|| current_db_id.clone());
    let content = format!(
        r#"<div class='text-center py-16'>
            <h1 class='text-5xl font-bold text-primary-600 mb-4'>{}</h1>
            <p class='text-lg text-gray-700 dark:text-gray-300 mb-8'>{}</p>
            <a href='/' class='inline-block px-6 py-3 bg-primary-600 text-white rounded shadow hover:bg-primary-700 transition'>Go to Dashboard</a>
        </div>"#,
        translations["not-found-title"], translations["not-found-message"]
    );
    let template = BaseTemplate::with_i18n(
        translations["not-found-title"].clone(),
        content,
        &state,
        &locale,
        current_db_label,
        current_db_id,
    )
    .await
    .unwrap();
    let html = match crate::handlers::templates::render_template_safely(template) {
        Ok(html) => html,
        Err(_) => {
            return crate::handlers::errors::render_404_page(&state, &headers)
                .await
                .into_response()
        }
    };
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header("Content-Type", "text/html; charset=utf-8")
        .body(html.into())
        .unwrap()
}
