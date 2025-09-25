use crate::templates::layout::BaseTemplate;
use crate::templates::cache_management::CacheManagementTemplate;
use crate::{AppState, i18n::get_translation};
use askama::Template;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Html,
    Form,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CacheManagementForm {
    pub action: String,
    pub cache_type: Option<String>,
}

pub async fn cache_management(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Html<String>, StatusCode> {
    let locale = crate::handlers::language::get_user_locale(&headers);

    // Get translations
    let title = get_translation(&state, &locale, "cache-management-title").await;

    // Get current cache statistics
    let cache_stats = state.db_manager.get_cache_stats().await;

    // Get current database info
    let current_db_id = crate::handlers::auth::get_selected_database(&headers);
    let current_db_label = state
        .db_manager
        .get_configs()
        .iter()
        .find(|db| db.id == current_db_id.as_deref().unwrap_or("primary"))
        .map(|db| db.label.clone())
        .unwrap_or_else(|| current_db_id.clone().unwrap_or_else(|| "primary".to_string()));

    // Create the cache management template
    let cache_template = CacheManagementTemplate::new(cache_stats);
    let content = match cache_template.render() {
        Ok(content) => content,
        Err(e) => {
            tracing::error!("Error rendering cache management template: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let template = match BaseTemplate::with_i18n(
        title,
        content,
        &state,
        &locale,
        current_db_label,
        current_db_id.unwrap_or_else(|| "primary".to_string()),
    )
    .await
    {
        Ok(template) => template,
        Err(e) => {
            tracing::error!("Error creating base template: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    match template.render() {
        Ok(content) => Ok(Html(content)),
        Err(e) => {
            tracing::error!("Error rendering template: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn cache_management_post(
    State(state): State<AppState>,
    Form(form): Form<CacheManagementForm>,
) -> Result<Html<String>, StatusCode> {
    match form.action.as_str() {
        "clear" => {
            if let Some(cache_type) = form.cache_type {
                state.db_manager.clear_cache_by_type(&cache_type).await;
                tracing::info!("Cleared cache type: {}", cache_type);
            } else {
                state.db_manager.clear_all_caches().await;
                tracing::info!("Cleared all caches");
            }
        }
        _ => {
            tracing::warn!("Unknown cache management action: {}", form.action);
        }
    }

    // Redirect back to cache management page
    Ok(Html(format!(
        r#"<script>window.location.href = '/cache-management';</script>"#
    )))
}
